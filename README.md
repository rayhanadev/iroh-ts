# iroh

TypeScript/JavaScript bindings for [iroh](https://github.com/n0-computer/iroh) — peer-to-peer networking with QUIC.

Iroh is a library for establishing direct connections between devices. It uses QUIC for transport with built-in NAT traversal via relay servers and hole punching.

## Installation

```bash
npm install @rayhanadev/iroh
# or
bun add @rayhanadev/iroh
```

## Quick Start

```typescript
import { Endpoint, initLogging } from "@rayhanadev/iroh";

// Optional: enable logging
initLogging("info");

// Create an endpoint
const endpoint = await Endpoint.create();
console.log("Node ID:", endpoint.nodeId());

// Wait for relay connection
await endpoint.online();
console.log("Online!");

// Clean up
await endpoint.close();
```

## Examples

### Echo Server

```typescript
import { Endpoint } from "@rayhanadev/iroh";

const ALPN = "my-app/1";

async function server() {
  const endpoint = await Endpoint.createWithOptions({
    alpns: [ALPN],
  });

  console.log("Server node ID:", endpoint.nodeId());
  await endpoint.online();

  const conn = await endpoint.accept();
  if (!conn) return;

  console.log("Connection from:", conn.remoteNodeId());

  const { send, recv } = await conn.acceptBi();
  const data = await recv.readToEnd(1024);
  console.log("Received:", data.toString());

  await send.writeAll(data);
  await send.finish();

  await endpoint.close();
}

server();
```

### Echo Client

```typescript
import { Endpoint } from "@rayhanadev/iroh";

const ALPN = "my-app/1";
const SERVER_NODE_ID = "..."; // Get this from the server

async function client() {
  const endpoint = await Endpoint.create();

  const conn = await endpoint.connect(SERVER_NODE_ID, ALPN);
  console.log("Connected to:", conn.remoteNodeId());

  const { send, recv } = await conn.openBi();

  await send.writeAll(Buffer.from("Hello, iroh!"));
  await send.finish();

  const response = await recv.readToEnd(1024);
  console.log("Response:", response.toString());

  conn.close();
  await endpoint.close();
}

client();
```

## API Reference

### `initLogging(level?: string)`

Initialize logging. Levels: `"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"`.

### `Endpoint`

The main entry point for creating and accepting connections.

| Method | Description |
|--------|-------------|
| `static create(): Promise<Endpoint>` | Create endpoint with defaults |
| `static createWithOptions(options): Promise<Endpoint>` | Create with custom options |
| `nodeId(): string` | Get the endpoint's public key (node ID) |
| `addr(): string` | Get full address info (for debugging) |
| `online(): Promise<void>` | Wait until connected to a relay |
| `connect(nodeId, alpn): Promise<Connection>` | Connect to a remote endpoint |
| `accept(): Promise<Connection \| null>` | Accept an incoming connection |
| `close(): Promise<void>` | Close the endpoint |
| `isClosed(): boolean` | Check if closed |

#### `EndpointOptions`

```typescript
interface EndpointOptions {
  alpns?: string[];      // ALPN protocols to accept
  secretKey?: string;    // 32-byte hex string for deterministic identity
}
```

### `Connection`

A QUIC connection to a remote peer.

| Method | Description |
|--------|-------------|
| `remoteNodeId(): string` | Get remote peer's node ID |
| `alpn(): Buffer` | Get the ALPN protocol used |
| `openBi(): Promise<BiStream>` | Open a bidirectional stream |
| `openUni(): Promise<SendStream>` | Open a send-only stream |
| `acceptBi(): Promise<BiStream>` | Accept a bidirectional stream |
| `acceptUni(): Promise<RecvStream>` | Accept a receive-only stream |
| `close(errorCode?, reason?)` | Close the connection |
| `closed(): Promise<string>` | Wait for connection to close |
| `rtt(): number` | Get round-trip time in ms |
| `sendDatagram(data: Buffer)` | Send unreliable datagram |
| `readDatagram(): Promise<Buffer>` | Receive unreliable datagram |

### `SendStream`

A stream for sending data.

| Method | Description |
|--------|-------------|
| `write(data: Buffer): Promise<number>` | Write data, returns bytes written |
| `writeAll(data: Buffer): Promise<void>` | Write all data |
| `finish(): Promise<void>` | Signal end of stream |
| `reset(errorCode: number): Promise<void>` | Abort the stream |
| `id(): Promise<string>` | Get stream ID |

### `RecvStream`

A stream for receiving data.

| Method | Description |
|--------|-------------|
| `read(maxLength: number): Promise<Buffer \| null>` | Read up to maxLength bytes |
| `readExact(length: number): Promise<Buffer>` | Read exactly length bytes |
| `readToEnd(maxLength: number): Promise<Buffer>` | Read all remaining data |
| `stop(errorCode: number): Promise<void>` | Stop reading |
| `id(): Promise<string>` | Get stream ID |

## How It Works

1. **Identity**: Each endpoint has a unique node ID (Ed25519 public key)
2. **Discovery**: Endpoints connect to relay servers to be discoverable
3. **Connection**: When connecting, iroh tries direct UDP hole punching first, falling back to relay if needed
4. **Streams**: QUIC multiplexes multiple streams over a single connection

## Building from Source

Requirements:
- Rust (latest stable)
- Node.js 18+ or Bun
- napi-rs CLI: `npm install -g @napi-rs/cli`

```bash
# Install dependencies
bun install

# Build native module
bun run build

# Build debug version
bun run build:debug
```

## Platform Support

Pre-built binaries are available for:
- macOS (x64, arm64)
- Linux (x64)
- Windows (x64)

## License

MIT
