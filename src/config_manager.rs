use crate::*;

pub fn manage_config() -> Config{
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

pub fn write_to_json(file_path: &str, config: &Config) {
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
