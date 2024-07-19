#![allow(non_snake_case)]
#![allow(unused)]
use std::collections::btree_map::Range;
use std::env::args;
use std::io::{self, Write}; 
use std::fs::{self, File, OpenOptions}; 
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::Command;
use colored::*; 
use git_actions::*;
use serde::{Serialize, Deserialize};
use reqwest::header;

mod config_manager;
mod command_line;
mod git_actions;
#[derive(PartialEq)]
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

#[derive(Serialize, Deserialize)]
struct Config {
    api_key: String,
    username: String,
    project_path: String,
}

fn avoid_index_error(args: &mut Vec<&str>){
    args.push(" ");
    args.push(" ");
    args.push(" ");
    args.push(" ");
    args.push(" ");
    args.push(" ");
    args.push(" ");
    args.push(" ");
    args.push(" ");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Searching your project directory for git repositories, this may take a while depending on your project directory size...");

    let mut currentState = State::Home;

    // init config
    let mut user_config = config_manager::manage_config();


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
        let input = command_line::get_git_input(&currentState);
        
        // arguements for the commands and rawArgs for names -> for uppercase and lowercase letters
        let mut arguements = input.0.split_whitespace().collect::<Vec<&str>>();
        let mut rawArgs = input.1.split_whitespace().collect::<Vec<&str>>();
        
        // push some empty strings so no index error can occur
        avoid_index_error(&mut arguements);
        avoid_index_error(&mut rawArgs);

        // command loop
        match arguements[0] {
            "set" => {
                    let arg = arguements[1];
                    let change = rawArgs[2];
                    if command_line::check_if_empty_and_print_info(&arg.to_string(), "set username,set key,set path"){
                        if command_line::check_name(&change.to_string(), "Empty change is not valid"){

                            match arg {
                                "username" => {
                                    user_config.username = change.to_string();
                                    config_manager::write_to_json("./config.json", &user_config);
                                }
                                "key" => {
                                    user_config.api_key = change.to_string();
                                    config_manager::write_to_json("./config.json", &user_config);
                                }
                                "path" => {
                                    user_config.project_path = change.to_string();
                                    config_manager::write_to_json("./config.json", &user_config);
                                }
                                other => {
                                    command_line::throw_error(format!("Arguement {} is not valid", other).as_str());
                                }
                            }
                        
                    }
                }
            
                    
            }

            "show" => {
                if currentState == State::Config {
                    println!("{}: {}", "username".blue().underline(), &user_config.username);
                    println!("{}: {}", "api key".blue().underline(), &user_config.api_key);
                    println!("{}: {}", "Project path".blue().underline(), &user_config.project_path);
                }

                else {
                    if command_line::check_if_empty_and_print_info(arguements[1], "show config"){
                        if arguements[1] == "config"{
                            println!("{}: {}", "username".blue().underline(), &user_config.username);
                            println!("{}: {}", "api key".blue().underline(), &user_config.api_key);
                            println!("{}: {}", "Project path".blue().underline(), &user_config.project_path);
                        }
                        else {
                            command_line::throw_error(format!("Arguement '{}' not found", arguements[1]).as_str())
                        }
                    }
                }
            } 

            "open" =>{
                if !repo_names_list.contains(&rawArgs[1].to_string()){
                    git_actions::update_repos(&mut repo_list, &mut repo_names_list, &mut repo_path_list, &user_config); 
                }
                if command_line::check_if_empty_and_print_info(&arguements[1].to_string(), "open config,open <filename>"){

                    if arguements[1] == "config"{
                        currentState = State::Config;
                    }

                    else if repo_names_list.contains(&rawArgs[1].to_string()){
                        // only if the file exists
                        currentState = State::Repo(rawArgs[1].to_string());
                        command_line::print_in_file_infos();
                    }
                    else {
                        command_line::throw_error("File not found")
                    }
                }
            }

            "help" => {
                if State::Config == currentState {
                    command_line::print_config_infos();
                }
                if let State::Repo(ref _reponame) = currentState {
                    command_line::print_in_file_infos();
                }
                if State::Home == currentState {
                    command_line::print_infos();
                }
            }

            "upload" => {
                if arguements[1] == "all" {
                    for repo in &repo_list{
                        let mut force = false;
                        for arg in &arguements{
                            if arg == &"--force"{
                                force = true;
                            }
                        }
                        git_actions::upload(&repo.Path, &"commited by Git-Manager".to_string(), force);
                    }
                }

                else {
                    
                    // upload a specific file
                    // upload <file> <commit message> (--force)
                    // upload das hier ist die 
                    if let State::Repo(ref reponame) = currentState{
                        let force = git_actions::get_force(&arguements);
                        let commit_msg = git_actions::get_commit_msg(&rawArgs, 1);

                        git_actions::upload(&reponame.to_string(), &&commit_msg.to_string(), force);
                    }
                    else {
                        let name = rawArgs[1].to_string().clone();
                        if !repo_names_list.contains(&name){
                            git_actions::update_repos(&mut repo_list, &mut repo_names_list, &mut repo_path_list, &user_config);
                        }
                        if command_line::check_if_empty_and_print_info(&name, "upload all,upload <name> (commit message) (--force)"){
                            for repo in &repo_list{
                                if &repo.Name == &name {
                                    let force = git_actions::get_force(&arguements);
                                    let commit_msg = git_actions::get_commit_msg(&rawArgs, 2);
                                    git_actions::upload(&repo.Path, &commit_msg.to_string(), force);
                                    }
                            

                            }
                            if !repo_names_list.contains(&name){

                                command_line::throw_error("File not found");
                            }
                            
                        }
                    }
                }
            }

            "download" => {

                if command_line::check_if_empty_and_print_info(&arguements[1].to_string(), "download all from <user>,download <repo> from <user>"){
                    if arguements[1] == "all"{
                        if arguements[2] == "from" {
                            let username = rawArgs[3];
                            if command_line::check_name(&username.to_string(), "Username is missing"){
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
                            command_line::throw_error("Keyword 'from' is missing: download all from <user>");
                        }
                    }
                    else {
                        let repoName = rawArgs[1];
                        
                        if command_line::check_name(&repoName.to_string(), "Repository name is missing"){
                            if arguements[2] == "from" {
                                let username = rawArgs[3];
                                if command_line::check_name(&username.to_string(), "Username is missing"){
                                    git_actions::download(&repoName.to_string(), &username.to_string(), &user_config.project_path)
                                }
                            }
                            else{
                                command_line::throw_error("Keyword 'from' is missing: download <repo> from <user>");
                            }
                        }
                    }
                }
            }

            "update" => {
                if arguements[1] == "all" {
                    for repo in &repo_list {
                        let force = git_actions::get_force(&arguements);
                        git_actions::update(&repo.Path, force);
                    }
                }
                else {
                    // update a specific file
                    if let State::Repo(ref reponame) = currentState{
                        let force = git_actions::get_force(&arguements);
                        git_actions::update(&reponame.to_string(), force);
                    }
                    else {
                        let name = rawArgs[1].to_string().clone();
                        if !repo_names_list.contains(&name){
                            git_actions::update_repos(&mut repo_list, &mut repo_names_list, &mut repo_path_list, &user_config);
                        }
                        if command_line::check_if_empty_and_print_info(&name, "update all (--force),update <name> (--force)"){
                            if repo_names_list.contains(&name) {
                                let force = git_actions::get_force(&arguements);
                                git_actions::update(&name, force);
                            }
                            else {
                                command_line::throw_error("File not found");
                            }
                        }
                   }
                }
            }
            
            "delete" => {
                let secondArg = arguements[1];
                let mut message ;
                if let State::Repo(ref _reponame) = currentState{
                    message = "delete repo,delete branch <name>"
                }
                else {
                    message = "delete repo <name>,delete branch <name> from <repo>,delete folder <name>"
                }
                
                if command_line::check_if_empty_and_print_info(&secondArg.to_string(), &message){
                    match secondArg{
                        "folder" => {
                            let name = rawArgs[2];
                            let path = git_actions::find_file_in_path(&user_config.project_path, &name);
                            if command_line::check_name(&name.to_string(), "Folder name is missing"){
                                // delete folder <name>
                                match path {
                                    Ok(filePath) => {
                                        git_actions::deleteDir(&filePath);
                                    }
                                    Err(git_actions::SearchError::NotFound) => command_line::throw_error(format!("Folder '{}' not found in {}", &name, &user_config.project_path).as_str()),
                                    Err(git_actions::SearchError::MultipleFound) => command_line::throw_error(format!("Found multiple projects with the name '{}'", &name).as_str()),
                                    other => command_line::throw_error("Unknown error ocurred while migrating"),
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
                                if command_line::check_name(&name.to_string(), "Repository name is missing"){
                                    if repo_names_list.contains(&name.to_string()) {
                                        git_actions::delete_repo(&name.to_string(), &user_config.api_key);
                                    }
                                    else {
                                        command_line::throw_error("Repository not found")
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
                                if command_line::check_name(&branchName.to_string(), "Branch name is missing"){
                                    if arguements[3] == "from" {
                                        let repoName = rawArgs[4];
                                        if command_line::check_name(&repoName.to_string(), "Repository name is missing"){

                                        git_actions::delete_branch(&repoName.to_string(), &branchName.to_string(), &user_config.api_key);
                                        }
                                    }
                                    else {
                                        command_line::throw_error("Keyword 'from' is missing: delete branch <name> from <repo>")
                                    }
                                }
                            } 
                        }
                        other => command_line::throw_error(format!("Arguement '{}' is not valid", secondArg).as_str())
                        


                    }

                }

                // update repo list, names, and path in case a repo got added or deleted
                git_actions::update_repos(&mut repo_list, &mut repo_names_list, &mut repo_path_list, &user_config);
            }

            "create" => {
                let name = rawArgs[2].to_string();
                let mut message = "";
                if let State::Repo(ref reponame) = currentState{
                    message = "create branch <name>"
                }
                else {
                    message = "create repo <name>,create branch <name> in <repo>"
                }
                
                if command_line::check_if_empty_and_print_info(&arguements[1].to_string(), &message){

                    if arguements[1] == "branch"{
                        if command_line::check_name(&name, "Branch name is missing"){
                            if let State::Repo(ref repoName) = currentState {
                                git_actions::create_branch(&repoName, &name, &user_config.api_key);
                            }
                            else {
                                if arguements[3] == "in"{
                                    let repoName = rawArgs[4];
                                    if command_line::check_name(&repoName.to_string(), "Repository name is missing"){
                                        git_actions::create_branch(&repoName.to_string(), &name, &user_config.api_key);
                                    }
                                }

                                else {
                                    command_line::throw_error("Keyword 'in' is missing: create branch <name> in <repo>");
                                }
                            }
                        }    
                    }
                    else if arguements[1] == "repo" {
                        if command_line::check_name(&name, "Repository name is missing"){
                            // create repo <name> <public / private> 
                            let name = rawArgs[2];
                            let privacy = arguements[3];
                            git_actions::create_repo(&name.to_string(), &true, &"sdfsdfsdfsf".to_string(), &"sdfsfsdf".to_string());

                        }       
                    } 
                    else {
                        command_line::throw_error(format!("Arguement '{}' is not valid", arguements[1]).as_str())
                    }
                }

                // update repo list, names, and path in case a repo got added or deleted
                git_actions::update_repos(&mut repo_list, &mut repo_names_list, &mut repo_path_list, &user_config);
            }

            "migrate" => {

                let name = rawArgs[1].to_string();

                let projectPath = String::from(&user_config.project_path);
                if command_line::check_if_empty_and_print_info(&name.to_string(), "migrate <project> <public / private>"){
                    match git_actions::find_file_in_path(&projectPath, &name) {
                        Ok(file_path) => {
                            if !repo_names_list.contains(&name) {
                                let privacy = arguements[2];
                                if command_line::check_name(&privacy.to_string(), "Privacy arguement is missing: migrate <project> <public/private>"){
                                    match privacy {
                                        "public" => {
                                            migrate(&file_path, &name, true, &user_config.api_key);
                                        } 
                                        "private" => {
                                            migrate(&file_path, &name, true, &user_config.api_key);
                                        }
                                        other => {
                                            command_line::throw_error(format!("Privacy arguement '{}' is invalid (public/private)", privacy).as_str());
                                        }
                                    }
                                }
                            }
                            
                            else {
                                command_line::throw_error("Project is already on git")
                            }
                        }
                        Err(git_actions::SearchError::NotFound) => command_line::throw_error(format!("Project '{}' not found in {}", &name, &projectPath).as_str()),
                        Err(git_actions::SearchError::MultipleFound) => command_line::throw_error(format!("Found multiple projects with the name '{}'", &name).as_str()),
                        other => command_line::throw_error("Unknown error ocurred while migrating"),
                    }

                }   
                git_actions::update_repos(&mut repo_list, &mut repo_names_list, &mut repo_path_list, &user_config);

            }

            "list" => {

                // update repo list, names, and path in case a repo got added or deleted
                git_actions::update_repos(&mut repo_list, &mut repo_names_list, &mut repo_path_list, &user_config);
                if repo_path_list.len() == 0 {
                    command_line::throw_error("No git projects found")
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
            other => command_line::throw_error(format!("Command '{}' not found", other).as_str())

        };        
    
    }
    Ok(())
}
