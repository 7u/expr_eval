mod ast;
// pub mod wrong_example;

use std::collections::HashSet;

pub use ast::AstNode;

pub const SEPARATOR_LIST: &[char] = &['+', '-', '*', '/', '^', '(', ')'];

pub fn tokenize(s: &str) -> Vec<String> {
    split_keep_separator(s, SEPARATOR_LIST)
}

fn split_keep_separator(s: &str, separator_list: &[char]) -> Vec<String> {
    let sep_set: HashSet<_> = separator_list.iter().collect();
    let mut index = 0;
    let mut result: Vec<String> = vec![];
    for (i, c) in s.chars().enumerate() {
        if sep_set.contains(&c) {
            if index != i {
                let token = s[index..i].trim();
                if !token.is_empty() {
                    result.push(token.to_string());
                }
            }
            result.push(c.to_string());
            index = i + 1;
        }
    }

    if index < s.len() {
        let token = s[index..].trim();
        if !token.is_empty() {
            result.push(token.to_string());
        }
    }
    result
}
