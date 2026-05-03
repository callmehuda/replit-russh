# Workspace

## Overview

pnpm workspace monorepo using TypeScript. Each package manages its own dependencies.
Also includes a standalone Rust application: `ws-ssh-server/`.

## Stack

- **Monorepo tool**: pnpm workspaces
- **Node.js version**: 24
- **Package manager**: pnpm
- **TypeScript version**: 5.9
- **API framework**: Express 5
- **Database**: PostgreSQL + Drizzle ORM
- **Validation**: Zod (`zod/v4`), `drizzle-zod`
- **API codegen**: Orval (from OpenAPI spec)
- **Build**: esbuild (CJS bundle)

## Rust SSH-over-WebSocket Server (`ws-ssh-server/`)

A standalone Rust application that runs an SSH server over WebSocket transport (no sshd required).

### Stack
- **Runtime**: Tokio async
- **SSH**: russh 0.44
- **WebSocket**: tokio-tungstenite 0.21
- **PTY**: portable-pty 0.8
- **Logging**: tracing + tracing-subscriber

### Source files
- `src/main.rs` — Entry point, TCP listener + WebSocket upgrade + host key management
- `src/transport.rs` — `WsTransport`: wraps WebSocket as `AsyncRead + AsyncWrite`
- `src/handler.rs` — `SshHandler`: implements `russh::server::Handler`
- `src/pty.rs` — `PtySession`: spawns shell, relays I/O to SSH channel
- `src/auth.rs` — `AuthStore`: validates username/password

### Default credentials
- `admin` / `secret123`
- `guest` / `guest`

### How to connect
```bash
# Install websocat, then in one terminal:
websocat -b tcp-l:127.0.0.1:2222 ws://localhost:8022

# In another terminal:
ssh -o StrictHostKeyChecking=no -p 2222 admin@localhost
# password: secret123
```

### Workflow
- **SSH-over-WebSocket Server**: `cd ws-ssh-server && cargo run --bin server`
- Listens on port 8022
- Host key auto-generated and persisted to `ws-ssh-server/host_key` on first run

## Key Commands (Node.js monorepo)

- `pnpm run typecheck` — full typecheck across all packages
- `pnpm run build` — typecheck + build all packages
- `pnpm --filter @workspace/api-spec run codegen` — regenerate API hooks and Zod schemas from OpenAPI spec
- `pnpm --filter @workspace/db run push` — push DB schema changes (dev only)
- `pnpm --filter @workspace/api-server run dev` — run API server locally

See the `pnpm-workspace` skill for workspace structure, TypeScript setup, and package details.
