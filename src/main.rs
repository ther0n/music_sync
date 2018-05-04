use std::collections::LinkedList;
use std::io;
use std::io::Write;
use std::fs::File;
use std::fs;
use std::io::Read;
use std::process::Command;
//TODO multithread conversion
//use std::thread;

extern crate yaml_rust;
use yaml_rust::YamlLoader;
extern crate glob;
use glob::glob;

use std::path::Path;

fn main() {
    // Load the config file

    let mut input = String::new(); // used for input from the user
                                   //let config_file = File::open("config.yaml").unwrap(); // load the config file
    let mut config_file = match File::open("config.yaml") {
        Ok(x) => x,
        Err(err) => {
            println!("ERROR: config.yaml not found!");
            return Err(err).unwrap();
        }
    };
    let mut config_data = String::new(); // create a new string to hold the data in the config
    config_file.read_to_string(&mut config_data).unwrap(); // read config file into string
    let config_yaml = match YamlLoader::load_from_str(&config_data) {
        Ok(x) => x,
        Err(err) => {
            println!("ERROR: config.yaml invalid!");
            return Err(err).unwrap();
        }
    }; // parse the data from the config
    let config = &config_yaml[0]; //extract the first item from the parsed data (the actual config)

    // Load options from config file

    let source = config["source"].as_str().unwrap(); // get the source directory from the config file
    let destination = config["destination"].as_str().unwrap(); // get the destination directory from the config file
    let split_formats = config["formats"].as_str().unwrap().split(" "); // get list of formats to convert/copy
    let formats = split_formats.collect::<Vec<&str>>(); // collect the formats into a Vec

    // Display options loaded from the config file and prompt user to continue

    println!("Source: {}", source);
    println!("Destination: {}", destination);
    println!("Copy/Convert these formats: {:?}", formats);
    println!("Press enter to continue...");
    io::stdin().read_line(&mut input).ok();

    // Create list of files to convert/copy and sync

    let mut path_list: LinkedList<String> = LinkedList::new(); // Create a linked list to hold each of the files that will be copied
    let mut progress: String; // Create a string that will display the number of files found while scanning
    let mut num_files_found: u32 = 0; // Holds the number of files found
    for format in formats {
        let mut pattern = String::from(source); // Create a string to hold the search pattern for glob
                                                // Use backslash on windows
        #[cfg(target_family = "windows")]
        pattern.push_str("**\\*.");
        // Forward slash on linux/unix
        #[cfg(target_family = "unix")]
        pattern.push_str("**/*.");

        pattern.push_str(format); // Push the format extension to the string
        println!("Searching pattern: {}", pattern); // Display the search pattern
        let mut num_files_pre_scan = num_files_found; // Save the number of files before scan to later display number of new files found with pattern
        for entry in glob(&pattern).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => path_list.push_back(path.display().to_string()), // Push the path to the list of file paths
                Err(e) => println!("{:?}", e),
            }
            num_files_found += 1; // Increment files found
                                  // Update progress string
            progress = String::from("Number of files to copy: ");
            progress.push_str(&num_files_found.to_string());
            update_line(progress); // Display progress on current line without printing new line
        }
        // Display number of new files found with pattern
        println!(
            "Found {} more files to sync",
            num_files_found - num_files_pre_scan
        );
    }
    println!("There are {} files to sync", num_files_found); // Display number of files to sync to the user
        for file_path in path_list {
            let source_ext = Path::new(&file_path).extension().unwrap().to_str().unwrap(); // Get the file extension for the file
            let mut command =
                String::from(config["convert"][source_ext]["command"].as_str().unwrap()); // Load the command to execute for the extension from config
            let dest_ext = String::from(config["convert"][source_ext]["output"].as_str().unwrap()); // Load the extension for the destination file
            command = command.replace("$source", &file_path); // Replace $source in the config file with the source path
                                                              // Create a path to the destination file based on the source path
            let mut dest_file_path = String::from(
                Path::new(&file_path)
                    .with_extension(dest_ext)
                    .to_str()
                    .unwrap(),
            );
            dest_file_path = dest_file_path.replace(source, destination); // Replace the source path with the destination path
            command = command.replace("$destination", &dest_file_path); // Replace $destination int he config file with the destination path

            // Skip command if destination file exists
            // Otherwise execute command to convert and sync file
            if Path::new(&dest_file_path).exists() {
                println!("Skipping {} because it already exists", &dest_file_path);
            } else {
                fs::create_dir_all(Path::new(&dest_file_path).parent().unwrap()).ok(); // Create directory leading to destination file
                convert(&command); // Execute command
            }
        }
        println!("\nDone!");
}

fn convert(command: &String) {
    println!("{}", command); // Display the command that will be executed

    // Executes the new command using sh -c
    // TODO: add windows support

    //let output = 
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");
}

fn update_line(string_to_print: String) {
    print!("{}\r", string_to_print);
    io::stdout().flush().ok().expect("Could not flush stdout");
}
