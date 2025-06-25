# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is `bevy_http_client`, a Bevy plugin that provides HTTP client capabilities for both native and WASM platforms.
It's a Rust crate that integrates with the Bevy game engine using the ECS (Entity Component System) architecture.

## Development Commands

### Testing

```bash
cargo test
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run clippy linting
cargo clippy -- -D warnings
```

### Building

```bash
# Build the library
cargo build

# Build with release optimizations
cargo build --release
```

### Running Examples

Examples are located in the `examples/` directory:

```bash
# Run specific examples
cargo run --example ipinfo
cargo run --example typed
cargo run --example observer
cargo run --example window
```

## Architecture

### Core Components

- **HttpClientPlugin**: Main plugin that sets up the HTTP client system
- **HttpClient**: Builder pattern for creating HTTP requests with support for GET, POST, PUT, PATCH, DELETE, HEAD
  methods
- **TypedRequest/TypedResponse**: Generic typed wrapper for strongly-typed JSON deserialization
- **HttpClientSetting**: Resource for configuring concurrent request limits (default: 5)

### Key Systems

- **handle_request**: Processes HTTP requests asynchronously using Bevy's IoTaskPool
- **handle_tasks**: Manages completion of async HTTP tasks and updates ECS world
- **handle_typed_request**: Handles typed requests with automatic JSON deserialization

### Event-Driven Architecture

The plugin uses Bevy's event system:

- `HttpRequest/HttpResponse/HttpResponseError` for untyped requests
- `TypedRequest<T>/TypedResponse<T>/TypedResponseError<T>` for typed requests

### Platform Support

- Native: Uses `ehttp` with async support
- WASM: Full WASM compatibility with fetch API

## Code Patterns

### HTTP Requests

```rust
// Basic HTTP request
let request = HttpClient::new()
.get("https://api.example.com")
.headers(& [("Content-Type", "application/json")])
.build();

// Typed HTTP request
let typed_request = HttpClient::new()
.get("https://api.example.com")
.with_type::<MyResponseType>();
```

### Registering Request Types

```rust
app.register_request_type::<MyType>();
```

### Response Handling

- Events are triggered both globally and on specific entities
- Supports entity-specific request tracking via `from_entity`
- Automatic entity cleanup for orphaned requests

## File Structure

- `src/lib.rs`: Main plugin and core HTTP client implementation
- `src/typed.rs`: Typed request/response system
- `src/prelude.rs`: Public API exports
- `examples/`: Usage examples demonstrating different features

## Dependencies

- Bevy 0.16.0 (latest supported version)
- ehttp for HTTP functionality
- serde/serde_json for JSON serialization
- crossbeam-channel for async communication

## Memories

- Claude.md 和 生成的报告不在git里提交
- 如果要删除原有的函数、方法, 先不删除, 先打上 deprecated 标签
- 和我交互使用中文.
- git 注释, 代码注释, README, CHANGELOG 使用英文