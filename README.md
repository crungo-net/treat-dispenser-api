# Treat Dispenser API

A simple REST API for controlling a treat dispenser, built with [Axum](https://github.com/tokio-rs/axum) and async Rust.

## Features

- **Dispense treats** via a REST endpoint
- **Authorization** using a bearer token
- **Structured logging** with [`tracing`](https://docs.rs/tracing/)
- **Thread ID tracking** in logs
- **Hardware integration** with GPIO control for stepper motors
- **Graceful shutdown** support
- **Docker support** for containerized deployment

## Endpoints

### `GET /`

Returns a simple status message.

**Example:**
```sh
curl http://localhost:3500/
```
_Response:_
```
Treat dispenser is online! Binky time!
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
- `Dispensing started, please wait...` on success
- Error message with appropriate status code on failure

---

### `GET /health`

Simple health check endpoint.

**Example:**
```sh
curl http://localhost:3500/health
```

_Response:_
```
OK
```

---

### `GET /health/detailed`

Returns detailed health status information including GPIO availability, motor status, and uptime.

**Example:**
```sh
curl http://localhost:3500/health/detailed
```

_Response:_ JSON object containing system status information.

## Setup

1. **Clone the repository**

2. **Set environment variables**  
   Create a `.env` file or set these in your environment:
   ```
   DISPENSER_API_TOKEN=your_secret_token
   DISPENSER_API_PORT=3500  # Optional, defaults to 3500
   RUST_LOG=info  # Optional, controls log level
   ```

3. **Run the server**
   ```sh
   cargo run
   ```

   The server will listen on `0.0.0.0:3500` by default or the port specified in your environment.

4. **Test the endpoints**  
   Use `curl` or any HTTP client.

## Logging

- Logs are output to stdout with thread IDs and names included.
- Log level can be set with the `RUST_LOG` environment variable (default: `info`).
- Example:  
  ```sh
  RUST_LOG=debug cargo run
  ```

## Docker Support

Build and run the application in a Docker container:

```sh
# Build the image
make build

# Run with debug logging
make run-debug

# Push to registry
make push
```

## Hardware Integration

The application is designed to control a 28BYJ-48 stepper motor (with a ULN2003 driver) connected via GPIO pins:

- Pin 26: Motor coil 1
- Pin 19: Motor coil 2
- Pin 13: Motor coil 3
- Pin 6: Motor coil 4

Supported step modes for 28BYJ-48:
- **Full step** (2048 steps per rotation, more torque, slower to avoid overheating)
- **Half step** (4096 steps per rotation, smoother motion)

Other step modes (quarter, eighth, sixteenth) are defined but not implemented for this motor.
Support for the NEMA-14 motor type and the A4988 stepper driver is planned.

The motor control logic enforces a 5-second cooldown after each dispensing operation to protect hardware.

### Motor Type Configuration

You can configure which motor type is used by setting the `MOTOR_TYPE` environment variable. The default is `Stepper28BYJ48`.

Example for 28BYJ-48 (default):
```
MOTOR_TYPE=Stepper28BYJ48 cargo run
```

## Code Structure

- `src/main.rs` – Application entry point, sets up routes, logging, and server.
- `src/dispenser.rs` – Treat dispensing logic with stepper motor control.
- `src/state.rs` – System state tracking and health monitoring.
- `src/auth.rs` – Authorization extractor for validating API requests.
- `src/error.rs` – Error handling and HTTP response mapping.
- `src/route.rs` – API endpoint implementations.

## Requirements

- Rust (edition 2021 or later)
- [Axum](https://github.com/tokio-rs/axum)
- [tokio](https://tokio.rs/)
- [tracing](https://docs.rs/tracing/)
- [rppal](https://docs.rs/rppal/) for Raspberry Pi GPIO control
- [dotenv](https://docs.rs/dotenv/)

## Continuous Integration (CI)

This project uses a GitLab CI pipeline (see `.gitlab-ci.yml`) to automate building and publishing container images for multiple architectures.

- **Build System:** Uses [moby/buildkit](https://github.com/moby/buildkit) for efficient Docker builds and caching.
- **Multi-Arch Builds:**
  - `build-and-push` builds and pushes an x86_64 (amd64) image.
  - `build-and-push-arm64` builds and pushes an ARM64 image (runs on `main`, `release`, or `arm-test-*` branches).
- **Image Tags:** Each build pushes images with tags for `latest`, the short commit SHA, and the branch name.
- **Build Caching:** Build cache is stored in the registry to speed up subsequent builds.
- **Artifacts:** The built binaries for each architecture are saved as CI artifacts for one week.
- **Authentication:** Docker credentials are injected for pushing to the private registry (`harbor.crungo.net`).

## License

MIT


