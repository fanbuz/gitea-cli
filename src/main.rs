fn main() {
    let code = gitea_cli::app::run(std::env::args_os());
    std::process::exit(code);
}
