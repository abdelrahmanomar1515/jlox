use crate::expr::{Expr, Visitor};

// pub struct Printer;
// impl Printer {
//     pub fn print(&mut self, expr: &Expr) -> String {
//         expr.accept(self)
//     }
// }

// impl Visitor for Printer {
//     type Out = String;
//
//     fn visit_literal(&mut self, value: &crate::token::Token) -> Self::Out {
//         value.text.clone()
//     }
//
//     fn visit_unary(&mut self, operator: &crate::token::Token, right: &Expr) -> Self::Out {
//         format!("({} {})", operator.text, right.accept(self))
//     }
//
//     fn visit_grouping(&mut self, expr: &Expr) -> Self::Out {
//         format!("(group {})", expr.accept(self))
//     }
//
//     fn visit_binary(
//         &mut self,
//         left: &Expr,
//         operator: &crate::token::Token,
//         right: &Expr,
//     ) -> Self::Out {
//         format!(
//             "({} {} {})",
//             operator.text,
//             left.accept(self),
//             right.accept(self)
//         )
//     }
// }

#[cfg(test)]
mod test {
    use crate::expr::*;
    use crate::print::Printer;
    use crate::token::*;

    #[test]
    fn it_prints_tree() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token {
                    token_type: TokenType::Minus,
                    text: "-".to_string(),
                    line: 1,
                },
                right: Box::new(Expr::Literal {
                    value: Token {
                        token_type: TokenType::Number(123.0),
                        text: "123.0".to_string(),
                        line: 1,
                    },
                }),
            }),
            operator: Token::new(TokenType::Star, "*".to_string(), 1),
            right: Box::new(Expr::Grouping {
                expr: Box::new(Expr::Literal {
                    value: Token::new(TokenType::Number(45.21), 45.21.to_string(), 1),
                }),
            }),
        };

        let mut printer = Printer;

        assert_eq!("(* (- 123.0) (group 45.21))", printer.print(&expr))
    }
}
