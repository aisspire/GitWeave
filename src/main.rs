fn main() {
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    if let Err(error) = gitweave::run(&mut stdout, &mut stderr) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
