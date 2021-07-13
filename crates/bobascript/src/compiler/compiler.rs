use std::rc::Rc;

use bobascript_parser::ast::{Ast, Expr, Stmt};

use super::{CompileContext, CompileError, CompileResult, FunctionType, Local};
use crate::{
  chunk::{JumpDirection, OpCode, Upvalue},
  debug::disassemble_chunk,
  value::{Function, Value},
};

pub struct Compiler {
  contexts: Vec<CompileContext>,
  errors: Vec<CompileError>,
}
impl Compiler {
  pub fn new() -> Self {
    Self {
      contexts: vec![CompileContext::new(FunctionType::TopLevel)],
      errors: vec![],
    }
  }

  pub fn compile(&mut self, ast: &Ast) -> CompileResult<Rc<Function>> {
    for stmt in ast {
      self.statement(stmt);
    }

    self.emit_opcode(OpCode::Tuple(0));
    let function = self.end_compiler();

    if self.errors.len() > 0 {
      let first = self.errors.pop().unwrap();
      self.errors.clear();
      Err(first)
    } else {
      Ok(function)
    }
  }

  /// Compiles a single expression and returns its containing function.
  pub fn compile_expr(&mut self, expr: &Box<Expr>) -> CompileResult<Rc<Function>> {
    self.expression(expr);

    let function = self.end_compiler();
    if self.errors.len() > 0 {
      let first = self.errors.pop().unwrap();
      self.errors.clear();
      Err(first)
    } else {
      Ok(function)
    }
  }

  pub(super) fn set_error(&mut self, error: CompileError) {
    self.errors.push(error);
  }

  pub(super) fn with_context<F>(&mut self, fn_type: FunctionType, f: F) -> CompileContext
  where
    F: FnOnce(&mut Compiler) -> (),
  {
    self.contexts.push(CompileContext::new(fn_type));
    f(self);
    self.contexts.pop().unwrap()
  }

  pub(super) fn context(&self) -> &CompileContext {
    &self.contexts.last().unwrap()
  }

  pub(super) fn context_mut(&mut self) -> &mut CompileContext {
    self.contexts.last_mut().unwrap()
  }

  pub(super) fn block(&mut self, stmts: &Vec<Box<Stmt>>, expr: &Option<Box<Expr>>) {
    for stmt in stmts {
      self.statement(&stmt);
    }

    if let Some(expr) = expr {
      self.expression(&expr);
    } else {
      self.emit_opcode(OpCode::Tuple(0));
    }
  }

  pub(super) fn function(
    &mut self,
    fn_type: FunctionType,
    ident: &String,
    args: &Vec<String>,
    block: &Box<Expr>,
  ) {
    let context = self.with_context(fn_type, |c| {
      if fn_type != FunctionType::TopLevel {
        c.context_mut().function.name = ident.clone();
      }
      c.begin_scope();

      for arg in args {
        if c.context().function.arity == u8::MAX {
          // todo: throw too many parameters error
        } else {
          c.context_mut().function.arity += 1;
        }

        // parse the parameter
        let idx = c.declare_variable(arg);
        c.define_variable(idx);
      }

      if let Expr::Block(stmts, expr) = &**block {
        c.block(stmts, expr);
        c.emit_opcode(OpCode::Return);
      } else {
        // throw big error dang
      }
    });

    if crate::DEBUG && (self.errors.is_empty() || crate::SUPER_DEBUG) {
      disassemble_chunk(
        &context.function.chunk,
        if context.function.name.len() > 0 {
          &context.function.name
        } else {
          "[script]"
        },
      );
    }

    let idx = self.make_constant(Value::Function(Rc::new(context.function)));
    self.emit_opcode(OpCode::Closure(idx, context.upvalues));
  }

  pub(super) fn make_constant(&mut self, value: Value) -> usize {
    self.context_mut().chunk_mut().add_constant(value)
  }

  pub(super) fn emit_opcode(&mut self, opcode: OpCode) {
    // let line_no = self.parser.previous().unwrap().line;
    // self.current_context_mut().current_chunk_mut().write(opcode, line_no);
    self.context_mut().chunk_mut().write(opcode);
  }

  /// Emits the given `OpCode` and returns its index in the chunk.
  pub(super) fn emit_opcode_idx(&mut self, opcode: OpCode) -> usize {
    // let line_no = self.parser.previous().unwrap().line;
    // self.current_context_mut().current_chunk_mut().write(opcode, line_no)
    self.context_mut().chunk_mut().write(opcode)
  }

  /// This just emits a `Jump` instruction, but backwards
  pub(super) fn emit_loop(&mut self, start: usize) {
    let offset = self.context_mut().chunk_mut().code.len() - start + 1;
    self.emit_opcode(OpCode::Jump(JumpDirection::Backwards, offset));
  }

  pub(super) fn patch_jump(&mut self, offset: usize) {
    let new_jump = self.context_mut().chunk_mut().code.len() - 1 - offset;
    let opcode = &self.context_mut().chunk_mut().code[offset];
    self.context_mut().chunk_mut().code[offset] = match opcode {
      OpCode::Jump(direction, _) => OpCode::Jump(*direction, new_jump),
      OpCode::JumpIfFalse(_) => OpCode::JumpIfFalse(new_jump),
      _ => unreachable!(),
    };
  }

  pub(super) fn with_scope<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Compiler) -> (),
  {
    self.begin_scope();
    f(self);
    self.end_scope();
  }

  fn begin_scope(&mut self) {
    self.context_mut().scope_depth += 1;
  }

  fn end_scope(&mut self) {
    self.context_mut().scope_depth -= 1;

    // let mut count: usize = 0;
    for i in (0..self.context().locals.len()).rev() {
      if self.context().locals[i].depth > self.context().scope_depth {
        if self.context().locals[i].is_captured {
          self.emit_opcode(OpCode::CloseUpvalue);
        } else {
          self.emit_opcode(OpCode::Pop);
        }
        self.context_mut().locals.remove(i);
        // count += 1;
      } else {
        break;
      }
    }

    /* todo: fix this so PopN isn't useless
    if count == 1 {
      self.emit_opcode(OpCode::Pop);
    } else if count > 1 {
      self.emit_opcode(OpCode::PopN(count));
    }
    */
  }

  /// Adds a variable to the scope
  pub(super) fn declare_variable(&mut self, name: &String) -> usize {
    if self.context().scope_depth > 0 {
      let name_exists = self
        .context()
        .locals
        .iter()
        .rev()
        .filter(|local| local.depth != -1 && local.depth < self.context().scope_depth)
        .find(|local| name == &local.name)
        .is_some();

      if name_exists {
        self.set_error(CompileError::VariableAlreadyExists(name.clone()));
      } else {
        self.context_mut().locals.push(Local {
          name: name.to_string(),
          depth: -1,
          is_captured: false,
        });
      }
      0
    } else {
      self.identifier_constant(name.to_string())
    }
  }

  pub(super) fn mark_initialized(&mut self) {
    if self.context().scope_depth != 0 {
      let idx = self.context().locals.len() - 1;
      self.context_mut().locals[idx].depth = self.context().scope_depth;
    }
  }

  /// Initializes a variable in the scope for use
  pub(super) fn define_variable(&mut self, global: usize) {
    if self.context().scope_depth > 0 {
      self.mark_initialized();
    } else {
      self.emit_opcode(OpCode::DefineGlobal(global));
    }
  }

  pub(super) fn resolve_variable(&mut self, name: &String) -> (OpCode, OpCode) {
    if let Some(idx) = self.resolve_local(name, 0) {
      (OpCode::GetLocal(idx), OpCode::SetLocal(idx))
    } else if let Some(idx) = self.resolve_upvalue(name, 0) {
      (OpCode::GetUpvalue(idx), OpCode::SetUpvalue(idx))
    } else {
      let idx = self.identifier_constant(name.clone());
      (OpCode::GetGlobal(idx), OpCode::SetGlobal(idx))
    }
  }

  fn identifier_constant(&mut self, lexeme: String) -> usize {
    self.make_constant(Value::String(lexeme))
  }

  fn resolve_local(&mut self, name: &String, context_idx: usize) -> Option<usize> {
    let context = self.contexts.iter_mut().nth_back(context_idx);
    let context = match context {
      Some(c) => c,
      // we've run out of compile contexts
      None => return None,
    };

    match context.resolve_local(name) {
      Ok(local) => local,
      Err(err) => {
        self.set_error(err);
        None
      }
    }
  }

  fn add_upvalue(&mut self, index: usize, is_local: bool, context_idx: usize) -> usize {
    // make sure we don't already have the upvalue before adding
    let upvalues = &mut self
      .contexts
      .iter_mut()
      .nth_back(context_idx)
      .unwrap()
      .upvalues;
    for i in 0..upvalues.len() {
      let upvalue = upvalues[i];
      if let Upvalue::Local(local_index) = upvalue {
        if local_index == index {
          return i;
        }
      }
    }

    upvalues.push(if is_local {
      Upvalue::Local(index)
    } else {
      Upvalue::Upvalue(index)
    });
    upvalues.len() - 1
  }

  fn resolve_upvalue(&mut self, name: &String, context_idx: usize) -> Option<usize> {
    if let Some(idx) = self.resolve_local(name, context_idx + 1) {
      self
        .contexts
        .iter_mut()
        .nth_back(context_idx + 1)
        .unwrap()
        .locals[idx]
        .is_captured = true;
      Some(self.add_upvalue(idx, true, context_idx))
    } else if context_idx + 1 >= self.contexts.len() {
      // if we continue here, we'd get stuck in an infinite loop until the stack overflows
      // this is because we are out of contexts to check
      None
    } else if let Some(idx) = self.resolve_upvalue(name, context_idx + 1) {
      Some(self.add_upvalue(idx, false, context_idx))
    } else {
      None
    }
  }

  fn end_compiler(&mut self) -> Rc<Function> {
    self.emit_opcode(OpCode::Return);
    let context = self.contexts.pop().unwrap();

    if crate::DEBUG && (self.errors.is_empty() || crate::SUPER_DEBUG) {
      disassemble_chunk(
        &context.function.chunk,
        if context.function.name.len() > 0 {
          &context.function.name
        } else {
          "[script]"
        },
      );
    }

    Rc::new(context.function)
  }
}
