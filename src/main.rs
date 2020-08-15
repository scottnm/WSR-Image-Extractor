extern crate bytesize;
use bytesize::ByteSize;
use std::io::BufRead;

fn get_path_from_args() -> Result<String, &'static str> {
    // the 0th arg is always the program name so skip it
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        return Err("missing path argument");
    }

    // the first argument should be the path
    Ok(args.remove(0))
}

struct Base64ImageFile {
    name: String,
    data: Vec<u8>
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
            },

            ScanPhase::CheckBase64 => {
                if line == "Content-Transfer-Encoding: base64" {
                    current_scan_state.scan_phase = ScanPhase::LookForImageName;
                } else {
                    println!("Skipping non-base64 image entry! {}", line);
                    current_scan_state = ScanState::new();
                }
            },

            ScanPhase::LookForImageName => {
                const IMAGE_NAME_PREFIX: &'static str = "Content-Location: ";
                if line.starts_with(IMAGE_NAME_PREFIX) {
                    let image_name = &line[IMAGE_NAME_PREFIX.len()..];
                    println!("image: {}", image_name);
                    current_scan_state.name = image_name.to_string();
                    current_scan_state.scan_phase = ScanPhase::FindImageDataStart;
                } else {
                    println!("Missing image name! {}", line);
                    current_scan_state = ScanState::new();
                }
            },

            ScanPhase::FindImageDataStart => {
                if line.is_empty() {
                    current_scan_state.scan_phase = ScanPhase::ReadJpeg;
                } else {
                    println!("Expected empty line! {}", line);
                    current_scan_state = ScanState::new();
                }
            },

            ScanPhase::ReadJpeg => {
                if line.is_empty() { // we finished reading the jpeg data lines
                    // consume the tmp scan state and reset it
                    let completed_scan_state = std::mem::replace(&mut current_scan_state, ScanState::new());
                    jpegs.push(Base64ImageFile { name: completed_scan_state.name, data: completed_scan_state.data });
                } else {
                    // we have found our first jpeg data line. Print a preview for debug purposes.
                    if current_scan_state.data.is_empty() {
                        println!("    image data preview: {}", line);
                    }
                    current_scan_state.data.extend(line.as_bytes());
                }
            },
        };
    }

    jpegs
}

fn main() {
    let get_path_result: Result<String, &'static str> = get_path_from_args();
    if let Err(err_msg) = get_path_result {
        println!("Invalid arguments: {}", err_msg);
        println!("Usage: wsr_image <path to file>");
        std::process::exit(1);
    }

    let file_path = get_path_result.unwrap();
    println!("Got path! {}", file_path);

    let base64_encoded_images: Vec<Base64ImageFile> = extract_base64_encoded_jpegs(&file_path);

    // Print out the sizes of each image for debug purposes
    println!("Collected images:");
    for image in base64_encoded_images {
        println!("    {}\n    size: {}\n", image.name, ByteSize(image.data.len() as u64));
    }
}
