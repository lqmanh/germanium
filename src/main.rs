#[cfg(test)]
mod tests;

use chrono::{DateTime, Utc};
use clap::{self, Clap};
use regex::Regex;
use reqwest::{Client, Response, Result as HttpResult, Url};
use serde::Serialize;
use std::{
    convert::From,
    io::{self, BufRead, Error as IOError, ErrorKind as IOErrorKind, Result as IOResult},
};
use tokio;

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    let stdin = io::stdin();
    let stdin_lock = stdin.lock();
    let trap = read_trap(stdin_lock).expect("Unable to read trap message");

    // TODO: asynchronously send traps, not consecutively
    for address in opts.address {
        send_trap(&address, &trap)
            .await
            .expect(&format!("Unable to forward trap message to {}", address));
    }
}

#[derive(Clap)]
#[clap(version = clap::crate_version!(), author = clap::crate_authors!())]
struct Opts {
    #[clap(
        required = true,
        multiple = true,
        validator = is_valid_address,
        help = "HTTP URL to forward trap messages to"
    )]
    address: Vec<String>,
}

#[derive(Serialize, Debug, Eq, PartialEq)]
struct TrapMessage {
    remote_hostname: String,
    transport_address: TransportAddress,
    varbinds: Vec<VarBind>,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize, Debug, Eq, PartialEq)]
struct TransportAddress {
    protocol: String,
    remote_address: String,
    local_address: String,
}
impl From<&str> for TransportAddress {
    fn from(address: &str) -> Self {
        // "UDP: [127.0.0.1]:57517->[127.0.0.1]:162"
        let re = Regex::new(r"(.+): (\[.+](:\d+)?)->(\[.+](:\d+)?)").unwrap();
        let captures = re.captures(address).unwrap();
        TransportAddress {
            protocol: String::from(&captures[1]),
            remote_address: String::from(&captures[2]),
            local_address: String::from(&captures[4]),
        }
    }
}

#[derive(Serialize, Debug, Eq, PartialEq)]
struct VarBind {
    oid: String,
    value: String,
}

fn is_valid_address(address: String) -> Result<(), String> {
    match Url::parse(&address) {
        Ok(url) => match url.scheme() {
            "http" | "https" => Ok(()),
            scheme => Err(format!("Invalid URL scheme: {}", scheme)),
        },
        Err(_) => Err(format!("Invalid URL: {}", address)),
    }
}

fn read_trap<R: BufRead>(input: R) -> IOResult<TrapMessage> {
    let mut lines = input.lines();
    let remote_hostname = match lines.next() {
        Some(value) => String::from(value?.trim()),
        None => return Err(IOError::new(IOErrorKind::InvalidData, "Malformed input")),
    };
    let transport_address = match lines.next() {
        Some(value) => TransportAddress::from(value?.trim()),
        None => return Err(IOError::new(IOErrorKind::InvalidData, "Malformed input")),
    };
    let mut varbinds = Vec::new();
    for line in lines {
        if let [oid, value] = line?.split_whitespace().take(2).collect::<Vec<&str>>()[..] {
            varbinds.push(VarBind {
                oid: String::from(oid),
                value: String::from(value),
            });
        };
    }

    Ok(TrapMessage {
        remote_hostname,
        transport_address,
        varbinds,
        timestamp: Utc::now(),
    })
}

async fn send_trap(address: &str, trap: &TrapMessage) -> HttpResult<Response> {
    let client = Client::new();
    let res = client.post(address).json(trap).send().await?;
    res.error_for_status()
}
