use bobascript_parser::ast::{Expr, Stmt};

use crate::chunk::OpCode;

use super::{compiler::Compiler, CompileError, FunctionType};

impl Compiler {
  pub fn statement(&mut self, stmt: &Box<Stmt>) {
    match &**stmt {
      Stmt::Function(ident, args, block) => self.function_stmt(ident, args, block),
      Stmt::Const(_, _) => self.const_stmt(),
      Stmt::Let(ident, expr) => self.let_stmt(ident, expr),
      Stmt::Return(expr) => self.return_stmt(expr),
      Stmt::Break(_) => self.break_stmt(),
      Stmt::Expression(expr) => self.expression_stmt(expr),
    }
  }

  fn function_stmt(&mut self, ident: &str, args: &Vec<String>, block: &Box<Expr>) {
    let global_idx = self.declare_variable(ident);
    self.mark_initialized();
    self.function(FunctionType::Function, ident, args, block);
    self.define_variable(global_idx);
  }

  fn const_stmt(&mut self) {
    todo!("add const statement");
  }

  fn let_stmt(&mut self, ident: &str, expr: &Option<Box<Expr>>) {
    let global = self.declare_variable(ident);

    if let Some(expr) = expr {
      self.expression(&expr);
    } else {
      self.emit_opcode(OpCode::Tuple(0));
    }

    self.define_variable(global);
  }

  fn return_stmt(&mut self, expr: &Option<Box<Expr>>) {
    if let FunctionType::TopLevel = self.context().fn_type {
      self.set_error(CompileError::TopLevelReturn);
    }

    if let Some(expr) = expr {
      self.expression(&expr);
    }
    self.emit_opcode(OpCode::Return);
  }

  fn break_stmt(&mut self) {
    todo!("add break statement");
  }

  fn expression_stmt(&mut self, expr: &Box<Expr>) {
    self.expression(&expr);
    self.emit_opcode(OpCode::Pop);
  }
}
