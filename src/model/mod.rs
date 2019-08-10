

pub mod repository;
pub mod object;
pub mod id;





#[cfg(test)]
mod tests {
    use crate::fs::*;

    #[test]
    fn test_lookup_prefix() {
        let fs = OsFs {};
        assert_eq!(fs.ls_files("./Cargo").take(2).count(), 2);
    }
}