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

fn send_request(mut ev_request: EventWriter<TypedRequest<IpInfo>>) {
    ev_request.write(
        HttpClient::new()
            .get("https://api.ipify.org?format=json")
            .with_type::<IpInfo>(),
    );
}

/// consume TypedResponse<IpInfo> events
fn handle_response(mut events: ResMut<Events<TypedResponse<IpInfo>>>) {
    for response in events.drain() {
        let response: IpInfo = response.into_inner();
        println!("ip info: {:?}", response);
    }
}

fn handle_error(mut ev_error: EventReader<TypedResponseError<IpInfo>>) {
    for error in ev_error.read() {
        println!("Error retrieving IP: {}", error.err);
    }
}
