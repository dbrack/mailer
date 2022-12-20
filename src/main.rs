use chrono::Local;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::{env, io, thread, time};

use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

static TIME_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

#[derive(Deserialize, Debug)]
struct Config {
    smtp_server: String,
    from: String,
    to: String,
    subject: String,
    interval: u64,
    messages: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Please provide the path to the config.json as argument",
        )));
    }

    let smtp_username =
        env::var("GMAIL_USERNAME").expect("GMAIL_USERNAME environment variable not set");
    let smtp_password =
        env::var("GMAIL_PASSWORD").expect("GMAIL_PASSWORD environment variable not set");

    let file = File::open(args[1].as_str()).unwrap();
    let config: Config = serde_json::from_reader(file).unwrap();

    println!(
        "{} Daemon started successfully",
        Local::now().format(TIME_FORMAT).to_string()
    );

    loop {
        // by default,trigger at least once every 60 minutes
        let sleep_time = rand::thread_rng().gen_range(1..config.interval);
        println!(
            "{} Sleeping for {} seconds",
            Local::now().format(TIME_FORMAT).to_string(),
            sleep_time.to_string()
        );
        let sleep_duration = time::Duration::from_secs(sleep_time);
        thread::sleep(sleep_duration);
        send_mail(
            &config.smtp_server,
            &smtp_username,
            &smtp_password,
            &config.from,
            &config.to,
            &config.subject,
            &config.messages,
        );
    }
}

fn send_mail(
    smtp_server: &String,
    smtp_username: &String,
    smtp_password: &String,
    from: &String,
    to: &String,
    subject: &String,
    messages: &Vec<String>,
) {
    println!(
        "{} Preparing to send email",
        Local::now().format(TIME_FORMAT).to_string()
    );

    let random_msg_body = messages.choose(&mut thread_rng()).unwrap();

    let email = Message::builder()
        .from(from.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .body(String::from(random_msg_body.to_string()))
        .unwrap();

    let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());

    let mailer = SmtpTransport::relay(smtp_server)
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!(
            "{} Email sent successfully! Message body: {}",
            Local::now().format(TIME_FORMAT).to_string(),
            random_msg_body
        ),
        Err(e) => panic!("Could not send email: {:?}", e),
    }
}
