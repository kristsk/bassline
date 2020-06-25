use std::env;

use futures::prelude::*;
use irc::client::prelude::*;

mod bassline;

#[tokio::main]
async fn main() -> irc::error::Result<()> {
    let config_filename = env::args().skip(1).next()
        .expect("Configuration file must be first argument!");

    let config = Config::load(config_filename)?;

    let db_filename = config.options.get("db_filename")
        .expect("In configuration file [options] section must have key 'db_filename', pointing to the database file");

    let db_connection = rusqlite::Connection::open(&db_filename).unwrap();

    let mut bassline = bassline::Bassline::new(&db_connection).unwrap();

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;
    let sender = client.sender();

    while let Some(message) = stream.next().await.transpose()? {
        match message.command {
            Command::PRIVMSG(ref target, ref msg) => {
                let nickname = message.source_nickname().unwrap();

                if msg.contains(client.current_nickname()) {
                    sender.send_privmsg(target, "Ko tau purn grib?!")?;
                }

                if msg == "!read" {
                    sender.send_privmsg(
                        target,
                        match bassline.respond_to_read(nickname) {
                            Ok(response) => response,
                            Err(error) => {
                                println!("read! failed: {}", error);
                                "Akvai, shodien kaukaa nesanaak.".to_string()
                            }
                        },
                    ).unwrap();
                }
                if msg.starts_with("!write ") {
                    let source = message.response_target().unwrap();
                    sender.send_privmsg(
                        nickname,
                        match bassline.respond_to_write(nickname, &msg[7..], source) {
                            Ok(response) => response,
                            Err(error) => {
                                println!("write! failed: {}", error);
                                "Nesanaaca pierakstiit, pameegjini veelaak?".to_string()
                            }
                        },
                    ).unwrap();
                }
            }
            _ => (),
        }
    }

    Ok(())
}