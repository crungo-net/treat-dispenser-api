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

## Quick Setup

For a quick development environment setup, run:

```bash
chmod +x setup_dev_env.sh
./setup_dev_env.sh
```

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

### `GET /status`

Returns detailed health status information including GPIO availability, motor status, and uptime.

**Example:**
```sh
curl http://localhost:3500/status
```

_Response:_ JSON object containing system status information.

## Setup

1. **Clone the repository**

2. **Set environment variables**  
   Create a `.env` file or set these in your environment (see [Environment Variables](#environment-variables) section for details)

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

The motor type can be configured using the `MOTOR_TYPE` environment variable (see [Environment Variables](#environment-variables) section). This allows switching between hardware implementations and the mock implementation for testing.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DISPENSER_API_TOKEN` | Authentication token for API access | (Required) |
| `DISPENSER_API_PORT` | Port to run the server on | `3500` |
| `RUST_LOG` | Log level (trace, debug, info, warn, error) | `info` |
| `MOTOR_TYPE` | Type of motor to use (Stepper28BYJ48, StepperMock) | `Stepper28BYJ48` |

Example `.env` file:
```
DISPENSER_API_TOKEN=your_secret_token
DISPENSER_API_PORT=3500
RUST_LOG=info
MOTOR_TYPE=Stepper28BYJ48
```

## Code Structure

- `src/main.rs` – Application entry point, sets up routes, logging, and server.
- `src/lib.rs` – Library exports and app factory (used for tests and integration).
- `src/state.rs` – System state tracking and health monitoring.
- `src/error.rs` – Error handling and HTTP response mapping.
- `src/motor/` – Stepper motor trait, real and mock implementations, and motor selection logic.
- `src/services/` – Business logic layer (hardware control, treat dispensing, etc.)
    - `mod.rs` – Exports service modules
    - `dispenser.rs` – Treat dispensing logic
- `src/routes/` – API route handlers (HTTP endpoints)
    - `mod.rs` – Exports route modules
    - `dispense.rs` – Dispense endpoint handler
    - `status.rs` – Status endpoint handler
- `src/middleware/` – API middleware (e.g., authentication)
    - `mod.rs` – Exports middleware modules
    - `auth.rs` – Authentication middleware
- `src/utils/` – Utility functions and helpers
    - `mod.rs` – Exports utility modules
    - `datetime.rs` – Date/time formatting utilities
    - `state_helpers.rs` – State manipulation helpers

This structure separates business logic, hardware integration, HTTP interface, and utility functions for clarity and maintainability. Each module has a single responsibility, making the codebase easier to test and extend as new features are added.

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

## Testing

The project includes both unit tests and integration tests:

### Unit Tests

Run unit tests with:
```sh
cargo test --lib
```

Unit tests cover individual components like date formatting utilities, motor control logic, and error handling.

### Integration Tests

Run integration tests with:
```sh
cargo test --test integration
```

Integration tests verify the full API functionality by starting a test server and making HTTP requests to the endpoints.

For better test parallelism, tests that require sequential execution (like testing busy states) are grouped together in single test functions.

## License

MIT


