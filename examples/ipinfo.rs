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
