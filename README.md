# bevy_http_client

[![CI](https://github.com/foxzool/bevy_http_client/workflows/CI/badge.svg)](https://github.com/foxzool/bevy_http_client/actions)
[![Crates.io](https://img.shields.io/crates/v/bevy_http_client)](https://crates.io/crates/bevy_http_client)
[![Downloads](https://img.shields.io/crates/d/bevy_http_client)](https://crates.io/crates/bevy_http_client)
[![Documentation](https://docs.rs/bevy_http_client/badge.svg)](https://docs.rs/bevy_http_client)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Seldom-SE/seldom_pixel#license)

A simple HTTP client Bevy Plugin for both native and WASM.

## Example

```rust, no_run
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
        .add_systems(Update, (handle_response, handle_error))
        .add_systems(
            Update,
            send_request.run_if(on_timer(std::time::Duration::from_secs(1))),
        );
    app.register_request_type::<IpInfo>();
    app.run();
}

fn send_request(mut ev_request: MessageWriter<TypedRequest<IpInfo>>) {
    if let Ok(request) = HttpClient::new()
        .get("https://api.ipify.org?format=json")
        .try_with_type::<IpInfo>()
    {
        ev_request.write(request);
    }
}

/// consume TypedResponse<IpInfo> events
fn handle_response(mut events: ResMut<Messages<TypedResponse<IpInfo>>>) {
    for response in events.drain() {
        let response: IpInfo = response.into_inner();
        println!("ip info: {:?}", response);
    }
}

fn handle_error(mut ev_error: MessageReader<TypedResponseError<IpInfo>>) {
    for error in ev_error.read() {
        println!("Error retrieving IP: {}", error.err);
    }
}

```

## Supported Versions

| bevy | bevy_http_client |
|------|------------------|
| 0.17 | 0.9.0            |
| 0.16 | 0.8.3            |
| 0.15 | 0.7              |
| 0.14 | 0.6              |
| 0.13 | 0.4, 0,5         |
| 0.12 | 0.3              |
| 0.11 | 0.1              |

## License

Dual-licensed under either:

- [`MIT`](LICENSE-MIT): [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT)
- [`Apache 2.0`](LICENSE-APACHE): [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0)

At your option. This means that when using this crate in your game, you may choose which license to use.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dually licensed as above, without any additional terms or conditions.
