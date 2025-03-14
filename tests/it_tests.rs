use expr_eval::tokenize;
use expr_eval::AstNode;

#[test]
fn it_parse_expr() {
    let token_list: Vec<String> = vec!["2", "+", "3", "*", "4", "*", "5", "-", "6"]
        .iter()
        .map(|x| x.to_string())
        .collect();
    let root = AstNode::parse_expression(token_list.into_iter());
    assert_eq!("((2 + ((3 * 4) * 5)) - 6)", root.unwrap().to_string());

    let token_list: Vec<String> = vec!["2", "+", "3", "^", "2", "*", "3", "+", "4"]
        .iter()
        .map(|x| x.to_string())
        .collect();
    let root = AstNode::parse_expression(token_list.into_iter());
    assert_eq!("((2 + ((3 ^ 2) * 3)) + 4)", root.unwrap().to_string());
}

#[test]
fn it_parse_expr_with_brackets() {
    let token_list: Vec<String> = vec!["2", "*", "(", "3", "+", "5", ")", "*", "7"]
        .iter()
        .map(|x| x.to_string())
        .collect();
    let root = AstNode::parse_expression(token_list.into_iter());
    assert_eq!("((2 * (3 + 5)) * 7)", root.unwrap().to_string());
}

#[test]
fn it_token() {
    let token_str = "2+3*4*5-6";
    assert_eq!(
        vec!["2", "+", "3", "*", "4", "*", "5", "-", "6"],
        tokenize(token_str)
    );

    let token_str = "2 + 3 ^ 2 * 3 + 4";
    assert_eq!(
        vec!["2", "+", "3", "^", "2", "*", "3", "+", "4"],
        tokenize(token_str)
    );
}

#[test]
fn it_token_with_brackets() {
    let token_str = "2 * (3 + 5) * 7";
    assert_eq!(
        vec!["2", "*", "(", "3", "+", "5", ")", "*", "7"],
        tokenize(token_str)
    );
}
