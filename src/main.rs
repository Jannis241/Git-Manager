#![allow(non_snake_case)]
#![allow(unused)]
use std::fmt::{format, Debug};
use std::io::{self, Write, Read, Seek, prelude::*};
use colored::*;
use crate::io::stdout;
use std::{clone, fs};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use reqwest::header;
use std::process::Command;

#[derive(PartialEq)]
#[derive(Debug)]
enum State {
    Home,
    Repo(String), // repo name
    Config,
}


#[derive(Deserialize)]
struct Repository {
    Name: String,
    Path: String,
    clone_url: String,
}
#[derive(Deserialize)]
struct CloneData {
    clone_url: String
}

fn new_lines(num_of_lines: usize) {
    println!("{}", "\n".repeat(num_of_lines));
}

fn clear_terminal() {
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().unwrap();
}

fn input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input.");
    input.trim().to_string()
}
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct Config {
    api_key: String,
    username: String,
    project_path: String,
}


fn find_git_repos(path: &Path) -> Vec<PathBuf> {
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

fn manage_config() -> Config{
    let config_path = "./config.json";
    let path = Path::new(&config_path);

    let mut config: Config;

    if path.exists() {
        // Read existing configuration from file
        let file = File::open(&path).expect("Failed to open config file.");
        config = serde_json::from_reader(file).expect("Failed to parse config file.");
        println!("Config file loaded...");
        config
    } else {
        println!("Creating new configuration:");

        // Prompt user for configuration input
        let config = Config {
            api_key: input("API Key: "),
            username: input("Username: "),
            project_path: input("Project path: "),
        };

        // Write configuration to file
        let serialized = serde_json::to_string(&config).expect("Failed to serialize config.");
        let mut file = File::create(&path).expect("Failed to create config file.");
        file.write_all(serialized.as_bytes()).expect("Failed to write config to file.");
        config
    }
}
// Function to write data to JSON file
fn write_to_json(file_path: &str, config: &Config) {
    // Convert the Config instance to JSON format
    let json = serde_json::to_string_pretty(config).expect("Failed to serialize config");

    // Open the file in read/write mode, creating it if it doesn't exist
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)
        .expect("Failed to open file");

    // Write the JSON data to the file
    file.write_all(json.as_bytes())
        .expect("Failed to write to file");

    println!("Data written to config.json successfully");
}


pub fn create_repo(repoName: String){
    // path auf dem pc
    // public or private
    // name
    // ich glaube ein file muss schon drinne sein, einfach ein text file oder so
    println!("the user wants to create a repo called: {}", repoName)
}

pub fn create_branch(repoName: String, branchName: String){
    println!("the user wants to create a branch in {} with the name {}", repoName, branchName)
}

pub fn upload(repoPath: &String){
    println!("the user wants to upload: {}", repoPath);
    
}

pub fn update(repoPath: &String){
    println!("the user wants to update: {}", repoPath);
}


pub fn download(repo_name: String, username: String, path: &str) {
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

pub fn migrage(projectName: String){
    println!("the user wants to migrate: {}", projectName);
}

fn print_in_file_infos() {
    new_lines(1);
    use colored::*;

    let commands_info = vec![
        ("upload", "Uploads this repository directly"),
        ("create branch <name>", "Creates a new branch"),
        ("update", "Get the newest version of this project directly"),
        ("close / back", "Get back to the home state"),
        ("delete repo", "Deletes the repository from your account"),
        ("delete branch <name>", "Deletes the branch of the project you are currently in"),
        ("delete file", "Deletes the file of the project you are currently in"),
        ("exit / q", "Exit the Git-Manager"),
    ];

    let max_command_length = commands_info.iter()
        .map(|(cmd, _)| cmd.len())
        .max()
        .unwrap_or(0);

    println!("{}", "You are currently in a file, so some commands changed:".bold().underline().green());
    for (command, description) in commands_info {
        let padding = max_command_length - command.len();
        println!(
            "{}{}    {}",
            command.bold().blue(),
            " ".repeat(padding),
            description.italic().white()
        );
    }
    
    println!("{}", "These are just the commands that changed, others like 'download all' still work.".italic().white());
    println!()
}

fn print_infos() {
    use colored::*;

    let commands_info = vec![
        ("upload <repo name>", "Upload a specific repository"),
        ("upload all", "Upload all repositories"),
        ("create branch <branch name>", "Create a new branch (you have to open this repository first)"),
        ("create repo <repo name>", "Create a new repository"),
        ("delete repo <name>", "Deletes the repository from your Github account"),
        ("delete branch <name> in <repo name>" , "Deletes branch in repository"),
        ("delete folder <name>", "Deletes the file from your system"),
        ("update <repo name>", "Get the newest version of a project"),
        ("download all from <github name>", "Download all repositories from your account"),
        ("download <repo name> from <github name>", "Download a repository from another user"),
        ("migrate <project name>", "Migrate a non git project to git"),
        ("list", "List all known git projects"),
        ("open config", "Open the config file"),
        ("open <repo name>", "Open a specific repository"),
        ("close / back", "Get back to the home state"),
        ("exit / q", "Exit the Git-Manager"),
        ("clear", "Clear the terminal"),
    ];

    let max_command_length = commands_info.iter()
        .map(|(cmd, _)| cmd.len())
        .max()
        .unwrap_or(0);

    println!("{}", "Commands:".bold().underline().green());
    for (command, description) in commands_info {
        let padding = max_command_length - command.len();
        println!(
            "{}{}    {}",
            command.bold().blue(),
            " ".repeat(padding),
            description.italic().white()
        );
    }
    println!("\n{}", "Extra info:".bold().underline().green());
    println!("{}", "To execute some commands, you need to have opened a repository.".italic().white());
}


fn throw_error(msg: &str){
    println!("{}: {}", "ERROR".bold().red().underline(), msg.red())
}


fn check_name(name: &String, error_msg: &str) -> bool {
    if name.trim() == ""{
        throw_error(error_msg);
        return false;
    }
    else {
        return true;
    }
}


async fn clone_all_repos(username: &str, token: Option<&str>, target_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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

// Extrahiere den Repository-Namen aus der URL
fn extract_repo_name(clone_url: &str) -> &str {
    Path::new(clone_url)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown-repo")
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    new_lines(1);
    // getting user config
    let mut currentState = State::Home;
    let user_config = manage_config();

    // getting all of the git project paths and names


    new_lines(1);
    println!("{}","Welcome to Git-Manager".white().bold().underline());



    // print the found repos
    let project_path_str = user_config.project_path.as_str();
    let project_path = Path::new(project_path_str);
    let git_repos_paths = find_git_repos(project_path);

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

    let mut repoNames = Vec::new();
    for repo in &repo_list {
        repoNames.push(repo.Name.clone());
    }


    new_lines(1);
    println!("{}", "Your repositories:".green().bold().underline());

    for path in git_repos_paths.clone() {
        let cleanPath = path.to_str().unwrap_or("error -> path is none");
        let cleanPathSplit = cleanPath.split_inclusive("/").collect::<Vec<&str>>();
        
        println!("{}{}", cleanPathSplit[0..cleanPathSplit.len() - 2].join("").italic(), cleanPathSplit.last().unwrap().blue().italic().bold());
    }


    new_lines(1);
    print_infos();
    new_lines(1);

    loop {
        let project_path_str = user_config.project_path.as_str();
        let project_path = Path::new(project_path_str);
        let git_repos_paths = find_git_repos(project_path);
    
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
    
        let mut repoNames = Vec::new();
        for repo in &repo_list {
            repoNames.push(repo.Name.clone());
        }

        match currentState {
            State::Home => {
                print!("<Git> ");
            }
            State::Repo(ref repoName) => {
                print!("<{}> ", &repoName);
            }
            State::Config => {
                print!("<Config> ");
            }
        }

        let _ = io::stdout().flush();
        let mut rawInput = String::new();
        io::stdin().read_line(&mut rawInput).expect("Error");
        let rawInput = rawInput.trim().to_string();
        let input = rawInput.to_lowercase();
        let mut arguments = input.split_whitespace().collect::<Vec<&str>>();
        let mut rawArgs = rawInput.split_whitespace().collect::<Vec<&str>>();

        arguments.push(" ");
        arguments.push(" ");
        arguments.push(" ");
        arguments.push(" ");
        arguments.push(" ");

        rawArgs.push(" ");
        rawArgs.push(" ");
        rawArgs.push(" ");
        rawArgs.push(" ");
        rawArgs.push(" ");


        if arguments.len() > 0 {
            match arguments[0] {
                
                // vorschläge wenn man einen namen falsch schreibt

                // auto correction
                // pull-push combi 
                // merge feature -> git add, git commit, git pull --rebase, git push
                // --> generell auch die features einbauen die man braucht um zusammen zu arbeiten
                // wenn ein project nicht gefunden wird soll eine warnung kommen dass dieses projekt im project folder sein muss
                
                "open" =>{
                    if arguments[1] == "config"{
                        currentState = State::Config;
                        // reset
                        // set username:
                        // set api::
                        // ....
                    }

                    else if check_name(&arguments[1].to_string(),"Empty file name is not valid"){                      
                        
                        // if the raw arg just matches (with uppercase letters)
                        if repoNames.contains(&rawArgs[1].to_string()){
                            // only if the file exists
                            currentState = State::Repo(rawArgs[1].to_string());
                            print_in_file_infos();
                        }
                        else {
                            throw_error("File not found")
                        }
                    }
                }

                "help" => {
                    new_lines(1);
                    print_infos();
                    new_lines(1);
                }

                // force abfrage falls es fehler gibt fehlt noch
                "upload" => {
                    if arguments[1] == "all" {
                        for repo in &repo_list{
                            update(&repo.Path);
                        }
                    }
                    else {
                        // upload a specific file
                        if let State::Repo(ref reponame) = currentState{
                            // hier muss der path sein und nicht der name
                            upload(&reponame.to_string());
                        }
                        else {
                            let name = rawArgs[1].to_string().clone();
                            if repoNames.contains(&name) {
                            // hier muss der path sein und nicht der name
                                upload(&name);
                            }
                            else {
                                throw_error("File not found");
                            }
                        }

                        
                    }
                }

                "download" => {

                    // download from <github name> <reponame>
                    // download all from <github name> <reponame>
                    // vielleicht noch einen optionalen parameter "path" der bestimmt wohin das geklont werden soll (nicht pflicht)
                    if arguments[1] == "all" && arguments[2] == "from"{
                        // irgendwie mit dem username alle sachen von dem acc downloaden oder so
                        let username = rawArgs[3];
                        
                        if username == &user_config.username {
                            // from own acc so also private repos
                            let api_key = Some(user_config.api_key.as_str());
                            clone_all_repos(username, api_key, &user_config.project_path).await?;
                        }
                        else {
                            let api_key = None;
                            clone_all_repos(username, api_key, &user_config.project_path).await?;
                        }
                        
                    }
                    else {
                        let repoName = rawArgs[1];
                        if arguments[2] == "from" {
                            let username = rawArgs[3];
                            download(repoName.to_string(), username.to_string(), &user_config.project_path)

                        }
                    }
                }

                // force abfrage falls es fehler gibt fehlt noch
                // wenn ein file nicht gefunden wird, vielleicht den input alles lowercase machen und gucken ob das zu dem repo namen
                // in lowercase match, und falls das der fall ist den namen vorschlagen da es oft passiert dass nur die groß und klein schreibung falsch ist
                // das selbe auch bei open, uppload usw machen
                "update" => {
                    if arguments[1] == "all" {
                        for repo in &repo_list {
                            update(&repo.Path);
                        }
                    }
                    else {
                        // update a specific file
                        if let State::Repo(ref reponame) = currentState{

                            // hier muss der path sein und nicht der name
                            update(&reponame.to_string());
                        }
                        else {
                            let name = rawArgs[1].to_string().clone();
                            if repoNames.contains(&name) {
                            // hier muss der path sein und nicht der name
                                update(&name);
                            }
                            else {
                                throw_error("File not found");
                            }
                        }

                        
                    }
                }
                "delete" => {
                    // folder
                    // repo
                    // branch 
                    // man muss nohchmal bestätigen falls man etwas löschen will
                    // error nachricht falls das repo nicht dir gehört (user config checken)
                }

                "create" => {
                    let name = rawArgs[2].to_string();
                    
                    if arguments[1] == "branch"{
                        if check_name(&name, "Empty branch name is not valid"){
                            if let State::Repo(ref repoName) = currentState {
                                create_branch(repoName.clone(), name);
                            }
                            else {
                                throw_error("You have to open a specific repo to create a branch")
                            }
                        }    
                    }
                    else if arguments[1] == "repo" {
                        if check_name(&name, "Empty repository name is not valid"){
                            create_repo(name);
                            // man muss daran denken hier werden alle repos zu bekommen oder zumindest den path 
                            // einfach hinzufügen um zeit zu sparen

                            // gucken ob es das repo schon gibt

                        }       
                    } 
                }

                "migrate" => {
                    migrage("skibidi ahhh".to_string());
                }

                "list" => {
                    new_lines(1);
                    println!("{}", "Your repositories:".green().bold().underline());

                    for path in git_repos_paths.clone() {
                        let cleanPath = path.to_str().unwrap_or("error -> path is none");
                        let cleanPathSplit = cleanPath.split_inclusive("/").collect::<Vec<&str>>();
                        
                        println!("{}{}", cleanPathSplit[0..cleanPathSplit.len() - 2].join("").italic(), cleanPathSplit.last().unwrap().blue().italic().bold());
                    }
                }
                "close" => {
                    currentState = State::Home;
                }
                "back" => {
                    currentState = State::Home;
                }

                "exit" => break,
                "q" => break,
                "quit" => break,
                "esc" => break,
                "clear" => clear_terminal(),
                "" => print!(""),
                " " => print!(""),
                other => println!("command '{}' not found", other.bold()),
            };        
        }
    }
    Ok(())
}
