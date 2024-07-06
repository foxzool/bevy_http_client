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
        .insert_resource(Msaa::Off)
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

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((Camera2dBundle::default(), IsDefaultUiCamera));

    let text_section = move |color: Srgba, value: &str| {
        TextSection::new(
            value,
            TextStyle {
                font_size: 40.0,
                color: color.into(),
                ..default()
            },
        )
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            z_index: ZIndex::Global(i32::MAX),
            background_color: Color::BLACK.with_alpha(0.75).into(),
            ..default()
        })
        .with_children(|c| {
            c.spawn((
                TextBundle::from_sections([
                    text_section(LIME, "Status: "),
                    text_section(AQUA, ""),
                    text_section(LIME, "\nIp: "),
                    text_section(AQUA, ""),
                ]),
                ResponseText,
            ));
        });
}

fn send_request(
    mut ev_request: EventWriter<HttpRequest>,
    mut query: Query<&mut Text, With<ResponseText>>,
) {
    let mut text = query.single_mut();
    text.sections[1].value = "Requesting".to_string();
    text.sections[3].value = "".to_string();
    let request = HttpClient::new().get("https://api.ipify.org").build();
    ev_request.send(request);
}

fn handle_response(
    mut ev_resp: EventReader<HttpResponse>,
    mut query: Query<&mut Text, With<ResponseText>>,
) {
    for response in ev_resp.read() {
        let mut text = query.single_mut();
        let ip = response.text().unwrap_or_default();

        text.sections[1].value = "Got ".to_string();
        text.sections[3].value = ip.to_string();
    }
}

fn handle_error(mut ev_error: EventReader<HttpResponseError>) {
    for error in ev_error.read() {
        println!("Error retrieving IP: {}", error.err);
    }
}
