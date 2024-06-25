#![doc = include_str!("../README.md")]

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use crossbeam_channel::Receiver;

use crate::prelude::TypedRequest;
use ehttp::{Headers, Request, Response};

pub mod prelude;
mod typed;

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
        if !app.world().contains_resource::<HttpClientSetting>() {
            app.init_resource::<HttpClientSetting>();
        }
        app.add_event::<HttpRequest>();
        app.add_event::<HttpResponse>();
        app.add_event::<HttpResponseError>();
        app.add_systems(Update, (handle_request, handle_tasks));
    }
}

/// The setting of http client.
/// can set the max concurrent request.
#[derive(Resource, Debug)]
pub struct HttpClientSetting {
    /// max concurrent request
    pub client_limits: usize,
    current_clients: usize,
}

impl Default for HttpClientSetting {
    fn default() -> Self {
        Self {
            client_limits: 5,
            current_clients: 0,
        }
    }
}

impl HttpClientSetting {
    /// create a new http client setting
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            client_limits: max_concurrent,
            current_clients: 0,
        }
    }

    /// check if the client is available
    #[inline]
    pub fn is_available(&self) -> bool {
        self.current_clients < self.client_limits
    }
}

#[derive(Event, Debug, Clone)]
pub struct HttpRequest {
    pub from_entity: Option<Entity>,
    pub request: Request,
}

/// builder  for ehttp request
#[derive(Component, Debug, Clone)]
pub struct HttpClient {
    /// The entity that the request is associated with.
    from_entity: Option<Entity>,
    /// "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", …
    method: Option<String>,

    /// https://…
    url: Option<String>,

    /// The data you send with e.g. "POST".
    body: Vec<u8>,

    /// ("Accept", "*/*"), …
    headers: Option<Headers>,

    /// Request mode used on fetch. Only available on wasm builds
    #[cfg(target_arch = "wasm32")]
    pub mode: ehttp::Mode,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            from_entity: None,
            method: None,
            url: None,
            body: vec![],
            headers: Some(Headers::new(&[("Accept", "*/*")])),
            #[cfg(target_arch = "wasm32")]
            mode: ehttp::Mode::default(),
        }
    }
}

impl HttpClient {
    /// This method is used to create a new `HttpClient` instance.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct.
    ///
    /// # Examples
    ///
    /// ```
    /// let http_client = HttpClient::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// This method is used to create a `GET` HTTP request.
    ///
    /// # Arguments
    ///
    /// * `url` - A value that can be converted into a string. This is the URL to which the HTTP request will be sent.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// let http_client = HttpClient::new().get("http://example.com");
    /// ```
    pub fn get(mut self, url: impl ToString) -> Self {
        self.method = Some("GET".to_string());
        self.url = Some(url.to_string());
        self
    }

    /// This method is used to create a `POST` HTTP request.
    ///
    /// # Arguments
    ///
    /// * `url` - A value that can be converted into a string. This is the URL to which the HTTP request will be sent.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// let http_client = HttpClient::new().post("http://example.com");
    /// ```
    pub fn post(mut self, url: impl ToString) -> Self {
        self.method = Some("POST".to_string());
        self.url = Some(url.to_string());
        self
    }

    /// This method is used to create a `PUT` HTTP request.
    ///
    /// # Arguments
    ///
    /// * `url` - A value that can be converted into a string. This is the URL to which the HTTP request will be sent.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// let http_client = HttpClient::new().put("http://example.com");
    /// ```
    pub fn put(mut self, url: impl ToString) -> Self {
        self.method = Some("PUT".to_string());
        self.url = Some(url.to_string());
        self
    }

    /// This method is used to create a `PATCH` HTTP request.
    ///
    /// # Arguments
    ///
    /// * `url` - A value that can be converted into a string. This is the URL to which the HTTP request will be sent.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// let http_client = HttpClient::new().patch("http://example.com");
    /// ```
    pub fn patch(mut self, url: impl ToString) -> Self {
        self.method = Some("PATCH".to_string());
        self.url = Some(url.to_string());
        self
    }

    /// This method is used to create a `DELETE` HTTP request.
    ///
    /// # Arguments
    ///
    /// * `url` - A value that can be converted into a string. This is the URL to which the HTTP request will be sent.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// let http_client = HttpClient::new().delete("http://example.com");
    /// ```
    pub fn delete(mut self, url: impl ToString) -> Self {
        self.method = Some("DELETE".to_string());
        self.url = Some(url.to_string());
        self
    }

    /// This method is used to create a `HEAD` HTTP request.
    ///
    /// # Arguments
    ///
    /// * `url` - A value that can be converted into a string. This is the URL to which the HTTP request will be sent.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// let http_client = HttpClient::new().head("http://example.com");
    /// ```
    pub fn head(mut self, url: impl ToString) -> Self {
        self.method = Some("HEAD".to_string());
        self.url = Some(url.to_string());
        self
    }

    /// This method is used to set the headers of the HTTP request.
    ///
    /// # Arguments
    ///
    /// * `headers` - A slice of tuples where each tuple represents a header. The first element of the tuple is the header name and the second element is the header value.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// let http_client = HttpClient::new().post("http://example.com")
    ///     .headers(&[("Content-Type", "application/json"), ("Accept", "*/*")]);
    /// ```
    pub fn headers(mut self, headers: &[(&str, &str)]) -> Self {
        self.headers = Some(Headers::new(headers));
        self
    }

    /// This method is used to set the body of the HTTP request as a JSON payload.
    /// It also sets the "Content-Type" header of the request to "application/json".
    ///
    /// # Arguments
    ///
    /// * `body` - A reference to any type that implements the `serde::Serialize` trait. This is the data that will be serialized to JSON and set as the body of the HTTP request.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Panics
    ///
    /// * This method will panic if the serialization of the `body` to JSON fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let http_client = HttpClient::new().post("http://example.com")
    ///     .json(&data);
    /// ```
    pub fn json(mut self, body: &impl serde::Serialize) -> Self {
        if let Some(headers) = self.headers.as_mut() {
            headers.insert("Content-Type".to_string(), "application/json".to_string());
        } else {
            self.headers = Some(Headers::new(&[
                ("Content-Type", "application/json"),
                ("Accept", "*/*"),
            ]));
        }

        self.body = serde_json::to_vec(body).unwrap();
        self
    }

    /// This method is used to set the properties of the `HttpClient` instance using an `Request` instance.
    /// This version of the method is used when the target architecture is not `wasm32`.
    ///
    /// # Arguments
    ///
    /// * `request` - An instance of `Request` which includes the HTTP method, URL, body, and headers.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// let request = Request {
    ///     method: "POST".to_string(),
    ///     url: "http://example.com".to_string(),
    ///     body: vec![],
    ///     headers: Headers::new(&[("Content-Type", "application/json"), ("Accept", "*/*")]),
    /// };
    /// let http_client = HttpClient::new().request(request);
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn request(mut self, request: Request) -> Self {
        self.method = Some(request.method);
        self.url = Some(request.url);
        self.body = request.body;
        self.headers = Some(request.headers);

        self
    }

    /// Associates an `Entity` with the `HttpClient`.
    ///
    /// This method is used to associate an `Entity` with the `HttpClient`. This can be useful when you want to track which entity initiated the HTTP request.
    ///
    /// # Parameters
    ///
    /// * `entity`: The `Entity` that you want to associate with the `HttpClient`.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `HttpClient`. This is used to allow method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// let entity = commands.spawn().id();
    /// let http_client = HttpClient::new().entity(entity);
    /// ```
    pub fn entity(mut self, entity: Entity) -> Self {
        self.from_entity = Some(entity);
        self
    }

    /// This method is used to set the properties of the `HttpClient` instance using an `Request` instance.
    /// This version of the method is used when the target architecture is `wasm32`.
    ///
    /// # Arguments
    ///
    /// * `request` - An instance of `Request` which includes the HTTP method, URL, body, headers, and mode.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// let request = Request {
    ///     method: "POST".to_string(),
    ///     url: "http://example.com".to_string(),
    ///     body: vec![],
    ///     headers: Headers::new(&[("Content-Type", "application/json"), ("Accept", "*/*")]),
    ///     mode: Mode::Cors,
    /// };
    /// let http_client = HttpClient::new().request(request);
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn request(mut self, request: Request) -> Self {
        self.method = Some(request.method);
        self.url = Some(request.url);
        self.body = request.body;
        self.headers = Some(request.headers);
        self.mode = request.mode;

        self
    }

    /// Builds an `HttpRequest` from the `HttpClient` instance.
    ///
    /// This method is used to construct an `HttpRequest` from the current state of the `HttpClient` instance. The resulting `HttpRequest` includes the HTTP method, URL, body, headers, and mode (only available on wasm builds).
    ///
    /// # Returns
    ///
    /// An `HttpRequest` instance which includes the HTTP method, URL, body, headers, and mode (only available on wasm builds).
    ///
    /// # Panics
    ///
    /// This method will panic if the HTTP method, URL, or headers are not set in the `HttpClient` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// let http_request = HttpClient::new().post("http://example.com")
    ///     .headers(&[("Content-Type", "application/json"), ("Accept", "*/*")])
    ///     .json(&data)
    ///     .build();
    /// ```
    ///
    /// # Note
    ///
    /// This method consumes the `HttpClient` instance, meaning it can only be called once per instance.
    pub fn build(self) -> HttpRequest {
        HttpRequest {
            from_entity: self.from_entity,
            request: Request {
                method: self.method.expect("method is required"),
                url: self.url.expect("url is required"),
                body: self.body,
                headers: self.headers.expect("headers is required"),
                #[cfg(target_arch = "wasm32")]
                mode: self.mode,
            },
        }
    }

    pub fn with_type<T: for<'a> serde::Deserialize<'a>>(self) -> TypedRequest<T> {
        TypedRequest::new(
            Request {
                method: self.method.expect("method is required"),
                url: self.url.expect("url is required"),
                body: self.body,
                headers: self.headers.expect("headers is required"),
                #[cfg(target_arch = "wasm32")]
                mode: self.mode,
            },
            self.from_entity,
        )
    }
}

/// wrap for ehttp response
#[derive(Event, Debug, Clone, Deref)]
pub struct HttpResponse(pub Response);

/// wrap for ehttp error
#[derive(Event, Debug, Clone, Deref)]
pub struct HttpResponseError {
    pub err: String,
}

impl HttpResponseError {
    pub fn new(err: String) -> Self {
        Self { err }
    }
}

/// task for ehttp response result
#[derive(Component, Debug)]
pub struct RequestTask(pub Receiver<CommandQueue>);

fn handle_request(
    mut commands: Commands,
    mut req_res: ResMut<HttpClientSetting>,
    mut requests: EventReader<HttpRequest>,
) {
    let thread_pool = IoTaskPool::get();
    for request in requests.read() {
        if req_res.is_available() {
            let req = request.clone();
            let (entity, has_from_entity) = if let Some(entity) = req.from_entity {
                (entity, true)
            } else {
                (commands.spawn_empty().id(), false)
            };
            let (tx, rx) = crossbeam_channel::bounded(1);

            thread_pool
                .spawn(async move {
                    let mut command_queue = CommandQueue::default();

                    let response = ehttp::fetch_async(req.request).await;
                    command_queue.push(move |world: &mut World| {
                        match response {
                            Ok(res) => {
                                world
                                    .get_resource_mut::<Events<HttpResponse>>()
                                    .unwrap()
                                    .send(HttpResponse(res));
                            }
                            Err(e) => {
                                world
                                    .get_resource_mut::<Events<HttpResponseError>>()
                                    .unwrap()
                                    .send(HttpResponseError::new(e.to_string()));
                            }
                        }

                        if has_from_entity {
                            world.entity_mut(entity).remove::<RequestTask>();
                        } else {
                            world.entity_mut(entity).despawn_recursive();
                        }
                    });

                    tx.send(command_queue).unwrap();
                })
                .detach();

            commands.entity(entity).insert(RequestTask(rx));
            req_res.current_clients += 1;
        }
    }
}

fn handle_tasks(
    mut commands: Commands,
    mut req_res: ResMut<HttpClientSetting>,
    mut request_tasks: Query<&RequestTask>,
) {
    for task in request_tasks.iter_mut() {
        if let Ok(mut command_queue) = task.0.try_recv() {
            commands.append(&mut command_queue);
            req_res.current_clients -= 1;
        }
    }
}
