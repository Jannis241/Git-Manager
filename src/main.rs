#![allow(non_snake_case)]
#![allow(unused)]
use std::io::{self, Write, Read, Seek}; 
use std::fs::{self, File, OpenOptions}; 
use std::path::{Path, PathBuf};
use std::process::Command;
use colored::*; 
use git_actions::{find_file_in_path, migrate};
use serde::{Serialize, Deserialize};
use reqwest::header;
use thiserror::Error;

mod command_line;
mod git_actions;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum State {
    Home,
    Repo(String), // repo name
    Config,
}


#[derive(Deserialize)]
pub struct Repository {
    Name: String,
    Path: String,
    clone_url: String,
}

#[derive(Deserialize)]
struct CloneData {
    clone_url: String
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct Config {
    api_key: String,
    username: String,
    project_path: String,
}

fn manage_config() -> Config{
    let config_path = "./config.json";
    let path = Path::new(&config_path);

    let mut config: Config;

    if path.exists() {
        // Read existing configuration from file
        let file = File::open(&path).expect("Failed to open config file.");
        config = serde_json::from_reader(file).expect("Failed to parse config file.");
        config
    } else {
        // Prompt user for configuration input
        let config = Config {
            api_key: command_line::input("API Key: "),
            username: command_line::input("Username: "),
            project_path: command_line::input("Project path: "),
        };
        write_to_json(&config_path, &config);
        // Write configuration to file
        //let serialized = serde_json::to_string(&config).expect("Failed to serialize config.");
        //let mut file = File::create(&path).expect("Failed to create config file.");
        //file.write_all(serialized.as_bytes()).expect("Failed to write config to file.");
        config
    }
}

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Searching your project directory for git repositories, this may take a while...");

    let mut currentState = State::Home;

    // init config
    let mut user_config = manage_config();


    // getting all of the repos
    let mut repo_list = git_actions::get_all_repositories(&user_config);
    let mut repo_names_list = Vec::new();
    let mut repo_path_list = Vec::new();
    for repo in &repo_list {
        repo_names_list.push(repo.Name.clone());
        repo_path_list.push(repo.Path.clone());
    }

    command_line::print_intro();

    loop {
        // update repo list, names, and path in case a repo got added or deleted
        git_actions::update_repos(&mut repo_list, &mut repo_names_list, &mut repo_path_list, &user_config);

        let input = command_line::get_git_input(&currentState);
        

        // arguements for the commands and rawArgs for names -> for uppercase and lowercase letters
        let mut arguements = input.0.split_whitespace().collect::<Vec<&str>>();
        let mut rawArgs = input.1.split_whitespace().collect::<Vec<&str>>();
        
        // push some empty strings so no index error can occur
        arguements.push(" ");
        arguements.push(" ");
        arguements.push(" ");
        arguements.push(" ");
        arguements.push(" ");

        rawArgs.push(" ");
        rawArgs.push(" ");
        rawArgs.push(" ");
        rawArgs.push(" ");
        rawArgs.push(" ");

        // command loop
        match arguements[0] {
            "set" => {
                if currentState == State::Config {
                    let arg = arguements[1];
                    let change = rawArgs[2];
                    if check_name(&arg.to_string(), "Arguement is missing"){
                        if check_name(&change.to_string(), "Empty change is not valid"){

                            match arg {
                                "username" => {
                                    user_config.username = change.to_string();
                                    write_to_json("./config.json", &user_config);
                                }
                                "key" => {
                                    user_config.api_key = change.to_string();
                                    write_to_json("./config.json", &user_config);
                                }
                                "path" => {
                                    user_config.project_path = change.to_string();
                                    write_to_json("./config.json", &user_config);
                                }
                                other => {
                                    throw_error(format!("Arguement {} is not valid", other).as_str());
                                }
                            }
                        }
                }
            }
                else {
                    throw_error("open config to change your settings")
                }
                    
            }

            "show" => {
                if currentState == State::Config {
                    println!("username: {}", &user_config.username);
                    println!("api key: {}", &user_config.api_key);
                    println!("project path: {}", &user_config.project_path);
                }
                else {
                    throw_error("open config to see your settings")
                }
            } 

            "open" =>{
                if check_name(&arguements[1].to_string(), "File name is missing"){

                    if arguements[1] == "config"{
                        currentState = State::Config;
                    }

                    else if repo_names_list.contains(&rawArgs[1].to_string()){
                        // only if the file exists
                        currentState = State::Repo(rawArgs[1].to_string());
                        command_line::print_in_file_infos();
                    }
                    else {
                        throw_error("File not found")
                    }
                }
            }

            "help" => {
                if State::Config == currentState {
                    command_line::print_config_infos();
                }
                if let State::Repo(ref repoName) = currentState {
                    command_line::print_in_file_infos();
                }
                if State::Home == currentState {
                    command_line::print_infos();
                }
            }

            "upload" => {
                if arguements[1] == "all" {
                    for repo in &repo_list{
                        git_actions::update(&repo.Path);
                    }
                }

                else {
                    // upload a specific file
                    
                    if let State::Repo(ref reponame) = currentState{
                        git_actions::upload(&reponame.to_string(), &"temp commit message".to_string());
                    }
                    else {
                        let name = rawArgs[1].to_string().clone();
                        if check_name(&name, "Project name is missing"){
                            if repo_names_list.contains(&name) {
                                git_actions::upload(&name, &"temp commit message".to_string());
                            }
                            else {
                                throw_error("File not found");
                            }
                        }
                    }
                }
            }

            "download" => {

                if check_name(&arguements[1].to_string(), "Arguement is missing"){
                    if arguements[1] == "all"{
                        if arguements[2] == "from" {
                            let username = rawArgs[3];
                            if check_name(&username.to_string(), "Username is missing"){
                                if username == user_config.username {
                                    // download own repos so also privat repos
                                    if username == &user_config.username {
                                        // from own acc so also private repos
                                        let api_key = Some(user_config.api_key.as_str());
                                        git_actions::clone_all_repos(username, api_key, &user_config.project_path).await?;
                                    }
                                    else {
                                        let api_key = None;
                                        git_actions::clone_all_repos(username, api_key, &user_config.project_path).await?;
                                    }  
                                }
                            }
                        }
                        else {
                            throw_error("Keyword 'from' is missing: download all from <user>");
                        }
                    }
                    else {
                        let repoName = rawArgs[1];
                        
                        if check_name(&repoName.to_string(), "Repository name is missing"){
                            if arguements[2] == "from" {
                                let username = rawArgs[3];
                                if check_name(&username.to_string(), "Username is missing"){
                                    git_actions::download(&repoName.to_string(), &username.to_string(), &user_config.project_path)
                                }
                            }
                            else{
                                throw_error("Keyword 'from' is missing: download <repo> from <user>");
                            }
                        }
                    }
                }
                
            }

            "update" => {
                if arguements[1] == "all" {
                    for repo in &repo_list {
                        git_actions::update(&repo.Path);
                    }
                }
                else {
                    // update a specific file
                    if let State::Repo(ref reponame) = currentState{
                        git_actions::update(&reponame.to_string());
                    }
                    else {
                        let name = rawArgs[1].to_string().clone();
                        if check_name(&name, "Project name is missing"){
                            if repo_names_list.contains(&name) {
                                git_actions::update(&name);
                            }
                            else {
                                throw_error("File not found");
                            }
                        }
                   }
                }
            }
            
            "delete" => {
                let secondArg = arguements[1];
                if check_name(&secondArg.to_string(), "Arguement is missing"){
                    match secondArg{
                        "folder" => {
                            let name = rawArgs[2];
                            let path = git_actions::find_file_in_path(&user_config.project_path, &name);
                            if check_name(&name.to_string(), "Folder name is missing"){
                                // delete folder <name>
                                match path {
                                    Ok(filePath) => {
                                        git_actions::deleteDir(&filePath);
                                    }
                                    Err(git_actions::SearchError::NotFound) => throw_error(format!("Folder '{}' not found in {}", &name, &user_config.project_path).as_str()),
                                    Err(git_actions::SearchError::MultipleFound) => throw_error(format!("Found multiple projects with the name '{}'", &name).as_str()),
                                    other => throw_error("Unknown error ocurred while migrating"),
                            }

                            }
                        }
                        "repo" => {
                            if let State::Repo(ref reponame) = currentState{
                                git_actions::delete_repo(&reponame, &user_config.api_key);
                                currentState = State::Home; // move back to home since the repo is deleted
                            }
                            else {
                                // chekcne ob es das repo gibt
                                let name = rawArgs[2];
                                if check_name(&name.to_string(), "Repository name is missing"){
                                    if repo_names_list.contains(&name.to_string()) {
                                        git_actions::delete_repo(&name.to_string(), &user_config.api_key);
                                    }
                                    else {
                                        throw_error("Repository not found")
                                    }
                                }


                            }                
                        }
                        "branch" => {
                            if let State::Repo(ref reponame) = currentState{
                                let branchName = rawArgs[2];
                                git_actions::delete_branch(&reponame,&branchName.to_string(),&user_config.api_key);
                            }
                            else{
                                let branchName = rawArgs[2];
                                if check_name(&branchName.to_string(), "Branch name is missing"){
                                    if arguements[3] == "from" {
                                        let repoName = rawArgs[4];
                                        if check_name(&repoName.to_string(), "Repository name is missing"){

                                        git_actions::delete_branch(&repoName.to_string(), &branchName.to_string(), &user_config.api_key);
                                        }
                                    }
                                    else {
                                        throw_error("Keyword 'from' is missing: delete branch <name> from <repo>")
                                    }
                                }
                            } 
                        }
                        other => throw_error(format!("Arguement '{}' is not valid", secondArg).as_str())
                        


                    }

                }
            }

            "create" => {
                let name = rawArgs[2].to_string();
                if check_name(&arguements[1].to_string(), "Arguement is missing"){

                    if arguements[1] == "branch"{
                        if check_name(&name, "Branch name is missing"){
                            if let State::Repo(ref repoName) = currentState {
                                git_actions::create_branch(&repoName, &name, &user_config.api_key);
                            }
                            else {
                                if arguements[3] == "in"{
                                    let repoName = rawArgs[4];
                                    if check_name(&repoName.to_string(), "Repository name is missing"){
                                        git_actions::create_branch(&repoName.to_string(), &name, &user_config.api_key);
                                    }
                                }

                                else {
                                    throw_error("Keyword 'in' is missing: create branch <name> in <repo>");
                                }
                            }
                        }    
                    }
                    else if arguements[1] == "repo" {
                        if check_name(&name, "Repository name is missing"){
                            // create repo <name> <public / private> 
                            let name = rawArgs[2];
                            let privacy = arguements[3];
                            git_actions::create_repo(&name.to_string(), &true, &"sdfsdfsdfsf".to_string(), &"sdfsfsdf".to_string());

                        }       
                    } 
                    else {
                        throw_error(format!("Arguement '{}' is not valid", arguements[1]).as_str())
                    }
                }
            }

            "migrate" => {

                let name = rawArgs[1].to_string();

                let projectPath = String::from(&user_config.project_path);
                if check_name(&name.to_string(), "Project name is missing"){
                    match git_actions::find_file_in_path(&projectPath, &name) {
                        Ok(file_path) => {
                            if !repo_names_list.contains(&name) {
                                let privacy = arguements[2];
                                if check_name(&privacy.to_string(), "Privacy arguement is missing: migrate <project> <public/private>"){
                                    match privacy {
                                        "public" => {
                                            migrate(&file_path, &name, true, &user_config.api_key);
                                        } 
                                        "private" => {
                                            migrate(&file_path, &name, true, &user_config.api_key);
                                        }
                                        other => {
                                            throw_error(format!("Privacy arguement '{}' is invalid (public/private)", privacy).as_str());
                                        }
                                    }
                                }
                            }
                            
                            else {
                                throw_error("Project is already on git")
                            }
                        }
                        Err(git_actions::SearchError::NotFound) => throw_error(format!("Project '{}' not found in {}", &name, &projectPath).as_str()),
                        Err(git_actions::SearchError::MultipleFound) => throw_error(format!("Found multiple projects with the name '{}'", &name).as_str()),
                        other => throw_error("Unknown error ocurred while migrating"),
                    }

                }   
            }

            "list" => {
                if repo_path_list.len() == 0 {
                    throw_error("No git projects found")
                }
                else {
                    git_actions::print_repo_list(&repo_path_list);
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
            "clear" => command_line::clear_terminal(),
            "" => print!(""),
            " " => print!(""),
            other => throw_error(format!("Command '{}' not found", other).as_str())
        };        
    
    }
    Ok(())
}
