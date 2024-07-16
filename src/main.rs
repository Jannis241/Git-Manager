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

        // Write configuration to file
        let serialized = serde_json::to_string(&config).expect("Failed to serialize config.");
        let mut file = File::create(&path).expect("Failed to create config file.");
        file.write_all(serialized.as_bytes()).expect("Failed to write config to file.");
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
    let mut currentState = State::Home;
    let user_config = manage_config();

    command_line::new_lines(1);
    println!("{}","Welcome to Git-Manager".white().bold().underline());

    let repo_list = git_actions::get_all_repositories(&user_config);
    let mut repo_names_list = Vec::new();
    let mut repo_path_list = Vec::new();
    for repo in &repo_list {
        repo_names_list.push(repo.Name.clone());
        repo_path_list.push(repo.Path.clone());
    }
    
    git_actions::print_repo_list(&repo_path_list);
    command_line::print_infos();

    loop {
        // update repo list, names, and path in case a repo got added or deleted
        let repo_list = git_actions::get_all_repositories(&user_config);
        let mut repo_names_list = Vec::new();
        let mut repo_path_list = Vec::new();
        for repo in &repo_list {
            repo_names_list.push(repo.Name.clone());
            repo_path_list.push(repo.Path.clone());
        }

        let input = command_line::get_git_input(&currentState);

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

        
        match arguements[0] {
            "open" =>{
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

            "help" => {
                command_line::new_lines(1);
                command_line::print_infos();
                command_line::new_lines(1);
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
                        if repo_names_list.contains(&name) {
                            git_actions::upload(&name, &"temp commit message".to_string());
                        }
                        else {
                            throw_error("File not found");
                        }
                    }
                }
            }

            "download" => {
                if arguements[1] == "all" && arguements[2] == "from"{
                    let username = rawArgs[3];
                    
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
                else {
                    let repoName = rawArgs[1];
                    if arguements[2] == "from" {
                        let username = rawArgs[3];
                        git_actions::download(&repoName.to_string(), &username.to_string(), &user_config.project_path)

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
                        if repo_names_list.contains(&name) {
                            git_actions::update(&name);
                        }
                        else {
                            throw_error("File not found");
                        }
                    }
                }
            }
            "delete" => {
            }

            "create" => {
                let name = rawArgs[2].to_string();
                
                if arguements[1] == "branch"{
                    if check_name(&name, "Empty branch name is not valid"){
                        if let State::Repo(ref repoName) = currentState {
                            git_actions::create_branch(&repoName, &name, &"sdfsfsffdsfs".to_string());
                        }
                        else {
                            throw_error("You have to open a specific repo to create a branch")
                        }
                    }    
                }
                else if arguements[1] == "repo" {
                    if check_name(&name, "Empty repository name is not valid"){
                        git_actions::create_repo(&"sfsdfsdgs".to_string(), &true, &"sdfsdfsdfsf".to_string(), &"sdfsfsdf".to_string());

                    }       
                } 
            }

            "migrate" => {
                git_actions::migrate("sfsd".to_string(), "fsdfsdf".to_string(), true, "sdfsdfsdfs".to_string());
            }

            "list" => {
                git_actions::print_repo_list(&repo_path_list);
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
            other => println!("command '{}' not found", other.bold()),
        };        
    
    }
    Ok(())
}
