use crate::expr::Expr;

pub trait Print {
    fn print(&self) -> String;
}

impl Print for Expr {
    fn print(&self) -> String {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => format!("({} {} {})", operator.text, left.print(), right.print()),
            Expr::Unary { operator, right } => {
                format!("({} {})", operator.text, right.print())
            }
            Expr::Grouping { expr } => format!("(group {})", expr.print()),
            Expr::Literal { value } => value.text.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::expr::*;
    use crate::print::*;
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

        assert_eq!("(* (- 123.0) (group 45.21))", expr.print())
    }
}
