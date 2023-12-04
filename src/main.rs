use watermarker::Watermarker;

fn main() {
    env_logger::init();
    match eframe::run_native(
        "Watermarker",
        Default::default(),
        Box::new(Watermarker::new),
    ) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
