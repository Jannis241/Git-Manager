use crate::{Config, Repository};
use crate::*;

pub fn create_repo(repoName: &String, public: &bool, path: &String, api_key: &String){
    println!("the user wants to create a repo:");
    println!("Name: {}", repoName);
    println!("Path (wo das repo ist, Optional): {}", path);
    println!("Public: {}", public);
    println!("API-Key: {}", api_key);
}

pub fn create_branch(repoName: &String, branchName: &String, api_key: &String){
    println!("the user wants to create a branch in {} with the name {} -> api key: {}", repoName, branchName, api_key);
}

pub fn upload(repoPath: &String, commitMessage: &String){
    println!("the user wants to upload: {}", repoPath);
    
}

pub fn update(repoPath: &String){
    println!("the user wants to update: {}", repoPath);
}

pub fn download(repo_name: &String, username: &String, path: &str) {
    let clone_url = format!("https://github.com/{}/{}.git", username, repo_name);
    let target_path = path;
    let output = Command::new("git")
    .arg("clone")
    .arg(&clone_url)
    .arg(format!("{}/{}", target_path, extract_repo_name(&clone_url)))
    .output()
    .expect("Failed to execute git command");

    if !output.status.success() {
        eprintln!("Failed to clone repo: {}", clone_url);
    } else {
        println!("Successfully cloned: {}", clone_url);
    }
}

pub fn migrate(projectPath: String, repoName: String, public: bool, api_key: String){
    println!("the user wants to migrate a project:");
    println!("repo name: {}", repoName);
    println!("current project path: {}", projectPath);
    println!("Public: {}", public);
    println!("API-Key: {}", api_key);
}

pub fn find_git_repos(path: &Path) -> Vec<PathBuf> {
    let mut git_repos = Vec::new();

    // Rekursiv alle Einträge im Pfad durchgehen
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    // Wenn das Verzeichnis noch nicht als Git-Repo gefunden wurde
                    if !git_repos.iter().any(|repo: &PathBuf | repo.starts_with(&entry_path)) {
                        // Nach .git-Dateien suchen
                        let git_dir = entry_path.join(".git");
                        if git_dir.exists() {
                            git_repos.push(entry_path);
                        } else {
                            // Rekursiv in Unterverzeichnisse suchen
                            git_repos.extend(find_git_repos(&entry_path));
                        }
                    }
                }
            }
        }
    }

    git_repos
}

pub async fn clone_all_repos(username: &str, token: Option<&str>, target_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Erstelle den Client
    let client = reqwest::Client::new();

    // Erstelle die URL
    let url = format!("https://api.github.com/users/{}/repos?per_page=100", username);

    // Füge optional den Authorization Header hinzu
    let mut request = client.get(&url).header("User-Agent", "rust-github-client");

    if let Some(token) = token {
        let auth_value = format!("token {}", token);
        request = request.header(header::AUTHORIZATION, auth_value);
    }

    // Sende die Anfrage
    let response = request.send().await?;

    // Überprüfe den Status der Antwort
    if !response.status().is_success() {
        eprintln!("Failed to fetch repos: {}", response.status());
        return Ok(());
    }

    // Parste die JSON Antwort
    let repos: Vec<CloneData> = response.json().await?;

    // Klone jedes Repository
    for repo in repos {
        let output = Command::new("git")
            .arg("clone")
            .arg(&repo.clone_url)
            .arg(format!("{}/{}", target_path, extract_repo_name(&repo.clone_url)))
            .output()
            .expect("Failed to execute git command");

        if !output.status.success() {
            eprintln!("Failed to clone repo: {}", repo.clone_url);
        } else {
            println!("Successfully cloned: {}", repo.clone_url);
        }
    }

    Ok(())
}

pub fn extract_repo_name(clone_url: &str) -> &str {
    Path::new(clone_url)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown-repo")
}

pub fn get_all_repositories(user_config: &Config) -> Vec<Repository>{
    let project_path_str = user_config.project_path.as_str();
    let project_path = Path::new(project_path_str);
    let git_repos_paths: Vec<PathBuf> = git_actions::find_git_repos(project_path);

    let mut repo_list = Vec::new();
    for path in git_repos_paths.clone() {
        let cleanPath = path.to_str().unwrap_or("error -> path is none");
        let cleanPathSplit = cleanPath.split_inclusive("/").collect::<Vec<&str>>();
        let name = cleanPathSplit.last().unwrap().to_string();
        let repo = Repository {
            Name: name.clone(),
            Path: cleanPath.to_string(),
            clone_url: "https://github.com/".to_string() + &user_config.username+ "/" + &name + ".git",
        };
        repo_list.push(repo);
        
    }

    repo_list
}

pub fn print_repo_list(repo_paths: &Vec<String>) {
    command_line::new_lines(1);
    println!("{}", "Your repositories:".green().bold().underline());

    for path in repo_paths.clone() {
        let cleanPathSplit = path.split_inclusive("/").collect::<Vec<&str>>();
        
        println!("{}{}", cleanPathSplit[0..cleanPathSplit.len() - 2].join("").italic(), cleanPathSplit.last().unwrap().blue().italic().bold());
    }
}