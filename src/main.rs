use clap::Parser;

/// A simple CLI application
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let message = xcom_rs::greet(args.name.as_deref());
    println!("{}", message);

    Ok(())
}
