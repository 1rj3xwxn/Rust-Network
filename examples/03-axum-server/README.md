# 03 - Axum Server

A minimal HTTP server built with Axum and Tokio.

## Run

From this directory:

```bash
cargo run
```

The server listens on `127.0.0.1:8080`.

## Endpoints

- `GET /` — returns a greeting.
- `GET /health` — returns the health status.
- `GET /node` — returns the node name from shared application state.

## Try it

```bash
curl http://127.0.0.1:8080/
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/node
```

## Learning focus

- `Router`: matches an HTTP method and path to a handler.
- Handler functions: process matched requests and produce responses.
- `State`: makes application-wide dependencies available to handlers.
- `Arc`: allows the state to be shared through reference-counted ownership.

This is a learning example, not a production-ready server.
