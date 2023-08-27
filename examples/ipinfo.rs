use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;

use bevy_http_client::{HttpClientPlugin, HttpRequest};

fn main() {
    App::new().add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        ))),
    ).add_plugins(HttpClientPlugin).insert_resource(WeatherApiTimer(Timer::from_seconds(
        1.0,
        TimerMode::Once,
    )))
        .add_systems(Update, spawn_reqwest)
        .run()
}

#[derive(Resource, Deref, DerefMut)]
pub struct WeatherApiTimer(pub Timer);

fn spawn_reqwest(mut commands: Commands, time: Res<Time>, mut timer: ResMut<WeatherApiTimer>) {
    timer.tick(time.delta());

    if timer.just_finished() {
        let req = ehttp::Request::get("https://api.ipify.org?format=json");
        commands.spawn(HttpRequest(req));
    }
}
