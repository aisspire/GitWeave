fn main() {
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    if gitweave::run(&mut stdout, &mut stderr).is_err() {
        std::process::exit(1);
    }
}
