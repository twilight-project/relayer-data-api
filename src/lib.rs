mod archiver;
pub mod database;
pub mod kafka;

pub use archiver::DatabaseArchiver;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
