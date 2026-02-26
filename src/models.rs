use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Repo {
    pub name: String,
    pub ssh_url: String,
}
