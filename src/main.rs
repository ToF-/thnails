use thumbnailer::{create_thumbnails, Thumbnail, ThumbnailSize};
use std::fs::File;
use std::io::{Write,BufReader, BufWriter};
use std::io::Cursor;

fn main() {
    let input_file = File::open("tests/assets/test.png").unwrap();
    let reader = BufReader::new(input_file);
    let mut  thumbnails = create_thumbnails(reader, mime::IMAGE_PNG, [ThumbnailSize::Small, ThumbnailSize::Medium]).unwrap();

    let thumbnail = thumbnails.pop().unwrap();
    let mut buf = Cursor::new(Vec::<u8>::new());
    let mut output_file = File::create("tests/assets/test-th-small.png").unwrap();
    thumbnail.write_png(&mut output_file).unwrap();
}
