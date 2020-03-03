use clap::{self, Clap};
use reqwest::{Client, Response, Result as HttpResult, Url};
use serde::Serialize;
use std::io::{self, BufRead, Result as IOResult};
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
    let hostname = match lines.next() {
        Some(hostname) => hostname?.trim().to_string(),
        None => String::new(),
    };
    let address = match lines.next() {
        Some(address) => address?.trim().to_string(),
        None => String::new(),
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
        hostname,
        address,
        varbinds,
    })
}

async fn send_trap(address: &str, trap: &TrapMessage) -> HttpResult<Response> {
    let client = Client::new();
    let res = client.post(address).json(trap).send().await?;
    res.error_for_status()
}
