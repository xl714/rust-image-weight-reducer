use filetime::set_file_times;
use filetime_creation::FileTime;
use image::imageops::FilterType;
use image::{io::Reader, ImageFormat}; // ImageOutputFormat
use regex::Regex;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self};
use std::path::Path;
use std::path::PathBuf; // Import the Path type
use std::time::{Duration, Instant};
// use std::process;
// use filetime::{set_file_times, FileTime};
// use std::fs::Metadata;
// use std::time::{SystemTime, UNIX_EPOCH};

// fn main() -> std::io::Result<()> {
// fn main() -> Result<(), Box<dyn std::error::Error>> {

fn set_creation_and_updated_date_default(
    entry: &fs::DirEntry,
    full_img_small_name: &Path,
) -> Result<(), std::io::Error> {
    let metadata = fs::metadata(&entry.path())?;
    let creation_time: FileTime = metadata.created()?.into();
    println!("   => creation_time {} ...", creation_time);
    let last_access_time: FileTime = metadata.modified()?.into();
    set_file_times(&full_img_small_name, creation_time, last_access_time)?;

    let new_metadata = fs::metadata(&full_img_small_name)?;
    let new_creation_time: FileTime = new_metadata.created()?.into();
    println!("   => new_creation_time {} ...", new_creation_time);
    if creation_time != new_creation_time {
        println!("   => creation_time != new_creation_time ==> trying another way");
        //let _ = set_creation_date_specific(&creation_time, &entry, &full_img_small_name)?;
        match set_creation_date_specific(&creation_time, &entry, &full_img_small_name) {
            Ok(()) => {
                println!("    set_creation_date_specific => OK");
            }
            Err(err) => {
                println!("    => Error set_creation_date_specific: {}", err);
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn set_creation_date_specific(
    creation_time: &FileTime,
    entry: &fs::DirEntry,
    full_img_small_name: &Path,
) {
    use filetime_creation::set_file_ctime;
    // set_file_ctime(&entry.path(), creation_time)?;
    match set_file_ctime(&entry.path(), creation_time) {
        Ok(()) => (),
        Err(err) => {
            println!("    => Error setting creation time 2: {}", err);
            // Handle the error gracefully here
        }
    }
    let new_metadata_2 = fs::metadata(&full_img_small_name)?;
    let new_creation_time_2: FileTime = new_metadata_2.created()?.into();
    if creation_time != new_creation_time_2 {
        println!("   => creation_time != new_creation_time_2 ==> impossible to set creation time");
    }
}

fn file_time_to_string(file_time: &FileTime) -> String {
    use chrono::prelude::*;
    let datetime =
        NaiveDateTime::from_timestamp_opt(file_time.seconds(), file_time.nanoseconds() as u32)
            .expect("Invalid timestamp");
    let formatted = datetime.format("%Y%m%d%H%M");
    formatted.to_string()
}

#[cfg(target_os = "linux")]
fn set_creation_date_specific(
    creation_time: &FileTime,
    _entry: &fs::DirEntry,
    full_img_small_name: &Path,
) -> io::Result<()> {
    use std::process::Command;
    // Execute a command line

    let date_str = file_time_to_string(creation_time);
    println!("    Set date: {}", date_str);
    let output = Command::new("touch")
        .arg("-t")
        .arg(date_str)
        //.arg("202001151600.59")
        .arg(&full_img_small_name)
        .output()
        .expect("Failed to execute command");
    let output = String::from_utf8_lossy(&output.stdout); // output.stdout is a Vec<u8>, which is a vector of bytes.
    println!("    Output: {}", output);

    /*
    // ########## TODO: TRY
    #!/bin/bash

    # Step 1: Save the current date and time
    current_datetime=$(date +"%Y-%m-%d %H:%M:%S")

    # Step 2: Change the date and time to your desired values
    desired_datetime="2024-02-07 12:00:00"  # Change this to your desired date and time
    sudo date -s "$desired_datetime"

    # Step 3: Create the file
    touch your_file_name_here.txt

    # Step 4: Restore the original date and time
    sudo date -s "$current_datetime"

    */

    // ########## try failed
    // use libc::{timespec, utimensat, AT_FDCWD, UTIME_NOW};
    // use std::ffi::CString; // Import CString
    // let atime = timespec {
    //     tv_sec: creation_time.seconds(),
    //     tv_nsec: creation_time.nanoseconds() as i64,
    // };
    // let times = [atime, atime];
    // // Convert Path to CString
    // let c_path = CString::new(full_img_small_name.to_str().unwrap())?;
    // unsafe {
    //     utimensat(
    //         AT_FDCWD,
    //         c_path.as_ptr(),
    //         times.as_ptr(),
    //         UTIME_NOW.try_into().unwrap(),
    //     );
    // }
    Ok(())
}

// fn set_creation_date_specific(
//     _creation_time: &FileTime,
//     _entry_: &fs::DirEntry,
//     _full_img_small_name_: &Path,
// ) {
//     //
// }

fn main() -> Result<(), Box<dyn Error>> {
    logo();
    let max_tries = 100;
    let mut max_mo = 1.0;
    let mut suffix = "_small".to_string();
    let args: Vec<String> = env::args().collect();
    let mut start_time = Instant::now();

    // if first arg is "default" then print "using default values"
    if !(args.len() > 1 && args[1] == "default") {
        // let name = &args[1];
        // println!("Hello, {}!", name);

        loop {
            println!("Quelle taille de fichier chaque image ne doit pas dépasser ? en méga octets  (par défaut c'est {} Mb)", max_mo);
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            input = input.trim().to_string();

            if input.is_empty() {
                break;
            }
            match input.parse::<f64>() {
                Ok(val) => {
                    if val >= 1.0 && val <= 10.0 {
                        max_mo = val;
                        break;
                    } else {
                        println!("Valeur non valide. Veuillez entrer un nombre entre 1 et 10.");
                    }
                }
                Err(_) => {
                    println!("Valeur non valide. Veuillez entrer un nombre entre 1 et 10.");
                }
            }
        }

        start_time = Instant::now();

        let regex = Regex::new(r"^[0-9a-zA-Z\_\-\.]+$").unwrap();
        loop {
            println!(
                "Quel suffixe utiliser pour les fichiers créés ?  (par défaut c'est '{}')",
                suffix
            );
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            input = input.trim().to_string();

            if input.is_empty() {
                break;
            } else {
                if regex.is_match(&input) {
                    suffix = format!("_{}", input);
                    break;
                } else {
                    println!("Valeur non valide. Le suffixe ne peut contenir que des chiffres et des lettres.");
                }
            }
        }
    }

    println!(
        "Vous avez choisi le suffixe : {}, et la taille maximum : {}.",
        suffix, max_mo
    );

    let mut num_images = 0;
    let mut num_decreased = 0;
    let mut errors = Vec::new();

    for entry in fs::read_dir(".")? {
        let entry = entry?;
        if entry.path().is_file() {
            if let Some(extension) = entry.path().extension() {
                if extension == "jpg" || extension == "jpeg" || extension == "png" {
                    num_images += 1;

                    let full_img_name = entry
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned();
                    let path = entry.path();
                    let img_ext = path.extension().unwrap().to_str().unwrap();
                    let img_name = full_img_name.trim_end_matches(img_ext);

                    let mut full_img_small_name = PathBuf::new();
                    full_img_small_name
                        .set_file_name(format!("{}{}.{}", img_name, suffix, img_ext));
                    let full_img_small_name = full_img_small_name.as_path();

                    if full_img_small_name.exists() {
                        continue;
                    }

                    let file_size = fs::metadata(entry.path())?.len() as f64 / 1_000_000_f64;

                    if file_size < max_mo {
                        continue;
                    }

                    println!("Image {}: {}", full_img_name, file_size);

                    let mut img = Reader::open(entry.path())?.decode()?;
                    let mut num_tries = 0;
                    while fs::metadata(full_img_small_name)
                        .map(|meta| meta.len() as f64 / 1_000_000_f64)
                        .unwrap_or(999.0)
                        > max_mo
                        && num_tries < max_tries
                    {
                        if num_tries == 0 {
                            println!("   => Reducing {} ...", full_img_name);
                        }
                        img = img.resize_exact(
                            (img.width() as f32 * 0.95) as u32,
                            (img.height() as f32 * 0.95) as u32,
                            FilterType::Lanczos3,
                        );
                        match img.save_with_format(full_img_small_name, ImageFormat::Jpeg) {
                            Ok(_) => {
                                println!("   => Reduced #{} done", (num_tries + 1));
                            }
                            Err(_e) => {
                                errors.push(full_img_small_name.to_str().unwrap().to_owned())
                            }
                        }
                        num_tries += 1;
                    }

                    // SET CREATION AND MODIFIED DATE
                    let _ = set_creation_and_updated_date_default(&entry, &full_img_small_name);

                    println!("Image {} resized {} times.", full_img_name, num_tries);
                    num_decreased += 1;
                }
            }
        }
    }

    println!("Number of files in the directory: {}", num_images);
    println!("Number of image files: {}", num_images);
    println!("Number of images decreased: {}", num_decreased);
    if !errors.is_empty() {
        println!("Errors occurred while processing the following files:");
        for error in errors {
            println!("{}", error);
        }
    };

    let end_time = Instant::now();
    let elapsed_time = end_time - start_time;
    println!("Script execution time: {}", format_duration(elapsed_time));

    Ok(())
}

// fn dx(msg: &str) {
//     println!("{}!", msg);
//     process::exit(0);
// }

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let millis = duration.subsec_millis();
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        seconds / 3600,
        (seconds / 60) % 60,
        seconds % 60,
        millis
    )
}

fn logo() {
    println!(
        r#"

             ____  __  __    __     ___  ____             
            (_  _)(  \/  )  /__\   / __)( ___)            
             _)(_  )    (  /(__)\ ( (_-. )__)             
            (____)(_/\/\_)(__)(__) \___/(____)            
      _____  __  __    __    __    ____  ____  _  _      
     (  _  )(  )(  )  /__\  (  )  (_  _)(_  _)( \/ )     
      )(_)(  )(__)(  /(__)\  )(__  _)(_   )(   \  /      
     (___/\\(______)(__)(__)(____)(____) (__)  (__)      
 ____   ____   ___  ____  ____    __    ___  ____  ____ 
(  _ \ ( ___) / __)(  _ \( ___)  /__\  / __)( ___)(  _ \
 )(_) ) )__) ( (__  )   / )__)  /(__)\ \__ \ )__)  )   /
(____/ (____) \___)(_)\_)(____)(__)(__)(___/(____)(_)\_)
        ____  _  _    _  _  __    ___   __   __          
       (  _ \( \/ )  ( \/ )(  )  (__ ) /  ) /. |         
        ) _ < \  /    )  (  )(__  / /   )( (_  _)        
       (____/ (__)   (_/\_)(____)(_/   (__)  (_)         
              ____  _____  __  __  ____                    
             (  _ \(  _  )(  )(  )(  _ \                   
              )___/ )(_)(  )(__)(  )   /                   
             (__)  (_____)(______)(_)\_)                   
    ___  _____  _  _    ____    __    ____    __         
   / __)(  _  )( \( )  (  _ \  /__\  (  _ \  /__\        
   \__ \ )(_)(  )  (    )___/ /(__)\  )___/ /(__)\       
   (___/(_____)(_)\_)  (__)  (__)(__)(__)  (__)(__) 

    "#
    );
}

// https://docs.rs/image_hasher/latest/image_hasher/enum.FilterType.html
// Nearest	31 ms
// Triangle	414 ms
// CatmullRom	817 ms
// Gaussian	1180 ms
// Lanczos3: 1170 ms
// img = img.resize(downsampled_width, downsampled_height, FilterType::Lanczos3);
// img = img.resize_exact(downsampled_width, downsampled_height, FilterType::Lanczos3);
// img = img.thumbnail(downsampled_width, downsampled_height, FilterType::Lanczos3);
