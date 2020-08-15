extern crate base64;
extern crate bytesize;

use bytesize::ByteSize;
use std::io::{BufRead, Write};

#[derive(Debug)]
struct RunOptions {
    file_path: String,
    output_directory: String,
    is_preview: bool,
}

fn get_run_options_from_args() -> Result<RunOptions, &'static str> {
    // the 0th arg is always the program name so skip it
    // only need the first 3 args but the last one is optional
    let mut args: Vec<String> = std::env::args().skip(1).take(3).collect();

    match args.len() {
        0 => return Err("missing input path argument"),
        1 => return Err("missing output directory path argument"),
        _ => (),
    };

    let is_preview: bool = if args.len() > 2 {
        args.remove(2) == "--preview"
    } else {
        false
    };

    if args.len() <= 1 {
        return Err("missing output directory path argument");
    }
    let output_directory = args.remove(1);

    if args.is_empty() {
        return Err("missing input path argument");
    }
    let file_path = args.remove(0);

    assert!(args.is_empty()); // should have consumed all arguments

    // the first argument should be the path
    Ok(RunOptions {
        file_path,
        output_directory,
        is_preview,
    })
}

// TODO: instead of copying all of the bytes to different in-memory buffers,
// copy to one large in-memory buffer and just collect each file as a list of
// pointers into the buffer + names
struct Base64ImageFile {
    name: String,
    data: Vec<u8>,
}

fn extract_base64_encoded_jpegs(file_path: &str) -> Vec<Base64ImageFile> {
    #[derive(Debug, Copy, Clone)]
    enum ScanPhase {
        LookForJpeg,
        CheckBase64,
        LookForImageName,
        FindImageDataStart,
        ReadJpeg,
    }

    #[derive(Debug)]
    struct ScanState {
        name: String,
        data: Vec<u8>,
        scan_phase: ScanPhase,
    }

    impl ScanState {
        fn new() -> ScanState {
            ScanState {
                name: String::new(),
                data: Vec::<u8>::new(),
                scan_phase: ScanPhase::LookForJpeg,
            }
        }
    }

    let mut jpegs = Vec::<Base64ImageFile>::new();
    let mut current_scan_state = ScanState::new();

    let file = std::fs::File::open(file_path).unwrap();
    for line_it in std::io::BufReader::new(file).lines() {
        let line = line_it.unwrap();

        match current_scan_state.scan_phase {
            ScanPhase::LookForJpeg => {
                if line == "Content-Type: image/jpeg" {
                    current_scan_state.scan_phase = ScanPhase::CheckBase64;
                }
            }

            ScanPhase::CheckBase64 => {
                if line == "Content-Transfer-Encoding: base64" {
                    current_scan_state.scan_phase = ScanPhase::LookForImageName;
                } else {
                    println!("Skipping non-base64 image entry! {}", line);
                    current_scan_state = ScanState::new();
                }
            }

            ScanPhase::LookForImageName => {
                const IMAGE_NAME_PREFIX: &'static str = "Content-Location: ";
                if line.starts_with(IMAGE_NAME_PREFIX) {
                    let image_name = &line[IMAGE_NAME_PREFIX.len()..];
                    current_scan_state.name = image_name.to_string();
                    current_scan_state.scan_phase = ScanPhase::FindImageDataStart;
                } else {
                    println!("Missing image name! {}", line);
                    current_scan_state = ScanState::new();
                }
            }

            ScanPhase::FindImageDataStart => {
                if line.is_empty() {
                    current_scan_state.scan_phase = ScanPhase::ReadJpeg;
                } else {
                    println!("Expected empty line! {}", line);
                    current_scan_state = ScanState::new();
                }
            }

            ScanPhase::ReadJpeg => {
                if line.is_empty() {
                    // we finished reading the jpeg data lines
                    // consume the tmp scan state and reset it
                    let completed_scan_state =
                        std::mem::replace(&mut current_scan_state, ScanState::new());
                    jpegs.push(Base64ImageFile {
                        name: completed_scan_state.name,
                        data: completed_scan_state.data,
                    });
                } else {
                    current_scan_state.data.extend(line.as_bytes());
                }
            }
        };
    }

    jpegs
}

fn decode_and_write_base64_file(file_path: &std::path::Path, data: &[u8]) {
    let b64_decoded_bytes = match base64::decode(data) {
        Err(why) => panic!("couldn't decode bytes, {:?}! {}", data, why),
        Ok(decoded_bytes) => decoded_bytes,
    };

    let display = file_path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match std::fs::File::create(&file_path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    // Write the `LOREM_IPSUM` string to `file`, returns `io::Result<()>`
    match file.write_all(&b64_decoded_bytes) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

fn main() {
    let run_options = match get_run_options_from_args() {
        Err(usage_err) => {
            println!("Invalid arguments: {}", usage_err);
            println!("Usage: wsr_image <path to file> <output directory> [--preview]");
            return;
        }
        Ok(options) => options,
    };

    println!("Run options: {:?}", run_options);

    let base64_encoded_images: Vec<Base64ImageFile> =
        extract_base64_encoded_jpegs(&run_options.file_path);

    // Print out the sizes of each image for debug purposes
    let output_directory_path = std::path::Path::new(&run_options.output_directory);
    for image in base64_encoded_images {
        let full_image_path_buffer = output_directory_path.join(image.name);
        let full_image_path = full_image_path_buffer.as_path();
        if run_options.is_preview {
            println!(
                "    would write {}\n    size: {}\n",
                full_image_path.display(),
                ByteSize(image.data.len() as u64)
            );
        } else {
            decode_and_write_base64_file(&full_image_path, &image.data);
        }
    }
}
