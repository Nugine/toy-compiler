pub fn number_width(n: usize) -> usize {
    if n < 10 {
        1
    } else {
        ((n as f64).log10() as usize) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_number_width() {
        assert_eq!(number_width(0), 1);
        assert_eq!(number_width(9), 1);
        assert_eq!(number_width(10), 2);
        assert_eq!(number_width(100), 3);
        assert_eq!(number_width(1000), 4);
        assert_eq!(number_width(10000), 5);
    }
}
