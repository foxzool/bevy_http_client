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
        .add_systems(Update, (send_request, handle_response))
        .run()
}

#[derive(Resource, Deref, DerefMut)]
pub struct ApiTimer(pub Timer);

impl Default for ApiTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

fn send_request(mut commands: Commands, time: Res<Time>, mut timer: ResMut<ApiTimer>) {
    timer.tick(time.delta());

    if timer.just_finished() {
        let req = ehttp::Request::get("https://api.ipify.org?format=json");
        commands.spawn(HttpRequest(req));
    }
}

fn handle_response(mut commands: Commands, responses: Query<(Entity, &HttpResponse)>) {
    for (entity, response) in responses.iter() {
        println!("response: {:?}", response.text());
        commands.entity(entity).despawn_recursive();
    }
}
