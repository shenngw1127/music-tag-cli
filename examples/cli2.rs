use std::path::PathBuf;
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_enum)]
    profile: Profile,

    #[arg(short, long)]
    #[arg(help = "Force to output if the output file exists")]
    force: bool,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Assign the path of your SOURCE file. It must point to a file path.")]
    input_path: Option<PathBuf>,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Assign the path of your DESTINATION file. It must point to a file path.")]
    output_path: Option<PathBuf>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Profile {
    T2s,
    S2t,
    Tw2s,
    HK2s,
}

fn main() {
    let args = Args::parse();

    println!("force is {}.", args.force);
    if args.input_path.is_some() {
        println!("input_path: {}", args.input_path.unwrap().display());
    }
    if !args.output_path.is_none() {
        println!("input_path: {}", args.output_path.unwrap().display());
    }
}
