use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: SubCommands
}

#[derive(Debug, Subcommand)]
pub(crate) enum SubCommands {
    /// Run the bot, fetching notifications and processing them
    Run(RunArgs),

    /// Run the ROM, based on user input
    Exec {
        /// ROM to load and execute
        rom: PathBuf,

        /// Output file (MP4 video)
        output: PathBuf,

        /// Input from file
        #[clap(long, short)]
        input: Option<PathBuf>,

        /// Maximum length of lines
        #[clap(env, long, default_value_t = 64)]
        max_line_length: u8,

        /// Maximum number of lines
        #[clap(env, long, default_value_t = 64)]
        max_num_lines: u8,

        /// Use the native Uxn implementation
        #[clap(long)]
        native: bool,
            
        /// Arguments to pass into the VM
        #[arg(last = true)]
        args: Vec<String>,
    }
}

#[derive(Debug, Args)]
pub(crate) struct RunArgs {
    /// ROM to load and execute
    pub(crate) rom: PathBuf,

    /// Don't post on Mastodon
    #[clap(long)]
    pub(crate) do_not_post: bool,

    /// Minimum interval between requests from the same account (seconds)
    #[clap(env, long, default_value_t = 30)]
    pub(crate) min_wait_interval: usize,

    /// Maximum number of requests from the same account in an hour
    #[clap(env, long, default_value_t = 10)]
    pub(crate) max_requests_hour: usize,

    /// Maximum length of lines
    #[clap(env, long, default_value_t = 32)]
    pub(crate) max_line_length: u8,

    /// Maximum number of lines
    #[clap(env, long, default_value_t = 32)]
    pub(crate) max_num_lines: u8,

    /// Location of history file
    #[clap(env, long, default_value = "history.csv")]
    pub(crate) history_file: PathBuf,

    /// Tag which should be mentioned for the code to be run
    #[clap(env, long, default_value = "run")]
    pub(crate) run_tag: String,    

    /// Mastodon instance URL
    #[clap(env, long, required = true)]
    pub(crate) mastodon_instance_url: String,

    /// Mastodon access token
    #[clap(env, long, required = true)]
    pub(crate) mastodon_access_token: String,

    /// Use the native Uxn implementation
    #[clap(long)]
    pub(crate) native: bool,
        
    /// Arguments to pass into the VM
    #[arg(last = true)]
    pub(crate) args: Vec<String>,
}