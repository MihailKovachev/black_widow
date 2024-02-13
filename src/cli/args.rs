use std::path::PathBuf;

use clap::*;

#[derive(Parser, Debug)]
#[command(author = "Mihail Kovachev", version, about, long_about = None)]
pub struct Args {

    #[arg(short = 't', long = "targets", value_name = "Targets File", help = "The target hosts")]
    pub targets: PathBuf,

    #[arg(short = 's', long = "crawl-subdomains", default_value_t = false, help = "Whether to also crawl subdomains of the targets as they are found.")]
    pub crawl_subdomains: bool,

    // #[arg(short = 'o', long = "output-dir", value_name = "Output Directory")]
    // pub output_dir: PathBuf

}