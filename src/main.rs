use expr_eval::tokenize;
use expr_eval::AstNode;

fn main() {
    let token_str = "(3 + 4) * 2^5^6";
    let token_list = tokenize(token_str);
    let root = AstNode::parse_expression(token_list.into_iter());
    println!("{}", root.unwrap());

    // let _root = AstNode::parse_expression_wrong(tokenize(token_str).into_iter().peekable(), 0);

    let token_str = "a+(b+c)";
    let token_list = tokenize(token_str);
    let root = AstNode::parse_expression(token_list.into_iter());
    println!("{}", root.unwrap());
}
