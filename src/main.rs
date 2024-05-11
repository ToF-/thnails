use std::io;
use std::io::{Error,ErrorKind};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use clap::Parser;
use thumbnailer::{create_thumbnails, ThumbnailSize};
use thumbnailer::error::{ThumbResult, ThumbError};
use std::fs::{File, create_dir_all};
use std::io::BufReader;

// . -> thumbnails
// tests/assets -> thumbnails/tests/assets

type Entry = String;
#[derive(Debug)]
struct EntryPair {
    source: Entry,
    target: Entry,
}

// declarative setting of arguments
/// Thnails
#[derive(Parser, Clone, Debug)]
#[command(infer_subcommands = true, infer_long_args = true, author, version, about, long_about = None)]
/// Pattern that displayed files must have
struct Args {
    /// file only (default: diretory)
    #[arg(short, long, default_value_t = false)] 
    file: bool,

    /// Source directory (or file) to search
    #[arg(short, long)]
    source: String,
    /// Target directory (or file) to create
    #[arg(short, long)]
    target: String,
}

fn get_target_folders(source_path: &Path, target: String) -> io::Result<Vec<Entry>> {
    let mut entries: Vec<Entry> = Vec::new();
    for entry in WalkDir::new(source_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() {
            let path = entry.into_path();
            let sub_path = path.strip_prefix(source_path).unwrap();
            let target_path = Path::new(&target).join(sub_path);
            entries.push(target_path.display().to_string());
        }
    };
    Ok(entries)
}

fn get_entry_pairs(source_path: &Path, target: String) -> io::Result<Vec<EntryPair>> {
    let mut pairs: Vec<EntryPair> = Vec::new();
    for entry in WalkDir::new(source_path).into_iter().filter_map(|e| e.ok()) {
        let path: PathBuf = entry.into_path();
        let valid_ext = if let Some(ext) = path.extension() {
            ext == "jpg" || ext == "jpeg" || ext == "png" || ext == "JPG" || ext == "JPEG" || ext == "PNG"
        } else {
            false
        };
        if valid_ext {
            let sub_path = path.strip_prefix(source_path).unwrap();
            let mut work_path = Path::new(&target).join(sub_path);
            let extension = work_path.extension().unwrap();
            let file_stem = work_path.file_stem().unwrap();
            let new_file_name = format!("{}THUMB.{}",
                    file_stem.to_str().unwrap(),
                    extension.to_str().unwrap());
            work_path.set_file_name(new_file_name);
            let target_path: PathBuf = Path::new(work_path.as_path()).to_path_buf();
            pairs.push(EntryPair { 
                source: path.into_os_string().into_string().unwrap(),
                target: work_path.into_os_string().into_string().unwrap(), });
        }
    };
    Ok(pairs)
}

fn create_thumbnail(source: String, target: String, number: Option<usize>) -> ThumbResult<()> {
    if let Some(n) = number {
        println!("{:6} {}", n, target.clone())
    } else {
        println!("{}", target.clone())
    };
    match File::open(source.clone()) {
        Err(err) => {
            println!("error opening file {}: {}", source, err);
            return Err(ThumbError::IO(err))
        },
        Ok(input_file) => {
            let source_path = Path::new(source.as_str());
            let source_extension = match source_path.extension().and_then(OsStr::to_str) {
                None => {
                    println!("error: file {} has no extension", source.clone());
                    return Err(ThumbError::IO(Error::new(ErrorKind::Other, "no extension")))
                },
                Some(s) => s,
            };
            let reader = BufReader::new(input_file);
            let mut output_file = match File::create(target.clone()) {
                Err(err) => {
                    println!("error while creating file {}: {}",
                        target.clone(),
                        err);
                    return Err(ThumbError::IO(err))
                },
                Ok(file) => file,
            };
            match source_extension {
                "jpg" | "jpeg" | "JPG" | "JPEG" => {
                    let mut thumbnails = match create_thumbnails(reader, mime::IMAGE_JPEG, [ThumbnailSize::Small]) {
                        Ok(tns) => tns,
                        Err(err) => {
                            println!("error while creating thumbnails:{:?}", err);
                            return Err(err)
                        },
                    };
                    let thumbnail = thumbnails.pop().unwrap();
                    let _ = match thumbnail.write_jpeg(&mut output_file,255) {
                        Err(err) => {
                            println!("error while writing jpeg:{}", err);
                            return Err(err)
                        },
                        Ok(()) => (),
                    };
                    Ok(())
                },
                "png" | "PNG" => {
                    let mut thumbnails = match create_thumbnails(reader, mime::IMAGE_PNG, [ThumbnailSize::Small]) {
                        Err(err) => {
                            println!("error while creating thumbnails:{}", err);
                            return Err(err)
                        },
                        Ok(tns) => tns,
                    };
                    let thumbnail = thumbnails.pop().unwrap();
                    match thumbnail.write_png(&mut output_file) {
                        Err(err) => {
                            println!("error while writing png:{}", err);
                            return Err(err)
                        },
                        Ok(()) => (),
                    };
                    Ok(())
                },
                _ => Ok(()),
            }
        }
}
}

fn create_all_thumbnails(pairs: Vec<EntryPair>) -> io::Result<()> {
    let mut i: usize = 0;
    for pair in pairs {
        create_thumbnail(pair.source, pair.target, Some(i));
        i += 1
    }
    Ok(())
}

fn create_target_folders(folders: Vec<Entry>) -> io::Result<()> {
    for entry in folders{
        println!("creating folder {}", entry);
        if let Err(err) = create_dir_all(entry) {
            return Err(err)
        }
    };
    Ok(())
}
fn main() {
    let args = Args::parse();
    if args.file {
        create_thumbnail(args.source, args.target, None);
    } else {
        let path = Path::new(&args.source);
        match get_target_folders(path, args.target.clone()) {
            Ok(folders) => match create_target_folders(folders) {
                Ok(_) => match get_entry_pairs(path, args.target) {
                    Ok(pairs) => {
                        println!("{}", pairs.len());
                        match create_all_thumbnails(pairs) {
                            Ok(()) => (),
                            Err(err) => println!("{}", err),
                        }
                    }
                    Err(err) => println!("{}", err),
                }
                Err(err) => println!("{}", err)
            },
            Err(err) => println!("{}", err)
        }
    }
}
