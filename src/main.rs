fn get_path_from_args() -> Result<String, &'static str> {
    // the 0th arg is always the program name so skip it
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        return Err("missing path argument");
    }

    // the first argument should be the path
    Ok(args.remove(0))
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
}
