use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_http_client::typed::*;
use bevy_http_client::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct IpInfo {
    pub ip: String,
}

fn main() {
    let mut binding = App::new();
    let app =
        binding
            .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
                Duration::from_secs_f64(1.0 / 60.0),
            )))
            .add_plugins(HttpClientPlugin)
            .init_resource::<ApiTimer>()
            .add_systems(Update, (send_request, handle_response));
    let app = bevy_http_client::register_request_type::<IpInfo>(app);
    app.run();
}

#[derive(Resource, Deref, DerefMut)]
pub struct ApiTimer(pub Timer);

impl Default for ApiTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(3.0, TimerMode::Repeating))
    }
}

fn send_request(mut commands: Commands, time: Res<Time>, mut timer: ResMut<ApiTimer>) {
    timer.tick(time.delta());

    if timer.just_finished() {
        let req = ehttp::Request::get("https://api.ipify.org?format=json");
        commands.spawn((HttpRequest(req), RequestType::<IpInfo>::default()));
    }
}

fn handle_response(mut commands: Commands, responses: Query<(Entity, &TypedResponse<IpInfo>)>) {
    for (entity, response) in responses.iter() {
        match response.parse() {
            Some(v) => {
                println!("response: {:?}", v);
            }
            None => {
                println!("Failed to parse: {:?}", response.result);
            }
        }
        commands.entity(entity).despawn_recursive();
    }
}
