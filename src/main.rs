use clap::{self, Clap};
use regex::Regex;
use reqwest::{Client, Response, Result as HttpResult, Url};
use serde::Serialize;
use std::{
    convert::From,
    io::{self, BufRead, Result as IOResult},
};
use tokio;

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

#[derive(Serialize, Debug)]
struct TrapMessage {
    remote_hostname: String,
    transport_address: TransportAddress,
    varbinds: Vec<VarBind>,
}

#[derive(Serialize, Debug)]
struct TransportAddress {
    protocol: String,
    remote_address: String,
    local_address: String,
}

#[derive(Serialize, Debug)]
struct VarBind {
    oid: String,
    value: String,
}

impl TransportAddress {
    fn new() -> Self {
        TransportAddress {
            protocol: String::new(),
            remote_address: String::new(),
            local_address: String::new(),
        }
    }
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

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let trap = read_trap().expect("Unable to read trap message");
    // TODO: asynchronously send traps, not consecutively
    for address in opts.address {
        send_trap(&address, &trap)
            .await
            .expect(&format!("Unable to forward trap message to {}", address));
    }
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

fn read_trap() -> IOResult<TrapMessage> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let remote_hostname = match lines.next() {
        Some(value) => String::from(value?.trim()),
        None => String::new(),
    };
    let transport_address = match lines.next() {
        Some(value) => TransportAddress::from(value?.trim()),
        None => TransportAddress::new(),
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
    })
}

async fn send_trap(address: &str, trap: &TrapMessage) -> HttpResult<Response> {
    let client = Client::new();
    let res = client.post(address).json(trap).send().await?;
    res.error_for_status()
}
