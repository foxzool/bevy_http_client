use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use ehttp::{Request, Response};
use futures_lite::future;

pub use ehttp;

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

#[derive(Resource)]
pub struct HttpClientSetting {
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
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            current_clients: 0,
        }
    }

    pub fn is_available(&self) -> bool {
        self.current_clients < self.max_concurrent
    }
}


#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct HttpRequest(pub Request);

#[derive(Component)]
pub struct RequestTask(pub Task<Result<Response, ehttp::Error>>);

fn handle_request(
    mut commands: Commands,
    mut req_res: ResMut<HttpClientSetting>,
    requests: Query<(Entity, &HttpRequest), Without<RequestTask>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (entity, request) in requests.iter() {
        if req_res.is_available() {
            let req = request.clone();
            println!("{:?}", req);
            let s = thread_pool.spawn( async {
                ehttp::fetch_async(req.0).await
            });

            commands.entity(entity).insert(RequestTask(s));
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

        println!("{:?}", task.0);
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            println!("{:?}", result);
            commands.entity(entity).remove::<RequestTask>();
            req_res.current_clients -= 1;
        }
    }
}
