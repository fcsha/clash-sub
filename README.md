# Clash Subscription Converter

A lightweight Clash subscription converter API built with Rust for Cloudflare Workers.

## Features

- **Auto Region Detection**: Automatically groups proxies by region based on naming patterns
- **Load-Balance Groups**: Each region becomes a load-balance group with `consistent-hashing` strategy
- **Info Node Preservation**: Traffic, expiry, and URL info nodes are kept in a "信息" group
- **Simple Rules**: China IP direct, everything else through proxy
- **Fast Edge Computing**: Powered by Cloudflare Workers and WebAssembly

## How It Works

### Input

The converter accepts any Clash subscription URL and extracts only the `proxies` section.

### Output

A simplified Clash configuration with:

```yaml
port: 7890
socks-port: 7891
allow-lan: true
mode: Rule
log-level: info
external-controller: 127.0.0.1:9090

proxies:
  # All proxies from source (info nodes + valid proxies)

proxy-groups:
  - name: 代理
    type: select
    proxies:
      - 信息 # Info nodes group
      - 香港 # Auto-detected regions...
      - 日本
      - DIRECT

  - name: 信息
    type: load-balance
    proxies:
      - "剩余流量: 100GB"
      - "过期时间: 2024-12-31"
    # ...

  - name: 香港
    type: load-balance
    proxies:
      - 香港-01
      - 香港-02
    # ...

rules:
  - GEOIP,CN,DIRECT
  - MATCH,代理
```

### Auto Region Detection

Proxies are automatically grouped by their name prefix:

| Proxy Name     | Detected Region |
| -------------- | --------------- |
| `香港-01`      | 香港            |
| `US-Server-01` | US-Server       |
| `Japan 01`     | Japan           |
| `HK_Node_1`    | HK_Node         |
| `SG01`         | SG              |

Supported delimiters: `-`, `_`, ` `, `|`, `·`, `#`, `@`, or trailing numbers.

## API

### GET /convert

Fetches and converts a subscription from the specified URL.

**Query Parameters:**

| Parameter | Required | Description                                      |
| --------- | -------- | ------------------------------------------------ |
| `url`     | Yes      | The target subscription URL to fetch and convert |

**Example:**

```
GET /convert?url=https://example.com/subscription
```

**Response:**

- `200 OK`: Returns converted YAML configuration
- `400 Bad Request`: Missing or invalid `url` parameter
- `500 Internal Server Error`: Failed to fetch or convert

## Development

### Prerequisites

- Rust toolchain
- Node.js and npm
- Cloudflare account (for deployment)

### Setup

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown
```

### Local Development

```bash
# Start local server
npx wrangler dev

# Test the endpoint
curl "http://localhost:8787/convert?url=https://example.com/subscription"
```

### Deploy

```bash
npx wrangler deploy
```

## Project Structure

```
clash-sub/
├── src/
│   ├── lib.rs          # HTTP handler for Cloudflare Workers
│   └── converter.rs    # Subscription conversion logic
├── Cargo.toml          # Rust dependencies
├── wrangler.toml       # Cloudflare Workers configuration
└── README.md           # This file
```

## License

MIT
