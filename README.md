# Gateway API

A Rust-based API Gateway that functions as a reverse proxy, routing incoming HTTP requests to backend services based on a configuration file. It includes built-in rate limiting, logging, and a real-time monitoring dashboard via WebSockets.

## Features

- **Reverse Proxy:** Forwards requests to upstream services defined in `config.json`.
- **Rate Limiting:** Prevents abuse by limiting the number of requests per second.
- **Observability:**
  - **REST API:** Endpoints to retrieve metrics (`/api/metrics`) and logs (`/api/logs`).
  - **WebSockets:** Real-time data feed for live monitoring of traffic and errors.
- **Concurrency:** Built on `tokio` and `axum` for asynchronous request handling.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Node.js](https://nodejs.org/en/download) (only if you want the logging monitor UI - see below)

## Configuration

The routing logic is defined in `config.json`. Example structure:

```json
{
    "routes": [
        {
            "path": "/bin",
            "backend_url": "http://httpbin.org"
        },
        {
            "path": "/can",
            "backend_url": "http://httpcan.org"
        }
    ]
}
```

- **path**: The incoming request path prefix to match.
- **backend_url**: The upstream server URL to forward requests to.

## Running the Server

1. Ensure `config.json` is present in the root directory.
2. Start the server:

```bash
cargo run
```

You can also run it with the RUST_LOG environment variable set to `info` to see some logs in the terminal.

```bash
RUST_LOG=info cargo run
```

The server will start on `127.0.0.1:3000`.

Define the allowed routes in the `config.json` file, then you can send requests to it using some HTTP client.

```bash
curl localhost:3000/bin/get
```

## API Endpoints

### Monitoring

- `GET /api/metrics`: Returns aggregated stats (total requests, errors, active connections).
- `GET /api/logs`: Returns the 50 most recent request logs.
- `GET /api/routes`: Returns the current routing table.
- `WS /ws`: WebSocket endpoint for real-time updates.

### Health Check

- `GET /health`: Returns a 200 OK status if the gateway is running.

## Development

### Running Examples

To verify the WebSocket functionality, run the included monitor example while the server is running:

```bash
cargo run --example ws_monitor
```

This script connects to the WebSocket endpoint and prints live traffic logs and metric updates to the terminal.

## Logging UI

```bash
cd frontend
npm install
npm run dev
```

This will start a React app on `localhost:5173` which will display the logs coming in via WebSocket.

## License

This project is open source.
