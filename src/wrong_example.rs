/// 来时路，记录错误写法
use super::ast::AstNode;

/// 问题： 多层嵌套的迭代器适配器组合，导致编译器推导出无限递归的类型结构
/// 具体原因：在递归函数中不断包装迭代器，形成了这样的类型推导链：
///       Peekable<A>
///         → Peekable<Peekable<A>>
///           → Peekable<Peekable<Peekable<A>>>
///             → ...
///  这导致编译器需要推导无限嵌套的类型结构
/// 解决方法：保持迭代器传递一致性，始终使用同一层迭代器
///
/// 说明：以下代码不是完整的功能代码，只是选取了部分，为了演示问题
///
/// cargo build 报错如下：
///    error[E0275]: overflow evaluating the requirement `&mut Peekable<std::vec::IntoIter<String>>: Sized`
///    |
///    = help: consider increasing the recursion limit by adding a `#![recursion_limit = "256"]` attribute to your crate (`expr_eval`)
///    = note: required for `Peekable<&mut Peekable<std::vec::IntoIter<String>>>` to implement `Iterator`
///    = note: 128 redundant requirements hidden
///    = note: required for `Peekable<&mut Peekable<&mut Peekable<&mut Peekable<&mut Peekable<...>>>>>` to implement `Iterator`
///    = note: the full name for the type has been written to '/Users/u/forest/rustspace/toy-projects/expr-eval/target/debug/deps/expr_eval-f0f7e14b27788895.long-type-11311293446330987924.txt'
///    = note: consider using `--verbose` to print the full type name to the console
/// 错误信息虽然不是很明显，但是只要注意观察，就可以找到关键线索：`Peekable<&mut Peekable<&mut Peekable<&mut Peekable<&mut Peekable<...>>>>>`
/// 进而，打开expr_eval-f0f7e14b27788895.long-type-11311293446330987924.txt这个文件，就可以更加直观的看到无数层嵌套的 Peekbale 迭代器结构
///
impl AstNode {
    pub fn parse_expression_wrong<T: Iterator<Item = String>>(
        mut token_iter: T,
        min_precedence: u32,
    ) -> Result<Self, &'static str> {
        let _node = Self::parse_atom_wrong(&mut token_iter)?;
        let mut token_iter = token_iter.peekable();
        if token_iter.peek().is_some() {
            token_iter.next();
            let _right_node = Self::parse_expression_wrong(&mut token_iter, min_precedence)?;
        }
        Err("")
    }

    fn parse_atom_wrong<T: Iterator<Item = String>>(token_iter: T) -> Result<Self, &'static str> {
        let mut token_iter = token_iter.peekable();
        match token_iter.peek() {
            Some(_) => {
                token_iter.next();
                let node = Self::parse_expression_wrong(&mut token_iter, 0);
                node
            }
            _ => Err(""),
        }
    }
}
