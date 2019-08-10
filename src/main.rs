extern crate structopt;

use structopt::StructOpt;

use rust_git::cmd::catfile::*;
use rust_git::errors::*;
 
#[derive(Debug, StructOpt)]
#[structopt(name = "git", about = "the rust git command")]
enum Opt {
    #[structopt(name = "cat-file")]
    CatFile(CatFileOpt)
}

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        use error_chain::ChainedError; // trait which holds `display_chain`
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "{}", e.display_chain()).expect(errmsg);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut clap = Opt::clap();
    let opt = Opt::from_args();
    
    match opt {
        Opt::CatFile(opt) => cat_file(&mut clap, opt),
    }
}
