# bevy_http_client

[![Crates.io](https://img.shields.io/crates/v/bevy_http_client)](https://crates.io/crates/bevy_http_client)
[![crates.io](https://img.shields.io/crates/d/bevy_http_client)](https://crates.io/crates/bevy_cronjob)
[![Documentation](https://docs.rs/bevy_http_client/badge.svg)](https://docs.rs/bevy_http_client)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Seldom-SE/seldom_pixel#license)

A simple HTTP client for Bevy. It works on WASM and native platforms.

## Example

```rust
use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_http_client::*;

fn main() {
    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
        )
        .add_plugins(HttpClientPlugin)
        .init_resource::<ApiTimer>()
        .add_systems(Update, (send_reqwest, handle_response))
        .run()
}

#[derive(Resource, Deref, DerefMut)]
pub struct ApiTimer(pub Timer);

impl Default for ApiTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

fn send_reqwest(mut commands: Commands, time: Res<Time>, mut timer: ResMut<ApiTimer>) {
    timer.tick(time.delta());

    if timer.just_finished() {
        let req = ehttp::Request::get("https://api.ipify.org?format=json");
        commands.spawn(HttpRequest(req));
    }
}

fn handle_response(
    mut commands: Commands,
    mut responses: Query<(Entity, &HttpResponse), Without<HttpResponseError>>,
) {
    for (entity, response) in responses.iter() {
        println!("response: {:?}", response.text());
        commands.entity(entity).despawn_recursive();
    }
}

```