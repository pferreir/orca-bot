use std::{
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use chrono::prelude::*;
use clap::Parser;
use tempfile::TempDir;
use tokio::time;

mod encoding;
mod history;
mod mastodon;
mod parser;
mod vm;

use history::Log;
use mastodon::Client;
use parser::{parse_html, ParseConfig};

/// Uxn runner
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// ROM to load and execute
    rom: PathBuf,

    /// Use the native Uxn implementation
    #[clap(long)]
    native: bool,

    /// Don't post on Mastodon
    #[clap(long)]
    do_not_post: bool,

    /// Minimum interval between requests from the same user (seconds)
    #[clap(env, long, default_value_t = 30)]
    min_wait_interval: usize,

    /// Maximum number of requests from the same user in an hour
    #[clap(env, long, default_value_t = 5)]
    max_requests_hour: usize,

    /// Maximum length of lines
    #[clap(env, long, default_value_t = 16)]
    max_line_length: u8,

    /// Maximum number of lines
    #[clap(env, long, default_value_t = 16)]
    max_num_lines: u8,

    /// Location of history file
    #[clap(env, long, default_value = "history.csv")]
    history_file: PathBuf,
    
    /// Mastodon instance URL
    #[clap(env, long, required = true)]
    mastodon_instance_url: String,
    
    /// Mastodon access token
    #[clap(env, long, required = true)]
    mastodon_access_token: String,

    /// Arguments to pass into the VM
    #[arg(last = true)]
    args: Vec<String>,
}

/// Run a simulation and video encoding job
fn run_job<'t>(
    rom: impl AsRef<Path>,
    input: impl Iterator<Item = &'t [char]>,
    native: bool,
    args: &'t Vec<String>,
) -> Result<TempDir> {
    let screen_dir = tempfile::tempdir()?;
    let audio_file = screen_dir.as_ref().join("audio.pcm");
    let out_file = screen_dir.as_ref().join("out.mp4");

    let vm = vm::VMWrapper::new(&screen_dir, &audio_file, args, native);
    let (width, height) = vm
        .run(rom, input, 600)
        .context("Couldn't run the VM properly")?;

    log::debug!("Generating {width}x{height} video...");

    encoding::encode(&screen_dir, (width, height), &audio_file, &out_file)
        .context("Can't encode video")?;

    log::debug!("Done!");

    Ok(screen_dir)
}

/// Check that the user rate limits haven't been crossed
fn user_rate_is_ok(
    history: &Log,
    username: &str,
    min_wait_interval: Duration,
    max_requests_hour: usize,
) -> bool {
    if history
        .iter_from_user(Local::now() - min_wait_interval, username)
        .next()
        .is_some()
    {
        // there is at least one history entry from this user in the last N seconds
        false
    } else if history
        .iter_from_user(Local::now() - Duration::from_secs(60 * 60), username)
        .count()
        >= max_requests_hour
    {
        // there are more then N requests from this user in the last hours
        false
    } else {
        true
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let env = env_logger::Env::default()
        .filter_or("LOG", "info")
        .write_style_or("LOG", "always");
    env_logger::init_from_env(env);

    let args = Args::parse();

    let mut history = Log::new(args.history_file)?;

    log::info!("orca-bot has started! 🎛️ 🤖");

    let parse_config = ParseConfig {
        max_line_size: args.max_line_length,
        max_num_lines: args.max_num_lines,
    };

    let client = Client::new(args.mastodon_instance_url, args.mastodon_access_token)?;

    loop {
        for (notif_id, post_id, (username, url), content) in client.get_notifications().await? {
            log::info!("Processing post {post_id} from {username} ({url})");

            if !user_rate_is_ok(
                &history,
                &username,
                Duration::from_secs(args.min_wait_interval as u64),
                args.max_requests_hour,
            ) {
                log::warn!("Request from {username} ignored due to rate limit");
                client.clear_notification(&notif_id).await?;
                continue;
            }

            match parse_html(&content, &parse_config) {
                Ok(source) => {
                    log::debug!("HTML OK");
                    match run_job(&args.rom, source.iter_lines(), args.native, &args.args) {
                        Ok(video_path) => {
                            let out_file = video_path.as_ref().join("out.mp4");
                            log::info!(
                                "File is {} KB long",
                                std::fs::metadata(&out_file)?.size() / 1024
                            );
                            if !args.do_not_post {
                                log::info!("Posting to mastodon, replying to {post_id}");

                                // Post on Mastodon
                                let url =
                                    client.post_result(&username, &post_id, &out_file).await?;
                                client.clear_notification(&notif_id).await?;
                                log::info!("All done! {url}");
                                history.log(Utc::now(), &username, &url)?;
                            } else {
                                log::info!("All done! (wink wink!)");
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to run job for post {post_id}: {e}");
                        }
                    }
                }
                Err(e) => {
                    log::error!("Can't parse post {post_id}: {e}");
                }
            }
        }

        time::sleep(Duration::from_secs(60)).await;
    }
}
