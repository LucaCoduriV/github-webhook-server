use serde::Deserialize;
use crate::dto::GithubEventTypes;

#[derive(Deserialize, Debug)]
pub struct Repo {
    pub repo:String,
    pub command:Option<String>,
    #[serde(default = "Vec::default")]
    pub args:Vec<String>,
    pub secret:Option<String>,
    #[serde(default = "Vec::default")]
    pub events:Vec<GithubEventTypes>,
    pub repo_directory:String,
    #[serde(default = "default_branch")]
    pub branch:String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub repos: Vec<Repo>
}

fn default_branch() -> String{
    "main".to_string()
}