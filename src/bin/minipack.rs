fn main() {
    if let Err(err) = minipacked::run_pack(&std::env::args().collect::<Vec<_>>()) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
