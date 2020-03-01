use clap::{self, Clap};
use reqwest::{Client, Url};
use std::{collections::HashMap, error::Error, process};
use tokio;

#[derive(Clap)]
#[clap(version = clap::crate_version!(), author = clap::crate_authors!())]
struct Opts {
    address: String,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let address = opts.address;

    match Url::parse(&address) {
        Ok(url) => match url.scheme() {
            "http" | "https" => (),
            _ => display_error_then_exit(&format!("Invalid HTTP URL: {}", address)),
        },
        Err(_) => display_error_then_exit(&format!("Invalid URL: {}", address)),
    }

    let mut req_body = HashMap::new();
    req_body.insert("hostname", "sample hostname");
    req_body.insert("address", "sample address");
    req_body.insert("varbinds", "sample varbinds");

    match send_request(&address, &req_body).await {
        Ok(_) => (),
        Err(_) => display_error_then_exit("Unable to forward trap message"),
    };
}

fn display_error_then_exit(message: &str) {
    println!("{}", message);
    process::exit(0);
}

async fn send_request(address: &str, body: &HashMap<&str, &str>) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let res = client.post(address).json(body).send().await?;
    match res.error_for_status() {
        Ok(_) => Ok(()),
        Err(err) => Err(Box::from(err)),
    }
}
