
use structopt::StructOpt;
use structopt::clap::App;
use crate::errors::*;

#[derive(Debug, StructOpt)]
pub struct CatFileOpt {
        #[structopt(short = "p" )]
        ///Pretty-print the contents of <object> based on its type.
        pretty_print: bool,    

        #[structopt(short = "t")]
        ///Instead of the content, show the object type identified by <object>.
        show_type: bool,

        #[structopt(short = "e")]
        /// Exit with zero status if <object> exists and is a valid object. If <object> is
        /// of an invalid format exit with non-zero and emits an error on stderr.
        check_error: bool,

        #[structopt(short = "s")]
        ///Instead of the content, show the object size identified by <object>.
        show_size: bool,

        #[structopt()]
        object: String,
}


pub fn cat_file(clap: &mut App, opt: CatFileOpt) -> Result<()> {
        println!("{:?}", opt);
        clap.print_help().expect("");
        Ok(())
}