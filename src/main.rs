use std::env;
use std::time::{Duration, Instant};

use futures::prelude::*;
use irc::client::prelude::*;

mod bassline;

const MIN_RECONNECT_DELAY_SECS: u64 = 5;
const MAX_RECONNECT_DELAY_SECS: u64 = 300;
const STABLE_SESSION_SECS: u64 = 60;

#[tokio::main]
async fn main() -> irc::error::Result<()> {
    let config_filename = env::args()
        .skip(1)
        .next()
        .expect("Configuration file must be first argument!");

    let config = Config::load(config_filename)?;

    let db_filename = config.options.get("db_filename")
        .expect("In configuration file [options] section must have key 'db_filename', pointing to the database file");

    let db_connection = rusqlite::Connection::open(&db_filename).unwrap();

    let mut bassline = bassline::Bassline::new(&db_connection).unwrap();

    let mut reconnect_delay = MIN_RECONNECT_DELAY_SECS;

    loop {
        let session_started = Instant::now();

        match run_session(config.clone(), &mut bassline).await {
            Ok(()) => println!("Server closed the connection"),
            Err(error) => println!("Connection failed: {:?}", error),
        }

        if session_started.elapsed() >= Duration::from_secs(STABLE_SESSION_SECS) {
            reconnect_delay = MIN_RECONNECT_DELAY_SECS;
        }

        println!("Reconnecting in {}s", reconnect_delay);
        tokio::time::delay_for(Duration::from_secs(reconnect_delay)).await;

        reconnect_delay = (reconnect_delay * 2).min(MAX_RECONNECT_DELAY_SECS);
    }
}

async fn run_session(
    config: Config,
    bassline: &mut bassline::Bassline<'_>,
) -> irc::error::Result<()> {
    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;
    let sender = client.sender();

    println!("Connected!");

    while let Some(message) = stream.next().await.transpose()? {
        match message.command {
            Command::PRIVMSG(ref target, ref msg) => {
                let nickname = match message.source_nickname() {
                    Some(nickname) => nickname,
                    None => continue,
                };

                if msg.contains(client.current_nickname()) {
                    sender.send_privmsg(target, "Ko tau purn grib?!")?;
                }

                if msg.starts_with("!read") {
                    sender.send_privmsg(
                        target,
                        bassline.respond_to_read(nickname).unwrap_or_else(|error| {
                            println!("read! failed: {}", error);
                            "Akvai, shodien kaukaa nesanaak.".to_string()
                        }),
                    )?;
                }
                if msg.starts_with("!write ") {
                    let source = message.response_target().unwrap();
                    sender.send_privmsg(
                        nickname,
                        bassline
                            .respond_to_write(nickname, &msg[7..], source)
                            .unwrap_or_else(|error| {
                                println!("write! failed: {}", error);
                                "Nesanaaca pierakstiit, pameegjini veelaak?".to_string()
                            }),
                    )?;
                }
            }
            _ => (),
        }
    }

    Ok(())
}
