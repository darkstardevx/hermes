//! Hermes -- messenger-god bot for the Cybercore portfolio. Slash commands
//! only (no privileged gateway intents needed): /status, /releases,
//! /downloads, each a thin wrapper over GitHub's REST API.

mod github;
mod repos;

use anyhow::Context as _;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct Data {
    http: reqwest::Client,
    github_token: Option<String>,
    started_at: Instant,
    dev_log_enabled: bool,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Every public repo's CI status on its default branch, one line each.
#[poise::command(slash_command)]
async fn status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let results = github::all_ci_status(&ctx.data().http, ctx.data().github_token.as_deref()).await;

    let mut lines = Vec::with_capacity(results.len());
    for (repo, outcome) in &results {
        let icon = match outcome {
            Ok(true) => "\u{2705}",
            Ok(false) => "\u{274c}",
            Err(_) => "\u{2753}",
        };
        lines.push(format!("{icon} `{repo}`"));
    }

    let embed = serenity::CreateEmbed::new()
        .title("Cybercore portfolio -- CI status")
        .description(lines.join("\n"))
        .color(0x4d_a6_ff);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Lightweight operational health summary for the always-on deployment.
#[poise::command(slash_command)]
async fn health(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let uptime = data.started_at.elapsed().as_secs();
    let hours = uptime / 3_600;
    let minutes = (uptime % 3_600) / 60;
    let seconds = uptime % 60;
    let github_mode = if data.github_token.is_some() {
        "authenticated GitHub API"
    } else {
        "unauthenticated public GitHub API"
    };
    let dev_log = if data.dev_log_enabled {
        "enabled"
    } else {
        "disabled"
    };

    let embed = serenity::CreateEmbed::new()
        .title("🪽 Hermes Watch health")
        .description("Gateway command succeeded; Hermes is online and responding.")
        .field("Uptime", format!("{hours}h {minutes}m {seconds}s"), true)
        .field("GitHub", github_mode, true)
        .field("#dev-log", dev_log, true)
        .footer(serenity::CreateEmbedFooter::new(
            "Hermes Watch • minimum access • no message reading",
        ))
        .color(0x35_d07f);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Latest release for one tool.
#[poise::command(slash_command)]
async fn releases(
    ctx: Context<'_>,
    #[description = "Tool name, e.g. cybervault"] tool: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let Some(repo) = repos::resolve(&tool) else {
        ctx.say(format!(
            "Unknown tool `{tool}`. Try one of: {}",
            repos::names_list()
        ))
        .await?;
        return Ok(());
    };

    match github::latest_release(&ctx.data().http, repo, ctx.data().github_token.as_deref()).await {
        Ok(Some(rel)) => {
            let embed = serenity::CreateEmbed::new()
                .title(format!("{repo} {}", rel.tag_name))
                .url(rel.html_url)
                .description(rel.body.unwrap_or_else(|| "_(no release notes)_".into()))
                .color(0x4d_a6_ff);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Ok(None) => {
            ctx.say(format!("`{repo}` has no releases yet.")).await?;
        }
        Err(e) => {
            ctx.say(format!("Couldn't reach GitHub for `{repo}`: {e}"))
                .await?;
        }
    }
    Ok(())
}

/// Portfolio directory with live release and CI context for one project.
#[poise::command(slash_command, rename = "project")]
async fn project(
    ctx: Context<'_>,
    #[description = "Project name, e.g. gateflow or agentforge"] tool: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let tool_name = tool.trim().to_ascii_lowercase();
    let Some(repo) = repos::resolve(&tool_name) else {
        ctx.say(format!(
            "Unknown project {tool}. Try one of: {}",
            repos::names_list()
        ))
        .await?;
        return Ok(());
    };

    let release =
        github::latest_release(&ctx.data().http, repo, ctx.data().github_token.as_deref())
            .await
            .ok()
            .flatten();
    let ci =
        github::ci_status_for(&ctx.data().http, repo, ctx.data().github_token.as_deref()).await;
    let ci_summary = match ci {
        Ok(true) => "✅ Passing".to_owned(),
        Ok(false) => "❌ Not passing".to_owned(),
        Err(_) => "❔ Unavailable".to_owned(),
    };
    let release_summary = match release.as_ref() {
        Some(release) => format!("[{}]({})", release.tag_name, release.html_url),
        None => "No published release".to_owned(),
    };
    let project_link = repos::site(&tool_name)
        .map(|url| format!("[Project page]({url})"))
        .unwrap_or_else(|| "No project page published yet".to_owned());

    let embed = serenity::CreateEmbed::new()
        .title(format!("🛰️ {tool_name}"))
        .url(format!("https://github.com/{repo}"))
        .description(repos::description(&tool_name))
        .field("Latest release", release_summary, true)
        .field("CI signal", ci_summary, true)
        .field(
            "Links",
            format!("[GitHub](https://github.com/{repo})\n{project_link}"),
            false,
        )
        .footer(serenity::CreateEmbedFooter::new(
            "Hermes Watch • GitHub read-only • no message reading",
        ))
        .color(0x16_dc_ff);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Compact directory of every project Hermes can currently understand.
#[poise::command(slash_command, rename = "projects")]
async fn projects(ctx: Context<'_>) -> Result<(), Error> {
    let lines = repos::all_named()
        .map(|(name, repo)| {
            format!(
                "• **{name}** — {}\n  https://github.com/{repo}",
                repos::description(name)
            )
        })
        .collect::<Vec<_>>();
    let embed = serenity::CreateEmbed::new()
        .title("🗺️ Cybercore project directory")
        .description(lines.join("\n"))
        .footer(serenity::CreateEmbedFooter::new(
            "Use /project <name> for live release and CI context",
        ))
        .color(0x95_64_ff);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Total download count across a tool's latest release assets.
#[poise::command(slash_command)]
async fn downloads(
    ctx: Context<'_>,
    #[description = "Tool name, e.g. cybervault"] tool: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let Some(repo) = repos::resolve(&tool) else {
        ctx.say(format!(
            "Unknown tool `{tool}`. Try one of: {}",
            repos::names_list()
        ))
        .await?;
        return Ok(());
    };

    match github::latest_release(&ctx.data().http, repo, ctx.data().github_token.as_deref()).await {
        Ok(Some(rel)) => {
            let total: u64 = rel
                .assets
                .iter()
                .filter(|a| !a.name.ends_with(".sha256"))
                .map(|a| a.download_count)
                .sum();
            ctx.say(format!("`{repo}` {}: **{total}** downloads", rel.tag_name))
                .await?;
        }
        Ok(None) => {
            ctx.say(format!("`{repo}` has no releases yet.")).await?;
        }
        Err(e) => {
            ctx.say(format!("Couldn't reach GitHub for `{repo}`: {e}"))
                .await?;
        }
    }
    Ok(())
}

/// Private, opt-in troubleshooting intake for one Cybercore tool.
#[poise::command(slash_command, rename = "troubleshoot", guild_only)]
async fn troubleshoot(
    ctx: Context<'_>,
    #[description = "Tool name, e.g. gateflow or agentforge"] tool: String,
    #[description = "Short description of what went wrong"] symptom: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let tool_name = tool.trim().to_ascii_lowercase();
    let Some(repo) = repos::resolve(&tool_name) else {
        ctx.send(
            poise::CreateReply::default()
                .content(format!(
                    "Unknown tool `{tool}`. Try one of: {}",
                    repos::names_list()
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    let (family, first_check) = troubleshooting_guidance(&tool_name);
    let symptom_summary = if symptom.trim().is_empty() {
        "No symptom summary was provided.".to_owned()
    } else {
        symptom.trim().chars().take(180).collect()
    };
    let embed = serenity::CreateEmbed::new()
        .title(format!("🧭 Troubleshooting: {tool_name}"))
        .url(format!("https://github.com/{repo}"))
        .description(format!(
            "Private triage started for **{tool_name}**. Hermes does not store this report.\n\n**Symptom noted:** {symptom_summary}"
        ))
        .field(
            "1. First checks",
            format!(
                "• {first_check}\n• Confirm the exact version or commit.\n• Reproduce with the smallest safe example.\n• Check the latest release notes with `/releases {tool_name}`."
            ),
            false,
        )
        .field(
            "2. Include in an issue",
            "• OS and architecture\n• Tool version or commit\n• Exact command/config used\n• Expected versus actual behavior\n• Relevant logs or a short traceback\n• A minimal reproduction, if possible",
            false,
        )
        .field(
            "3. Protect your data",
            "Never post passwords, API keys, private keys, tokens, secret files, customer data, or complete production configuration. Redact first.",
            false,
        )
        .field("Area", family, true)
        .field(
            "Next step",
            format!("Open an issue at https://github.com/{repo}/issues after redacting sensitive data."),
            false,
        )
        .color(0xff_7f_41)
        .footer(serenity::CreateEmbedFooter::new(
            "Hermes Watch • opt-in troubleshooting • no message reading",
        ));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}

fn troubleshooting_guidance(tool: &str) -> (&'static str, &'static str) {
    match tool {
        "agentforge" => (
            "AI-assisted workflows",
            "Run `forge doctor`, verify the project root is clean, and inspect the task contract, capabilities, gates, and audit evidence.",
        ),
        "cybervault" | "keysmith" => (
            "Crypto & secrets",
            "Reproduce with synthetic test data only and record the algorithm/version; never include a real secret or private key.",
        ),
        "cyberterm" | "cyberplug" | "cyberplug-bar-widget" | "apexdaemon" => (
            "System & terminal",
            "Record your OS, desktop/session, hardware, and whether the issue survives a clean configuration.",
        ),
        "wraithflow" | "echo" | "aetherscope" | "sentrygrid" | "vortexwall" | "ghostport"
        | "gateflow" => (
            "Network & sandboxing",
            "Record the interface/topology, kernel, namespace or firewall context, and a sanitized command/output reproduction.",
        ),
        _ => (
            "Cybercore tool",
            "Confirm the installation path, version, platform, and smallest reproducible command before collecting logs.",
        ),
    }
}

/// Linked preview cards for the Cybercore tools shared in #dev-tools.
#[poise::command(slash_command)]
async fn tools(ctx: Context<'_>) -> Result<(), Error> {
    let mut reply = poise::CreateReply::default().content(
        "**:tools: Dev & Core**\nShared design system and toolchain — open a card for the project site.",
    );

    for (name, description, site) in repos::featured() {
        let embed = serenity::CreateEmbed::new()
            .title(name)
            .url(site)
            .description(description)
            .color(0x8b_5c_f6)
            .footer(serenity::CreateEmbedFooter::new(
                "#dev-tools • darkstar_dev",
            ));
        reply = reply.embed(embed);
    }

    ctx.send(reply).await?;
    Ok(())
}

/// Read-only channel layout and permission-overwrite audit for the current server.
#[poise::command(slash_command, rename = "server-audit", guild_only)]
async fn server_audit(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let Some(guild_id) = ctx.guild_id() else {
        return Ok(());
    };

    let channels = guild_id.channels(ctx.http()).await?;
    let mut categories: Vec<&serenity::GuildChannel> = channels
        .values()
        .filter(|channel| channel.kind == serenity::ChannelType::Category)
        .collect();
    categories.sort_by_key(|channel| (channel.position, channel.name.to_ascii_lowercase()));

    let category_names: BTreeMap<serenity::ChannelId, &str> = channels
        .values()
        .filter(|channel| channel.kind == serenity::ChannelType::Category)
        .map(|channel| (channel.id, channel.name.as_str()))
        .collect();

    let mut channels_by_name: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for channel in channels
        .values()
        .filter(|channel| channel.kind != serenity::ChannelType::Category)
    {
        let parent = channel
            .parent_id
            .and_then(|id| category_names.get(&id).copied())
            .unwrap_or("Uncategorized");
        channels_by_name
            .entry(channel.name.trim().to_ascii_lowercase())
            .or_default()
            .push(format!("{parent} / #{}", channel.name));
    }

    let custom_overwrite_channels: Vec<&serenity::GuildChannel> = channels
        .values()
        .filter(|channel| !channel.permission_overwrites.is_empty())
        .collect();
    let role_overwrites: usize = custom_overwrite_channels
        .iter()
        .flat_map(|channel| channel.permission_overwrites.iter())
        .filter(|overwrite| matches!(overwrite.kind, serenity::PermissionOverwriteType::Role(_)))
        .count();
    let member_overwrites: usize = custom_overwrite_channels
        .iter()
        .flat_map(|channel| channel.permission_overwrites.iter())
        .filter(|overwrite| matches!(overwrite.kind, serenity::PermissionOverwriteType::Member(_)))
        .count();

    let mut type_counts = BTreeMap::<&str, usize>::new();
    for channel in channels.values() {
        *type_counts.entry(channel.kind.name()).or_default() += 1;
    }

    let mut report = String::new();
    writeln!(report, "Hermes Watch server audit — guild `{guild_id}`").unwrap();
    writeln!(report, "Channels: {} total", channels.len()).unwrap();
    writeln!(
        report,
        "Types: {}",
        type_counts
            .iter()
            .map(|(kind, count)| format!("{kind}={count}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
    .unwrap();
    writeln!(
        report,
        "Custom permission overwrites: {} channels ({} role, {} member)",
        custom_overwrite_channels.len(),
        role_overwrites,
        member_overwrites
    )
    .unwrap();

    writeln!(report, "\nChannel layout:").unwrap();
    for category in categories {
        writeln!(report, "📁 {}", category.name).unwrap();
        let mut children: Vec<&serenity::GuildChannel> = channels
            .values()
            .filter(|channel| channel.parent_id == Some(category.id))
            .collect();
        children.sort_by_key(|channel| (channel.position, channel.name.to_ascii_lowercase()));
        if children.is_empty() {
            writeln!(report, "  (empty)").unwrap();
        } else {
            for child in children {
                let marker = if child.kind == serenity::ChannelType::Voice {
                    "🔊"
                } else {
                    "#"
                };
                let overwrite_note = if child.permission_overwrites.is_empty() {
                    String::new()
                } else {
                    format!(" [{} overwrites]", child.permission_overwrites.len())
                };
                writeln!(report, "  {marker} {}{overwrite_note}", child.name).unwrap();
            }
        }
    }

    let mut uncategorized: Vec<&serenity::GuildChannel> = channels
        .values()
        .filter(|channel| {
            channel.kind != serenity::ChannelType::Category
                && channel
                    .parent_id
                    .and_then(|id| category_names.get(&id))
                    .is_none()
        })
        .collect();
    uncategorized.sort_by_key(|channel| (channel.position, channel.name.to_ascii_lowercase()));
    if !uncategorized.is_empty() {
        writeln!(report, "📂 Uncategorized").unwrap();
        for channel in uncategorized {
            writeln!(report, "  # {}", channel.name).unwrap();
        }
    }

    let duplicate_names: Vec<String> = channels_by_name
        .into_iter()
        .filter(|(_, locations)| locations.len() > 1)
        .map(|(name, locations)| format!("`{name}` → {}", locations.join(", ")))
        .collect();
    writeln!(report, "\nPotential overlaps:").unwrap();
    if duplicate_names.is_empty() {
        writeln!(report, "✅ No duplicate channel names detected.").unwrap();
    } else {
        for duplicate in duplicate_names {
            writeln!(report, "⚠️ {duplicate}").unwrap();
        }
    }
    if custom_overwrite_channels.is_empty() {
        writeln!(
            report,
            "✅ No channel-specific permission overwrites detected."
        )
        .unwrap();
    } else {
        writeln!(
            report,
            "ℹ️ Review channels marked with `[N overwrites]`; custom rules can create access overlap."
        )
        .unwrap();
    }
    writeln!(
        report,
        "\nNo messages, message content, or member lists were read."
    )
    .unwrap();

    for (index, chunk) in split_report(&report, 1_800).into_iter().enumerate() {
        let heading = if index == 0 {
            String::new()
        } else {
            "Audit continued:\n".to_owned()
        };
        ctx.send(
            poise::CreateReply::default()
                .content(format!("{heading}```text\n{chunk}\n```"))
                .ephemeral(true),
        )
        .await?;
    }
    Ok(())
}

fn split_report(report: &str, max_bytes: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for line in report.lines() {
        let line_len = line.len() + usize::from(!current.is_empty());
        if !current.is_empty() && current.len() + line_len > max_bytes {
            chunks.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
    }

    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

#[derive(Default, Deserialize, Serialize)]
struct DevLogState {
    repos: BTreeMap<String, DevLogRepoState>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct DevLogRepoState {
    pushed_at: Option<String>,
    release_tag: Option<String>,
}

fn load_dev_log_state(path: &Path) -> DevLogState {
    match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => DevLogState::default(),
    }
}

fn save_dev_log_state(path: &Path, state: &DevLogState) {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        if let Err(error) = std::fs::create_dir_all(parent) {
            eprintln!("dev-log: could not create state directory: {error}");
            return;
        }
    }

    match serde_json::to_string_pretty(state) {
        Ok(contents) => {
            if let Err(error) = std::fs::write(path, contents) {
                eprintln!("dev-log: could not save state: {error}");
            }
        }
        Err(error) => eprintln!("dev-log: could not serialize state: {error}"),
    }
}

/// Poll GitHub and publish a compact, stateful digest when #dev-log is enabled.
async fn dev_log_poll(
    discord_http: Arc<serenity::Http>,
    github_http: reqwest::Client,
    github_token: Option<String>,
    channel_id: serenity::ChannelId,
    interval_secs: u64,
    state_path: PathBuf,
) {
    let mut state = load_dev_log_state(&state_path);
    let mut first_run = state.repos.is_empty();

    loop {
        let mut next_repos = state.repos.clone();
        let mut changes = Vec::new();

        for repo in repos::all() {
            let activity =
                match github::repo_activity(&github_http, repo, github_token.as_deref()).await {
                    Ok(activity) => activity,
                    Err(error) => {
                        eprintln!("dev-log: GitHub activity check failed for {repo}: {error}");
                        continue;
                    }
                };

            let previous = state.repos.get(repo);
            let pushed_changed = previous.and_then(|entry| entry.pushed_at.as_deref())
                != activity.pushed_at.as_deref();

            let latest_release = if first_run || pushed_changed {
                match github::latest_release(&github_http, repo, github_token.as_deref()).await {
                    Ok(release) => release,
                    Err(error) => {
                        eprintln!("dev-log: release check failed for {repo}: {error}");
                        None
                    }
                }
            } else {
                None
            };

            let release_changed = latest_release.as_ref().is_some_and(|release| {
                previous.and_then(|entry| entry.release_tag.as_deref())
                    != Some(release.tag_name.as_str())
            });
            let release_tag = latest_release
                .as_ref()
                .map(|release| release.tag_name.clone())
                .or_else(|| previous.and_then(|entry| entry.release_tag.clone()));

            next_repos.insert(
                repo.to_owned(),
                DevLogRepoState {
                    pushed_at: activity.pushed_at.clone(),
                    release_tag,
                },
            );

            if !first_run && (pushed_changed || release_changed) {
                let mut line = format!("• **{repo}** — {}", activity.html_url);
                if let Some(release) = latest_release.filter(|_| release_changed) {
                    line.push_str(&format!(
                        " · release {}: {}",
                        release.tag_name, release.html_url
                    ));
                }
                changes.push(line);
            }
        }

        if first_run {
            state.repos = next_repos;
            save_dev_log_state(&state_path, &state);
            first_run = false;
        } else if !changes.is_empty() {
            let omitted = changes.len().saturating_sub(10);
            changes.truncate(10);
            if omitted > 0 {
                changes.push(format!("…and {omitted} more update(s)."));
            }
            let content = format!("🛰️ **Cybercore project log**\n{}", changes.join("\n"));
            match channel_id
                .send_message(
                    discord_http.as_ref(),
                    serenity::CreateMessage::new().content(content),
                )
                .await
            {
                Ok(_) => {
                    state.repos = next_repos;
                    save_dev_log_state(&state_path, &state);
                }
                Err(error) => eprintln!("dev-log: Discord post failed: {error}"),
            }
        } else {
            state.repos = next_repos;
            save_dev_log_state(&state_path, &state);
        }

        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let _ = dotenvy::dotenv();
    let token =
        std::env::var("DISCORD_TOKEN").context("DISCORD_TOKEN not set (see .env.example)")?;
    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let dev_log_channel = match std::env::var("DEV_LOG_CHANNEL_ID") {
        Ok(raw) => match raw.parse::<u64>() {
            Ok(id) => Some(serenity::ChannelId::new(id)),
            Err(error) => {
                eprintln!("dev-log: invalid DEV_LOG_CHANNEL_ID: {error}");
                None
            }
        },
        Err(_) => None,
    };
    let dev_log_interval_secs = std::env::var("DEV_LOG_INTERVAL_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(3_600)
        .max(300);
    let dev_log_state_path = PathBuf::from(
        std::env::var("DEV_LOG_STATE_FILE")
            .unwrap_or_else(|_| ".hermes/dev-log-state.json".to_owned()),
    );
    let dev_log_enabled = dev_log_channel.is_some();

    let intents = serenity::GatewayIntents::empty();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                health(),
                status(),
                releases(),
                project(),
                projects(),
                downloads(),
                troubleshoot(),
                tools(),
                server_audit(),
            ],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                if let Some(channel_id) = dev_log_channel {
                    tokio::spawn(dev_log_poll(
                        ctx.http.clone(),
                        reqwest::Client::new(),
                        github_token.clone(),
                        channel_id,
                        dev_log_interval_secs,
                        dev_log_state_path.clone(),
                    ));
                }
                Ok(Data {
                    http: reqwest::Client::new(),
                    github_token,
                    started_at: Instant::now(),
                    dev_log_enabled,
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .context("failed to build Discord client")?;

    client.start().await.context("client error")?;
    Ok(())
}
