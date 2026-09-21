# Hermes Watch roadmap

Hermes is the Cybercore messenger and control plane: highly capable, deliberately narrow, and useful without becoming invasive.

## Product law

> Maximum usefulness. Minimum access.

Hermes keeps privileged message-content access disabled. Interactive features use slash commands, buttons, modals, webhooks, and explicit subscriptions. Sensitive responses are ephemeral. Hermes never executes arbitrary shell commands from Discord and never stores secrets or raw user diagnostics by default.

## Capability map

### 1. Herald — project updates

- [x] Stateful #dev-log activity digest
- [x] Release and CI lookup commands
- [x] Per-project subscriptions with /watch and /unwatch
- [ ] Scheduled weekly portfolio digest
- [ ] GitHub webhooks with polling fallback

### 2. Oracle — project intelligence

- [x] /project <tool> live project card
- [x] /projects project directory
- [ ] Open issue and pull-request summaries
- [ ] What changed since last release comparison
- [ ] Dependency and security signal summaries

### 3. Gatekeeper — server operations

- [x] Read-only channel and permission audit
- [ ] Configurable role/channel policy checks
- [ ] Setup diagnostics and least-privilege recommendations
- [ ] Maintenance and incident announcement helpers

### 4. Guide — troubleshooting

- [x] Private opt-in troubleshooting intake
- [x] Tool-specific first checks and issue templates
- [ ] Modal-based structured intake
- [ ] Sanitized attachment and log guidance
- [ ] Follow-up issue draft generation

### 5. Courier — notifications

- [ ] Role-based notification subscriptions
- [ ] CI failure and release alerts
- [ ] Quiet hours and notification preferences
- [ ] Optional DM delivery, off by default

### 6. Archivist — documentation

- [ ] /docs <tool> project-page and docs.rs lookup
- [ ] README and release-note search
- [ ] Install and download guidance
- [ ] Contextual links in every project response

### 7. Armory — extensibility

- [ ] SQLite-backed state and migrations
- [ ] Domain services separated from Discord commands
- [ ] GitHub, Discord, and future adapter interfaces
- [ ] Event/audit stream for bot actions
- [ ] Configuration validation and startup diagnostics

### 8. Dashboard — operations surface

- [x] Dedicated Hermes Watch GitHub Page
- [x] Lightweight health, uptime, and configuration summary
- [ ] Read-only server/project dashboard
- [ ] Health, rate-limit, and delivery metrics
- [ ] Operator configuration view without message access

## Delivery order

1. Foundation: configuration, persistence, rate-limit handling, structured errors, and module boundaries.
2. Project intelligence: /project, /projects, documentation lookup, and richer release/CI context.
3. Event delivery: webhook ingestion, subscriptions, dev-log routing, and notification preferences.
4. Support and operations: modal troubleshooting, issue drafts, policy checks, and audit history.
5. Extensibility: adapters, SQLite migrations, dashboard data, and a stable 1.0 contract.

Every milestone must preserve the privacy boundary and ship with an explicit data-retention note.
