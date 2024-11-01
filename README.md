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
```rust
use dotenvy::dotenv;
use rustyaci::ACI;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let server = std::env::var("APIC_HOST").unwrap();
    let username = std::env::var("APIC_USERNAME").unwrap();
    let password = std::env::var("APIC_PASSWORD").unwrap();

    let aci = match ACI::new(server, username, password).await {
        Ok(aci) => aci,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    if let Ok(epgs) = aci.get_json(String::from("class/fvAEPg.json")).await {
        for epg in epgs {
            println!(
                "EPG: {:?}",
                epg["fvAEPg"]["attributes"]["name"].as_str().unwrap()
            );
        }
    }
}
```

## Macro to create ACI Structs
The crate provides a Macro `aci_struct` that allows you to generate a Flatted ACI structure. It assumes the following JSON structure.
```
"fvTenant": {
    "attributes": {
        "name": "TenantName"
    }
}
```

This example would work for the ACI Tenant Object.

```rust
let json_data = r#"
{
    "fvTenant": {
        "attributes": {
            "name": "TenantName",
            "bytes": 500
        }
    }
}"#;
let tenant: Tenant = serde_json::from_str(json_data).expect("Failed to deserialize");
```

Many objects have the same Structure for the nested `attributes`. This macro saves some boilerplate. These objects can be used with the `get` function to get the Struct from the ACI API directly.
```rust
use dotenvy::dotenv;
use rustyaci::ACI;

rustyaci::aci_struct!(
    Tenant,
    "fvTenant",
    {
        name: String,
        dn: String,
    }
);

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let server = std::env::var("APIC_HOST").unwrap();
    let username = std::env::var("APIC_USERNAME").unwrap();
    let password = std::env::var("APIC_PASSWORD").unwrap();

    let aci = match ACI::new(server, username, password).await {
        Ok(aci) => aci,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    if let Ok(tenants) = aci.get::<Tenant>(String::from("class/fvTenant.json")).await {
        for tenant in tenants {
            println!("Tenant: {:?}", tenant);
        }
    }
}
```
