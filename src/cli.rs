use std::path::PathBuf;

use clap::*;

#[derive(Parser, Debug)]
#[command(author = "Mihail Kovachev", version, about, long_about = None)]
pub struct Cli {

    #[arg(short = 'u', long = "url", value_name = "Base URL")]
    pub base_url: String,

    #[arg(short = 'o', long = "output-dir", value_name = "Output Directory")]
    pub output_dir: PathBuf
}