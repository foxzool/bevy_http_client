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

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, HttpClientPlugin))
        .add_systems(Update, handle_response)
        .add_systems(
            Update,
            send_request.run_if(on_timer(std::time::Duration::from_secs(1))),
        )
        .run()
}

fn send_request(mut commands: Commands) {
    let req = ehttp::Request::get("https://api.ipify.org?format=json");
    commands.spawn(HttpRequest(req));
}

fn handle_response(mut commands: Commands, responses: Query<(Entity, &HttpResponse)>) {
    for (entity, response) in responses.iter() {
        println!("response: {:?}", response.text());
        commands.entity(entity).despawn_recursive();
    }
}
```


## Supported Versions

| bevy | bevy_cronjob |
|------|--------------|
| 0.13 | 0.4          |
| 0.12 | 0.3          |
| 0.11 | 0.1          |

## License

Dual-licensed under either

- MIT
- Apache 2.0
