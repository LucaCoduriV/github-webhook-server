mod dto;
mod models;

use std::ffi::OsStr;
use std::io;
use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use std::net::SocketAddr;
use axum::http::HeaderMap;
use axum::response::Response;
use crate::dto::GithubEventTypes;
use sha2::Sha256;
use hmac::{Hmac, Mac};
use once_cell::sync::Lazy;
use crate::models::{Config, Repo};
use std::process::{Command, Output};


static USER_CONFIG: Lazy<Config> = Lazy::new(|| {
    let config_str = std::fs::read_to_string("./config.toml").expect("No configuration file found");
    toml::from_str(&config_str).expect("Wrong config format")
});

#[tokio::main]
async fn main() {
    println!("{:?}", USER_CONFIG.repos[0].repo_directory);

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/hook", post(hook))
        .route("/", get(root));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], USER_CONFIG.port));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!2"
}

async fn hook(header: HeaderMap, body: String) -> Response {
    let event = header.get("X-GitHub-Event").unwrap().to_str().unwrap();
    let event = event.parse::<GithubEventTypes>().unwrap();

    let body_json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let repo_full_name = body_json.get("repository").unwrap()
        .get("full_name").unwrap()
        .as_str().unwrap();

    let maybe_repo = USER_CONFIG.repos.iter().find(|repo| {
        return if repo.events.is_empty(){
             repo.repo == repo_full_name
        }else{
            repo.repo == repo_full_name && repo.events.contains(&event)
        }
    });

    let Some(repo) = maybe_repo else {
        eprintln!("REPO {} NOT IN CONFIG FILE FOR {}", repo_full_name, event);
        return (StatusCode::NOT_MODIFIED, "repo is not in config file").into_response();
    };

    if let Some(encoded_secret) = header.get("X-Hub-Signature-256") {
        let Some(secret) = &repo.secret else {
            eprintln!("[{}][{}]SECRET IS MISSING", repo.repo, event);
            return (StatusCode::BAD_REQUEST, "NO SECRET SPECIFIED").into_response();
        };

        if !check_signature(secret, encoded_secret.to_str().unwrap(), &body) {
            eprintln!("WRONG SECRET");
            return (StatusCode::BAD_REQUEST, "WRONG SECRET").into_response();
        }
    }

    if !repo.events.is_empty() {
        if !repo.events.contains(&event) {
            println!("[{}][{}]NOTHING TO DO WITH THIS EVENT", repo.repo, event);
            return (StatusCode::NOT_MODIFIED, "Nothing to do for this event").into_response();
        }
    }

    let git_result = update_git_repo(&repo);
    if git_result.is_err() {
        eprintln!("[{}][{}]ERROR WITH GIT: {:?}",repo.repo, event, git_result.unwrap_err());
        return (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't update git repo").into_response();
    }else{
        println!("[{}][{}]GIT OUTPUT: {:#?}", repo.repo, event, git_result.unwrap());
    }

    // If there is a command to run
    if repo.command.is_some() {
        let Ok(output) = run_command(
            repo.command.as_ref().unwrap(),
            &repo.args,
            &repo.working_directory)
            else {
                eprintln!("[{}][{}]COULDN'T RUN THE COMMAND", repo.repo, event);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't run the commands").into_response();
            };
        println!("[{}][{}]COMMAND OUTPUT: {:#?}", repo.repo, event, output);
    }


    StatusCode::OK.into_response()
}

fn run_command<I, S>(command: &str, args: I, curr_dir: &str) -> io::Result<Output>
    where I: IntoIterator<Item=S>,
          S: AsRef<OsStr>, {

    let output = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .current_dir(curr_dir)
            .arg(command)
            .args(args)
            .output()?
    } else {
        Command::new("sh")
            .current_dir(curr_dir)
            .arg("-c")
            .arg(command)
            .args(args)
            .output()?
    };

    Ok(output)
}

fn git_fetch_all(repo_directory:&str) -> Result<Output, io::Error> {
    Command::new(&USER_CONFIG.git_directory)
        .arg("fetch")
        .arg("--all")
        .current_dir(repo_directory)
        .output()
}

fn git_reset(branch:&str, repo_directory:&str) -> Result<Output, io::Error> {
    Command::new(&USER_CONFIG.git_directory)
        .arg("reset")
        .arg("--hard")
        .arg(format!("origin/{}", branch))
        .current_dir(repo_directory)
        .output()
}

fn update_git_repo(repo: &Repo) -> Result<(), io::Error> {
    let location = &repo.repo_directory;
    let branch = &repo.branch;
    // let backup_branch = format!("backup-{}", branch);

    // backup
    // let output = run_command("git checkout", vec![backup_branch.as_str()], location)?;
    // println!("{}", String::from_utf8(output.stdout).unwrap());
    // let output = run_command("git reset", vec![format!("--hard {}", branch)], location)?;
    // println!("{}", String::from_utf8(output.stdout).unwrap());
    // let output = run_command("git checkout", vec![branch], location)?;
    // println!("{}", String::from_utf8(output.stdout).unwrap());

    // update
    let output = git_fetch_all(location)?;
    println!("GIT FETCH ALL: {:#?}", String::from_utf8(output.stdout));
    // let output = run_command("git branch", vec![backup_branch.as_str()], location)?;
    // println!("{}", String::from_utf8(output.stdout).unwrap());
    let output = git_reset(branch, location)?;
    println!("GIT RESET: {:#?}", String::from_utf8(output.stdout));
    Ok(())
}

fn check_signature(secret: &str, signature: &str, body: &str) -> bool {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body.clone().as_bytes());
    let result = mac.finalize().into_bytes();
    let encoded_secret = signature.replace("sha256=", "");
    let expected_result = hex::decode(encoded_secret).unwrap();
    expected_result == result.as_slice()
}