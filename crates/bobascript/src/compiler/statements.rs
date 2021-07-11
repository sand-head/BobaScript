use bobascript_parser::ast::{Expr, Stmt};

use super::compiler::Compiler;

impl Compiler {
  pub fn statement(&mut self, stmt: &Box<Stmt>) {
    match **stmt {
      Stmt::Function(ident, args, block) => self.function_stmt(ident, args, block),
      Stmt::Const(_, _) => todo!(),
      Stmt::Let(_, _) => todo!(),
      Stmt::Return(_) => todo!(),
      Stmt::Break(_) => todo!(),
      Stmt::Expression(_) => todo!(),
    }
  }

  fn function_stmt(&mut self, ident: String, args: Vec<String>, block: Box<Expr>) {}

  fn const_stmt(&mut self) {
    todo!("add const statement");
  }

  fn let_stmt(&mut self) {}

  fn return_stmt(&mut self) {}

  fn break_stmt(&mut self) {}

  fn expression_stmt(&mut self) {}
}
