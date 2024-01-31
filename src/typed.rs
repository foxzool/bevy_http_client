use crate::{HttpClientSetting, HttpResponseError};
use bevy::ecs::query::WorldQuery;
use bevy::prelude::{App, Update};
use bevy::prelude::{Commands, Component, Entity, Query, ResMut, Without};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use ehttp::{Request, Response};
use futures_lite::future;
use serde::Deserialize;
use std::marker::PhantomData;

pub fn register_request_type<T: Send + Sync + 'static>(app: &mut App) -> &mut App {
    app.add_systems(Update, handle_typed_response::<T>)
        .add_systems(Update, handle_typed_request::<T>)
}

/// task for ehttp response result
#[derive(Component)]
pub struct TypedRequestTask<T>
where
    T: Send + Sync,
{
    pub task: Task<Result<Response, ehttp::Error>>,
    res: PhantomData<T>,
}

/// wrap for ehttp request
#[derive(Component, Clone, Debug)]
pub struct TypedRequest<T>
where
    T: Send + Sync,
{
    pub request: Request,
    res: PhantomData<T>,
}

impl<T> TypedRequest<T>
where
    T: Send + Sync,
{
    pub fn new(request: Request) -> Self {
        Self {
            request,
            res: PhantomData,
        }
    }
}

/// wrap for ehttp response
#[derive(Component, Clone, Debug)]
pub struct TypedResponse<T>
where
    T: Send + Sync,
{
    pub response: Response,
    res: PhantomData<T>,
}

impl<T> TypedResponse<T>
where
    T: for<'a> Deserialize<'a> + Send + Sync,
{
    pub fn parse(&self) -> Option<T> {
        match &self.response.text() {
            Some(s) => match serde_json::from_str::<T>(s) {
                Ok(val) => Some(val),
                _ => None,
            },
            None => None,
        }
    }
}

#[derive(WorldQuery)]
pub struct TypedRequestFilter<T: Send + Sync + 'static> {
    without: Without<TypedRequestTask<T>>,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct TypedRequestQuery<T: Send + Sync + 'static> {
    pub entity: Entity,
    pub task: &'static mut TypedRequestTask<T>,
}

pub fn handle_typed_request<T: Send + Sync + 'static>(
    mut commands: Commands,
    mut req_res: ResMut<HttpClientSetting>,
    requests: Query<(Entity, &'static TypedRequest<T>), TypedRequestFilter<T>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    if !req_res.is_available() {
        return;
    }
    for (entity, request) in requests.iter() {
        let req = request.request.clone();

        let task = thread_pool.spawn(async move { ehttp::fetch_async(req).await });

        commands
            .entity(entity)
            .remove::<TypedRequest<T>>()
            .insert(TypedRequestTask::<T> {
                task,
                res: PhantomData,
            });
        req_res.current_clients += 1;
    }
}

pub fn handle_typed_response<T: Send + Sync + 'static>(
    mut commands: Commands,
    mut req_res: ResMut<'_, HttpClientSetting>,
    mut request_tasks: Query<TypedRequestQuery<T>>,
) {
    for mut entry in request_tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut entry.task.task)) {
            match result {
                Ok(response) => {
                    commands
                        .entity(entry.entity)
                        .insert(TypedResponse::<T> {
                            response,
                            res: PhantomData,
                        })
                        .remove::<TypedRequestTask<T>>();
                }
                Err(e) => {
                    commands
                        .entity(entry.entity)
                        .insert(HttpResponseError(e))
                        .remove::<TypedRequestTask<T>>();
                }
            }

            req_res.current_clients -= 1;
        }
    }
}
