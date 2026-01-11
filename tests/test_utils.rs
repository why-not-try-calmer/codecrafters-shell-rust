#[cfg(test)]
mod test_utils {
    use codecrafters_shell::utils::strip_bytes;

    #[test]
    fn test_clean_bytes_1() {
        let s = b"'Hello World'";
        let expected = b"Hello World";
        let results = strip_bytes(s.to_vec());
        assert_eq!(results, *expected);
    }
    #[test]
    fn test_clean_bytes_2() {
        let s = b"\"Hello World\"";
        let expected = b"Hello World";
        let results = strip_bytes(s.to_vec());
        assert_eq!(results, *expected);
    }
}
