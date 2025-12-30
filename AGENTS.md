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

### Proxy Group Structure

The converter generates a hierarchical group structure:

#### 1. 默认流量 (Default Traffic) - Select Group

Main selector group with the following options in order:

- **直接连接** (first option)
- **全部节点负载组** (all nodes load-balance group)
- All region load-balance groups (香港负载组, 台湾负载组, 日本负载组, etc.)
- All individual proxy nodes

#### 2. 全部节点负载组 (All Nodes Load-Balance Group)

Contains all proxies with `consistent-hashing` strategy.

#### 3. Region Load-Balance Groups

Fixed region groups with regex filters:

1. 香港负载组 (Hong Kong) - `(?i)港|hk|hongkong|hong kong`
2. 台湾负载组 (Taiwan) - `(?i)台|tw|taiwan`
3. 日本负载组 (Japan) - `(?i)日本?|jp|japan`
4. 新加坡负载组 (Singapore) - `(?i)新|sg|singapore`
5. 美国负载组 (USA) - `(?i)美|us|usa|united states|america`
6. 韩国负载组 (Korea) - `(?i)韩|kr|korea`
7. 英国负载组 (UK) - `(?i)英|uk|britain|united kingdom`
8. 德国负载组 (Germany) - `(?i)德|de|germany`
9. 法国负载组 (France) - `(?i)法|fr|france`
10. 加拿大负载组 (Canada) - `(?i)加|ca|canada`
11. 澳大利亚负载组 (Australia) - `(?i)澳|au|australia`
12. 马来西亚负载组 (Malaysia) - `(?i)马来|my|malaysia`
13. 土耳其负载组 (Turkey) - `(?i)土耳其|tr|turkey`
14. 阿根廷负载组 (Argentina) - `(?i)阿根廷|ar|argentina`
15. 其他负载组 (Others) - `.*` (matches all remaining)

Each group uses `include-all: true` with regex filtering.

#### 4. 直接连接 (Direct Connection) - Select Group

Contains only `DIRECT` option for direct connections.

### Output Configuration

The converter generates a config with:

- Fixed settings: `port: 7890`, `socks-port: 7891`, `allow-lan: true`
- **默认流量** select group (direct connection first, then all load-balance groups, then all proxies)
- **全部节点负载组** load-balance group (all proxies)
- 15 fixed region load-balance groups with regex filters and `consistent-hashing` strategy
- **直接连接** select group (DIRECT only)
- Simple rules: `GEOIP,CN,DIRECT` and `MATCH,默认流量`

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
