# Contributing to Tubefast

Thanks for wanting to help. Bug reports, fixes and features are all welcome.

## Before you start

- For a bug, open an issue with the steps, what you expected and what happened. The output of `tubefast --selftest` helps a lot when playback is involved.
- For a feature or a visual change, open an issue first so we can agree on the shape before you spend time on it.

## Working on the code

You need Rust 1.90 or newer.

```sh
cargo run                      # debug build
cargo run --release            # what users get
cargo run -- --profile dev     # separate library and sign-in, leaves your own untouched
cargo run -- --selftest        # search, stream, decode and seek against YouTube, no window
```

Before opening a pull request, make sure these pass. CI runs the same three:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

`cargo test -- --include-ignored` also drives the sign-in flow through a real headless Edge or Chrome.

## Where things live

| File | What it does |
|---|---|
| `src/ytm.rs` | The YouTube client: requests, response parsing, stream lookup. Client versions and other values YouTube may change sit at the top. |
| `src/player.rs` | Progressive download, decoding thread and audio output. |
| `src/auth.rs` | Sign-in: cookie handling, request signing, encrypted storage, the browser flow. |
| `src/art.rs` | Artwork download, decode and texture cache with a memory budget. |
| `src/app.rs` | Application state. Applies the actions the interface emits. |
| `src/ui.rs` | Everything drawn on screen. Emits actions, never changes state itself. |

## Ground rules

- **Nothing blocks the interface.** Network and decoding work happens on background threads and reports back through events.
- **Stay small.** No browser engine, no telemetry. Think twice before adding a dependency. Memory and startup time are features, so measure them when you touch caching or rendering.
- **Names over comments.** The codebase carries intent in names rather than comments. Please follow suit.
- **Custom drawn interface.** Default egui widgets look out of place here. Draw new controls in the style of the existing ones and add a screenshot to your pull request.
- **One topic per pull request**, with a short description of what changed and how you checked it.

## When YouTube breaks something

Playback or browsing can stop working when YouTube changes its internal API. Run `tubefast --selftest` to see which step fails. The fix is usually an updated client name, version or user agent in the constants at the top of `src/ytm.rs`. The [yt-dlp](https://github.com/yt-dlp/yt-dlp) project tracks the working values in `yt_dlp/extractor/youtube/_base.py`.
