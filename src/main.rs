use std::{num::ParseIntError, thread::sleep, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use clap::{Parser, Subcommand};
use embed::embed_file;
use rodio::{Decoder};
use nix::unistd::{fork, ForkResult};

// Embed the alarm audio into the binary
embed_file!{"./alarm.mp3", alarm}


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long)]
    /// Specifies that the program should not run as a daemon
    no_daemon: bool
}

#[derive(Subcommand)]
enum Commands {
    /// Waits a certain amount of time and then execute
    FromNow {
        /// Time to wait in HH:MM:SS format
        to_wait: String
    },
    /// Executes at the given time
    AtTime {
        /// Time to execute in HH:MM:SS format
        when: String
    }
}

fn wait_and_play(target_time: Instant) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream_handle = rodio::DeviceSinkBuilder::open_default_sink()
        .expect("open default audio stream");
    stream_handle.log_on_drop(false);
    let player = rodio::Player::connect_new(&stream_handle.mixer());
    let source = Decoder::try_from(std::io::Cursor::new(&alarm))?;
    sleep(target_time - Instant::now());
    player.append(source);
    player.sleep_until_end();
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let target_time: Instant;
    match args.command {
        Commands::FromNow { to_wait: x } => {
            let mut target = Instant::now();
            let mut time_to_wait = x.trim().split(':').rev().map(|x| x.parse::<u64>()).collect::<Vec<Result<u64, ParseIntError>>>().into_iter();
            target += Duration::from_secs(time_to_wait.next().unwrap()?);
            target += Duration::from_mins(time_to_wait.next().unwrap_or(Ok(0))?);
            target += Duration::from_hours(time_to_wait.next().unwrap_or(Ok(0))?);
            target_time = target;
        }
        Commands::AtTime { when: x } => {
            let secs_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let time_to_start_of_day = secs_since_epoch % 86400;
            let hours_mins_secs = x.trim().split(':').collect::<Vec<&str>>();
            if hours_mins_secs.len() != 3 || hours_mins_secs.iter().any(|x| x.len() != 2) {
                return Err("Must be in HH:MM:SS format".into());
            }
            let hour: u64 = hours_mins_secs[0].parse()?;
            let minute: u64 = hours_mins_secs[1].parse()?;
            let second: u64 = hours_mins_secs[2].parse()?;
            let mut target = Instant::now();
            target -= Duration::from_secs(time_to_start_of_day) + Duration::from_hours(1);
            target += Duration::from_hours(hour) + Duration::from_mins(minute) + Duration::from_secs(second);
            target_time = target;
        }
    }
    if !args.no_daemon {
        match unsafe{fork()} {
            Ok(ForkResult::Parent{..}) => {},
            Ok(ForkResult::Child) => {
                wait_and_play(target_time)?;
            },
            Err(y) => {
                return Err(Box::new(y));
            }
        }
    }
    else {
        wait_and_play(target_time)?;
    }
    Ok(())
}
