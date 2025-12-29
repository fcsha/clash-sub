# Clash Subscription Converter

A lightweight Clash subscription proxy API built with Rust for Cloudflare Workers.

## Features

- Fetch and proxy subscription content from any URL
- Fast edge computing powered by Cloudflare Workers
- Written in Rust, compiled to WebAssembly

## API

### GET /convert

Fetches content from the specified subscription URL.

**Query Parameters:**

| Parameter | Required | Description                          |
| --------- | -------- | ------------------------------------ |
| `url`     | Yes      | The target subscription URL to fetch |

**Example:**

```
GET /convert?url=https://example.com/subscription
```

## Development

### Prerequisites

- Rust toolchain
- Node.js and npm
- Cloudflare account

### Setup

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

## License

MIT
