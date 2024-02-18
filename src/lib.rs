#![doc = include_str!("../README.md")]

mod typed;

use bevy::prelude::*;
use bevy::tasks::{block_on, IoTaskPool, Task};

use ehttp::{Request, Response};
use futures_lite::future;

pub mod prelude {
    pub use super::typed::{register_request_type, RequestBundle, TypedResponse};
    pub use super::{
        HttpClientPlugin, HttpClientSetting, HttpRequest, HttpResponse, HttpResponseError,
        RequestTask,
    };
}

/// Plugin that provides support for send http request and handle response.
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_http_client::prelude::*;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(HttpClientPlugin).run();
/// ```
#[derive(Default)]
pub struct HttpClientPlugin;

impl Plugin for HttpClientPlugin {
    fn build(&self, app: &mut App) {
        if !app.world.contains_resource::<HttpClientSetting>() {
            app.init_resource::<HttpClientSetting>();
        }
        app.add_systems(Update, (handle_request, handle_response));
    }
}

/// The setting of http client.
/// can set the max concurrent request.
#[derive(Resource)]
pub struct HttpClientSetting {
    /// max concurrent request
    pub max_concurrent: usize,
    current_clients: usize,
}

impl Default for HttpClientSetting {
    fn default() -> Self {
        Self {
            max_concurrent: 5,
            current_clients: 0,
        }
    }
}

impl HttpClientSetting {
    /// create a new http client setting
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            current_clients: 0,
        }
    }

    /// check if the client is available
    #[inline]
    pub fn is_available(&self) -> bool {
        self.current_clients < self.max_concurrent
    }
}

/// wrap for ehttp request
#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct HttpRequest(pub Request);

impl HttpRequest {
    /// create a new http request
    pub fn new(request: Request) -> Self {
        Self(request)
    }

    /// create a new http get request
    pub fn get(url: impl ToString) -> Self {
        Self(Request::get(url))
    }

    /// create a new http post request
    pub fn post(url: &str, body: Vec<u8>) -> Self {
        Self(Request::post(url, body))
    }
}

/// wrap for ehttp response
#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct HttpResponse(pub Response);

/// wrap for ehttp error
#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct HttpResponseError(pub String);

/// task for ehttp response result
#[derive(Component)]
pub struct RequestTask(pub Task<Result<Response, ehttp::Error>>);

fn handle_request(
    mut commands: Commands,
    mut req_res: ResMut<HttpClientSetting>,
    requests: Query<(Entity, &HttpRequest), Without<RequestTask>>,
) {
    let thread_pool = IoTaskPool::get();
    for (entity, request) in requests.iter() {
        if req_res.is_available() {
            let req = request.clone();

            let s = thread_pool.spawn(async { ehttp::fetch_async(req.0).await });

            commands
                .entity(entity)
                .remove::<HttpRequest>()
                .insert(RequestTask(s));
            req_res.current_clients += 1;
        }
    }
}

fn handle_response(
    mut commands: Commands,
    mut req_res: ResMut<HttpClientSetting>,
    mut request_tasks: Query<(Entity, &mut RequestTask)>,
) {
    for (entity, mut task) in request_tasks.iter_mut() {
        if let Some(result) = block_on(future::poll_once(&mut task.0)) {
            match result {
                Ok(res) => {
                    commands
                        .entity(entity)
                        .insert(HttpResponse(res))
                        .remove::<RequestTask>();
                }
                Err(e) => {
                    commands
                        .entity(entity)
                        .insert(HttpResponseError(e))
                        .remove::<RequestTask>();
                }
            }

            req_res.current_clients -= 1;
        }
    }
}
