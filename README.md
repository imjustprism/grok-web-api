# grok-web-api

[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)

Self-hosted REST API wrapping grok.com into an OpenAI-compatible endpoint. Pure Rust, single binary, no headless browser, no JS engine. Reverse-engineers Grok's anti-bot challenge cryptographically so requests look like a real browser session.

Runs on your grok.com cookies, so inference bills against your own grok.com quota (free, SuperGrok, or Heavy). No xAI API key needed.

## Requirements

- Rust 1.85 or newer ([rustup](https://rustup.rs/))
- NASM
- LLVM and libclang (needed by BoringSSL in wreq)

## Install

```sh
git clone https://github.com/imjustprism/grok-web-api.git
cd grok-web-api
cp .env.example .env
```

## Configure

Edit `.env`. Five required values come in two groups.

### 1. Cookies (`GROK_SSO_COOKIE`, `GROK_SSO_RW_COOKIE`)

1. Log in to [grok.com](https://grok.com)
2. Open DevTools (F12) and go to **Application** (Chrome or Edge) or **Storage** (Firefox)
3. In the sidebar open **Cookies** and pick `https://grok.com`
4. Copy the `Value` of the row named `sso` into `GROK_SSO_COOKIE`
5. Copy the `Value` of the row named `sso-rw` into `GROK_SSO_RW_COOKIE`

Both rows typically hold the same JWT. Cookies rotate when you log out or clear storage.

### 2. Challenge (`CHALLENGE_HEADER_HEX`, `CHALLENGE_SUFFIX`, `CHALLENGE_TRAILER`)

Grok's web client signs every POST with an `x-statsig-id` header. This server needs three constants to reproduce the signature.

With [Void](https://github.com/imjustprism/Void) installed, paste this snippet in grok.com's browser console:

```js
var m=Void.findByProps("chatApi"),p=m.chatApi.configuration.middleware[0].pre,r=Math.random,d=Date.now,g=crypto.subtle.digest.bind(crypto.subtle),h;Math.random=()=>0;Date.now=()=>1e12;crypto.subtle.digest=async(a,b)=>{h=new TextDecoder().decode(b);return g(a,b)};var s=await p({url:"https://grok.com/rest/app-chat/x",init:{method:"POST",headers:{}}});Math.random=r;Date.now=d;crypto.subtle.digest=g;var t=new Uint8Array([...atob(s.init.headers["x-statsig-id"])].map(c=>c.charCodeAt(0)));console.log(`CHALLENGE_HEADER_HEX=${[...t.slice(0,49)].map(b=>b.toString(16).padStart(2,"0")).join("")}\nCHALLENGE_SUFFIX=${h.split("!").slice(2).join("!").replace(/^-?\d+/,"")}\nCHALLENGE_TRAILER=${t[69]}`)
```

Copy the three output lines into `.env`. Re-run the snippet whenever Grok ships a new build.

<details>
<summary>Without Void (manual extraction)</summary>

1. DevTools Network tab on grok.com, pick any POST carrying `x-statsig-id`
2. Base64-decode the header into 70 raw bytes
3. Bytes 0 through 48 (hex-encoded) become `CHALLENGE_HEADER_HEX`
4. Byte 69 becomes `CHALLENGE_TRAILER`
5. Breakpoint the middleware that calls `crypto.subtle.digest`, grab the string passed in, take everything after the second `!`, strip the leading counter digits. That's `CHALLENGE_SUFFIX`

</details>

### 3. Run

```sh
cargo run --release
```

Or Docker:

```sh
docker compose up -d
```

Server binds `http://0.0.0.0:3000` by default.

## Models

Five modes map to the options grok.com exposes in its UI.

| Model ID | UI label | Notes |
|----------|----------|-------|
| `auto` | Auto | Picks Fast or Expert per query |
| `fast` | Fast | Quick responses |
| `expert` | Expert | Deeper reasoning |
| `heavy` | Heavy | Multi-agent orchestration. Requires Heavy plan |
| `grok-4-3` | Grok 4.3 | Early access. Remaps to upstream `grok-420-computer-use-sa` so requests debit the 4.3 rate-limit bucket |

Unknown model IDs fall back to `auto` with a debug log. The server does not expose `grok-2`, `grok-3`, `grok-4`, `grok-4-mini`, or other legacy names because the web client no longer routes to them.

## OpenAI-Compatible API

### Non-streaming

```sh
curl http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"auto","messages":[{"role":"user","content":"hello"}]}'
```

### Streaming

```sh
curl -N http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"auto","stream":true,"messages":[{"role":"user","content":"hello"}]}'
```

SSE format matches OpenAI chunks. Final event is `data: [DONE]`.

### Python

```python
from openai import OpenAI

client = OpenAI(base_url="http://localhost:3000/v1", api_key="unused")
resp = client.chat.completions.create(
    model="auto",
    messages=[{"role": "user", "content": "hello"}],
)
print(resp.choices[0].message.content)
```

Any OpenAI-shaped client works: LiteLLM, Open WebUI, Cursor, Continue, aider, llm, etc.

### Ignored Request Fields

Grok's web API has no equivalent for these, so the server accepts and drops them without error:

- `temperature`
- `top_p`
- `max_tokens`
- `max_completion_tokens`
- `response_format`

### Tool Calling

Implemented as a prompted protocol. The server injects a system block describing the tools and a required XML call format, then parses `<tool_call>{...}</tool_call>` out of Grok's output and re-emits them as OpenAI `tool_calls`.

Supported request fields:

- `tools` (array of `{"type":"function","function":{...}}`)
- `tool_choice` (`"auto"`, `"none"`, `"required"`, or `{"type":"function","function":{"name":"X"}}`)
- `messages[].tool_calls` on assistant turns
- `messages[].tool_call_id` on `role:"tool"` turns

Supported response fields:

- `choices[0].message.tool_calls` in non-streaming mode
- `choices[0].delta.tool_calls` in streaming mode
- `finish_reason:"tool_calls"` when a call was emitted

Reliability notes based on live testing:

- `expert` with `tool_choice:"required"` is the most reliable path
- `auto` under default `tool_choice` often refuses to call tools and answers the user directly
- `tool_choice:"none"` skips the tool-block injection entirely

Example request:

```sh
curl http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "expert",
    "messages": [{"role": "user", "content": "What is the weather in Paris?"}],
    "tools": [{
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get current weather for a city",
        "parameters": {
          "type": "object",
          "properties": {"city": {"type": "string"}},
          "required": ["city"]
        }
      }
    }],
    "tool_choice": "required"
  }'
```

Example response:

```json
{
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "tool_calls": [{
        "id": "call_...",
        "type": "function",
        "function": {"name": "get_weather", "arguments": "{\"city\":\"Paris\"}"}
      }]
    },
    "finish_reason": "tool_calls"
  }]
}
```

Second turn feeds the result back:

```json
{
  "model": "expert",
  "messages": [
    {"role": "user", "content": "What is the weather in Paris?"},
    {"role": "assistant", "content": "",
     "tool_calls": [{"id": "call_1", "type": "function",
       "function": {"name": "get_weather", "arguments": "{\"city\":\"Paris\"}"}}]},
    {"role": "tool", "tool_call_id": "call_1",
     "content": "{\"temp_c\":18,\"conditions\":\"partly cloudy\"}"}
  ]
}
```

The server renders tool-call and tool-result history back into Grok's turn as text, so multi-step agent loops work.

### Usage Counts

`prompt_tokens`, `completion_tokens`, and `total_tokens` are always `0`. Grok's web API does not expose token counts.

## OpenClaw Support

[OpenClaw](https://openclaw.ai) is an open-source local agent runtime. Point it at this server as an OpenAI-compatible provider and Grok drives OpenClaw's full tool surface (shell, browser, filesystem, 50+ integrations) on your real host.

Add to `~/.openclaw/openclaw.json`:

```json5
{
  models: {
    providers: {
      grok_web: {
        baseUrl: "http://localhost:3000/v1",
        apiKey: "unused",
        api: "openai-completions",
        models: [
          { id: "auto",     name: "Grok Auto",    contextWindow: 131072, maxTokens: 8192 },
          { id: "fast",     name: "Grok Fast",    contextWindow: 131072, maxTokens: 8192 },
          { id: "expert",   name: "Grok Expert",  contextWindow: 131072, maxTokens: 8192 },
          { id: "heavy",    name: "Grok Heavy",   contextWindow: 131072, maxTokens: 8192 },
          { id: "grok-4-3", name: "Grok 4.3",     contextWindow: 131072, maxTokens: 8192 },
        ],
      },
    },
  },
}
```

Notes:

- `apiKey` is a placeholder. It only matters if you set `API_KEY` in this server's `.env`. Otherwise OpenClaw does not authenticate against this server and the real auth happens through your grok.com cookies.
- `contextWindow` and `maxTokens` depend on your grok.com plan. 131072 is a safe default for SuperGrok. Free tier is lower, Heavy is higher.
- For agent loops, prefer `expert`. Tell OpenClaw to use `tool_choice:"required"` when it needs a forced tool call.

## Native API

Everything under `/v1/*` not covered by the OpenAI surface is a typed wrapper around grok.com's internal REST endpoints.

```sh
curl -X POST http://localhost:3000/v1/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"hello","temporary":true}'

curl http://localhost:3000/v1/conversations
curl http://localhost:3000/v1/models
```

## Endpoints

#### OpenAI

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/chat/completions` | OpenAI-compatible chat, tools, streaming |
| `GET` | `/v1/models` | List supported models |
| `GET` | `/v1/models/:id` | Get a model |

#### Chat

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/chat` | New conversation (streaming) |
| `POST` | `/v1/chat/quick` | Quick answer without persisting a conversation |
| `POST` | `/v1/chat/:id/message` | Continue a conversation |
| `POST` | `/v1/chat/:id/stop` | Stop generation |
| `POST` | `/v1/chat/:id/cancel` | Cancel a response |
| `GET` | `/v1/chat/:id/reconnect` | Reconnect to an in-flight response |

#### Conversations

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/conversations` | List conversations |
| `DELETE` | `/v1/conversations` | Delete all conversations |
| `GET` | `/v1/conversations/deleted` | List soft-deleted |
| `GET` | `/v1/conversations/:id` | Get a conversation |
| `PUT` | `/v1/conversations/:id` | Update a conversation |
| `DELETE` | `/v1/conversations/:id` | Delete a conversation |
| `GET` | `/v1/conversations/:id/exists` | Check existence |
| `POST` | `/v1/conversations/:id/restore` | Restore deleted |
| `POST` | `/v1/conversations/:id/title` | Generate title |
| `GET` | `/v1/conversations/:id/responses` | List responses |
| `GET` | `/v1/conversations/:id/artifacts` | Get artifact metadata |

#### Files and Code

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/files` | Upload file (max 64 MB) |
| `GET` | `/v1/files/:id/metadata` | Get file metadata |
| `POST` | `/v1/code/run` | Execute code |

#### Voice

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/voice/read/:id` | TTS stream |
| `GET` | `/v1/voice/audio/:id` | TTS audio file |
| `POST` | `/v1/voice/tts` | Text-to-speech |
| `POST` | `/v1/voice/livekit/token` | Issue LiveKit JWT for realtime voice against `wss://livekit.grok.com` |

Realtime voice flow: call `POST /v1/voice/livekit/token`, take the returned `token`, connect any [LiveKit SDK](https://docs.livekit.io/) to `wss://livekit.grok.com` with it. The server-assigned room auto-admits the `prod` voice agent, which subscribes to your mic track and publishes synthesized audio back. The server proxies only the token issuance; media flows directly from client to LiveKit over WebRTC.

```bash
curl -X POST http://localhost:3000/v1/voice/livekit/token \
  -H "Authorization: Bearer $GROK_API_KEY"
```

#### Memory and Artifacts

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/memory/blurb` | Memory summary |
| `GET` | `/v1/memory/v2/:id` | Get memory |
| `PUT` | `/v1/memory/v2/:id` | Update memory |
| `DELETE` | `/v1/memory/v2/:id` | Delete memory |
| `DELETE` | `/v1/memory/v2/all/:companion_id` | Delete all memories for a companion |
| `GET` | `/v1/artifacts/:id` | Get artifact |
| `PUT` | `/v1/artifacts/:id` | Update artifact |
| `GET` | `/v1/artifacts/:id/content/:version_id` | Get artifact content |

#### Sharing

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/sharing/:id` | Share a conversation |
| `POST` | `/v1/sharing/:id/artifact` | Share an artifact |
| `GET` | `/v1/sharing/links` | List share links |
| `GET` | `/v1/sharing/links/:id` | Get share link |
| `DELETE` | `/v1/sharing/links/:id` | Delete share link |
| `POST` | `/v1/sharing/links/:id/clone` | Clone share link |
| `GET` | `/v1/sharing/artifacts/:id` | Get shared artifact |

#### Discovery

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/suggestions` | Search suggestions |
| `GET` | `/v1/suggestions/starters` | Conversation starters |
| `POST` | `/v1/suggestions/follow-up` | Follow-up suggestions |
| `GET` | `/v1/images` | List image generations |

#### Google Drive

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/google-drive/files` | List files |
| `GET` | `/v1/google-drive/files/:id` | Read file |

#### System

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Liveness check |
| `GET` | `/health/session` | Cookie validity |
| `GET` | `/status` | Request counters |
| `GET` | `/setup` | Challenge extraction helper |
| `ANY` | `/raw/*` | Raw passthrough to the Grok web API |

## Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `GROK_SSO_COOKIE` | yes | | `sso` cookie |
| `GROK_SSO_RW_COOKIE` | yes | | `sso-rw` cookie |
| `CHALLENGE_HEADER_HEX` | yes | | 49-byte anti-bot header, hex |
| `CHALLENGE_SUFFIX` | yes | | Anti-bot suffix string |
| `CHALLENGE_TRAILER` | no | `3` | Anti-bot trailer byte |
| `GROK_EXTRA_COOKIES` | no | | Extra cookies to forward (rarely needed) |
| `GROK_BASE_URL` | no | `https://grok.com` | Override upstream base |
| `TOKEN_PROVIDER_URL` | no | | External HTTP token provider instead of in-process challenge |
| `API_KEY` | no | | Bearer token required by this server's clients |
| `HOST` | no | `0.0.0.0` | Bind address |
| `PORT` | no | `3000` | Listen port |
| `SESSION_CHECK_INTERVAL_SECS` | no | `60` | Background cookie validity poll, minimum 30 |
| `LOG_LEVEL` | no | `info` | Log level filter |

## Errors

Every error response is RFC 7807 JSON:

```json
{"type":"bad_request","title":"Bad Request","status":400,"detail":"messages must contain at least one non-system message"}
```

Common types:

- `bad_request` (400): malformed input
- `unauthorized` (401): missing or wrong `API_KEY`
- `not_found` (404): unknown route or resource
- `auth_expired` (503): grok.com session cookies expired
- `upstream_error` (502): Grok returned an error

Streaming errors surface as SSE events with shape `{"error":{"message":"...","type":"..."}}`.

## How the Anti-Bot Bypass Works

Grok's web client runs obfuscated JS that generates a per-request `x-statsig-id`. Requests without it get rejected.

Token layout (70 bytes, base64-encoded):

```
header[49] + counter_le32[4] + sha256(method + "!" + path + "!" + counter + suffix)[0..16] + trailer[1]
```

Every byte is then XOR'd with a random key.

- `header`: 49-byte static fingerprint extracted once from the browser
- `counter`: seconds since a hardcoded epoch (May 1, 2023)
- `suffix`: static string baked into the challenge JS
- `trailer`: single constant byte

This server rebuilds that exact algorithm in ~60 lines of Rust with `sha2`. Tokens are cryptographically identical to browser output. No JS engine, no eval, no WebDriver.

On top, [wreq](https://github.com/0x676e67/wreq) (a reqwest fork using BoringSSL) produces a TLS fingerprint matching Chrome. Cloudflare JA3 and JA4 checks pass without a real browser.

## Using as a Library

`grok-client` works standalone.

```toml
[dependencies]
grok-client = { git = "https://github.com/imjustprism/grok-web-api.git" }
futures = "0.3"
tokio = { version = "1", features = ["full"] }
```

```rust
use futures::StreamExt;
use grok_client::{ChallengeConfig, GrokAuth, GrokClient};
use grok_client::types::chat::NewConversationRequest;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let auth = GrokAuth::new("sso_cookie", "sso_rw_cookie")?;
    let challenge = ChallengeConfig::new("header_hex", "suffix", 3)?;
    let client = GrokClient::new(auth)?.with_token_provider(challenge);

    let mut request = NewConversationRequest::new("Explain the Raft consensus algorithm");
    request.temporary = Some(true);

    let mut stream = client.create_conversation(&request).await?;
    while let Some(chunk) = stream.next().await {
        let _chunk = chunk?;
    }
    Ok(())
}
```

## Architecture

```
crates/
  grok-client/    Typed HTTP client, challenge token generator, streaming parser
  grok-server/    Axum REST API with OpenAI compatibility layer and tool-calling bridge
```

Split so `grok-client` is usable without pulling Axum.

## Why This One

Other wrappers either shell out to a real browser or skip the anti-bot challenge and break within days. This one solves it at the crypto layer. Single ~5 MB binary, full endpoint coverage, no runtime dependencies.

## Disclaimer

Reverse-engineers Grok's internal web API. Not affiliated with or endorsed by xAI. May violate xAI's Terms of Service. Use at your own risk.

## License

[MIT](LICENSE)
