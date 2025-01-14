# Orcabot

This is a Mastodon bot which runs [Orca](https://100r.co/site/orca.html
) source code and produces a video.

Example: https://fedi.turbofish.cc/@ubik/113822982174235813


It uses the [uxn](https://100r.co/site/uxn.html) [version of Orca](https://git.sr.ht/~rabbits/orca-toy), which is emulated thanks to the great [raven](https://github.com/mkeeter/raven/) emulator. Any ROM can be used, which means that this project could actually be repurposed to execute any other uxn ROM.

The bot will respond to mentions. The first line of the message must also include a "run tag" (defaults to `#run`). The rest should be Orca code. Lines should have the same length. Maximum dimensions for the grid can be set. `= (instrument, octave, note)` can be used to play sounds.

Orca is a two-dimensional esoteric programming by Hundred Rabbits. Learn more about Orca on their site:
https://100r.co/site/orca.html


## Usage

`orca-bot [OPTION] <ROM> [ARGS]`

The `ROM` parameter is the path to the Orca Uxn ROM file. Additional arguments (`ARGS`) can be passed directly to the Uxn VM (not very useful in Orca).

### Options

Most options can be specified both in the command line and through environment variables (in parenthesis).

 * `--mastodon-instance-url` (`MASTODON_INSTANCE_URL`) (required) - URL of the Mastodon instance
 * `--mastodon-access-token` (`MASTODON_ACCESS_TOKEN`) (required) - Mastodon access token for the bot account
 * `--min-wait-interval=<SECONDS>` (`MIN_WAIT_INTERVAL`) - minimum time an account should wait before requesting something from the bot again (defaults to `30`)
 * `--max-requests-hour=<N>` (`MAX_REQUESTS_HOUR`) - maximum number of requests from the same account in an hour (defaults to `10`)
 * `--max-line-length=<LEN>` (`MAX_LINE_LENGTH`) - maximum length of Orca source code code lines. Longer lines will be ignored (defaults to `16`)
 * `--max-num-lines=<LEN>` (`MAX_NUM_LINES`) - maximum number of lines of Orca source code. All lines beyond that will be ignored (defaults to `16`)
 * `--history-file=<PATH>` (`HISTORY_FILE`) - path to the CSV file where the history of processed posts is kept. Has to be writable (defaults to `history.csv`)
 * `--run-tag=<TAG>` (`RUN_TAG`) - name of #tag that the bot will look for in the first line, in order to interpret the rest of the post as code (defaults to `run`)
 * `--native` - run the emulator in native (ASM) mode. Raven only supports it in AARCH64.
 * `--do-not-post` - do not actually post anything on Mastodon (good for testing)

### Running in a Container

A Dockerfile is provided, which takes a `user` build arg. You can use it e.g. like `docker build . --build-arg=<UID>`.