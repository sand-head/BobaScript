use bobascript_parser::ast::{Expr, Stmt};

use crate::chunk::OpCode;

use super::compiler::Compiler;

impl Compiler {
  pub fn expression(&mut self, expr: &Box<Expr>) {
    match expr {
      Expr::Log(expr) => self.log_expr(expr),
      Expr::Block(stmts, expr) => self.block_expr(stmts, expr),
      Expr::If(condition, true_branch, false_branch) => {
        self.if_expr(condition, true_branch, false_branch)
      }
      Expr::While(_, _) => todo!(),
      Expr::Binary(_, _, _) => todo!(),
      Expr::Unary(_, _) => todo!(),
      Expr::Call(_, _) => todo!(),
      Expr::Constant(_) => todo!(),
    }
  }

  fn log_expr(&mut self, expr: &Box<Expr>) {
    self.expr(expr);
    self.emit_opcode(OpCode::Log);
  }

  fn block_expr(&mut self, stmts: &Vec<Box<Stmt>>, expr: &Option<Box<Expr>>) {}

  fn if_expr(&mut self, condition: &Box<Expr>, true_branch: &Box<Expr>, false_branch: &Box<Expr>) {}

  fn while_expr(&mut self) {}

  fn binary_expr(&mut self) {}

  fn unary_expr(&mut self) {}

  fn call_expr(&mut self) {}
}
