//! Hermes -- messenger-god bot for the Cybercore portfolio. Slash commands
//! only (no privileged gateway intents needed): /status, /releases,
//! /downloads, each a thin wrapper over GitHub's REST API.

mod github;
mod repos;

use anyhow::Context as _;
use poise::serenity_prelude as serenity;

pub struct Data {
    http: reqwest::Client,
    github_token: Option<String>,
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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let _ = dotenvy::dotenv();
    let token =
        std::env::var("DISCORD_TOKEN").context("DISCORD_TOKEN not set (see .env.example)")?;
    let github_token = std::env::var("GITHUB_TOKEN").ok();

    let intents = serenity::GatewayIntents::empty();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![status(), releases(), downloads()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    http: reqwest::Client::new(),
                    github_token,
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
