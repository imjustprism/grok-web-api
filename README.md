# grok-web-api

[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)

Self-hosted REST API that wraps Grok's web interface into an OpenAI-compatible endpoint. Pure Rust, single binary, no headless browser, no JS engine, no runtime dependencies. Reverse-engineers Grok's anti-bot challenge at the cryptographic level -- requests are indistinguishable from a real browser session.

## Quick Start

Requires [Rust](https://rustup.rs/) 1.85+, NASM, and LLVM/libclang (for BoringSSL).

```sh
git clone https://github.com/imjustprism/grok-web-api.git
cd grok-web-api
cp .env.example .env
```

**Get your cookies** -- open [grok.com](https://grok.com), DevTools > Network, grab `sso` and `sso-rw` from any request's `Cookie` header:

```
GROK_SSO_COOKIE=your_sso_value
GROK_SSO_RW_COOKIE=your_sso_rw_value
```

**Get challenge values** -- the anti-bot bypass needs three static values extracted once from Grok's web client. If you have [Void](https://github.com/imjustprism/Void) installed, paste this in grok.com's console (uses top-level await + `var` so it's safe to re-run and survives accidental edge-trim on paste):

```js
var m=Void.findByProps("chatApi"),p=m.chatApi.configuration.middleware[0].pre,r=Math.random,d=Date.now,g=crypto.subtle.digest.bind(crypto.subtle),h;Math.random=()=>0;Date.now=()=>1e12;crypto.subtle.digest=async(a,b)=>{h=new TextDecoder().decode(b);return g(a,b)};var s=await p({url:"https://grok.com/rest/app-chat/x",init:{method:"POST",headers:{}}});Math.random=r;Date.now=d;crypto.subtle.digest=g;var t=new Uint8Array([...atob(s.init.headers["x-statsig-id"])].map(c=>c.charCodeAt(0)));console.log(`CHALLENGE_HEADER_HEX=${[...t.slice(0,49)].map(b=>b.toString(16).padStart(2,"0")).join("")}\nCHALLENGE_SUFFIX=${h.split("!").slice(2).join("!").replace(/^-?\d+/,"")}\nCHALLENGE_TRAILER=${t[69]}`)
```

Copy the three output lines into `.env`.

<details>
<summary>Without Void (manual extraction)</summary>

1. DevTools > Network on grok.com, find any POST with an `x-statsig-id` header
2. Base64-decode the header into raw bytes
3. Bytes 0-48 = `CHALLENGE_HEADER_HEX` (hex-encode them)
4. Byte 69 = `CHALLENGE_TRAILER`
5. For `CHALLENGE_SUFFIX`: breakpoint the challenge middleware, intercept the string passed to `crypto.subtle.digest`, extract everything after the second `!` with leading counter digits stripped

</details>

**Run:**

```sh
cargo run --release
# or
docker compose up -d
```

Server starts on `http://localhost:3000`.

## Usage

### OpenAI-compatible

```sh
curl http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"grok-3","messages":[{"role":"user","content":"hello"}],"stream":true}'
```

Works with any OpenAI client -- just change the base URL:

```python
from openai import OpenAI

client = OpenAI(base_url="http://localhost:3000/v1", api_key="unused")
response = client.chat.completions.create(
    model="grok-3",
    messages=[{"role": "user", "content": "hello"}],
)
print(response.choices[0].message.content)
```

Compatible with LiteLLM, Open WebUI, Cursor, Continue, and anything else that speaks the OpenAI API.

### Native API

```sh
curl -X POST http://localhost:3000/v1/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"hello","temporary":true}'

curl http://localhost:3000/v1/conversations
curl http://localhost:3000/v1/models
```

## Endpoints

#### Chat

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/chat/completions` | OpenAI-compatible chat |
| `POST` | `/v1/chat` | New conversation (streaming) |
| `POST` | `/v1/chat/quick` | Quick answer (no conversation) |
| `POST` | `/v1/chat/:id/message` | Continue conversation |
| `POST` | `/v1/chat/:id/stop` | Stop generation |

#### Conversations

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/conversations` | List conversations |
| `GET` | `/v1/conversations/:id` | Get conversation |
| `PUT` | `/v1/conversations/:id` | Update conversation |
| `DELETE` | `/v1/conversations/:id` | Delete conversation |
| `POST` | `/v1/conversations/:id/restore` | Restore deleted |
| `POST` | `/v1/conversations/:id/title` | Generate title |
| `GET` | `/v1/conversations/:id/responses` | List responses |

#### Media and Code

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/files` | Upload file |
| `POST` | `/v1/code/run` | Execute code |
| `GET` | `/v1/images` | List image generations |

#### Voice

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/voice/read/:id` | TTS stream |
| `GET` | `/v1/voice/audio/:id` | TTS audio file |
| `POST` | `/v1/voice/tts` | Text-to-speech |

#### Memory and Artifacts

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/memory/blurb` | Memory summary |
| `GET` | `/v1/memory/v2/:id` | Get memory |
| `PUT` | `/v1/memory/v2/:id` | Update memory |
| `DELETE` | `/v1/memory/v2/:id` | Delete memory |
| `GET` | `/v1/artifacts/:id` | Get artifact |
| `PUT` | `/v1/artifacts/:id` | Update artifact |

#### Sharing and Discovery

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/sharing/:id` | Share conversation |
| `GET` | `/v1/sharing/links` | List share links |
| `GET` | `/v1/suggestions` | Search suggestions |
| `GET` | `/v1/models` | List models |

#### System

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Server health |
| `GET` | `/health/session` | Cookie validity |
| `ANY` | `/raw/*` | Raw passthrough to Grok API |

## Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `GROK_SSO_COOKIE` | yes | | SSO cookie from grok.com |
| `GROK_SSO_RW_COOKIE` | yes | | SSO-RW cookie from grok.com |
| `CHALLENGE_HEADER_HEX` | yes | | Anti-bot header (49 bytes, hex-encoded) |
| `CHALLENGE_SUFFIX` | yes | | Anti-bot suffix string |
| `CHALLENGE_TRAILER` | no | `3` | Anti-bot trailer byte |
| `API_KEY` | no | | Protect the server with a bearer token |
| `HOST` | no | `0.0.0.0` | Bind address |
| `PORT` | no | `3000` | Listen port |
| `LOG_LEVEL` | no | `info` | Log level filter |

## How the Anti-Bot Bypass Works

Grok's web client runs obfuscated JS that generates a per-request `x-statsig-id` token. Requests without a valid token are rejected.

The token is 70 bytes, base64-encoded:

```
header[49] + counter_le32[4] + sha256(method + "!" + path + "!" + counter + suffix)[0..16] + trailer[1]
```

Then every byte is XOR'd with a random key.

- `header` -- 49-byte static fingerprint, extracted once from the browser
- `counter` -- seconds since a hardcoded epoch (May 1, 2023)
- `suffix` -- static string baked into the challenge JS
- `trailer` -- single constant byte

This server replicates that algorithm in ~60 lines of Rust using `sha2`. Tokens are cryptographically identical to what the browser generates. No JS engine, no eval, no WebDriver.

On top of that, wreq (a fork of reqwest using BoringSSL) produces a TLS fingerprint matching Chrome. This passes Cloudflare's JA3/JA4 checks without a real browser.

## Why This One?

Every other wrapper either shells out to a real browser or ignores the anti-bot challenge entirely. This is the only one that solved it at the crypto level. Single ~5MB binary, 50+ endpoints, no runtime dependencies.

## Using as a Library

The `grok-client` crate works independently of the server.

```toml
[dependencies]
grok-client = { git = "https://github.com/imjustprism/grok-web-api.git" }
tokio = { version = "1", features = ["full"] }
```

```rust
use grok_client::{GrokClient, GrokAuth, ChallengeConfig};
use grok_client::types::chat::NewConversationRequest;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let auth = GrokAuth::new("sso_cookie", "sso_rw_cookie");
    let challenge = ChallengeConfig::new("header_hex", "suffix", 3)?;
    let client = GrokClient::new(auth)?.with_token_provider(challenge);

    let request = NewConversationRequest {
        message: "Explain the Raft consensus algorithm".into(),
        temporary: true,
        ..Default::default()
    };

    let mut stream = client.create_conversation(&request).await?;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        // text deltas, attachments, metadata
    }
    Ok(())
}
```

## Architecture

```
crates/
  grok-client/    Typed HTTP client + challenge token generator
  grok-server/    Axum REST API with OpenAI compatibility layer
```

Split so `grok-client` can be used standalone without server dependencies.

## Disclaimer

This project reverse-engineers Grok's internal web API. Not affiliated with or endorsed by xAI. May violate xAI's Terms of Service. Use at your own risk.

## License

[MIT](LICENSE)
