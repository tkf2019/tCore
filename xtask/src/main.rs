use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "tCore")]
#[clap(version, about, long_about = None)]
struct Commands {
    #[clap(subcommand)]
    inner: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    Make(BuildArgs),
    Run(RunArgs),
}

#[derive(Args, Default)]
struct BuildArgs {
    #[clap(short, long)]
    name: Option<String>,
}

impl BuildArgs {
    fn make(&self) {
        println!("Hello, world!");
    }
}

#[derive(Args, Default)]
struct RunArgs {
    #[clap(short, long)]
    name: Option<String>,
}

fn main() {
    match Commands::parse().inner {
        Subcommands::Make(args) => {
            println!("Hello, world2!");
            args.make();
        }
        Subcommands::Run(_) => todo!(),
    }
}
