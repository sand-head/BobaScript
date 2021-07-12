use std::convert::TryInto;

use bobascript_parser::ast::{AssignOp, BinaryOp, Constant, Expr, Stmt, UnaryOp};

use crate::{
  chunk::{JumpDirection, OpCode},
  value::Value,
};

use super::{compiler::Compiler, CompileError};

impl Compiler {
  pub fn expression(&mut self, expr: &Box<Expr>) {
    match &**expr {
      Expr::Log(expr) => self.log_expr(expr),
      Expr::Block(stmts, expr) => self.block_expr(stmts, expr),
      Expr::If(condition, true_branch, false_branch) => {
        self.if_expr(condition, true_branch, false_branch)
      }
      Expr::While(condition, stmts) => self.while_expr(condition, stmts),
      Expr::Assign(name, op, expr) => self.assign_expr(name, op, expr),
      Expr::Binary(lhs, op, rhs) => self.binary_expr(lhs, op, rhs),
      Expr::Unary(op, expr) => self.unary_expr(op, expr),
      Expr::Index(object, index) => self.index_expr(object, index),
      Expr::Call(function, args) => self.call_expr(function, args),
      Expr::Constant(constant) => self.constant_expr(constant),
    }
  }

  fn log_expr(&mut self, expr: &Box<Expr>) {
    self.expression(&expr);
    self.emit_opcode(OpCode::Log);
  }

  fn block_expr(&mut self, stmts: &Vec<Box<Stmt>>, expr: &Option<Box<Expr>>) {
    self.with_scope(|c| {
      c.block(stmts, expr);
    });
  }

  fn if_expr(
    &mut self,
    condition: &Box<Expr>,
    true_branch: &Box<Expr>,
    false_branch: &Option<Box<Expr>>,
  ) {
    self.expression(&condition);
    let then_jump = self.emit_opcode_idx(OpCode::JumpIfFalse(0));
    self.emit_opcode(OpCode::Pop);

    self.expression(&true_branch);

    let else_jump = self.emit_opcode_idx(OpCode::Jump(JumpDirection::Forwards, 0));
    self.patch_jump(then_jump);
    self.emit_opcode(OpCode::Pop);

    if let Some(false_branch) = false_branch {
      match &**false_branch {
        Expr::Block(stmts, expr) => self.block_expr(&stmts, &expr),
        Expr::If(condition, true_branch, false_branch) => {
          self.if_expr(&condition, &true_branch, &false_branch)
        }
        _ => self.set_error(CompileError::UndefinedBehavior(
          r#"An expression other than "if" or "block" was found in the else clause."#.to_string(),
        )),
      }
    }

    self.patch_jump(else_jump);
  }

  fn while_expr(&mut self, condition: &Box<Expr>, stmts: &Vec<Box<Stmt>>) {
    let loop_start = self.context_mut().chunk_mut().code.len();

    self.expression(&condition);
    let exit_jump = self.emit_opcode_idx(OpCode::JumpIfFalse(0));
    self.emit_opcode(OpCode::Pop);

    for stmt in stmts {
      self.statement(&stmt);
    }

    self.emit_loop(loop_start);
    self.patch_jump(exit_jump);
    self.emit_opcode(OpCode::Pop);

    // since this *is* still an expression, it must return *something*
    self.emit_opcode(OpCode::Tuple(0));
  }

  fn assign_expr(&mut self, name: &Box<Expr>, op: &AssignOp, expr: &Box<Expr>) {
    if let Expr::Constant(Constant::Ident(name)) = &**name {
      let (get_op, set_op) = self.resolve_variable(name);
      match op {
        AssignOp::Assign => {
          self.expression(&expr);
          self.emit_opcode(set_op);
        }
        AssignOp::AddAssign => {
          self.emit_opcode(get_op);
          self.expression(&expr);
          self.emit_opcode(OpCode::Add);
          self.emit_opcode(set_op);
        }
        AssignOp::SubtractAssign => {
          self.emit_opcode(get_op);
          self.expression(&expr);
          self.emit_opcode(OpCode::Subtract);
          self.emit_opcode(set_op);
        }
        AssignOp::MultiplyAssign => {
          self.emit_opcode(get_op);
          self.expression(&expr);
          self.emit_opcode(OpCode::Multiply);
          self.emit_opcode(set_op);
        }
        AssignOp::DivideAssign => {
          self.emit_opcode(get_op);
          self.expression(&expr);
          self.emit_opcode(OpCode::Divide);
          self.emit_opcode(set_op);
        }
        AssignOp::ExponentAssign => {
          self.emit_opcode(get_op);
          self.expression(&expr);
          self.emit_opcode(OpCode::Exponent);
          self.emit_opcode(set_op);
        }
        AssignOp::OrAssign => {
          self.emit_opcode(get_op);
          let else_jump = self.emit_opcode_idx(OpCode::JumpIfFalse(0));
          let end_jump = self.emit_opcode_idx(OpCode::Jump(JumpDirection::Forwards, 0));

          self.patch_jump(else_jump);
          self.emit_opcode(OpCode::Pop);

          self.expression(&expr);
          self.patch_jump(end_jump);
        }
        AssignOp::AndAssign => {
          self.emit_opcode(get_op);
          let end_jump = self.emit_opcode_idx(OpCode::JumpIfFalse(0));

          self.emit_opcode(OpCode::Pop);
          self.expression(&expr);

          self.patch_jump(end_jump);
        }
      }
    } else {
      self.set_error(CompileError::InvalidAssignmentTarget);
    }
  }

  fn binary_expr(&mut self, lhs: &Box<Expr>, op: &BinaryOp, rhs: &Box<Expr>) {
    match op {
      BinaryOp::Or => {
        self.expression(lhs);
        let else_jump = self.emit_opcode_idx(OpCode::JumpIfFalse(0));
        let end_jump = self.emit_opcode_idx(OpCode::Jump(JumpDirection::Forwards, 0));

        self.patch_jump(else_jump);
        self.emit_opcode(OpCode::Pop);

        self.expression(rhs);
        self.patch_jump(end_jump);
      }
      BinaryOp::And => {
        self.expression(lhs);
        let end_jump = self.emit_opcode_idx(OpCode::JumpIfFalse(0));

        self.emit_opcode(OpCode::Pop);
        self.expression(rhs);

        self.patch_jump(end_jump);
      }
      BinaryOp::Equal => {
        self.expression(&lhs);
        self.expression(&rhs);
        self.emit_opcode(OpCode::Equal);
      }
      BinaryOp::NotEqual => {
        self.expression(&lhs);
        self.expression(&rhs);
        self.emit_opcode(OpCode::Equal);
        self.emit_opcode(OpCode::Not);
      }
      BinaryOp::GreaterThan => {
        self.expression(&lhs);
        self.expression(&rhs);
        self.emit_opcode(OpCode::GreaterThan);
      }
      BinaryOp::GreaterEqual => {
        self.expression(&lhs);
        self.expression(&rhs);
        self.emit_opcode(OpCode::LessThan);
        self.emit_opcode(OpCode::Not);
      }
      BinaryOp::LessThan => {
        self.expression(&lhs);
        self.expression(&rhs);
        self.emit_opcode(OpCode::LessThan);
      }
      BinaryOp::LessEqual => {
        self.expression(&lhs);
        self.expression(&rhs);
        self.emit_opcode(OpCode::GreaterThan);
        self.emit_opcode(OpCode::Not);
      }
      BinaryOp::Add => {
        self.expression(&lhs);
        self.expression(&rhs);
        self.emit_opcode(OpCode::Add);
      }
      BinaryOp::Subtract => {
        self.expression(&lhs);
        self.expression(&rhs);
        self.emit_opcode(OpCode::Subtract);
      }
      BinaryOp::Multiply => {
        self.expression(&lhs);
        self.expression(&rhs);
        self.emit_opcode(OpCode::Multiply);
      }
      BinaryOp::Divide => {
        self.expression(&lhs);
        self.expression(&rhs);
        self.emit_opcode(OpCode::Divide);
      }
      BinaryOp::Exponent => {
        self.expression(&lhs);
        self.expression(&rhs);
        self.emit_opcode(OpCode::Exponent);
      }
    }
  }

  fn unary_expr(&mut self, op: &UnaryOp, expr: &Box<Expr>) {
    self.expression(&expr);

    // emit the operator
    match op {
      UnaryOp::Negate => self.emit_opcode(OpCode::Negate),
      UnaryOp::Not => self.emit_opcode(OpCode::Not),
    }
  }

  fn index_expr(&mut self, object: &Box<Expr>, index: &Box<Expr>) {
    self.expression(object);
    self.expression(index);
    self.emit_opcode(OpCode::Index);
  }

  fn call_expr(&mut self, function: &Box<Expr>, args: &Vec<Box<Expr>>) {
    self.expression(function);
    for arg in args {
      self.expression(&arg);
      if args.len() >= u8::MAX.into() {
        self.set_error(CompileError::TooManyArguments);
      }
    }
    self.emit_opcode(OpCode::Call(args.len().try_into().unwrap()));
  }

  fn constant_expr(&mut self, constant: &Constant) {
    match constant {
      Constant::True => self.emit_opcode(OpCode::True),
      Constant::False => self.emit_opcode(OpCode::False),
      Constant::Ident(ident) => {
        let (get_op, _) = self.resolve_variable(ident);
        self.emit_opcode(get_op);
      }
      Constant::Number(num) => {
        let num_idx = self.make_constant(Value::Number(num.clone()));
        self.emit_opcode(OpCode::Constant(num_idx));
      }
      Constant::String(str) => {
        // strip the leading and trailing quotation mark off the string:
        let string = str[1..(str.len() - 1)].to_string();
        let string_idx = self.make_constant(Value::String(string));
        self.emit_opcode(OpCode::Constant(string_idx))
      }
      Constant::Tuple(tuple) => {
        for expr in tuple {
          self.expression(&expr);
        }
        self.emit_opcode(OpCode::Tuple(tuple.len().try_into().unwrap()));
      }
      Constant::Record(record) => {
        todo!("records not implemented")
      }
    }
  }
}
