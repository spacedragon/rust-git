use std::path::PathBuf;

error_chain! {

    foreign_links {
        Io(::std::io::Error);
        Encoding(::std::str::Utf8Error);
        BadMode(::std::num::ParseIntError);
    }

    errors {
        BadId
        InvalidRepository(path: PathBuf){
            display("{:?} is not a valid repository", path)
        }
        ParseError
    }
}