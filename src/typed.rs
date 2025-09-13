use bevy_app::{App, PreUpdate};
use bevy_derive::Deref;
use bevy_ecs::{prelude::*, system::Commands, world::CommandQueue};
use bevy_tasks::IoTaskPool;
use ehttp::{Request, Response};
use serde::Deserialize;
use std::marker::PhantomData;

use crate::{get_channel, HttpClientSetting, RequestTask};

pub trait HttpTypedRequestTrait {
    /// Registers a new request type `T` to the application.
    ///
    /// This method is used to register a new request type `T` to the application. The request type
    /// `T` must implement the `Deserialize` trait, and be `Send` and `Sync`. This is necessary for
    /// the request type to be safely shared across threads and for it to be deserialized from a
    /// HTTP response.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the request. This type must implement `Deserialize`, `Send`, and `Sync`.
    ///
    /// # Returns
    ///
    /// A mutable reference to the application. This is used to allow method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_http_client::prelude::*;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize, Clone)]
    /// struct MyRequestType { id: u32, name: String }
    ///
    /// let mut app = App::new();
    /// app.register_request_type::<MyRequestType>();
    /// ```
    fn register_request_type<T: for<'a> Deserialize<'a> + Send + Sync + 'static + Clone>(
        &mut self,
    ) -> &mut Self;
}

impl HttpTypedRequestTrait for App {
    fn register_request_type<T: for<'a> Deserialize<'a> + Send + Sync + 'static + Clone>(
        &mut self,
    ) -> &mut Self {
        self.add_message::<TypedRequest<T>>();
        self.add_message::<TypedResponse<T>>();
        self.add_message::<TypedResponseError<T>>();
        self.add_systems(PreUpdate, handle_typed_request::<T>);
        self
    }
}

/// A struct that represents a typed HTTP request.
///
/// This struct is used to represent a typed HTTP request. The type `T` is the type of the data that
/// is expected to be returned by the HTTP request. The `Request` is the actual HTTP request that
/// will be sent.
///
/// # Type Parameters
///
/// * `T`: The type of the data that is expected to be returned by the HTTP request. This type must
///   implement `Deserialize`.
///
/// # Fields
///
/// * `request`: The actual HTTP request that will be sent.
/// * `inner`: A marker field that uses `PhantomData` to express that it may hold data of type `T`.
///
/// # Examples
///
/// ```
/// use bevy_http_client::prelude::*;
/// use ehttp::{Request, Headers};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct MyResponseType { id: u32, name: String }
///
/// let request = Request {
///     method: "GET".to_string(),
///     url: "https://api.example.com".to_string(),
///     body: vec![],
///     headers: Headers::new(&[("Accept", "application/json")]),
/// };
/// let typed_request = TypedRequest::<MyResponseType>::new(request, None);
/// ```
#[derive(Debug, Message, Event)]
pub struct TypedRequest<T>
where
    T: for<'a> Deserialize<'a> + 'static + Send + Sync,
{
    pub from_entity: Option<Entity>,
    pub request: Request,
    inner: PhantomData<T>,
}

impl<T: 'static + Send + Sync + for<'a> serde::Deserialize<'a>> TypedRequest<T> {
    pub fn new(request: Request, from_entity: Option<Entity>) -> Self {
        TypedRequest {
            from_entity,
            request,
            inner: PhantomData,
        }
    }
}

/// A struct that represents a typed HTTP response.
///
/// This struct is used to represent a typed HTTP response. The type `T` is the type of the data
/// that is expected to be contained in the HTTP response. The `inner` field is the actual data
/// contained in the HTTP response.
///
/// # Type Parameters
///
/// * `T`: The type of the data that is expected to be contained in the HTTP response. This type
///   must implement `Deserialize`.
///
/// # Fields
///
/// * `inner`: The actual data contained in the HTTP response.
///
/// # Examples
///
/// ```
/// use bevy_http_client::prelude::*;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct MyResponseType { id: u32, name: String }
///
/// // Note: TypedResponse is typically created internally by the system
/// // when processing HTTP responses. You would typically use it like this:
/// # fn example_usage(response: TypedResponse<MyResponseType>) {
/// let data = response.into_inner(); // Get the deserialized data
/// let id = data.id;
/// let name = data.name;
/// # }
/// ```
#[derive(Debug, Deref, Message, Event)]
pub struct TypedResponse<T>
where
    T: Send + Sync + 'static + for<'a> Deserialize<'a>,
{
    #[deref]
    inner: T,
}

impl<T: Send + Sync + for<'a> serde::Deserialize<'a>> TypedResponse<T> {
    /// Consumes the HTTP response and returns the inner data.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Access inner value T from a TypedResponse reference
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

#[derive(Message, Event, Debug, Clone, Deref)]
pub struct TypedResponseError<T>
where
    T: Send + Sync + 'static,
{
    #[deref]
    pub err: String,
    pub response: Option<Response>,
    phantom: PhantomData<T>,
}

impl<T: Send + Sync + 'static> TypedResponseError<T> {
    pub fn new(err: String) -> Self {
        Self {
            err,
            response: None,
            phantom: Default::default(),
        }
    }

    pub fn response(mut self, response: Response) -> Self {
        self.response = Some(response);
        self
    }
}

#[derive(Debug, EntityEvent)]
pub struct HttpObserved<T: Send + Sync + 'static> {
    pub entity: Entity,
    pub event: T,
}

impl<T: Send + Sync + 'static> HttpObserved<T> {
    pub fn new(entity: Entity, event: T) -> Self {
        HttpObserved { entity, event }
    }

    pub fn inner(&self) -> &T {
        &self.event
    }

    pub fn into_inner(self) -> T {
        self.event
    }
}

/// A system that handles typed HTTP requests.
fn handle_typed_request<T: for<'a> Deserialize<'a> + Send + Sync + Clone + 'static>(
    mut commands: Commands,
    mut req_res: ResMut<HttpClientSetting>,
    mut requests: MessageReader<TypedRequest<T>>,
    q_tasks: Query<&RequestTask>,
) {
    let thread_pool = IoTaskPool::get();
    for request in requests.read() {
        if req_res.is_available() {
            let (entity, has_from_entity) = if let Some(entity) = request.from_entity {
                (entity, true)
            } else {
                (commands.spawn_empty().id(), false)
            };
            let req = request.request.clone();
            let tx = get_channel(&mut commands, q_tasks, entity);
            thread_pool
                .spawn(async move {
                    let mut command_queue = CommandQueue::default();

                    let response = ehttp::fetch_async(req).await;
                    command_queue.push(move |world: &mut World| {
                        match response {
                            Ok(response) => {
                                let result: Result<T, _> =
                                    serde_json::from_slice(response.bytes.as_slice());

                                match result {
                                    // deserialize success, send response
                                    Ok(inner) => {
                                        if let Some(mut events) =
                                            world.get_resource_mut::<Messages<TypedResponse<T>>>()
                                        {
                                            events.write(TypedResponse {
                                                inner: inner.clone(),
                                            });
                                        } else {
                                            bevy_log::error!(
                                                "TypedResponse events resource not found"
                                            );
                                        }
                                        world.trigger(HttpObserved::new(
                                            entity,
                                            TypedResponse { inner },
                                        ));
                                    }
                                    // deserialize error, send error + response
                                    Err(e) => {
                                        if let Some(mut messages) = world
                                            .get_resource_mut::<Messages<TypedResponseError<T>>>()
                                        {
                                            messages.write(
                                                TypedResponseError::new(e.to_string())
                                                    .response(response.clone()),
                                            );
                                        } else {
                                            bevy_log::error!(
                                                "TypedResponseError events resource not found"
                                            );
                                        }

                                        world.trigger(HttpObserved::new(
                                            entity,
                                            TypedResponseError::<T>::new(e.to_string())
                                                .response(response),
                                        ));
                                    }
                                }
                            }
                            Err(e) => {
                                if let Some(mut events) =
                                    world.get_resource_mut::<Messages<TypedResponseError<T>>>()
                                {
                                    events.write(TypedResponseError::new(e.to_string()));
                                } else {
                                    bevy_log::error!(
                                        "TypedResponseError events resource not found"
                                    );
                                }
                            }
                        }

                        if !has_from_entity {
                            world.entity_mut(entity).despawn();
                        }
                    });

                    if let Err(e) = tx.send(command_queue) {
                        bevy_log::error!("Failed to send command queue: {}", e);
                    }
                })
                .detach();

            req_res.current_clients += 1;
        }
    }
}
