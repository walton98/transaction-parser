use clap::Parser;

#[derive(Parser)]
struct Args {
    path: String,
}

fn main() -> Result<(), csv::Error> {
    let args = Args::parse();
    transaction_manager::run(&args.path, std::io::stdout())
}
