# Expression Evaluation

## Inspired by

Inspired by roseduan's expr-eval[^1]

摘要一段该项目的 README 作为概览：

> #  表达式解析计算
>
> 根据运算符优先级来进行表达式计算，算法看起来非常简洁优雅，非常巧妙的利用优先级来解决运算的顺序和结合等问题。

随即，我查看了 Readme 链接的公众号文章 [^2]。当然，我只看了介绍部分，没看具体实现过程。文章顺带提到了表达式解析常用的算法，比如逆波兰式，这个词让我瞬间梦回大学时代。那时就知道只需一个栈的数据结构就可以实现逆波兰式，进而做出一个可以加减乘除简单运算的计算器（一次学院组织的编程比赛中就有这道题，小弟不才:)，比赛得了个优胜奖）。而其中的关键信息还是：`expr-eval` 采用了 “基于运算符优先级的算法叫做 `Precedence Climbing`”。



## 看一眼 Precedence Climbing 怎么个事儿

我让 DeepSeek ` 详细介绍一下 Precedence Climbing`。它的回答还算完整，也给出了伪码（Python——行走的伪码）：

```python
def parse_expression(min_precedence):
    node = parse_atom()  # 解析原子项（数字、括号等）
    while True:
        current_op = peek_next_token()  # 查看下一个运算符
        if current_op is None or current_op.precedence < min_precedence:
            break
        consume_token()  # 消耗当前运算符
        # 确定下一优先级阈值
        if current_op.associativity == LEFT:
            next_min = current_op.precedence + 1
        else:
            next_min = current_op.precedence
        right_node = parse_expression(next_min)  # 递归解析右侧
        node = create_ast_node(node, current_op, right_node)
    return node
```



并且提到了 ` 遇括号需递归解析子项 ` 等注意点。



### Reading list

在正式开始对着伪码手撸 Rust 实现之前，我又 Google 了一番，找到了关于这个算法引用最多的两篇：

* 一篇来自 Eli Bendersky 的博文：`Parsing expressions by precedence climbing`[^3]
* 一篇是 York College 的课程讲义：`CS 340: Lecture 6: Precedence Climbing, Abstract Syntax Trees`[^4]

另外两篇 Wiki[^5][^6]：



## 开撸

与 Rust 编译器斗智斗勇

### 确定对象和类型

开撸的第一步是理清算法涉及哪些对象，抽象成数据结构

伪码中主要出现的两个对象是 op 和 node。op，运算符，包含优先级（precedence）和 结合性（associativity）等属性，提供了函数递归的终止条件。而 node，语法树节点，则是算法迭代更新的关键对象；迭代到最后，相当于是构建了一个以运算符为根结点、以左右操作项（亦即原子项，涵盖了操作数和括号内子表达式）为左右子树的 AST。而二叉树的特点正好适用于二元运算的范畴，表达式的计算过程相当于是对树做了一次 middle order DSF。如此看来，op 也是一种 node。

那么，如何把这些表达为 Rust 类型呢？节点分为叶子节点和根节点，把运算符信息包含在根节点中，就有了如下结构：

```rust
struct LeafNode {
    operand: String,
}

struct RootNode {
    precedence: u32,
    associativity: Associativity,
    operator: String,
}

enum Associativity {
    Left,
    Right,
}
```

但根节点中还缺少左右子树的信息，并且所有节点类型都需要归一化、统一成一种类型，才可以在递归函数中迭代。所以：

```rust
enum AstNode {
    // 略去 LeafNode 结构体，以更加简洁的元组结构体代替
    Leaf(String),
    // Box 智能指针指向根节点，以免语法树太繁茂、根系太庞大
    Root(Box<RootNode>),
}

struct RootNode {
    precedence: u32,
    associativity: Associativity,
    operator: String,
    left_child: AstNode,
    right_child: AstNode,
}
```

好，已基本成型。但总觉得哪里有点别扭：虽然操作符（亦即运算符）处于根节点的位置，但在语法树的概念里，节点就是节点，就是 root、left_child、right_child。所以，把操作符信息单独抽象为一个类型、隔离出去，然后再用组合（favor composition over inheritance）的方式放到根节点中。此时，Root 所指也不再是 RootNode，它就是一颗 AstTree ：

```rust
enum AstNode {
    Leaf(String),
    Root(Box<AstTree>),
}  

pub struct AstTree {
    root: Operator,
    left_child: AstNode,
    right_child: AstNode,
}

struct Operator {
    mark: String,
    precedence: u32,
    associativity: Associativity,
}
```

（PS：参考了 `ConList`)



### 实现递归函数

有了以上结构体的定义，后续的实现就变得水到渠成了：对 token 迭代器进行循环遍历，生成节点，递归构造 AST ……

期间会尝到 `while let`、`match` 等语法糖，也能体会 `Peekable Iterator` 的用法。`peek()` 方法不消费而 “窥视” 下一个元素，`peek vs next` 有点类似其他语言中对栈的操作 `top vs take`。



### Token 迭代器的传递：在 Iterator 和 Peekable 之间反复横跳

我本来的做法是把解析表达式的功能代码放入 `lib.rs`，然后本 crate 对外可见 parse_expression 函数。那么函数参数 token 序列的类型，选择普通的 `Iterator`，比如 `vec::into_iter`，会更方便调用者。

但算法在一些边界条件的检查，比如遇到同级的左结合性操作符（+ -），比如左右括号配对，都需要 `peek` 一下 next token。当我把 parse_expression 函数和 parse_atom 函数的参数类型设为 `Iterator` 泛型的同时，也在函数体开头进行一次 `Iterator` 到 `Peekable` 的转换。后果是产生了一个错误的嵌套结构：

```rust
/// 说明：以下代码不是完整的功能代码，只是选取了部分，为了演示问题
pub fn parse_expression<T: Iterator<Item = String>>(
    mut token_iter: T,
    min_precedence: u32,
) -> Result<Self, &'static str> {
    let _node = Self::parse_atom(&mut token_iter)?;
    let mut token_iter = token_iter.peekable();
    if token_iter.peek().is_some() {
        token_iter.next();
        let _right_node = Self::parse_expression(&mut token_iter, min_precedence)?;
    }
    Err("")
}

fn parse_atom<T: Iterator<Item = String>>(
    token_iter: T,
) -> Result<Self, &'static str> {
    let mut token_iter = token_iter.peekable();
    match token_iter.peek() {
        Some(_) => {
            token_iter.next();
            let node = Self::parse_expression(&mut token_iter, 0);
            node
        }
        _ => Err(""),
    }
}
```



#### 编译通不过

多层嵌套的迭代器适配器组合，导致编译器推导出无限递归的类型结构：

```rust
error[E0275]: overflow evaluating the requirement `&mut Peekable<std::vec::IntoIter<String>>: Sized`
|
= help: consider increasing the recursion limit by adding a `#![recursion_limit = "256"]` attribute to your crate (`expr_eval`)
= note: required for `Peekable<&mut Peekable<std::vec::IntoIter<String>>>` to implement `Iterator`
= note: 128 redundant requirements hidden
= note: required for `Peekable<&mut Peekable<&mut Peekable<&mut Peekable<&mut Peekable<...>>>>>` to implement `Iterator`
= note: the full name for the type has been written to '/Users/u/forest/rustspace/toy-projects/expr-eval/target/debug/deps/expr_eval-f0f7e14b27788895.long-type-11311293446330987924.txt'
= note: consider using `--verbose` to print the full type name to the console
```



#### 分析原因

在递归函数中不断包装迭代器，形成了这样的类型推导链：
```
       Peekable<A>
         → Peekable<Peekable<A>>
           → Peekable<Peekable<Peekable<A>>>
             → ...
```
这导致编译器需要推导无限嵌套的类型结构



#### 解决方法

保持迭代器传递一致性，始终使用同一层迭代器：

```rust
fn do_parse_expression<T: Iterator<Item = String>>(
    token_iter: &mut Peekable<T>,
    min_precedence: u32,
) -> Result<Self, &'static str> {
}

fn parse_atom<T: Iterator<Item = String>>(
    token_iter: &mut Peekable<T>,
) -> Result<Self, &'static str> {
}
```



#### 仍要方便调用者

对外公开的函数中，仍然选择 `Iterator` 泛型作为 token 序列参数的类型：

```rust
pub fn parse_expression<T: Iterator<Item = String>>(
    token_iter: T,
) -> Result<Self, &'static str> {
    Self::do_parse_expression(&mut token_iter.peekable(), 0)
}
```



### 怎样才够 Rusty？一些小的细节

#### 不要把事情搞复杂：从懒加载 HashMap 到 match 表达式

每个操作符（Operator）都有对应的唯一的优先级和结合性等特性。在使用这些特性时，思维定式地想要用 HashMap 来存取，于是走了些弯路：

```rust
// precedence only
let operator_list = vec![
    ("+".to_string(), 1),
    ("-".to_string(), 1),
    ("*".to_string(), 2),
    ("/".to_string(), 2),
    ("^".to_string(), 3),
];
let operator_map: HashMap<_,_> = operator_list.into_iter().collect();

// 不想每次取值的时候都创建一次 map
// 用 OneLock 懒加载，只初始化一次
fn operator_map() -> &'static HashMap<String, u32> {
    static OPERATOR_MAP: OnceLock<HashMap<String, u32>> = OnceLock::new();
    OPERATOR_MAP.get_or_init(|| {
        vec![
            ("+".to_string(), 1),
            ("-".to_string(), 1),
            ("*".to_string(), 2),
            ("/".to_string(), 2),
            ("^".to_string(), 3),
        ]
        .into_iter()
        .collect()
    })
}

// 用的时候
let precedence = operator_map().get(op);

// 这样只有 precedence， associativity 怎么办呢：
let operator_list = vec![
    ("+".to_string(), (1, Associativity::Left)),
];
```

这样代码只会越来越丑，完全没必要如此大费周章。稍微运用一点 OOP 的思想，思路就豁然开朗：视 Operator 整体为一个对象，那么在初始化代码中（一般是关联函数 new）用 match 表达式匹配不同的操作符以生成不同类型的 Operator 对象即可以：

```rust
impl Operator {
    fn new(mark: &str) -> Option<Self> {
        match mark {
            "+" | "-" => Some(Self {
                mark: mark.to_string(),
                precedence: 1,
                associativity: Associativity::Left,
            }),
            // ...
        }
    }
```



#### 字符串参数类型用 String 还是 &str

接着 `Operator::new` 函数讲，其参数类型是 `String` 好呢还是 `&str ` ?

先说结论，`&str` 好

DeepSeek 详细分析了两者的优劣、适用场景：

> | 特性              | 使用 `&str` 参数             | 使用 `String` 参数           |
> | :---------------- | :--------------------------- | :--------------------------- |
> | **所有权处理**| 无所有权转移，更灵活         | 需要转移所有权               |
> | **调用灵活性**| 接受 `&str`/`String`/ 字面量 | 只接受 `String`              |
> | **内存效率**  | 总是需要克隆（内部转换）     | 已持有 `String` 时避免克隆   |
> | **典型使用场景**| 参数可能来自多种来源         | 调用方已持有 `String` 所有权 |
> | **API 友好度**| 更符合 Rust 生态的惯用模式   | 需要调用方处理所有权         |

然后，我把它对于 `new 函数 ` 这个场景的分析、衡量和选择，总结如下：

乍看，`Operator::new` 的实参来自 token 序列，也就是 `Vec<String>`，那么参数类型设为 `String` 似乎更顺理成章一些。因为只要所有权转移到 Operator 对象即可，不需要像 `&str` 那样再做一次克隆。但这几乎也是 `String` 唯一的优势。反观 `&str`，除了克隆可能带来的有限的性能开销，其他都是优点：

* 接受不同类型的实参，`String` 或者字符串字面量都可以自动解引用为 `&str`
* 调用者不需要担心类型转换或者所有权问题
* 根据 Rust 的最佳实践，通常由结构体来管理内部数据的所有权。接受 `&str` 并在内部转换为 `String` 更符合社区的惯用做法：Rust 标准库（如 `String::from_str`）和主流库的常见做法

总结一下，就是 `&str` 让 API 更简洁，调用者更灵活。所以，**除非有明确的性能需求或所有权管理需求，否则优先使用 `&str` 作为参数类型 **。



#### 结构体中的复合类型字段用类型本身还是引用

面对嵌套结构体的情况，我有了类似 ` 字符串类型选择 ` 的问题：选择使用结构体本身还是引用？

[DeepSeek 给出了决策流程](../compound_type_in_struct.md)：

>1. 优先考虑简单所有权模型
>2. 需要共享时首选不可变引用
>3. 必须共享可变状态时使用 `Arc<Mutex<T>>`
>4. 超大结构体（>1KB）使用 `Box` 或智能指针
>5. 生命周期无法保证时回退到所有权模式
>

最终，我选择保持原有结构，亦即除了对可能会成为超大结构体的 AstTree 使用智能指针指向以外，其他都是默认的直接持有所有权：

```rust
pub enum AstNode {
    Leaf(String),
    Root(Box<AstTree>),
}
```



## Reference

[^1]: https://github.com/rosedblabs/rust-practice/tree/main/expr-eval
[^2]: https://mp.weixin.qq.com/s/MuuaROoH7gI0wYVypEOoWw
[^3]: https://eli.thegreenplace.net/2012/08/02/parsing-expressions-by-precedence-climbing
[^4]: https://ycpcs.github.io/cs340-fall2018/lectures/lecture06.html
[^5]: https://en.wikipedia.org/wiki/Operator-precedence_parser
[^6]: https://en.wikipedia.org/wiki/Shunting_yard_algorithm
