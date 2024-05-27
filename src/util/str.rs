use std::string::String;

pub fn get_insert_from_beginning(original: &str,
                                 insert_pos: usize,
                                 addend: &str) -> Option<String> {
    let char_vec: Vec<char> = original.chars().collect();
    if insert_pos <= char_vec.len() {
        let (first_part, second_part) = char_vec.split_at(insert_pos);
        Some(first_part.iter().collect::<String>()
            + addend
            + &second_part.iter().collect::<String>())
    } else {
        None
    }
}

pub fn get_append_from_end(original: &str,
                           append_pos: usize,
                           addend: &str) -> Option<String> {
    let char_vec: Vec<char> = original.chars().collect();
    let len = char_vec.len();
    if append_pos <= len {
        let begin = len - append_pos;
        let (first_part, second_part) = char_vec.split_at(begin);
        Some(String::from_iter(first_part.iter())
            + addend
            + &String::from_iter(second_part.iter()))
    } else {
        None
    }
}

pub fn get_replaced_any(current: &str,
                        from: &str,
                        to: &str,
                        ignore_case: bool) -> Option<String> {
    if !ignore_case {
        Some(current.replace(from, to))
    } else {
        let mut index = 0;
        let len = from.len();
        let mut res = String::new();
        for (i, _) in current.to_lowercase().match_indices(&from.to_lowercase()) {
            res.push_str(&current[index..i]);
            res.push_str(to);
            index = i + len;
        }
        res.push_str(&current[index..]);
        Some(res)
    }
}

pub fn get_replaced_beginning(original: &str,
                              from: &str,
                              to: &str,
                              ignore_case: bool) -> Option<String> {
    let found = if ignore_case {
        original.to_lowercase().starts_with(&from.to_lowercase())
    } else {
        original.starts_with(from)
    };

    if found {
        let (_, last_part) = original.split_at(from.len());
        Some(to.to_owned() + last_part)
    } else {
        None
    }
}

pub fn get_replaced_end(original: &str,
                        from: &str,
                        to: &str,
                        ignore_case: bool) -> Option<String> {
    let found = if ignore_case {
        original.to_lowercase().ends_with(&from.to_lowercase())
    } else {
        original.ends_with(from)
    };

    if found {
        let (first_part, _) = original.split_at(original.len() - from.len());
        Some(first_part.to_owned() + to)
    } else {
        None
    }
}

pub fn get_replaced_last(original: &str,
                         from: &str,
                         to: &str,
                         ignore_case: bool) -> Option<String> {
    let pos = if ignore_case {
        original.to_lowercase().rfind(&from.to_lowercase())
    } else {
        original.rfind(from)
    };
    get_replaced_one(original, &pos, from.len(), to)
}

pub fn get_replaced_first(original: &str,
                          from: &str,
                          to: &str,
                          ignore_case: bool) -> Option<String> {
    let pos = if ignore_case {
        original.to_lowercase().find(&from.to_lowercase())
    } else {
        original.find(from)
    };
    get_replaced_one(original, &pos, from.len(), to)
}

fn get_replaced_one(original: &str,
                    found_pos: &Option<usize>,
                    from_len: usize,
                    to: &str) -> Option<String> {
    if found_pos.is_some() {
        let (first_part, _) = original.split_at(found_pos.unwrap());
        let (_, last_part) = original.split_at(found_pos.unwrap() + from_len);
        Some(first_part.to_owned() + to + last_part)
    } else {
        None
    }
}

pub fn get_remove_from_beginning(original: &str,
                                 begin: usize,
                                 end: &Option<usize>) -> Option<String> {
    let char_vec: Vec<char> = original.chars().collect();
    let len = char_vec.len();
    if begin <= len {
        if let Some(end) = end {
            if *end < len {
                if begin < *end {
                    let (first_part, ..) = char_vec.split_at(begin);
                    let (.., second_part) = char_vec.split_at(*end);
                    Some(first_part.iter().collect::<String>()
                        + &second_part.iter().collect::<String>())
                } else if begin == *end {
                    Some(original.to_owned())
                } else {
                    None
                }
            } else {
                let (first_part, ..) = char_vec.split_at(begin);
                Some(first_part.iter().collect::<String>())
            }
        } else {
            let (first_part, ..) = char_vec.split_at(begin);
            Some(first_part.iter().collect::<String>())
        }
    } else {
        None
    }
}

pub fn get_remove_from_end(original: &str,
                           begin: usize,
                           end: &Option<usize>) -> Option<String> {
    let char_vec: Vec<char> = original.chars().collect();
    let len = char_vec.len();
    if begin <= len {
        let end_index = len - begin;
        if let Some(end) = end {
            if *end < len {
                let begin_index = len - *end;
                if begin_index < end_index {
                    let (first_part, ..) = char_vec.split_at(begin_index);
                    let (.., second_part) = char_vec.split_at(end_index);
                    Some(first_part.iter().collect::<String>()
                        + &second_part.iter().collect::<String>())
                } else if begin_index == end_index {
                    Some(original.to_owned())
                } else {
                    None
                }
            } else {
                let (.., second_part) = char_vec.split_at(end_index);
                Some(second_part.iter().collect::<String>())
            }
        } else {
            let (.., second_part) = char_vec.split_at(end_index);
            Some(second_part.iter().collect::<String>())
        }
    } else {
        None
    }
}

pub fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

pub fn rtruncate(s: &str, max_chars: usize) -> &str {
    let len = s.chars().collect::<Vec<_>>().len();
    if max_chars >= len {
        s
    } else {
        let removed_chars = len - max_chars;
        match s.char_indices().nth(removed_chars) {
            None => "",
            Some((idx, _)) => {
                &s[idx..]
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{get_append_from_end, get_insert_from_beginning, get_replaced_any};
    use super::{get_remove_from_beginning, get_remove_from_end};
    use super::{get_replaced_beginning, get_replaced_end, get_replaced_first, get_replaced_last};
    use super::{rtruncate, truncate};

    #[test]
    fn test_get_insert_from_beginning() {
        let words = "The dream my father told me";

        let res = get_insert_from_beginning(words, 0, "000 ");
        assert_eq!(res.unwrap(), "000 The dream my father told me");

        let res = get_insert_from_beginning(words, 4, "111 ");
        assert_eq!(res.unwrap(), "The 111 dream my father told me");

        let res = get_insert_from_beginning(words, words.len() - 2, "333 ");
        assert_eq!(res.unwrap(), "The dream my father told 333 me");

        let res = get_insert_from_beginning(words, words.len(), ". 789");
        assert_eq!(res.unwrap(), "The dream my father told me. 789");

        let res = get_insert_from_beginning(words, words.len() + 1, "any");
        assert_eq!(res, None);

        let w_words = "鹿港小鎮03";
        let res = get_insert_from_beginning(w_words, 1, "_abc_");
        assert_eq!(res.unwrap(), "鹿_abc_港小鎮03");
    }

    #[test]
    fn test_get_append_from_end() {
        let words = "The dream my father told me";

        let res = get_append_from_end(words, 0, ". 789");
        assert_eq!(res.unwrap(), "The dream my father told me. 789");

        let res = get_append_from_end(words, 2, "333 ");
        assert_eq!(res.unwrap(), "The dream my father told 333 me");

        let res = get_append_from_end(words, words.len() - 4, "111 ");
        assert_eq!(res.unwrap(), "The 111 dream my father told me");

        let res = get_append_from_end(words, words.len(), "000 ");
        assert_eq!(res.unwrap(), "000 The dream my father told me");

        let res = get_append_from_end(words, words.len() + 1, "any");
        assert_eq!(res, None);

        let w_words = "鹿港小鎮03";
        let res = get_append_from_end(w_words, 4, "_abc_");
        assert_eq!(res.unwrap(), "鹿港_abc_小鎮03");
    }

    #[test]
    fn test_get_replaced_any() {
        let words = "Someone is better than Rust but Rust is better than C++";
        let res = get_replaced_any(words, "BET", "Af", true);
        assert_eq!(res.unwrap(), "Someone is After than Rust but Rust is After than C++");

        let words = "Chen洁冰; 王先生; 张先生; Chen先生; 李先生";
        let res = get_replaced_any(words, "chen", "C", true);
        assert_eq!(res.unwrap(), "C洁冰; 王先生; 张先生; C先生; 李先生");
    }

    #[test]
    fn test_get_replaced_beginning() {
        let words = "Someone is better than Rust but Rust is better than C++";

        let res = get_replaced_beginning(words, "Some", "Any", false);
        assert_eq!(res.unwrap(), "Anyone is better than Rust but Rust is better than C++");

        let res = get_replaced_beginning(words, "Something", "Anything", false);
        assert_eq!(res, None);

        let res = get_replaced_beginning(words, "is", "other", false);
        assert_eq!(res, None);
    }

    #[test]
    fn test_get_replaced_end() {
        let words = "Someone is better than Rust but Rust is better than C++";

        let res = get_replaced_end(words, "than C++", "THAN Java", false);
        assert_eq!(res.unwrap(), "Someone is better than Rust but Rust is better THAN Java");

        let res = get_replaced_end(words, "N C++", "Anything", false);
        assert_eq!(res, None);

        let res = get_replaced_end(words, "is", "other", false);
        assert_eq!(res, None);
    }

    #[test]
    fn test_get_replaced_first() {
        let words = "Someone is better than Rust but Rust is better than C++";

        let res = get_replaced_first(words, "Rust", "Java", false);
        assert_eq!(res.unwrap(), "Someone is better than Java but Rust is better than C++");

        let res = get_replaced_first(words, " Rust", " _Java", false);
        assert_eq!(res.unwrap(), "Someone is better than _Java but Rust is better than C++");

        let res = get_replaced_first(words, "then", "Anything", false);
        assert_eq!(res, None);

        let res = get_replaced_first(words, "  Rust", "other", false);
        assert_eq!(res, None);
    }

    #[test]
    fn test_get_replaced_last() {
        let words = "Someone is better than Rust but Rust is better than C++";

        let res = get_replaced_last(words, "better", "_BEST_", false);
        assert_eq!(res.unwrap(), "Someone is better than Rust but Rust is _BEST_ than C++");

        let res = get_replaced_last(words, " better", "  OK ", false);
        assert_eq!(res.unwrap(), "Someone is better than Rust but Rust is  OK  than C++");

        let res = get_replaced_last(words, "then", "Anything", false);
        assert_eq!(res, None);

        let res = get_replaced_last(words, "  Rust", "other", false);
        assert_eq!(res, None);
    }

    #[test]
    fn test_get_remove_from_beginning() {
        assert_eq!(get_remove_from_beginning("倚天屠龙记", 0, &Some(2)).unwrap(),
                   "屠龙记");
        assert_eq!(get_remove_from_beginning("倚天屠龙记", 0, &None).unwrap(),
                   "");
        assert_eq!(get_remove_from_beginning("倚天屠龙记", 2, &None).unwrap(),
                   "倚天");
        assert_eq!(get_remove_from_beginning("倚天屠龙记", 2, &Some(10)).unwrap(),
                   "倚天");
        assert_eq!(get_remove_from_beginning("倚天屠龙记", 2, &Some(4)).unwrap(),
                   "倚天记");

        assert_eq!(get_remove_from_beginning("倚天屠龙记", 2, &Some(2)).unwrap(),
                   "倚天屠龙记");

        assert_eq!(get_remove_from_beginning("倚天屠龙记", 2, &Some(1)), None);
    }

    #[test]
    fn test_get_remove_from_end() {
        assert_eq!(get_remove_from_end("倚天屠龙记", 0, &Some(2)).unwrap(),
                   "倚天屠");
        assert_eq!(get_remove_from_end("倚天屠龙记", 0, &None).unwrap(),
                   "");
        assert_eq!(get_remove_from_end("倚天屠龙记", 2, &None).unwrap(),
                   "龙记");
        assert_eq!(get_remove_from_end("倚天屠龙记", 2, &Some(10)).unwrap(),
                   "龙记");
        assert_eq!(get_remove_from_end("倚天屠龙记", 2, &Some(4)).unwrap(),
                   "倚龙记");

        assert_eq!(get_remove_from_end("倚天屠龙记", 2, &Some(2)).unwrap(),
                   "倚天屠龙记");

        assert_eq!(get_remove_from_end("倚天屠龙记", 2, &Some(1)), None);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("倚天屠龙记", 0), "");
        assert_eq!(truncate("倚天屠龙记", 4), "倚天屠龙");
        assert_eq!(truncate("倚天屠龙记", 100), "倚天屠龙记");

        assert_eq!(truncate("hello", 4), "hell");
        assert_eq!(truncate("hello", 100), "hello");
    }

    #[test]
    fn test_rtruncate() {
        assert_eq!(rtruncate("倚天屠龙记", 0), "");
        assert_eq!(rtruncate("倚天屠龙记", 4), "天屠龙记");
        assert_eq!(rtruncate("倚天屠龙记", 100), "倚天屠龙记");

        assert_eq!(rtruncate("hello", 4), "ello");
        assert_eq!(rtruncate("hello", 100), "hello");

        assert_eq!(rtruncate("Hi倚天屠龙mao记", 7), "天屠龙mao记");
    }
}
