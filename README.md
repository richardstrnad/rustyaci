<!-- [![Crates.io](https://img.shields.io/crates/v/rustyaci.svg)](https://crates.io/crates/rustyaci) -->
<!-- [![Documentation](https://docs.rs/rustyaci/badge.svg)](https://docs.rs/rustyaci/) -->
[![Codecov](https://codecov.io/github/richardstrnad/rustyaci/coverage.svg?branch=main)](https://codecov.io/gh/richardstrnad/rustyaci)
[![Dependency status](https://deps.rs/repo/github/richardstrnad/rustyaci/status.svg)](https://deps.rs/repo/github/richardstrnad/rustyaci)

# RustyACI - A Rust SDK for Cisco ACI
This crate provides an easy way to interact with the Cisco ACI API.
Like [aciClient](https://github.com/richardstrnad/aciClient) but in Rust!

## Usage
Provide a `.env` file that defines the required ENV vars (APIC_HOST, APIC_USERNAME, APIC_PASSWORD).
Or modify the script to provide this information in another way.
```
use dotenvy::dotenv;
use rustyaci::ACI;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let server = std::env::var("APIC_HOST").unwrap();
    let username = std::env::var("APIC_USERNAME").unwrap();
    let password = std::env::var("APIC_PASSWORD").unwrap();

    let aci = ACI::new(server, username, password).await;

    if let Ok(epgs) = aci.get_json(String::from("class/fvAEPg.json")).await {
        for epg in epgs.as_array().unwrap() {
            println!(
                "EPG: {:?}",
                epg["fvAEPg"]["attributes"]["name"].as_str().unwrap()
            );
        }
    }
}
```
