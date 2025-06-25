#![doc = include_str!("../README.md")]

use bevy_app::{App, Plugin, Update};
use bevy_derive::Deref;
use bevy_ecs::{prelude::*, world::CommandQueue};
use bevy_tasks::IoTaskPool;
use crossbeam_channel::{Receiver, Sender};
use ehttp::{Headers, Request, Response};

use crate::prelude::TypedRequest;

pub mod prelude;
mod typed;

/// JSON serialization fallback strategy when serialization fails
#[derive(Debug, Clone, Default)]
pub enum JsonFallback {
    /// Use empty object {} as fallback
    #[default]
    EmptyObject,
    /// Use empty array [] as fallback  
    EmptyArray,
    /// Use null as fallback
    Null,
    /// Use custom data as fallback
    Custom(Vec<u8>),
}

/// JSON serialization error type
#[derive(Debug, Clone)]
pub enum JsonSerializationError {
    SerializationFailed {
        message: String,
        fallback_used: JsonFallback,
    },
}

impl std::fmt::Display for JsonSerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonSerializationError::SerializationFailed {
                message,
                fallback_used,
            } => {
                write!(
                    f,
                    "JSON serialization failed: {}, fallback: {:?}",
                    message, fallback_used
                )
            }
        }
    }
}

impl std::error::Error for JsonSerializationError {}

/// HTTP client builder error type
#[derive(Debug, Clone)]
pub enum HttpClientBuilderError {
    MissingMethod,
    MissingUrl,
    MissingHeaders,
}

impl std::fmt::Display for HttpClientBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpClientBuilderError::MissingMethod => write!(f, "HTTP method is required"),
            HttpClientBuilderError::MissingUrl => write!(f, "URL is required"),
            HttpClientBuilderError::MissingHeaders => write!(f, "Headers are required"),
        }
    }
}

impl std::error::Error for HttpClientBuilderError {}

/// Plugin that provides support for send http request and handle response.
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_http_client::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins(DefaultPlugins)
///    .add_plugins(HttpClientPlugin);
/// // Note: Don't call .run() in doctests as it starts the event loop
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
    /// use bevy_http_client::HttpClient;
    /// let http_client = HttpClient::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// his method is used to create a new `HttpClient` instance with `Entity`.
    ///
    /// # Arguments
    ///
    /// * `entity`: Target Entity
    ///
    /// returns: HttpClient
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_http_client::HttpClient;
    /// use bevy_ecs::entity::Entity;
    ///
    /// let entity = Entity::from_raw(42); // Example entity
    /// let http_client = HttpClient::new_with_entity(entity);
    /// ```
    pub fn new_with_entity(entity: Entity) -> Self {
        Self {
            from_entity: Some(entity),
            ..Default::default()
        }
    }

    /// This method is used to create a `GET` HTTP request.
    ///
    /// # Arguments
    ///
    /// * `url` - A value that can be converted into a string. This is the URL to which the HTTP
    ///   request will be sent.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_http_client::HttpClient;
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
    /// * `url` - A value that can be converted into a string. This is the URL to which the HTTP
    ///   request will be sent.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_http_client::HttpClient;
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
    /// * `url` - A value that can be converted into a string. This is the URL to which the HTTP
    ///   request will be sent.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_http_client::HttpClient;
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
    /// * `url` - A value that can be converted into a string. This is the URL to which the HTTP
    ///   request will be sent.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_http_client::HttpClient;
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
    /// * `url` - A value that can be converted into a string. This is the URL to which the HTTP
    ///   request will be sent.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_http_client::HttpClient;
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
    /// * `url` - A value that can be converted into a string. This is the URL to which the HTTP
    ///   request will be sent.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_http_client::HttpClient;
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
    /// * `headers` - A slice of tuples where each tuple represents a header. The first element of
    ///   the tuple is the header name and the second element is the header value.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_http_client::HttpClient;
    /// let http_client = HttpClient::new().post("http://example.com")
    ///     .headers(&[("Content-Type", "application/json"), ("Accept", "*/*")]);
    /// ```
    pub fn headers(mut self, headers: &[(&str, &str)]) -> Self {
        self.headers = Some(Headers::new(headers));
        self
    }

    /// Safe JSON serialization method with fallback strategy
    ///
    /// This method safely serializes the body to JSON and sets the Content-Type header.
    /// If serialization fails, it uses a fallback strategy instead of panicking.
    ///
    /// # Arguments
    /// * `body` - Data to serialize to JSON
    /// * `fallback` - Fallback strategy when serialization fails
    ///
    /// # Returns
    /// Returns HttpClient instance for method chaining
    ///
    /// # Examples
    /// ```
    /// use bevy_http_client::{HttpClient, JsonFallback};
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct MyData { name: String }
    /// let data = MyData { name: "test".to_string() };
    ///
    /// let client = HttpClient::new()
    ///     .post("http://example.com")
    ///     .json_with_fallback(&data, JsonFallback::EmptyObject);
    /// ```
    pub fn json_with_fallback(
        mut self,
        body: &impl serde::Serialize,
        fallback: JsonFallback,
    ) -> Self {
        // Set Content-Type header
        if let Some(headers) = self.headers.as_mut() {
            headers.insert("Content-Type".to_string(), "application/json".to_string());
        } else {
            self.headers = Some(Headers::new(&[
                ("Content-Type", "application/json"),
                ("Accept", "*/*"),
            ]));
        }

        // Safe serialization with fallback strategy
        self.body = match serde_json::to_vec(body) {
            Ok(bytes) => {
                // Check for unreasonably large payloads
                if bytes.len() > 50 * 1024 * 1024 {
                    // 50MB limit
                    bevy_log::warn!(
                        "JSON payload is very large ({} bytes), this might cause performance issues",
                        bytes.len()
                    );
                }
                bytes
            }
            Err(e) => {
                // Get fallback data
                let fallback_data = match &fallback {
                    JsonFallback::EmptyObject => b"{}".to_vec(),
                    JsonFallback::EmptyArray => b"[]".to_vec(),
                    JsonFallback::Null => b"null".to_vec(),
                    JsonFallback::Custom(data) => data.clone(),
                };

                // Log error using bevy's logging system
                bevy_log::error!(
                    "JSON serialization failed: {}. Using fallback: {:?}",
                    e,
                    fallback
                );

                fallback_data
            }
        };

        self
    }

    /// Result-returning safe JSON serialization method
    ///
    /// # Arguments
    /// * `body` - Data to serialize to JSON
    ///
    /// # Returns
    /// * `Ok(HttpClient)` - Serialization successful
    /// * `Err(JsonSerializationError)` - Serialization failed
    ///
    /// # Examples
    /// ```
    /// use bevy_http_client::HttpClient;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct MyData { name: String }
    /// let data = MyData { name: "test".to_string() };
    ///
    /// match HttpClient::new().post("http://example.com").json_safe(&data) {
    ///     Ok(client) => { /* use client */ },
    ///     Err(e) => { /* handle error */ },
    /// }
    /// ```
    pub fn json_safe(
        mut self,
        body: &impl serde::Serialize,
    ) -> Result<Self, JsonSerializationError> {
        // Set Content-Type header
        if let Some(headers) = self.headers.as_mut() {
            headers.insert("Content-Type".to_string(), "application/json".to_string());
        } else {
            self.headers = Some(Headers::new(&[
                ("Content-Type", "application/json"),
                ("Accept", "*/*"),
            ]));
        }

        // Try serialization
        self.body = serde_json::to_vec(body).map_err(|e| {
            JsonSerializationError::SerializationFailed {
                message: e.to_string(),
                fallback_used: JsonFallback::EmptyObject, // Record intended fallback
            }
        })?;

        Ok(self)
    }

    /// Improved json method with safe fallback - maintains backward compatibility
    ///
    /// This method will automatically use empty object {} as fallback when serialization fails,
    /// instead of panicking. This maintains backward compatibility while providing better error handling.
    ///
    /// # Arguments
    /// * `body` - Data to serialize to JSON
    ///
    /// # Returns
    /// Returns HttpClient instance for method chaining
    ///
    /// # Examples
    /// ```
    /// use bevy_http_client::HttpClient;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct MyData { name: String }
    /// let my_data = MyData { name: "test".to_string() };
    ///
    /// let client = HttpClient::new()
    ///     .post("http://example.com")
    ///     .json(&my_data);  // Now safe, won't panic
    /// ```
    pub fn json(self, body: &impl serde::Serialize) -> Self {
        // Use default fallback strategy (empty object)
        self.json_with_fallback(body, JsonFallback::default())
    }

    /// This method is used to set the properties of the `HttpClient` instance using an `Request`
    /// instance. This version of the method is used when the target architecture is not
    /// `wasm32`.
    ///
    /// # Arguments
    ///
    /// * `request` - An instance of `Request` which includes the HTTP method, URL, body, and
    ///   headers.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_http_client::HttpClient;
    /// use ehttp::{Request, Headers};
    ///
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
    /// This method is used to associate an `Entity` with the `HttpClient`. This can be useful when
    /// you want to track which entity initiated the HTTP request.
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
    /// use bevy_http_client::HttpClient;
    /// use bevy_ecs::entity::Entity;
    ///
    /// let entity = Entity::from_raw(42); // Example entity
    /// let http_client = HttpClient::new().entity(entity);
    /// ```
    pub fn entity(mut self, entity: Entity) -> Self {
        self.from_entity = Some(entity);
        self
    }

    /// This method is used to set the properties of the `HttpClient` instance using an `Request`
    /// instance. This version of the method is used when the target architecture is `wasm32`.
    ///
    /// # Arguments
    ///
    /// * `request` - An instance of `Request` which includes the HTTP method, URL, body, headers,
    ///   and mode.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the instance of the `HttpClient` struct, allowing for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_http_client::HttpClient;
    /// use ehttp::{Request, Headers, Mode};
    ///
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
    /// This method is used to construct an `HttpRequest` from the current state of the `HttpClient`
    /// instance. The resulting `HttpRequest` includes the HTTP method, URL, body, headers, and mode
    /// (only available on wasm builds).
    ///
    /// # Returns
    ///
    /// An `HttpRequest` instance which includes the HTTP method, URL, body, headers, and mode (only
    /// available on wasm builds).
    ///
    /// # Panics
    ///
    /// This method will panic if the HTTP method, URL, or headers are not set in the `HttpClient`
    /// instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_http_client::HttpClient;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct MyData { name: String }
    /// let data = MyData { name: "test".to_string() };
    ///
    /// let http_request = HttpClient::new().post("http://example.com")
    ///     .headers(&[("Content-Type", "application/json"), ("Accept", "*/*")])
    ///     .json(&data)
    ///     .build();
    /// ```
    ///
    /// # Note
    ///
    /// This method consumes the `HttpClient` instance, meaning it can only be called once per
    /// instance.
    #[deprecated(
        since = "0.8.3",
        note = "Use `try_build()` instead for better error handling"
    )]
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

    /// Safe version of build() that returns a Result instead of panicking
    ///
    /// This method safely builds an `HttpRequest` from the `HttpClient` instance.
    /// Returns an error if required fields (method, url, headers) are missing.
    ///
    /// # Returns
    ///
    /// * `Ok(HttpRequest)` - Successfully built HTTP request
    /// * `Err(HttpClientBuilderError)` - Missing required fields
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_http_client::HttpClient;
    ///
    /// let result = HttpClient::new().post("http://example.com")
    ///     .headers(&[("Content-Type", "application/json")])
    ///     .try_build();
    ///
    /// match result {
    ///     Ok(request) => { /* use request */ },
    ///     Err(e) => eprintln!("Build failed: {}", e),
    /// }
    /// ```
    pub fn try_build(self) -> Result<HttpRequest, HttpClientBuilderError> {
        let method = self.method.ok_or(HttpClientBuilderError::MissingMethod)?;
        let url = self
            .url
            .filter(|u| !u.trim().is_empty())
            .ok_or(HttpClientBuilderError::MissingUrl)?;
        let headers = self.headers.ok_or(HttpClientBuilderError::MissingHeaders)?;

        Ok(HttpRequest {
            from_entity: self.from_entity,
            request: Request {
                method,
                url,
                body: self.body,
                headers,
                #[cfg(target_arch = "wasm32")]
                mode: self.mode,
            },
        })
    }

    #[deprecated(
        since = "0.8.3",
        note = "Use `try_with_type()` instead for better error handling"
    )]
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

    /// Safe version of with_type() that returns a Result instead of panicking
    ///
    /// This method safely creates a typed request from the `HttpClient` instance.
    /// Returns an error if required fields (method, url, headers) are missing.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The expected response type that implements Deserialize
    ///
    /// # Returns
    ///
    /// * `Ok(TypedRequest<T>)` - Successfully built typed request
    /// * `Err(HttpClientBuilderError)` - Missing required fields
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_http_client::HttpClient;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct MyResponseType { id: u32, name: String }
    ///
    /// let result = HttpClient::new().get("https://api.example.com")
    ///     .try_with_type::<MyResponseType>();
    ///
    /// match result {
    ///     Ok(request) => { /* use typed request */ },
    ///     Err(e) => eprintln!("Build failed: {}", e),
    /// }
    /// ```
    pub fn try_with_type<T: for<'a> serde::Deserialize<'a>>(
        self,
    ) -> Result<TypedRequest<T>, HttpClientBuilderError> {
        let method = self.method.ok_or(HttpClientBuilderError::MissingMethod)?;
        let url = self
            .url
            .filter(|u| !u.trim().is_empty())
            .ok_or(HttpClientBuilderError::MissingUrl)?;
        let headers = self.headers.ok_or(HttpClientBuilderError::MissingHeaders)?;

        Ok(TypedRequest::new(
            Request {
                method,
                url,
                body: self.body,
                headers,
                #[cfg(target_arch = "wasm32")]
                mode: self.mode,
            },
            self.from_entity,
        ))
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
pub struct RequestTask {
    tx: Sender<CommandQueue>,
    rx: Receiver<CommandQueue>,
}

fn handle_request(
    mut commands: Commands,
    mut req_res: ResMut<HttpClientSetting>,
    mut requests: EventReader<HttpRequest>,
    q_tasks: Query<&RequestTask>,
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

            let tx = get_channel(&mut commands, q_tasks, entity);

            thread_pool
                .spawn(async move {
                    let mut command_queue = CommandQueue::default();

                    let response = ehttp::fetch_async(req.request).await;
                    command_queue.push(move |world: &mut World| {
                        match response {
                            Ok(res) => {
                                if let Some(mut events) =
                                    world.get_resource_mut::<Events<HttpResponse>>()
                                {
                                    events.send(HttpResponse(res.clone()));
                                } else {
                                    bevy_log::error!("HttpResponse events resource not found");
                                }
                                world.trigger_targets(HttpResponse(res), entity);
                            }
                            Err(e) => {
                                if let Some(mut events) =
                                    world.get_resource_mut::<Events<HttpResponseError>>()
                                {
                                    events.send(HttpResponseError::new(e.to_string()));
                                } else {
                                    bevy_log::error!("HttpResponseError events resource not found");
                                }
                                world
                                    .trigger_targets(HttpResponseError::new(e.to_string()), entity);
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

fn get_channel(
    commands: &mut Commands,
    q_tasks: Query<&RequestTask>,
    entity: Entity,
) -> Sender<CommandQueue> {
    if let Ok(task) = q_tasks.get(entity) {
        task.tx.clone()
    } else {
        let (tx, rx) = crossbeam_channel::bounded(5);

        commands.entity(entity).insert(RequestTask {
            tx: tx.clone(),
            rx: rx.clone(),
        });

        tx
    }
}

fn handle_tasks(
    mut commands: Commands,
    mut req_res: ResMut<HttpClientSetting>,
    mut request_tasks: Query<&RequestTask>,
) {
    for task in request_tasks.iter_mut() {
        if let Ok(mut command_queue) = task.rx.try_recv() {
            commands.append(&mut command_queue);
            req_res.current_clients -= 1;
        }
    }
}
