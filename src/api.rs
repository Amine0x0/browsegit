use anyhow::{Context, Result};

use crate::models::Repo;

const URL: &str = "https://api.github.com/user/repos?per_page=100&visibility=all";

pub async fn get_repos(token: &str) -> Result<Vec<Repo>> {
    let client = reqwest::Client::new();

    let repos = client
        .get(URL)
        .header("User-Agent", "browsegit")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .context("failed to call GitHub API")?
        .error_for_status()
        .context("GitHub API returned error status")?
        .json::<Vec<Repo>>()
        .await
        .context("failed to parse GitHub repos response")?;

    Ok(repos)
}
