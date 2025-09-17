use clap::Parser;
use image::GenericImageView;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{error::Error, str::FromStr};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
struct Pos {
    x: u32,
    y: u32,
}

impl FromStr for Pos {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(format!("invalid pos: {}", s));
        }
        let x = parts[0]
            .parse()
            .map_err(|_| format!("invalid x: {}", parts[0]))?;
        let y = parts[1]
            .parse()
            .map_err(|_| format!("invalid y: {}", parts[1]))?;

        Ok(Pos { x, y })
    }
}

/// Minecraft Item Icon Cropper
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// ファイルパス
    #[arg(short, long, required_unless_present = "directory")]
    file: Option<String>,

    /// ディレクトリパス
    #[arg(short, long, required_unless_present = "file")]
    directory: Option<String>,

    /// 出力パス(デフォルトはoutput)
    #[arg(short, long, default_value = "output")]
    output: String,

    /// 背景透過設定(例: "255,255,0")
    #[arg(short, long)]
    color: Option<String>,

    /// アイテムフレームサイズ(実際のアイテムがあるわけではない枠の部分)
    #[arg(short = 'S', long)]
    framesize: Option<u32>,

    /// アイテムサイズ
    #[arg(short, long, required = true)]
    size: u32,

    /// 実際にアイテムリストが始まる位置(--pos x,y)
    #[arg(long, required = true)]
    pos: Pos,
}

fn get_filename(path: &Path) -> String {
    if let Some(stem) = path.file_stem() {
        stem.to_string_lossy().to_string()
    } else {
        "Err".to_string()
    }
}

fn file_processor(image_path: &Path, args: &Args) -> Result<()> {
    println!(
        "Start processing...: {}",
        image_path.to_string_lossy().to_string()
    );
    let now = Instant::now();

    let image = image::open(&image_path)?;
    let (width, height) = image.dimensions();
    let image_name = get_filename(&image_path);

    //create output/$image_name directory
    fs::create_dir_all(&format!("output/{}", image_name))?;

    (0..9)
        .flat_map(|x| (0..5).map(move |y| (x, y)))
        .enumerate()
        .par_bridge()
        .for_each(|(i, (x, y))| {
            let x_pos = args.pos.x + x * args.size;
            let y_pos = args.pos.y + y * args.size;
            if x_pos + args.size < width && y_pos + args.size < height {
                let sub = image.view(x_pos, y_pos, args.size, args.size);

                let mut path = PathBuf::from(&args.output);
                path.push(&image_name);
                path.push(format!("{}.png", i));

                sub.to_image()
                    .save(path)
                    .expect("Error: with saving image.");
            } else {
                eprintln!(
                    "File({}) skipped due to attempt to access outside index pixel.",
                    image_path.to_string_lossy()
                );
            }
        });
    let end = now.elapsed();
    println!(
        "Process complete:{}",
        image_path.to_string_lossy().to_string()
    );
    println!("time: {:?}", end);
    Ok(())
}

fn directory_processor(directory_path: &Path, args: &Args) -> Result<()> {
    println!(
        "Start directory processing...: {}",
        directory_path.to_string_lossy().to_string()
    );
    let now = Instant::now();

    for image in fs::read_dir(directory_path)? {
        let image = image?;
        let path = image.path();
        let _ = file_processor(&path, args);
    }

    println!(
        "Direcotry process complete:{}",
        directory_path.to_string_lossy().to_string()
    );
    println!("time: {:?}", now.elapsed());
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Minecraft Item Icon Cropper");
    println!("===========================");

    match (&args.file, &args.directory) {
        (Some(file), None) => {
            println!("Running with file processing mode.");
            let path = Path::new(&file);
            file_processor(&path, &args)?;
        }
        (None, Some(dir)) => {
            println!("Running with directory processing mode.");
            let path = Path::new(&dir);
            directory_processor(&path, &args)?;
        }
        (Some(file), Some(dir)) => {
            eprintln!("両方指定されました！");
            eprintln!("ディレクトリフラグとファイル指定は共存できません。");
            eprintln!("file: {}, dir: {}", file, dir);
        }
        (None, None) => {
            eprintln!("エラー: file及び--directoryの指定がアリませんでした。");
            std::process::exit(1);
        }
    }

    Ok(())
}
