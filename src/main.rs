use std::{env::args, fs::{create_dir_all, File}, io::Write, path::Path, process::{self, Command}};
use curl::easy::{Easy, WriteError};

fn download_with_curl(url: &str, output_path: &str) -> Result<(), String> {
    let mut file = File::create(output_path).map_err(|e| e.to_string())?;

    let mut easy = Easy::new();
    easy.url(url).map_err(|e| e.to_string())?;

    easy.write_function(move |data| {
        file.write_all(data)
            .map_err(|_| WriteError::Pause)
            .map(|_| data.len())
    }).map_err(|e| e.to_string())?;

    easy.perform().map_err(|e| e.to_string())?;

    let response_code = easy.response_code().map_err(|e| e.to_string())?;
    if response_code == 200 {
        println!("Downloaded and saved to {}", output_path);
        Ok(())
    } else {
        Err(format!("Failed to download file: HTTP {}", response_code))
    }
}

fn start() -> Result<(), String> {
    let args: Vec<String> = args().collect();
    let url = "https://api.papermc.io/v2/projects/paper/versions/1.21.1/builds/123/downloads/paper-1.21.1-123.jar";
    let filename = "paper-1.21.1-123.jar";
    let eula = "eula.txt";

    let path = if args.len() != 2 {
        Path::new("./").join(filename)
    } else {
        Path::new(&args[1]).join(filename)
    };

    let dir = path.parent().unwrap();
    create_dir_all(dir).map_err(|e| e.to_string())?;

    let eula_path = dir.join(eula);
    match File::create(&eula_path) {
        Ok(mut file) => {
            match file.write_all(b"eula=true") {
                Ok(_) => println!("Created {} file", eula),
                Err(e) => println!("Error writing to {} file: {}", eula, e),
            }
        }
        Err(e) => {
            println!("Error creating {} file: {}", eula, e);
        }
    }

    download_with_curl(url, &path.to_string_lossy())?;

    println!("Starting PaperMC server...");

    let start_script = if cfg!(target_os = "windows") {
        dir.join("start.bat")
    } else {
        dir.join("start.sh")
    };

    match File::create(&start_script) {
        Ok(mut file) => {
            let script_content = if cfg!(target_os = "windows") {
                format!("@echo off\njava -jar {} nogui\npause", filename)
            } else {
                format!("#!/bin/sh\njava -jar {} nogui", filename)
            };

            match file.write_all(script_content.as_bytes()) {
                Ok(_) => println!("Created start script"),
                Err(e) => println!("Error writing to start script: {}", e),
            }
        }
        Err(e) => {
            println!("Error creating start script: {}", e);
        }
    }

    if cfg!(target_os = "unix") {
        let _ = process::Command::new("chmod")
            .arg("+x")
            .arg(start_script.to_str().unwrap())
            .status()
            .map_err(|e| format!("Failed to make script executable: {}", e))?;
    }

    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", start_script.to_str().unwrap()])
            .status()
            .map_err(|e| format!("Failed to run the start script: {}", e))?
    } else {
        Command::new("sh")
            .arg(start_script.to_str().unwrap())
            .status()
            .map_err(|e| format!("Failed to run the start script: {}", e))?
    };

    if status.success() {
        println!("Server started successfully.");
    } else {
        return Err("Failed to start the server".to_string());
    }

    Ok(())
}

fn main() {
    match start() {
        Ok(()) => println!("Finished successfully"),
        Err(e) => println!("Error: {}", e),
    }    
}
