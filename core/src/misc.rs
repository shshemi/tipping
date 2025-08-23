use itertools::Itertools;
use fancy_regex::Regex;


pub fn compile_into_regex<Item, Iter>(regex_str: Iter) -> Regex
where
 Item: AsRef<str>,
 Iter: IntoIterator<Item = Item>
{
    Regex::new(
        regex_str
            .into_iter()
            .map(|s| format!(r"(?:{})", s.as_ref()))
            .join("|")
            .as_str(),
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let r = compile_into_regex([r"\d+", r"[a-zA-Z]+"]);
        assert!(r.is_match("123").unwrap());
        assert!(r.is_match("abc").unwrap());
        assert!(r.is_match("ABC").unwrap());

        assert!(!r.is_match("@").unwrap());
        assert!(!r.is_match("#").unwrap());
    }
}
