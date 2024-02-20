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
    app.register_request_type::<IpInfo>();
    app.run();
}

fn send_request(mut ev_request: EventWriter<TypedRequest<IpInfo>>) {
    ev_request.send(
        HttpClient::new()
            .get("https://api.ipify.org?format=json")
            .with_type::<IpInfo>(),
    );
}

fn handle_response(mut ev_response: EventReader<TypedResponse<IpInfo>>) {
    for response in ev_response.read() {
        println!("ip: {}", response.ip);
    }
}
