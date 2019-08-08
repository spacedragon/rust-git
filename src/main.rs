extern crate structopt;

use structopt::StructOpt;

 
#[derive(Debug, StructOpt)]
#[structopt(name = "git", about = "the rust git command")]
enum Opt {
    #[structopt(name = "cat-file")]
    CatFile {
            #[structopt(short = "p")]
            ///Pretty-print the contents of <object> based on its type.
            pretty_print: bool,    

            #[structopt(short = "t")]
            ///Instead of the content, show the object type identified by <object>.
            show_type: Option<String>,

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
}


fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
