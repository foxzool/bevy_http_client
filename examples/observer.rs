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
    mut ev_request: MessageWriter<HttpRequest>,
) {
    let requests: Vec<HttpRequest> = clients
        .iter()
        .filter_map(|c| match c.clone().try_build() {
            Ok(request) => Some(request),
            Err(e) => {
                eprintln!("Failed to build request: {}", e);
                None
            }
        })
        .collect();

    ev_request.write_batch(requests);
}

fn handle_response(response: On<HttpObserved<HttpResponse>>) {
    println!("response: {:?}", response.event().inner().text());
}

fn handle_error(error: On<HttpObserved<HttpResponseError>>) {
    println!("Error retrieving IP: {}", error.event().inner().err);
}
