use std::io;
use std::path::{Path,PathBuf};
use walkdir::WalkDir;
use clap::{Parser,ValueEnum};
use clap_num::number_range;
use thumbnailer::{create_thumbnails, Thumbnail, ThumbnailSize};
use std::fs::File;
use std::io::{Write,BufReader, BufWriter};
use std::io::Cursor;

// . -> thumbnails
// tests/assets -> thumbnails/tests/assets


// declarative setting of arguments
/// Thnails
#[derive(Parser, Clone, Debug)]
#[command(infer_subcommands = true, infer_long_args = true, author, version, about, long_about = None)]
/// Pattern that displayed files must have
struct Args {
    /// Directory to search
    #[arg(short, long, default_value=".")]
    directory: Option<String>,
}

fn append_dir(path: &Path, dir: &str) -> PathBuf {
    let dirs = path.parent().unwrap();
    dirs.join(dir).join(path.file_name().unwrap())
}

fn get_files_in_directory(dir_path: &str) -> io::Result<Vec<String>> {
    let mut entries: Vec<String> = Vec::new();
    let source_path = Path::new(dir_path);
    let dest_path = append_dir(source_path, "thumbnails");
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.into_path();
        println!("{} {}",dest_path.display(.display()), path.display());
    }
    Ok(entries)
}
fn main() {
    let args = Args::parse();
    println!("{:?}", args);
    if let Some(path) = args.directory {
        get_files_in_directory(path.as_str())
    } else {
        panic!("can't search directory");
    };
    let input_file = File::open("tests/assets/test.png").unwrap();
    let reader = BufReader::new(input_file);
    let mut  thumbnails = create_thumbnails(reader, mime::IMAGE_PNG, [ThumbnailSize::Small, ThumbnailSize::Medium]).unwrap();

    let thumbnail = thumbnails.pop().unwrap();
    let mut buf = Cursor::new(Vec::<u8>::new());
    let mut output_file = File::create("tests/assets/test-th-small.png").unwrap();
    thumbnail.write_png(&mut output_file).unwrap();
}
