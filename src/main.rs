use image::imageops::FilterType;
use image::{io::Reader, ImageFormat}; // ImageOutputFormat
use regex::Regex;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self};
use std::path::PathBuf;
use std::process; // , Write
// use filetime::FileTime;
use filetime::{set_file_times, FileTime};

// fn main() -> std::io::Result<()> {
// fn main() -> Result<(), Box<dyn std::error::Error>> {
fn main() -> Result<(), Box<dyn Error>> {
    logo();
    let max_tries = 100;
    let mut max_mo = 1.0;
    let mut suffix = "_small".to_string();
    let args: Vec<String> = env::args().collect();

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
                                let metadata = fs::metadata(&entry.path())?;
                                let creation_time:FileTime = metadata.created()?.into();
                                let last_access_time:FileTime = metadata.modified()?.into();
                                set_file_times(&full_img_small_name, creation_time, last_access_time)?;
                            },
                            Err(_e) => {
                                errors.push(full_img_small_name.to_str().unwrap().to_owned())
                            }
                        }
                        num_tries += 1;
                    }
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
    dx("Done");
    Ok(())
}

fn dx(msg: &str) {
    println!("{}!", msg);
    process::exit(0);
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
