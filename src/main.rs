use std::{thread::sleep, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use clap::{Parser, Subcommand};
use embed::embed_file;
use rodio::{Decoder};
use nix::{unistd::{fork, ForkResult}};

// Embed the alarm audio into the binary
embed_file!{"./alarm.mp3", alarm}


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    match unsafe{fork()} {
        Ok(ForkResult::Parent { .. }) => {},
        Ok(ForkResult::Child) => {
            let target_time: Instant;
            let mut stream_handle = rodio::OutputStreamBuilder::open_default_stream()
                .expect("open default audio stream");
            stream_handle.log_on_drop(false);
            let sink = rodio::Sink::connect_new(&stream_handle.mixer());
            let source = Decoder::try_from(std::io::Cursor::new(&alarm))?;
            match args.command {
                Commands::FromNow { to_wait: x } => {
                    let mut target = Instant::now();
                    let hours_mins_secs = x.trim().split(':').collect::<Vec<&str>>();
                    if hours_mins_secs.len() != 3 || hours_mins_secs.iter().any(|x| x.len() != 2) {
                        return Err("Must be in HH:MM:SS format".into());
                    }
                    let hour: u64 = hours_mins_secs[0].parse()?;
                    let minute: u64 = hours_mins_secs[1].parse()?;
                    let second: u64 = hours_mins_secs[2].parse()?;
                    target += Duration::from_hours(hour) + Duration::from_mins(minute) + Duration::from_secs(second);
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
            while Instant::now() < target_time {
                sleep(Duration::from_secs(1));
            }
            sink.append(source);
            sink.sleep_until_end();
        },
        Err(_) => {}
    }
    Ok(())
}
