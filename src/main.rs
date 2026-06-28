use std::{num::ParseIntError, thread::sleep, time::{Duration, SystemTime, UNIX_EPOCH}};
use clap::{Parser, Subcommand};
use rodio::{Decoder};
use nix::unistd::{fork, ForkResult};

// Embed the alarm audio into the binary
const ALARM: &[u8] = include_bytes!("./alarm.mp3");


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

fn wait_and_play(time_to_wait: Duration) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream_handle = rodio::DeviceSinkBuilder::open_default_sink()
        .expect("open default audio stream");
    stream_handle.log_on_drop(false);
    let player = rodio::Player::connect_new(&stream_handle.mixer());
    let source = Decoder::try_from(std::io::Cursor::new(&ALARM))?;
    sleep(time_to_wait);
    player.append(source);
    player.sleep_until_end();
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut to_sleep_for: Duration = Duration::new(0, 0);
    match args.command {
        Commands::FromNow { to_wait: x } => {
            let mut time_to_wait = x.trim().split(':').rev().map(|x| x.parse::<u64>()).collect::<Vec<Result<u64, ParseIntError>>>().into_iter();
            to_sleep_for += Duration::from_secs(time_to_wait.next().unwrap()?);
            to_sleep_for += Duration::from_mins(time_to_wait.next().unwrap_or(Ok(0))?);
            to_sleep_for += Duration::from_hours(time_to_wait.next().unwrap_or(Ok(0))?);
        }
        Commands::AtTime { when: x } => {
            let millis_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            let time_to_start_of_day: u64 = (millis_since_epoch % 86400000).try_into().expect("The amount of milliseconds since the start of day exceeds u64::MAX");
            let hours_mins_secs = x.trim().split(':').collect::<Vec<&str>>();
            if hours_mins_secs.len() != 3 || hours_mins_secs.iter().any(|x| x.len() != 2) {
                return Err("Must be in HH:MM:SS format".into());
            }
            let hour: u64 = hours_mins_secs[0].parse()?;
            let minute: u64 = hours_mins_secs[1].parse()?;
            let second: u64 = hours_mins_secs[2].parse()?;
            to_sleep_for += Duration::from_hours(hour) + Duration::from_mins(minute) + Duration::from_secs(second);
            if let Some(x) = to_sleep_for.checked_sub(Duration::from_millis(time_to_start_of_day) + Duration::from_hours(1)) {
                to_sleep_for = x;
            } else {
                return Err("The given time should not be before the current time".into());
            };
        }
    }
    if !args.no_daemon {
        match unsafe{fork()} {
            Ok(ForkResult::Parent{..}) => {},
            Ok(ForkResult::Child) => {
                wait_and_play(to_sleep_for)?;
            },
            Err(y) => {
                return Err(Box::new(y));
            }
        }
    }
    else {
        wait_and_play(to_sleep_for)?;
    }
    Ok(())
}
