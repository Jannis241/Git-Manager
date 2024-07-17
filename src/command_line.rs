#![allow(non_snake_case)]
#![allow(unused)]
use crate::*;

pub fn new_lines(num_of_lines: usize) {
    println!("{}", "\n".repeat(num_of_lines));
}

pub fn clear_terminal() {
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().unwrap();
}

pub fn input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input.");
    input.trim().to_string()
}

pub fn print_in_file_infos() {
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

pub fn print_infos() {
    use colored::*;
    new_lines(1);
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
    println!("{}", "If you have a really large project directory some commands might take a while to execute.".italic().white());
    new_lines(1);
}



pub fn get_git_input(currentState: &State) -> (String, String) {
    match currentState {
        State::Home => {
            print!("<Git> ");
        }
        State::Repo(ref repoName) => {
            print!("<{}> ", &repoName);
        }
        State::Config => {
            print!("<config> ");
        }
    }

    let _ = io::stdout().flush();
    let mut rawInput = String::new();
    io::stdin().read_line(&mut rawInput).expect("Error");
    let rawInput = rawInput.trim().to_string();
    let input = rawInput.to_lowercase();
    (input, rawInput)
}


pub fn print_intro() {
    clear_terminal();
    println!("{}","Welcome to Git-Manager".white().bold().underline());
    print_infos();
}


pub fn print_config_infos() {
    use colored::*;
    new_lines(1);
    let commands_info = vec![
        ("set username <name>", "Change your username"),
        ("set path <path>", "Change your project Path"),
        ("set key <key>", "Change your api key"),
        ("show", "display your current settings"),
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
    new_lines(1);

    println!("{}", "These are just the commands that changed, others like 'download all' still work.".italic().white());
}