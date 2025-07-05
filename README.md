# Treat Dispenser API

A simple REST API for controlling a treat dispenser, built with [Axum](https://github.com/tokio-rs/axum) and async Rust.

## Features

- **Dispense treats** via a REST endpoint
- **Authorization** using a bearer token
- **Structured logging** with `env_logger`
- Ready for hardware integration (see `dispenser.rs`)

## Endpoints

### `GET /`

Returns a simple status message.

**Example:**
```sh
curl http://localhost:3500/
```
_Response:_
```
Treat dispenser is online!
```

---

### `GET /dispense`

Dispenses a treat.  
**Requires** an `Authorization` header with a bearer token.

**Example:**
```sh
curl -H "Authorization: Bearer <YOUR_TOKEN>" http://localhost:3500/dispense
```

_Response:_
- `Treat dispensed successfully!` on success
- `Failed to dispense treat.` on error

## Setup

1. **Clone the repository**

2. **Set environment variables**  
   Create a `.env` file or set these in your environment:
   ```
   DISPENSER_API_TOKEN=your_secret_token
   ```

3. **Run the server**
   ```sh
   cargo run
   ```

   The server will listen on `0.0.0.0:3500`.

4. **Test the endpoints**  
   Use `curl` or any HTTP client.

## Logging

- Logs are output to stdout.
- Log level can be set with the `RUST_LOG` environment variable (default: `info`).
- Example:  
  ```sh
  RUST_LOG=debug cargo run
  ```

## Code Structure

- `main.rs` – Application entry point, sets up routes and logging.
- `dispenser.rs` – Treat dispensing logic (placeholder for hardware integration).
- `auth.rs` – Authorization extractor for validating API requests.

## Requirements

- Rust (edition 2021 or later)
- [Axum](https://github.com/tokio-rs/axum)
- [tokio](https://tokio.rs/)
- [env_logger](https://docs.rs/env_logger/)
- [dotenv](https://docs.rs/dotenv/)

## License

MIT

---
