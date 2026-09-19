# 🪽 Hermes

`Rust` · `poise` · `serenity`

**Cybercore portfolio Discord bot.** Messenger-god name, messenger-god job:
relays release/CI/download status from the portfolio's GitHub repos into
Discord via slash commands. No privileged gateway intents, no message
reading -- slash commands only.

## 🚀 Commands

```
/status               CI status (pass/fail) across every tracked repo
/releases <tool>      latest release + notes for one tool
/downloads <tool>     total download count on the latest release
```

## 📦 Setup

```bash
cp .env.example .env   # then paste in your real DISCORD_TOKEN
cargo run
```

`GITHUB_TOKEN` in `.env` is optional -- raises the unauthenticated GitHub
API rate limit (60/hr -> 5000/hr) and lets `/status`/`/releases` reach
private repos too. Without it, everything still works for public repos
at the lower rate limit.

## 🧩 Layout

```
src/main.rs    slash command definitions + bot bootstrap
src/github.rs  GitHub REST API wrappers (releases, workflow runs)
src/repos.rs   the tracked-repo registry -- add a tool here once it's
               public and release-pipelined
```

## 🔒 Token handling

`DISCORD_TOKEN` and `GITHUB_TOKEN` live in `.env`, which is gitignored.
Never put a real token in `.env.example`, a commit message, or anywhere
else that ends up tracked.
