use std::{env, error::Error};
use serde_derive::Deserialize;
use std::path::Path;
use std::fs;
use serde::de::DeserializeOwned;
use image::{GenericImageView};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
struct Config {
    itemsize: u32,
    pos: Pos,
}

#[derive(Deserialize)]
struct Pos {
    x: u32,
    y: u32,
}

fn deserial_toml<T>(path: &Path) -> Result<T> where T: DeserializeOwned {
    let file = fs::read_to_string(path)?;
    let cfg = toml::from_str::<T>(&file)?;

    Ok(cfg)
}

fn you_need_the_help() {
    println!("つかいかた: mc-item-icons [options]... [file]...");
    println!("-d     file指定をディレクトリ指定に変更");
    println!();

    println!("mc-item-icons     file.png");
    println!("mc-item-icons -d  fileDir");
}

fn get_filename(path: &Path) -> String {
    if let Some(stem) = path.file_stem() {
        stem.to_string_lossy().to_string()
    } else {
        "Err".to_string()
    }
}

fn file_processor(cfg: &Config, image_path: &Path) -> Result<()> {
    let image = image::open(&image_path)?;
    let (width, height) = image.dimensions();
    let image_name = get_filename(&image_path);
    
    //create output/$image_name directory
    fs::create_dir_all(&format!("output/{}", image_name))?;
    for x in 0..9 {
        for y in 0..5 {
            let x_pos = cfg.pos.x + x * cfg.itemsize;
            let y_pos = cfg.pos.y + y * cfg.itemsize;
            if x_pos + cfg.itemsize < width && y_pos + cfg.itemsize < height {
                let croped_image = image.crop_imm(x_pos, y_pos, cfg.itemsize, cfg.itemsize);
                let path = format!("output/{}/{}.png", image_name, x + y * 9);
                croped_image.save(path)?;
            } else {
                eprintln!("File({}) skipped due to attempt to access outside index pixel.", image_path.to_string_lossy());
            }
        }
    }
    Ok(())
}

fn directory_processor(cfg: &Config, directory_path: &Path) -> Result<()> {
    for image in fs::read_dir(directory_path)? {
        let image = image?;
        let path = image.path();
        let _ = file_processor(cfg, &path);
    };
    
    Ok(())
}

fn main() -> Result<()> {
    fs::create_dir_all("output")?;

    let args: Vec<String> = env::args().collect();
    let config_path = Path::new("config.toml");
    let config: Config = deserial_toml::<Config>(&config_path)?;

    match args.get(1).map(|s| s.as_str()) {
        Some("-d") | Some("--directory") => {
            match args.get(2).map(|s| s.as_str()) {
                Some(dir) => {
                    directory_processor(&config, &Path::new(dir))?;
                }
                None => {
                    eprintln!("エラー: -dオプションの後に値が指定されていません。");
                }
            }
        },
        Some("-h") | Some("--help") => you_need_the_help(),
        Some(default) => file_processor(&config, &Path::new(default))?,
        None => {
            eprintln!("エラー: 引数がありません。");
            you_need_the_help();
        }
    };

    Ok(())
}
