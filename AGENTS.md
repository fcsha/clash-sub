# AGENTS.md

## Project Overview

This is a **Clash Subscription Converter API** built with Rust for Cloudflare Workers. It fetches subscription content from a given URL and converts it to a simplified Clash configuration with:

- Auto-detected region-based load-balance proxy groups
- Simple rules: China IP direct, others through proxy
- Info nodes (traffic, expiry) preserved in a separate group

## Tech Stack

- **Runtime**: Cloudflare Workers (WASM)
- **Language**: Rust
- **Framework**: `worker-rs` (Cloudflare's official Rust SDK)
- **Build Tool**: `wrangler`
- **YAML Parsing**: `serde_yaml`

## Project Structure

```
clash-sub/
├── src/
│   ├── lib.rs              # HTTP handler for Cloudflare Workers
│   └── converter.rs        # Subscription conversion logic
├── Cargo.toml              # Rust dependencies
├── Cargo.lock              # Dependency lock file
├── wrangler.toml           # Cloudflare Workers configuration
├── AGENTS.md               # This file
└── README.md               # Project documentation
```

## Conversion Features

### Auto Region Detection

Automatically detects and groups proxies by region based on naming patterns:

- `香港-01`, `香港-02` → Group "香港"
- `US-Server-01`, `US-Server-02` → Group "US-Server"
- `Japan 01`, `Japan 02` → Group "Japan"
- `HK_Node_1`, `HK_Node_2` → Group "HK_Node"
- `SG01`, `SG02` → Group "SG"

Supported delimiters: `-`, `_`, ` `, `|`, `·`, `｜`, `#`, `@`, or trailing numbers.

### Info Nodes

Info nodes (traffic, expiry, URL, etc.) are placed in a "信息" load-balance group:

- `剩余流量: 100GB`
- `过期时间: 2024-12-31`
- `最新网址: example.com`

### Output Configuration

The converter generates a simplified config with:

- Fixed settings: `port: 7890`, `socks-port: 7891`, `allow-lan: true`
- Main "代理" select group containing all region groups
- Each region as a load-balance group with `consistent-hashing` strategy
- Simple rules: `GEOIP,CN,DIRECT` and `MATCH,代理`

## API Endpoints

### GET /convert

Fetches and converts a subscription from the specified URL.

**Query Parameters:**

- `url` (required): The target subscription URL to fetch

**Example:**

```
GET /convert?url=https://example.com/subscription
```

**Responses:**

- `200 OK`: Returns converted YAML config
- `400 Bad Request`: Missing or invalid `url` parameter
- `500 Internal Server Error`: Failed to fetch, parse, or convert

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

Then test the endpoint:

```bash
curl "http://localhost:8787/convert?url=https://example.com/subscription"
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

# Check WASM target
cargo check --target wasm32-unknown-unknown
```
