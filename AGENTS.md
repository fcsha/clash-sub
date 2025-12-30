# AGENTS.md

## Project Overview

This is a **Clash Subscription Converter API** built with Rust for Cloudflare Workers. It fetches subscription content from a given URL and converts it to a simplified Clash configuration with:

- Dynamic region-based load-balance proxy groups (only includes regions with nodes)
- Node selector group for manual proxy selection
- Simple rules: China IP direct, others through proxy

## Tech Stack

- **Runtime**: Cloudflare Workers (WASM)
- **Language**: Rust
- **Framework**: `worker-rs` (Cloudflare's official Rust SDK)
- **Build Tool**: `wrangler`
- **YAML Parsing**: `serde_yaml`
- **Regex**: `regex` crate for pattern matching

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

The converter generates a hierarchical group structure with dynamic region detection:

#### 1. 默认流量 (Default Traffic) - Select Group

Main selector group with the following options in order:

- **节点选择** (first option - manual node selector)
- **直接连接** (direct connection)
- **全部节点负载组** (all nodes load-balance group)
- Active region load-balance groups (only regions with matching nodes)

#### 2. 节点选择 (Node Selection) - Select Group

Contains all individual proxy nodes for manual selection.

#### 3. 全部节点负载组 (All Nodes Load-Balance Group)

Contains all proxies with `consistent-hashing` strategy and 180s health check interval.

#### 4. Region Load-Balance Groups (Dynamic)

Only includes regions that have matching nodes. Supported regions with regex filters:

1. 香港负载组 (Hong Kong) - `(?i)港|hk|hongkong|hong kong`
2. 台湾负载组 (Taiwan) - `(?i)台|tw|taiwan`
3. 日本负载组 (Japan) - `(?i)日|jp|japan`
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
15. 其他负载组 (Others) - `.*` (always included, matches all remaining)

Each group uses:

- `include-all: true` with regex filtering
- `consistent-hashing` strategy
- 180s health check interval
- URL: `http://www.gstatic.com/generate_204`

#### 5. 直接连接 (Direct Connection) - Select Group

Contains only `DIRECT` option for direct connections.

### Output Configuration

The converter generates a minimal config with:

- **No fixed settings** - Only proxies, proxy-groups, and rules
- **默认流量** select group (node selector first, then direct connection, then all load-balance groups)
- **节点选择** select group (all individual nodes)
- **全部节点负载组** load-balance group (all proxies)
- **Dynamic region groups** - Only includes regions that have matching nodes
- **直接连接** select group (DIRECT only)
- Simple rules: `GEOIP,LAN,直接连接`, `GEOIP,CN,直接连接`, and `MATCH,默认流量`

### Routing Rules

The converter generates simple routing rules:

1. `GEOIP,LAN,直接连接` - LAN IPs go to "直接连接" group (local network)
2. `GEOIP,CN,直接连接` - China IPs go to "直接连接" group
3. `MATCH,默认流量` - All other traffic goes to "默认流量" group

**Note**: GEOIP rules do NOT include `no-resolve` parameter, as DNS resolution is required to determine the IP address for geo-location matching.

### YAML Optimization

The converter uses YAML merge keys (`<<:`) to significantly reduce redundancy for load-balance group configurations.

**First load-balance group** (defines the anchor):

```yaml
- name: 全部节点负载组
  type: load-balance
  include-all: true
  <<: &lb_common
    url: http://www.gstatic.com/generate_204
    interval: 180
    strategy: consistent-hashing
```

**Subsequent load-balance groups** (merge the common config):

```yaml
- name: 香港负载组
  type: load-balance
  include-all: true
  filter: "(?i)港|hk|hongkong|hong kong"
  <<: *lb_common
```

Instead of repeating 3 lines (url, interval, strategy) for each group, we only need 1 line (`<<: *lb_common`). This **reduces file size by ~60%** for the proxy-groups section and makes the config much more maintainable.

### Key Features

- ✅ **Dynamic region detection**: Only adds region groups if nodes exist for that region
- ✅ **No info node filtering**: All nodes are treated equally
- ✅ **Clean output**: No fixed port/mode/log-level configurations
- ✅ **Flexible selection**: Users can choose load-balance groups or individual nodes
- ✅ **Health checks**: All load-balance groups check node health every 180 seconds
- ✅ **YAML merge keys**: Load-balance common configs (url, interval, strategy) are reused via YAML merge key `<<:` for 60% smaller file size

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

# Build for production
cargo build --target wasm32-unknown-unknown --release
```
