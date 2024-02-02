use std::path::PathBuf;

use clap::*;

#[derive(Parser, Debug)]
#[command(author = "Mihail Kovachev", version, about, long_about = None)]
pub struct Cli {

    #[arg(short = 't', long = "targets", value_name = "Targets File")]
    pub targets: PathBuf,

    //#[arg(short = 'o', long = "output-dir", value_name = "Output Directory")]
    //pub output_dir: PathBuf
}