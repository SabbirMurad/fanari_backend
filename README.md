# Fanari backend

The Rust server behind **Fanari**, a social app with a feed, comments, one-to-one and group chat, voice and video calls, and in-app support. The Flutter client is [fanari_v2](https://github.com/SabbirMurad/fanari_v2).

One Actix Web binary serves the REST API, a WebSocket for real-time chat and call signalling, media upload and delivery, and a small server-rendered admin dashboard.

## Stack

| Concern | Choice |
| --- | --- |
| HTTP server | Actix Web 4 on Tokio, TLS through rustls |
| Real-time | `actix-web-actors` WebSockets, with one `Lobby` actor routing messages between connected users |
| Main data | MongoDB (accounts, posts, comments, replies, conversations, reports, support) |
| Sessions | Redis, through `actix-session` (admin dashboard) |
| Media and tokens | SQLite (`rusqlite`, bundled): image and emoji blobs, refresh tokens |
| Auth | JWT access and refresh tokens (`jsonwebtoken`); refresh tokens are stored server side, so they can be renewed and revoked |
| Images | Every upload is converted to WebP; the original is kept next to it |
| Video | ffmpeg turns uploads into HLS (H.264/AAC, 4-second segments) |
| Email | Resend: sign-up verification and password reset codes |
| Push | Firebase Cloud Messaging for comment and reply notifications |
| Templates | Tera, for the admin sign-in and dashboard pages |
| Logging | `flexi_logger`, async with compressed rotation |

## How it fits together

```
Flutter app ──HTTPS──► /api/*        handlers ──► MongoDB
            │                                 └─► SQLite (media, refresh tokens)
            └──WSS───► /api/ws/chat   Lobby actor ──► every socket in the conversation
```

- **Lobby.** Each socket registers with the `Lobby` actor on connect. The lobby keeps a room per conversation and fans each envelope out to its members: text, typing, seen receipts, new conversations, group membership changes, and WebRTC `call_signal` messages for calls.
- **Handlers.** Each endpoint lives in its own module under `src/handler/<area>/<action>.rs` and exposes a single `task` function. Routes are wired in `src/routes/<area>.rs`.
- **Access control.** `src/middleware/auth.rs` checks the bearer token and the account's role before a handler runs.
- **HTTPS only.** Plain HTTP is redirected to HTTPS and `www.` to the bare domain, unless `APP_HTTP=allow` is set for local work.

```
src/
├── main.rs          server setup: TLS, sessions, CORS, CSP, redirects, routes
├── builtins/        Mongo, SQLite, JWT, TLS, CORS, CSP, logger, cron
├── middleware/      bearer-token and role checks
├── routes/          one router per area
├── handler/         one module per endpoint, plus web_socket/ (lobby, messages)
├── model/           Mongo document types
├── integrations/    Resend (email) and Firebase (push)
└── utils/           responses, validation, Mongo helpers
```

## API

| Area | Endpoints |
| --- | --- |
| Auth | `POST /api/auth/sign-up`, `validate-email`, `resend-verification-code`, `sign-in`, `sign-out`, `refresh`, `forgot-password`, `verify-reset-code`, `reset-password` · `GET /api/auth/user/{email_or_username}` |
| Profile | `GET /api/profile/myself/details`, `GET /api/profile/search`, `POST /api/profile/list`, `PATCH /api/profile` |
| Posts | `POST`, `GET /api/post` · `DELETE /api/post/{uuid}` |
| Comments, replies | `POST`, `GET /api/comment` and `/api/reply` · `DELETE …/{uuid}` |
| Conversations | `POST /api/conversation/single`, `/group`, `/block` · `GET /single/{id}`, `/group/{id}`, `/list`, `/text/list` · `PATCH /favorite`, `/mute` · `DELETE /delete/{uuid}` |
| Chat socket | `GET /api/ws/chat` (WebSocket upgrade) |
| Images | `POST /image` · `GET /image/webp/{id}`, `/image/original/{id}`, `/image/metadata/{id}` · `POST /image/metadata` (bulk) |
| Emoji | `POST /api/emoji` · `GET /api/emoji/list`, `/webp/{id}`, `/original/{id}`, `/metadata/{id}` |
| Video | `POST /api/video/upload` · `GET /api/video/segment/{id}` (HLS playlist), `/api/video/segment/{id}/{segment}`, `/api/video/{id}/video.mp4` |
| Moderation | `POST /api/report/create`, `/api/report/resolve` |
| Support | `GET /api/support/start` · `POST /api/support/user-text`, `/support-text`, `/end` |
| App metadata | `GET /api/metadata/get`, `POST /api/metadata/update` |
| Admin pages | `GET /admin/authentication`, `GET /dashboard` |

## Running locally

You need Rust (edition 2024), MongoDB, Redis, and `ffmpeg` on the `PATH` for video uploads.

```bash
cp .env.dev .env       # then fill in the empty keys: JWT, Resend, Firebase
cargo run
```

`.env.dev` lists every variable the server reads. Use `APP_HTTP=allow` to serve plain HTTP on your machine, and a fresh 64-character hex `SESSION_KEY` for anything that isn't local.

For auto-reload on save, with the socket held open across restarts:

```bash
cargo install systemfd cargo-watch
systemfd --no-pid -s http::8080 -- cargo watch -x run
```

The longer project notes are in `docs/`, served by MkDocs (`mkdocs serve`).
