use std::fmt::format;

use crate::ast::BinaryOperator;
use crate::ast::Expression;
use crate::ast::LiteralValue;
use crate::ast::UnaryOperator;

pub struct AstFormatter;

impl AstFormatter {
    pub fn format_expression(&self, expression: &Expression) -> String {
        use Expression::*;
        match expression {
            Literal { value } => self.format_literal(value),
            Unary {
                operator,
                expression,
            } => self.format_unary(operator, expression),
            Binary {
                left,
                operator,
                right,
            } => self.format_binary(left, operator, right),
            Grouping { expression } => self.format_grouping(expression),
        }
    }

    fn format_literal(&self, literal: &LiteralValue) -> String {
        use LiteralValue::*;
        match literal {
            String(value) => format!("\"{}\"", value),
            Number(value) => format!("{}", value),
            Boolean(value) => match value {
                true => "True",
                false => "False",
            }
            .to_string(),
            Nil => "Nil".to_string(),
        }
    }

    fn format_unary(&self, operator: &UnaryOperator, expression: &Expression) -> String {
        let operator_str = match operator {
            UnaryOperator::Bang => "!",
            UnaryOperator::Minus => "-",
        };
        let expression_str = self.format_expression(expression);
        format!("({} {})", operator_str, expression_str)
    }

    fn format_binary(
        &self,
        left: &Expression,
        operator: &BinaryOperator,
        right: &Expression,
    ) -> String {
        let left_str = self.format_expression(left);
        let right_str = self.format_expression(right);

        let operator_str = match operator {
            BinaryOperator::EqualEqual => "==",
            BinaryOperator::BangEqual => "!=",
            BinaryOperator::Greater => ">",
            BinaryOperator::GreaterEqual => ">=",
            BinaryOperator::Less => "<",
            BinaryOperator::LessEqual => "<=",
            BinaryOperator::Minus => "-",
            BinaryOperator::Plus => "+",
            BinaryOperator::Slash => "/",
            BinaryOperator::Star => "*",
        };

        format!("({} {} {})", operator_str, left_str, right_str)
    }

    fn format_grouping(&self, expression: &Expression) -> String {
        let expression_str = self.format_expression(expression);
        format!("(group {})", expression_str)
    }
}
