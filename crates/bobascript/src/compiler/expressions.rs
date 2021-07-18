use std::{convert::TryInto, rc::Rc};

use bobascript_parser::ast::{AssignOp, BinaryOp, Constant, Expr, Stmt, UnaryOp};

use crate::{
  chunk::{JumpDirection, OpCode},
  value::Value,
};

use super::{compiler::Compiler, CompileError, FunctionType};

impl Compiler {
  pub fn expression(&mut self, expr: &Expr) {
    match &*expr {
      Expr::Log(expr) => self.log_expr(expr),
      Expr::Block(stmts, expr) => self.block_expr(stmts, expr),
      Expr::If(condition, true_branch, false_branch) => {
        self.if_expr(condition, true_branch, false_branch)
      }
      Expr::While(condition, stmts) => self.while_expr(condition, stmts),
      Expr::Assign(name, op, expr) => self.assign_expr(name, op, expr),
      Expr::Binary(lhs, op, rhs) => self.binary_expr(lhs, op, rhs),
      Expr::Unary(op, expr) => self.unary_expr(op, expr),
      Expr::Property(expr, prop) => self.property_expr(expr, prop),
      Expr::Index(expr, index) => self.index_expr(expr, index),
      Expr::Call(function, args) => self.call_expr(function, args),
      Expr::Constant(constant) => self.constant_expr(constant),
      Expr::Error => todo!(),
    }
  }

  fn log_expr(&mut self, expr: &Expr) {
    self.expression(&expr);
    self.emit_opcode(OpCode::Log);
  }

  fn block_expr(&mut self, stmts: &Vec<Box<Stmt>>, expr: &Option<Box<Expr>>) {
    // todo: maybe only compile blocks as functions in some cases instead of all
    // self.with_scope(|c| {
    //   c.block(stmts, expr);
    // });
    let context = self.with_context(FunctionType::Block, |c| {
      c.begin_scope();
      c.block(stmts, expr);
      c.emit_opcode(OpCode::Return);
    });

    // todo: disassemble the chunk

    let idx = self.make_constant(Value::Function(Rc::new(context.function)));
    self.emit_opcode(OpCode::Closure(idx, context.upvalues));
    self.emit_opcode(OpCode::Call(0));
  }

  fn if_expr(&mut self, condition: &Expr, true_branch: &Expr, false_branch: &Option<Box<Expr>>) {
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

  fn while_expr(&mut self, condition: &Expr, stmts: &Vec<Box<Stmt>>) {
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

  fn assign_expr(&mut self, name: &Expr, op: &AssignOp, expr: &Expr) {
    if let Expr::Constant(Constant::Ident(_, name)) = &*name {
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

  fn binary_expr(&mut self, lhs: &Expr, op: &BinaryOp, rhs: &Expr) {
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

  fn unary_expr(&mut self, op: &UnaryOp, expr: &Expr) {
    self.expression(&expr);

    // emit the operator
    match op {
      UnaryOp::Negate => self.emit_opcode(OpCode::Negate),
      UnaryOp::Not => self.emit_opcode(OpCode::Not),
    }
  }

  fn property_expr(&mut self, expr: &Expr, prop: &str) {
    self.expression(expr);
    self.emit_opcode(OpCode::GetProperty(prop.to_string()));
  }

  fn index_expr(&mut self, expr: &Expr, index: &Expr) {
    self.expression(expr);
    self.expression(index);
    self.emit_opcode(OpCode::Index);
  }

  fn call_expr(&mut self, function: &Expr, args: &Vec<Box<Expr>>) {
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
      Constant::Ident(_, ident) => {
        let (get_op, _) = self.resolve_variable(ident);
        self.emit_opcode(get_op);
      }
      Constant::Number(num) => {
        let num_idx = self.make_constant(Value::Number(*num));
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
        for (prop, expr) in record {
          self.expression(&expr);
          let prop = if prop.starts_with('"') {
            prop[1..(prop.len() - 1)].to_string()
          } else {
            prop.clone()
          };
          let idx = self.make_constant(Value::String(prop));
          self.emit_opcode(OpCode::Constant(idx));
        }
        self.emit_opcode(OpCode::Record(record.len().try_into().unwrap()));
      }
    }
  }
}
