//! Thin GitHub REST API wrappers. Unauthenticated by default (fine for the
//! low request volume a Discord bot's slash commands generate); pass
//! GITHUB_TOKEN to raise the rate limit or reach private repos.

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub html_url: String,
    pub body: Option<String>,
    pub assets: Vec<Asset>,
}

#[derive(Deserialize)]
pub struct Asset {
    pub name: String,
    pub download_count: u64,
}

#[derive(Deserialize)]
pub struct RepoActivity {
    pub html_url: String,
    pub pushed_at: Option<String>,
}

#[derive(Deserialize)]
struct WorkflowRuns {
    workflow_runs: Vec<WorkflowRun>,
}

#[derive(Deserialize)]
struct WorkflowRun {
    conclusion: Option<String>,
}

fn request(client: &reqwest::Client, url: &str, token: Option<&str>) -> reqwest::RequestBuilder {
    let mut req = client
        .get(url)
        .header("User-Agent", "hermes-discord-bot")
        .header("Accept", "application/vnd.github+json");
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    req
}

pub async fn latest_release(
    client: &reqwest::Client,
    repo: &str,
    token: Option<&str>,
) -> Result<Option<Release>, reqwest::Error> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let resp = request(client, &url, token).send().await?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    Ok(Some(resp.error_for_status()?.json::<Release>().await?))
}

/// Basic repository activity used by the scheduled #dev-log digest.
pub async fn repo_activity(
    client: &reqwest::Client,
    repo: &str,
    token: Option<&str>,
) -> Result<RepoActivity, reqwest::Error> {
    let url = format!("https://api.github.com/repos/{repo}");
    request(client, &url, token)
        .send()
        .await?
        .error_for_status()?
        .json::<RepoActivity>()
        .await
}

/// Most recent completed workflow run's conclusion, for the repo's default
/// branch -- Ok(true) green, Ok(false) red, Err if the request itself fails
/// (private repo without a token, rate limit, network blip).
pub async fn ci_status_for(
    client: &reqwest::Client,
    repo: &str,
    token: Option<&str>,
) -> Result<bool, reqwest::Error> {
    let url =
        format!("https://api.github.com/repos/{repo}/actions/runs?status=completed&per_page=1");
    let resp = request(client, &url, token)
        .send()
        .await?
        .error_for_status()?;
    let runs: WorkflowRuns = resp.json().await?;
    Ok(runs
        .workflow_runs
        .first()
        .and_then(|r| r.conclusion.as_deref())
        == Some("success"))
}

pub async fn all_ci_status(
    client: &reqwest::Client,
    token: Option<&str>,
) -> Vec<(&'static str, Result<bool, reqwest::Error>)> {
    let mut out = Vec::new();
    for repo in crate::repos::all() {
        let result = ci_status_for(client, repo, token).await;
        out.push((repo, result));
    }
    out
}
