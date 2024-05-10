use std::io;
use std::ffi::OsStr;
use std::path::Path;
use walkdir::WalkDir;
use clap::Parser;
use thumbnailer::{create_thumbnails, ThumbnailSize};
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
        let path = entry.into_path();
        let valid_ext = if let Some(ext) = path.extension() {
            ext == "jpg" || ext == "jpeg" || ext == "png" || ext == "JPG" || ext == "JPEG" || ext == "PNG"
        } else {
            false
        };
        if valid_ext {
            let sub_path = path.strip_prefix(source_path).unwrap();
            let target_path = Path::new(&target).join(sub_path);
            pairs.push(EntryPair { 
                source: path.into_os_string().into_string().unwrap(),
                target: target_path.into_os_string().into_string().unwrap(), });
        }
    };
    Ok(pairs)
}

// called `Result::unwrap()` on an `Err` value: Image(Decoding(DecodingError { format: Exact(Jpeg), underlying: Some(Format("first two bytes are not an SOI marker")) }))

fn get_extension_from_filename(path: &Path) -> Option<&str> {

    path.extension()
        .and_then(OsStr::to_str)
}
fn create_all_thumbnails(pairs: Vec<EntryPair>) -> io::Result<()> {
    let mut i: usize = 0;
    for pair in pairs {
        match File::open(pair.source.clone()) {
            Ok(input_file) => {
                let source_path = Path::new(pair.source.as_str());
                let source_extension = get_extension_from_filename(source_path).unwrap();
                let reader = BufReader::new(input_file);
                let mut output_file = File::create(pair.target.clone()).unwrap();
                println!("{} {}",i, pair.target);
                match source_extension {
                    "jpg" | "jpeg" | "JPG" | "JPEG" => {
                        let mut thumbnails = create_thumbnails(reader, mime::IMAGE_JPEG, [ThumbnailSize::Small]).unwrap();
                        let thumbnail = thumbnails.pop().unwrap();
                        thumbnail.write_jpeg(&mut output_file,255).unwrap()
                    },
                    "png" | "PNG" => {
                        let mut thumbnails = create_thumbnails(reader, mime::IMAGE_PNG, [ThumbnailSize::Small]).unwrap();
                        let thumbnail = thumbnails.pop().unwrap();
                        thumbnail.write_png(&mut output_file).unwrap()
                    },
                    _ => (),
                }
                // let reader = BufReader::new(input_file);
                // let mut  thumbnails = create_thumbnails(reader, mime::IMAGE_PNG, [ThumbnailSize::Small, ThumbnailSize::Medium]).unwrap();

                // let thumbnail = thumbnails.pop().unwrap();
                // let mut buf = Cursor::new(Vec::<u8>::new());
                // let mut output_file = File::create("tests/assets/test-th-small.png").unwrap();
                // thumbnail.write_png(&mut output_file).unwrap();
            }
            Err(_) => {
                println!("can't open: {}", pair.source)
            }
        };
        i+=1;
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
        let source_path = Path::new(&args.source);
        match File::open(&args.source.clone()) {
            Ok(input_file) => {
                let source_extension = get_extension_from_filename(source_path).unwrap();
                let reader = BufReader::new(input_file);
                let mut output_file = File::create(&args.target).unwrap();
                match source_extension {
                    "jpg" | "jpeg" | "JPG" | "JPEG" => {
                        let mut thumbnails = create_thumbnails(reader, mime::IMAGE_JPEG, [ThumbnailSize::Small]).unwrap();
                        let thumbnail = thumbnails.pop().unwrap();
                        thumbnail.write_jpeg(&mut output_file,255).unwrap()
                    },
                    "png" | "PNG" => {
                        let mut thumbnails = create_thumbnails(reader, mime::IMAGE_PNG, [ThumbnailSize::Small]).unwrap();
                        let thumbnail = thumbnails.pop().unwrap();
                        thumbnail.write_png(&mut output_file).unwrap()
                    },
                    _ => (),
                }
            },
            Err(err) => {
                println!("error: {}", err)
            }
        }
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
