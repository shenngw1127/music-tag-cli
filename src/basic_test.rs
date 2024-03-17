#[test]
fn test_int_to_string() {
    let a: u32 = 15;
    assert_eq!(&a.to_string(), "15");
}

#[test]
fn test_split_slash() {
    assert_split("/", '/', vec!("", ""));
    assert_split("1/", '/', vec!("1", ""));
    assert_split("1/-", '/', vec!("1", "-"));
    assert_split("1/10", '/', vec!("1", "10"));
    assert_split("/10", '/', vec!("", "10"));
    assert_split("-/-", '/', vec!("-", "-"));
}

fn assert_split(source: &str, pattern: char, x1: Vec<&str>) {
    let result1: Vec<&str> = source.split(pattern).collect();
    assert_eq!(result1, x1);
}

