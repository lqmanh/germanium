use clap::{self, Clap};
use reqwest::{Client, Url};
use serde::Serialize;
use std::{
    error::Error,
    io::{self, BufRead, Result as IOResult},
};
use tokio;

#[derive(Clap)]
#[clap(version = clap::crate_version!(), author = clap::crate_authors!())]
struct Opts {
    address: String,
}

#[derive(Serialize, Debug)]
struct TrapMessage {
    hostname: String,
    address: String,
    varbinds: Vec<VarBind>,
}

#[derive(Serialize, Debug)]
struct VarBind {
    oid: String,
    value: String,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let address = opts.address;

    match Url::parse(&address) {
        Ok(url) => match url.scheme() {
            "http" | "https" => (),
            _ => panic!("Invalid HTTP URL: {}", address),
        },
        Err(_) => panic!("Invalid URL: {}", address),
    }

    let trap = get_trap_message().expect("Unable to read trap message");

    send_trap(&address, &trap)
        .await
        .expect("Unable to forward trap message");
}

fn get_trap_message() -> IOResult<TrapMessage> {
    let stdin = io::stdin();
    let mut hostname = String::new();
    let mut address = String::new();
    let mut varbinds = Vec::new();

    stdin.read_line(&mut hostname)?;
    stdin.read_line(&mut address)?;
    for line in stdin.lock().lines() {
        if let [oid, value] = line?.split_whitespace().take(2).collect::<Vec<&str>>()[..] {
            varbinds.push(VarBind {
                oid: String::from(oid),
                value: String::from(value),
            });
        };
    }

    Ok(TrapMessage {
        hostname,
        address,
        varbinds,
    })
}

async fn send_trap(address: &str, trap: &TrapMessage) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let res = client.post(address).json(trap).send().await?;
    match res.error_for_status() {
        Ok(_) => Ok(()),
        Err(err) => Err(Box::from(err)),
    }
}
