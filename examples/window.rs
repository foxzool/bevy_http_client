use bevy::{
    color::palettes::css::{AQUA, LIME},
    prelude::*,
    time::common_conditions::on_timer,
};
use bevy_http_client::{
    HttpClient, HttpClientPlugin, HttpRequest, HttpResponse, HttpResponseError,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.4, 0.4, 0.4)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Wasm http request".to_string(),
                // Bind to canvas included in `index.html`
                canvas: Some("#bevy".to_owned()),
                // Tells wasm not to override default event handling, like F5 and Ctrl+R
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(HttpClientPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            send_request.run_if(on_timer(std::time::Duration::from_secs(2))),
        )
        .add_systems(Update, (handle_response, handle_error))
        .run();
}

#[derive(Component)]
struct ResponseText;

#[derive(Component)]
struct ResponseIP;

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((Camera2d, IsDefaultUiCamera));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(5.0)),
                display: Display::Grid,
                ..default()
            },
            ZIndex(i32::MAX),
            BackgroundColor(Color::BLACK.with_alpha(0.75)),
        ))
        .with_children(|parent| {
            let text_font = TextFont {
                font_size: 40.,
                ..default()
            };
            parent.spawn(Node::default()).with_children(|parent| {
                parent.spawn((
                    Text::new("Status: "),
                    TextColor(LIME.into()),
                    text_font.clone(),
                ));
                parent.spawn((
                    Text::new(""),
                    TextColor(AQUA.into()),
                    text_font.clone(),
                    ResponseText,
                ));
            });
            parent.spawn(Node::default()).with_children(|parent| {
                parent.spawn((Text::new("Ip: "), TextColor(LIME.into()), text_font.clone()));
                parent.spawn((
                    Text::new(""),
                    TextColor(AQUA.into()),
                    text_font.clone(),
                    ResponseIP,
                ));
            });
        });
}

fn send_request(
    mut ev_request: MessageWriter<HttpRequest>,
    mut status_query: Query<&mut Text, (With<ResponseText>, Without<ResponseIP>)>,
    mut ip_query: Query<&mut Text, (With<ResponseIP>, Without<ResponseText>)>,
) {
    if let Ok(mut text) = status_query.single_mut() {
        text.0 = "Requesting ".to_string();
    }
    if let Ok(mut ip) = ip_query.single_mut() {
        ip.0 = "".to_string();
    }

    match HttpClient::new().get("https://api.ipify.org").try_build() {
        Ok(request) => {
            ev_request.write(request);
        }
        Err(e) => {
            eprintln!("Failed to build request: {}", e);
            if let Ok(mut text) = status_query.single_mut() {
                text.0 = "Build Error".to_string();
            }
        }
    }
}

fn handle_response(
    mut ev_resp: MessageReader<HttpResponse>,
    mut status_query: Query<&mut Text, (With<ResponseText>, Without<ResponseIP>)>,
    mut ip_query: Query<&mut Text, (With<ResponseIP>, Without<ResponseText>)>,
) {
    for response in ev_resp.read() {
        if let Ok(mut text) = status_query.single_mut() {
            text.0 = "Got ".to_string();
        }
        if let Ok(mut ip) = ip_query.single_mut() {
            ip.0 = response.text().unwrap_or_default().to_string();
        }
    }
}
fn handle_error(mut ev_error: MessageReader<HttpResponseError>) {
    for error in ev_error.read() {
        println!("Error retrieving IP: {}", error.err);
    }
}
