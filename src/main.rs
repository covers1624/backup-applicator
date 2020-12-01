use clap::{app_from_crate, Arg};
use zip::read::ZipArchive;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use std::{io, fs};
use chrono::Local;

fn main() {
    let matches = app_from_crate!()
        .arg(
            Arg::new("instance")
                .short('i')
                .long("instance")
                .value_name("FOLDER")
                .about("The instance folder to apply the backup to.")
                .required(true)
                .takes_value(true)
        )
        .arg(
            Arg::new("backup")
                .short('b')
                .long("backup")
                .value_name("FILE")
                .about("The backup to apply.")
                .required(true)
                .takes_value(true)
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .multiple_occurrences(true)
                .about("Enables verbose logging, can be provided multiple times for increasingly verbose logging.")
        )
        .get_matches();
    let instance = Path::new(matches.value_of("instance").unwrap());
    let backup = Path::new(matches.value_of("backup").unwrap());
    let verbose = matches.occurrences_of("verbose");

    if !instance.exists() {
        eprintln!("Instance path {:?} does not exist.", instance);
        exit(2);
    }

    if !backup.exists() {
        eprintln!("Backup {:?} does not exist.", backup);
        exit(3);
    }

    let properties_file = instance.join("server.properties");

    let mut level_name: String = String::from("world");
    if properties_file.exists() {
        if verbose >= 1 {
            println!("Found server.properties")
        }
        let file = match File::open(&properties_file) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Failed to open file '{:?}': {}", properties_file, e);
                exit(4);
            }
        };
        let mut properties = match java_properties::read(file) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Failed to parse server.properties file: {}", e);
                exit(4);
            }
        };
        match properties.remove("level-name") {
            Some(e) => { level_name = e }
            None => {
                if verbose == 1 {
                    eprintln!("level-name not found in server.properties, assuming 'world'.")
                }
            }
        }
    } else {
        if verbose >= 1 {
            eprintln!("Unable to find server.properties.")
        }
    }

    if verbose >= 1 {
        eprintln!("Using level-name: {}", &level_name);
    }

    let world_folder = instance.join(level_name);

    if world_folder.exists() {
        let time = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
        let to = world_folder.to_str().unwrap().to_string() + "-" + time.as_str();
        if verbose >= 1 {
            println!("Moving old world to: {}", &to)
        }
        match std::fs::rename(world_folder.to_str().unwrap(), to) {
            Err(e) => {
                eprintln!("Unable to rename world folder: {}", e);
                exit(4);
            }
            _ => {}
        }
    }

    let backup_file = match File::open(backup) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to open backup file '{:?}': {:?}", backup, e);
            exit(4);
        }
    };
    let mut zip = match ZipArchive::new(backup_file) {
        Ok(e) => { e }
        Err(e) => {
            eprintln!("Failed to read zip file '{:?}': {:?}", backup, e);
            exit(4);
        }
    };
    let mut prefix: String = String::new();

    for i in 0..zip.len() {
        let file = zip.by_index(i).unwrap();
        let name = file.name();

        if name.ends_with("level.dat") {
            let prefix2 = name.replace("level.dat", "");
            if !prefix.is_empty() {
                eprintln!("Found duplicate level.dat, A: {}, B: {}", prefix, prefix2);
                exit(5);
            }
            prefix = prefix2;
            if verbose >= 1 {
                println!("Found level.dat at: {}", name);
            }
        }
    }

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        if file.name().starts_with(&prefix) {
            let name = file.name().replacen(&prefix, "", 1);
            let path = Path::new(name.as_str());
            let full_path = world_folder.join(path);
            if verbose >= 2 {
                println!("Extracting {:?} to {:?}", file.name(), full_path);
            }
            if name.is_empty() || name.ends_with("/") {
                fs::create_dir_all(&full_path).unwrap()
            } else {
                if let Some(p) = full_path.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).unwrap()
                    }
                }
                let mut out_file = match File::create(&full_path) {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("Failed to open file '{:?}': {}", properties_file, e);
                        exit(4);
                    }
                };
                io::copy(&mut file, &mut out_file).unwrap();
            }

            #[cfg(unix)]
            if let Some(mode) = file.unix_mode() {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&full_path, fs::Permissions::from_mode(mode)).unwrap()
            }
        }
    }
}
