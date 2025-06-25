use bevy::{prelude::*, time::common_conditions::on_timer};

use bevy_http_client::prelude::*;

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, HttpClientPlugin))
        .add_systems(Update, (handle_response, handle_error))
        .add_systems(
            Update,
            send_request.run_if(on_timer(std::time::Duration::from_secs(1))),
        )
        .run();
}

fn send_request(mut ev_request: EventWriter<HttpRequest>) {
    match HttpClient::new().get("https://api.ipify.org").try_build() {
        Ok(request) => {
            ev_request.write(request);
        }
        Err(e) => {
            eprintln!("Failed to build request: {}", e);
        }
    }
}

fn handle_response(mut ev_resp: EventReader<HttpResponse>) {
    for response in ev_resp.read() {
        println!("response: {:?}", response.text());
    }
}

fn handle_error(mut ev_error: EventReader<HttpResponseError>) {
    for error in ev_error.read() {
        println!("Error retrieving IP: {}", error.err);
    }
}
