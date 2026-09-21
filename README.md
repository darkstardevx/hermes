# 🪽 Hermes

`Rust` · `poise` · `serenity`

**Cybercore portfolio Discord bot.** Messenger-god name, messenger-god job:
relays release/CI/download status and opt-in troubleshooting guidance from the
portfolio's GitHub repos into Discord via slash commands. It can also publish
an opt-in, stateful project-update digest to `#dev-log`. No privileged gateway
intents or message reading are required.

## 🚀 Commands

```
/status               CI status (pass/fail) across every tracked repo
/releases <tool>      latest release + notes for one tool
/downloads <tool>     total download count on the latest release
/troubleshoot         private opt-in troubleshooting intake
/tools                linked preview cards for the Cybercore toolchain
/server-audit         private read-only channel layout and overlap audit
```

## 🛰️ Scheduled `#dev-log`

Set `DEV_LOG_CHANNEL_ID` to enable a background GitHub poller. Hermes seeds its
local state on first start, then posts a compact digest only when a tracked
repository receives new activity; release links are included when available.
The default interval is one hour (`DEV_LOG_INTERVAL_SECS=3600`). State is kept
in `.hermes/dev-log-state.json` by default and contains only public GitHub
timestamps and release tags. Leave the setting unset to keep this feature off.

To get the channel ID in Discord, enable **User Settings → Advanced → Developer
Mode**, then right-click `#dev-log` and choose **Copy Channel ID**. The bot
needs permission to view and send messages in that channel.

Run `/troubleshoot tool:<name> symptom:<what happened>` in `#troubleshooting`
for a private, tool-specific checklist and issue-report template. Hermes does
not read the channel or store the report. It only responds when a user
explicitly invokes the command.

Run `/tools` in `#dev-tools` to publish one clickable embed per featured project:
`cybercore`, `cyberdeck`, `diagprint`, `gateflow`, and `cybermeta`.

Run `/server-audit` from a server channel to get an ephemeral report of the
channel/category layout, duplicate channel names, and custom permission
overwrites. It does not read messages, message content, or member lists. The
audit only needs `View Channels`; no `Read Message History` permission is
required. Add `Embed Links` for the `/tools` preview cards. Do not grant
Administrator.

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
src/github.rs  GitHub REST API wrappers (activity, releases, workflow runs)
src/repos.rs   the tracked-repo registry -- add a tool here once it's
               public and release-pipelined
```

## 🔒 Token handling

`DISCORD_TOKEN` and `GITHUB_TOKEN` live in `.env`, which is gitignored.
Never put a real token in `.env.example`, a commit message, or anywhere
else that ends up tracked.
