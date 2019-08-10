use std::path::PathBuf;

error_chain! {


    errors {
        BadId
        InvalidRepository(path: PathBuf){
            display("{:?} is not a valid repository", path)
        }
    }
}