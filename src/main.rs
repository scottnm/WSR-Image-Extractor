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
    _name: String,
    _data: Vec<u8>
}

fn extract_base64_encoded_jpegs(file_path: &str) -> Vec<Base64ImageFile> {
    #[derive(Debug, Copy, Clone)]
    enum ScanState {
        LookForJpeg,
        CheckBase64,
        LookForImageName,
        FindImageDataStart,
        ReadJpeg,
    }

    let mut scan_state = ScanState::LookForJpeg;
    let file = std::fs::File::open(file_path).unwrap();

    for line_it in std::io::BufReader::new(file).lines() {
        let line = line_it.unwrap();
        match scan_state {
            ScanState::LookForJpeg => {
                if line == "Content-Type: image/jpeg" {
                    scan_state = ScanState::CheckBase64;
                }
            },
            ScanState::CheckBase64 => {
                if line == "Content-Transfer-Encoding: base64" {
                    scan_state = ScanState::LookForImageName;
                } else {
                    scan_state = ScanState::LookForJpeg;
                    println!("Skipping non-base64 image entry! {}", line);
                }
            }
            ScanState::LookForImageName => {
                const IMAGE_NAME_PREFIX: &'static str = "Content-Location: ";
                if line.starts_with(IMAGE_NAME_PREFIX) {
                    let image_name = &line[IMAGE_NAME_PREFIX.len()..];
                    println!("image: {}", image_name);
                    scan_state = ScanState::FindImageDataStart;
                } else {
                    println!("Missing image name! {}", line);
                    scan_state = ScanState::LookForJpeg;
                }
            },
            ScanState::FindImageDataStart => {
                if line.is_empty() {
                    scan_state = ScanState::ReadJpeg;
                } else {
                    println!("Expected empty line! {}", line);
                    scan_state = ScanState::LookForJpeg;
                }
            },
            ScanState::ReadJpeg => {
                println!("    image data preview: {}", line);
                scan_state = ScanState::LookForJpeg;
            },
        };
    }

    Vec::new()
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

    let _base64_encoded_images: Vec<Base64ImageFile> = extract_base64_encoded_jpegs(&file_path);
}
