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
    register_request_type::<IpInfo>(&mut app);
    app.run();
}

fn send_request(mut commands: Commands) {
    let req = ehttp::Request::get("https://api.ipify.org?format=json");
    commands.spawn(RequestBundle::<IpInfo>::new(req));
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
