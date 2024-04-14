pub fn decimal_to_padding_string(decimal: u32, padding: usize) -> String {
    format!("{:0width$}", decimal, width = padding)
}

#[cfg(test)]
mod test {
    use super::decimal_to_padding_string;

    use std::iter::successors;

    fn get_binary_len(input: u32) -> usize {
        get_number_len(input, 2)
    }

    fn get_digit_len(input: u32) -> usize {
        get_number_len(input, 10)
    }

    fn get_number_len(input: u32, base: u32) -> usize {
        successors(Some(input),
                   |&n: &u32| { (n >= base).then(|| n / base) })
            .count()
    }

    #[test]
    fn test_decimal_to_padding_string() {
        assert_eq!(decimal_to_padding_string(5, 3), "005");
        assert_eq!(decimal_to_padding_string(1234, 3), "1234");
        assert_eq!(decimal_to_padding_string(1234, 12), "000000001234");
    }

    #[test]
    fn test_get_digit_len() {
        assert_eq!(1, get_digit_len(0));
        assert_eq!(1, get_digit_len(1));
        assert_eq!(3, get_digit_len(100));
        assert_eq!(3, get_digit_len(999));
        assert_eq!(4, get_digit_len(1000));
    }

    #[test]
    fn test_get_binary_len() {
        assert_eq!(1, get_binary_len(0));
        assert_eq!(1, get_binary_len(1));
        assert_eq!(2, get_binary_len(2));
        assert_eq!(2, get_binary_len(3));

        assert_eq!(3, get_binary_len(7));
        assert_eq!(4, get_binary_len(8));
    }
}
