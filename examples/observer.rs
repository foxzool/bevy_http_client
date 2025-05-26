use bevy::{prelude::*, time::common_conditions::on_timer};

use bevy_http_client::prelude::*;

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, HttpClientPlugin))
        .add_systems(Startup, init_request)
        .add_systems(
            Update,
            send_request.run_if(on_timer(std::time::Duration::from_secs(1))),
        )
        .run();
}

#[derive(Component)]
struct IpRequestMarker;

fn init_request(mut commands: Commands) {
    let entity = commands.spawn(IpRequestMarker).id();
    let request = HttpClient::new_with_entity(entity).get("https://api.ipify.org");
    commands
        .entity(entity)
        .insert(request)
        .observe(handle_response)
        .observe(handle_error);
}

fn send_request(
    clients: Query<&HttpClient, With<IpRequestMarker>>,
    mut ev_request: EventWriter<HttpRequest>,
) {
    let requests = clients
        .iter()
        .map(|c| c.clone().build())
        .collect::<Vec<_>>();

    ev_request.write_batch(requests);
}

fn handle_response(response: Trigger<HttpResponse>) {
    println!("response: {:?}", response.text());
}

fn handle_error(error: Trigger<HttpResponseError>) {
    println!("Error retrieving IP: {}", error.err);
}
