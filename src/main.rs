extern crate structopt;


use structopt::StructOpt;

use rust_git::cmd::catfile::*;
 
#[derive(Debug, StructOpt)]
#[structopt(name = "git", about = "the rust git command")]
enum Opt {
    #[structopt(name = "cat-file")]
    CatFile(CatFileOpt)
}


fn main() {
    let mut clap = Opt::clap();
    let opt = Opt::from_args();
    
    match opt {
        Opt::CatFile(opt) => cat_file(&mut clap, opt),
    }
}
