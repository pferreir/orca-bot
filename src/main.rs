use std::{
    fs::{self, File},
    io::{stdin, Read},
    os::unix::fs::MetadataExt,
    path::Path,
    time::Duration,
};

use anyhow::{Context, Result};
use chrono::prelude::*;
use clap::Parser;
use cli::{RunArgs, SubCommands};
use tempfile::TempDir;
use tokio::time;

mod cli;
mod encoding;
mod history;
mod mastodon;
mod parser;
mod vm;

use history::Log;
use mastodon::Client;
use parser::{parse_html, parse_orca_code, ParseConfig};

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

/// Check that the aaccount rate limits haven't been crossed
fn user_rate_is_ok(
    history: &Log,
    username: &str,
    min_wait_interval: Duration,
    max_requests_hour: usize,
) -> bool {
    if history
        .iter_from_for_user(Local::now() - min_wait_interval, username)
        .next()
        .is_some()
    {
        // there is at least one history entry from this account in the last N seconds
        false
    } else if history
        .iter_from_for_user(Local::now() - Duration::from_secs(60 * 60), username)
        .count()
        >= max_requests_hour
    {
        // there are more then N requests from this account in the last hours
        false
    } else {
        true
    }
}

async fn run_cmd(args: RunArgs) -> Result<()> {
    let mut history = Log::new(args.history_file)?;

    log::info!("orca-bot has started! 🎛️ 🤖");

    let parse_config = ParseConfig {
        tag: &args.run_tag,
        max_line_length: args.max_line_length,
        max_num_lines: args.max_num_lines,
    };

    let client = Client::new(args.mastodon_instance_url, args.mastodon_access_token)?;

    loop {
        for (notif_id, post_id, (username, url), content) in client.get_notifications().await? {
            log::info!("Processing post {post_id} from {username} ({url})");

            // look for valid HTML
            match parse_html(&content, &parse_config) {
                Ok(source) => {
                    log::debug!("HTML OK");

                    // first of all, let's check that the account is not hammering us
                    if user_rate_is_ok(
                        &history,
                        &username,
                        Duration::from_secs(args.min_wait_interval as u64),
                        args.max_requests_hour,
                    ) {
                        match run_job(&args.rom, source.iter_lines(), args.native, &args.args) {
                            Ok(video_path) => {
                                // this means the encoding went well, let's log the size of the file and get to posting it
                                let out_file = video_path.as_ref().join("out.mp4");
                                log::info!(
                                    "File is {} KB long",
                                    std::fs::metadata(&out_file)?.size() / 1024
                                );
                                if !args.do_not_post {
                                    log::info!("Posting to mastodon, replying to {post_id}");

                                    // post on Mastodon
                                    let url =
                                        client.post_result(&username, &post_id, &out_file).await?;
                                    client.clear_notification(&notif_id).await?;
                                    log::info!("All done! {url}");
                                    history.log(Utc::now(), &username, &url)?;
                                } else {
                                    log::info!("All done! (wink wink!)");
                                }

                                // skip to next notification
                                continue;
                            }
                            Err(e) => {
                                log::error!("Failed to run job for post {post_id}: {e}");
                            }
                        }
                    } else {
                        log::warn!("Request from {username} ignored due to rate limit");
                    }
                }
                Err(e) => {
                    log::warn!("Ignored {post_id}: {e}");
                }
            };

            if !args.do_not_post {
                client.clear_notification(&notif_id).await?;
            }
        }

        time::sleep(Duration::from_secs(60)).await;
    }
}

async fn exec_cmd(
    rom: impl AsRef<Path>,
    input: Option<impl AsRef<Path>>,
    output: impl AsRef<Path>,
    parse_config: &ParseConfig<'_>,
    native: bool,
    args: &Vec<String>,
) -> Result<()> {
    let mut input: Box<dyn Read> = match input {
        Some(f) => Box::new(File::open(f.as_ref())?),
        None => Box::new(stdin()),
    };

    let mut text = Vec::new();

    input.read_to_end(&mut text)?;

    let source = parse_orca_code(&String::from_utf8(text)?, parse_config)?;

    let dir = run_job(rom, source.iter_lines(), native, args)?;

    fs::copy(dir.as_ref().join("out.mp4"), output)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let env = env_logger::Env::default()
        .filter_or("LOG", "info")
        .write_style_or("LOG", "always");
    env_logger::init_from_env(env);

    let args = cli::Cli::parse();

    match args.command {
        SubCommands::Run(args) => run_cmd(args).await?,
        SubCommands::Exec {
            rom,
            output,
            input,
            max_line_length,
            max_num_lines,
            native,
            args,
        } => {
            let parse_config = ParseConfig {
                max_line_length,
                max_num_lines,
                ..Default::default()
            };
            exec_cmd(rom, input, output, &parse_config, native, &args).await?
        }
    }

    Ok(())
}
