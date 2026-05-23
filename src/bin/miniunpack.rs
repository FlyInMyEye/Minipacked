fn main() {
    if let Err(err) = minipacked::run_unpack(&std::env::args().collect::<Vec<_>>()) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
