use std::fmt::Display;
use std::iter::Peekable;

pub enum AstNode {
    Leaf(String),
    Root(Box<AstTree>),
}

impl Display for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Leaf(lf) => write!(f, "{}", lf),
            Self::Root(rt) => write!(f, "({})", rt),
        }
    }
}

impl AstNode {
    pub fn parse_expression<T: Iterator<Item = String>>(
        token_iter: T,
    ) -> Result<Self, &'static str> {
        Self::do_parse_expression(&mut token_iter.peekable(), 0)
    }

    fn do_parse_expression<T: Iterator<Item = String>>(
        token_iter: &mut Peekable<T>,
        min_precedence: u32,
    ) -> Result<Self, &'static str> {
        let mut node = Self::parse_atom(token_iter)?;
        while let Some(token) = token_iter.peek() {
            // println!(
            //     "CURRENT node is:{}, token is: {}, min is {}",
            //     node, token, min_precedence
            // );

            if token == ")" {
                break;
            }

            if let Some(op) = Operator::new(token) {
                if op.precedence < min_precedence {
                    break;
                }

                token_iter.next();
                let next_min = match op.associativity {
                    Associativity::Left => op.precedence + 1,
                    _ => op.precedence,
                };
                let right_node = Self::do_parse_expression(token_iter, next_min)?;
                node = Self::new_root(op, node, right_node);
            } else {
                return Err("Invalid operator");
            }
        }

        Ok(node)
    }

    fn parse_atom<T: Iterator<Item = String>>(
        token_iter: &mut Peekable<T>,
    ) -> Result<Self, &'static str> {
        let parsed = match token_iter.peek() {
            Some(t) if Operator::is_operation(t) => Err("Operand expected, not operator!"),
            Some(t) if t == "(" => {
                // println!("CURRENT atom token is: {}", t);
                token_iter.next();
                let node = Self::do_parse_expression(token_iter, 0);
                if token_iter.next_if(|t| t == ")").is_none() {
                    return Err("Unmatched brackets!");
                }
                node
            }
            Some(_) => {
                // println!("CURRENT atom token is: {}", t);
                Ok(Self::Leaf(token_iter.next().unwrap()))
            }
            _ => Err("Fail parsing atom!"),
        };
        parsed
    }

    fn new_root(op: Operator, left: AstNode, right: AstNode) -> Self {
        Self::Root(Box::new(AstTree {
            root: op,
            left_child: left,
            right_child: right,
        }))
    }

    fn _new_leaf(value: &str) -> Self {
        Self::Leaf(value.to_string())
    }
}

pub struct AstTree {
    root: Operator,
    left_child: AstNode,
    right_child: AstNode,
}

impl Display for AstTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.left_child, self.root, self.right_child)
    }
}

enum Associativity {
    Left,
    Right,
}

struct Operator {
    mark: String,
    precedence: u32,
    associativity: Associativity,
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mark)
    }
}

impl Operator {
    fn new(mark: &str) -> Option<Self> {
        match mark {
            "+" | "-" => Some(Self {
                mark: mark.to_string(),
                precedence: 1,
                associativity: Associativity::Left,
            }),
            "*" | "/" => Some(Self {
                mark: mark.to_string(),
                precedence: 2,
                associativity: Associativity::Left,
            }),
            "^" => Some(Self {
                mark: mark.to_string(),
                precedence: 3,
                associativity: Associativity::Right,
            }),
            _ => None,
        }
    }

    fn is_operation(mark: &str) -> bool {
        Self::new(mark).is_some()
    }
}
