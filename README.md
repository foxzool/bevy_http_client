# bevy_http_client

[![Crates.io](https://img.shields.io/crates/v/bevy_http_client)](https://crates.io/crates/bevy_http_client)
[![crates.io](https://img.shields.io/crates/d/bevy_http_client)](https://crates.io/crates/bevy_cronjob)
[![Documentation](https://docs.rs/bevy_http_client/badge.svg)](https://docs.rs/bevy_http_client)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Seldom-SE/seldom_pixel#license)

A simple HTTP client Bevy Plugin for both native and WASM.

## Example

```rust
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_http_client::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct IpInfo {
    pub ip: String,
}

fn main() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, HttpClientPlugin))
        .add_systems(Update, handle_response)
        .add_systems(
            Update,
            send_request.run_if(on_timer(std::time::Duration::from_secs(1))),
        );
    app.register_request_type::<IpInfo>();
    app.run();
}

fn send_request(mut ev_request: EventWriter<TypedRequest<IpInfo>>) {
    ev_request.send(
        HttpClient::new()
            .get("https://api.ipify.org?format=json")
            .with_type::<IpInfo>(),
    );
}

fn handle_response(mut ev_response: EventReader<TypedResponse<IpInfo>>) {
    for response in ev_response.read() {
        println!("ip: {}", response.ip);
    }
}
```

## Supported Versions

| bevy | bevy_http_client |
|------|------------------|
| 0.13 | 0.4, 0,5         |
| 0.12 | 0.3              |
| 0.11 | 0.1              |

## License

Dual-licensed under either

- MIT
- Apache 2.0
