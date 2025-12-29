# AGENTS.md

## Project Overview

This is a **Clash Subscription Converter API** built with Rust for Cloudflare Workers. It provides a simple HTTP endpoint to fetch and proxy subscription content from a given URL.

## Tech Stack

- **Runtime**: Cloudflare Workers (WASM)
- **Language**: Rust
- **Framework**: `worker-rs` (Cloudflare's official Rust SDK)
- **Build Tool**: `wrangler`

## Project Structure

```
clash-sub/
├── src/
│   └── lib.rs          # Main application code
├── Cargo.toml          # Rust dependencies
├── Cargo.lock          # Dependency lock file
├── wrangler.toml       # Cloudflare Workers configuration
└── AGENTS.md           # This file
```

## API Endpoints

### GET /convert

Fetches content from the specified URL and returns it.

**Query Parameters:**

- `url` (required): The target subscription URL to fetch

**Example:**

```
GET /convert?url=https://example.com/subscription
```

**Responses:**

- `200 OK`: Returns the fetched content
- `400 Bad Request`: Missing or invalid `url` parameter
- `500 Internal Server Error`: Failed to fetch or read the target URL

## Development

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- Node.js and npm
- Cloudflare account

### Install Dependencies

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown
```

### Local Development

```bash
npx wrangler dev
```

### Deploy

```bash
npx wrangler deploy
```

## Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow standard Rust naming conventions
- Handle all `Result` types explicitly with proper error messages

## Testing

```bash
# Check for compilation errors
cargo check

# Run clippy
cargo clippy

# Test locally
npx wrangler dev
```

Then test the endpoint:

```bash
curl "http://localhost:8787/convert?url=https://example.com/test"
```
