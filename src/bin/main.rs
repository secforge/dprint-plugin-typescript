use dprint_plugin_typescript::{configuration, format_text, FormatTextOptions};
use dprint_core::configuration::{ConfigKeyMap, ConfigKeyValue};
use std::env;
use std::path::Path;
use std::fs;
use std::io::{self, Read};

use serde_json;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut config_file: Option<String> = None;
    let mut code_arg: Option<String> = None;
    
    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--config-file" || arg == "-c" {
            if i + 1 < args.len() {
                config_file = Some(args[i + 1].clone());
                i += 2;
            } else {
                eprintln!("Error: --config-file requires a path argument");
                std::process::exit(1);
            }
        } else if arg.starts_with("--config-file=") {
            config_file = Some(arg[14..].to_string());
            i += 1;
        } else if arg == "--help" || arg == "-h" {
            print_help(&args[0]);
            std::process::exit(0);
        } else if !arg.starts_with("--") {
            code_arg = Some(arg.clone());
            i += 1;
        } else {
            eprintln!("Unknown argument: {}", arg);
            print_help(&args[0]);
            std::process::exit(1);
        }
    }
    
    // Get code from argument or stdin
    let code = if let Some(code_arg) = code_arg {
        code_arg
    } else {
        let mut buffer = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buffer) {
            eprintln!("Error reading from stdin: {}", e);
            std::process::exit(1);
        }
        buffer
    };
    
    // Load config from file if specified
    let mut config_map = ConfigKeyMap::new();
    if let Some(config_path) = config_file {
        match load_config_file(&config_path) {
            Ok(file_config) => {
                config_map = file_config;
            }
            Err(e) => {
                eprintln!("Error loading config file '{}': {}", config_path, e);
                std::process::exit(1);
            }
        }
    }
    
    let config_result = configuration::resolve_config(config_map, &Default::default());
    let config = &config_result.config;
    
    
    let original_code = code.clone();
    let options = FormatTextOptions {
        path: Path::new("input.js"),
        extension: Some("js"),
        text: code,
        config,
        external_formatter: None,
    };
    
    match format_text(options) {
        Ok(Some(formatted)) => {
            println!("{}", formatted);
        }
        Ok(None) => {
            println!("{}", original_code);
        }
        Err(e) => {
            eprintln!("Error formatting code: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_help(program_name: &str) {
    println!("Usage: {} [OPTIONS] [<code>]", program_name);
    println!();
    println!("Format JavaScript/TypeScript code. Reads from stdin if no code argument provided.");
    println!();
    println!("OPTIONS:");
    println!("  -c, --config-file <PATH>    Load configuration from dprint.json file");
    println!("  -h, --help                  Show this help message");
    println!();
    println!("Examples:");
    println!("  {} 'if(x)console.log(\"hi\");'", program_name);
    println!("  cat file.js | {} --config-file dprint.json", program_name);
    println!("  {} --config-file dprint.json < input.js > output.js", program_name);
}

fn load_config_file(path: &str) -> Result<ConfigKeyMap, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    
    let mut config_map = ConfigKeyMap::new();
    
    // Extract TypeScript configuration
    if let Some(typescript_config) = json.get("typescript") {
        if let Some(obj) = typescript_config.as_object() {
            for (key, value) in obj {
                let config_value = match value {
                    serde_json::Value::Bool(b) => ConfigKeyValue::from_bool(*b),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            ConfigKeyValue::from_i32(i as i32)
                        } else if let Some(f) = n.as_f64() {
                            ConfigKeyValue::from_i32(f as i32)
                        } else {
                            continue;
                        }
                    }
                    serde_json::Value::String(s) => ConfigKeyValue::from_str(s),
                    _ => continue,
                };
                config_map.insert(key.clone(), config_value);
            }
        }
    }
    
    Ok(config_map)
}