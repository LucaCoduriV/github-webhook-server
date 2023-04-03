<p align="center">
  <img src="logo.png" />
</p>

## ❓ How to use

```
Github webhook server for CD                                                
                                                                            
Usage: rust-webhook-server [OPTIONS]                                    
                                                                            
Options:                                                                    
  -c, --config <CONFIG>  Path to to the config file [default: ./config.toml]
  -h, --help             Print help                                         
  -V, --version          Print version
```

## 🚧 Project configuration

The configuration follows the toml format. The top level keys contain common 
configurations, and the sections contain project specific configurations. Here's an example configuration:

```
port = 3003
git = "C:\\Program Files\\Git\\cmd\\git.exe"

[[repos]]
repo = "LucaCoduriV/rust-webhook-server"
secret = "MY_BIG_SECRET"
branch = "main"
repo_directory = "."
working_directory = "."
command = "powershell"
args = ["./testscript.ps1"]
events = ["push", "create"]

[[repos]]
repo = "2"
working_directory = "."
repo_directory = "../repo2/"
```

### Common configurations

| Key  | Description                | Required | Default         |
|------|----------------------------|----------|-----------------|
| port | port to run the server on  | Yes      |                 |
| git  | path to the git executable | No       | default git dir |

### Repo configurations
| Key               | Description                                       | Required | Default     |
|-------------------|---------------------------------------------------|----------|-------------|
| repo              | GitHub repository in format `username/repository` | Yes      |             |
| secret            | The secret set in Github                          | No       |             |
| branch            | Branch to update                                  | No       | main        |
| repo_directory    | path to the repo                                  | Yes      |             |
| working_directory | path from where to run the commands               | No       | current dir |
| command           | a program to execute                              | No       |             |
| args              | arguments of the command                          | No       |             |
| events            | a list of events (see below)                      | No       | All         |

### Creating a webhook in GitHub
See GitHub's Creating Webhooks guide. Currently, the server only supports JSON 
payloads. The payload needs to be sent to `http://yourserverip:port/hook`

### Available events
For mor details look at the Github webhook documentation
-   branch_protection_rule
-   check_run
-   check_suite
-   code_scanning_alert
-   commit_comment
-   create
-   delete
-   dependabot_alert
-   deploy_key
-   deployment
-   deployment_status
-   discussion
-   discussion_comment
-   fork
-   github_app_authorization
-   gollum
-   installation
-   installation_repositories
-   installation_target
-   issue_comment
-   issues
-   label
-   marketplace_purchase
-   member
-   membership
-   merge_group
-   meta
-   milestone
-   org_block
-   organization
-   package
-   page_build
-   personal_access_token_request
-   ping
-   project_card
-   project
-   project_column
-   project_v2
-   project_v2_item
-   public
-   pull_request
-   pull_request_review_comment
-   pull_request_review
-   pull_request_review_thread
-   push
-   registry_package
-   release
-   repository_advisory
-   repository
-   repository_dispatch
-   repository_import
-   repository_vulnerability_alert
-   secret_scanning_alert
-   secret_scanning_alert_location
-   security_advisory
-   security_and_analysis
-   sponsorship
-   star
-   status
-   team_add
-   team
-   watch
-   workflow_dispatch
-   workflow_job
-   workflow_ru
