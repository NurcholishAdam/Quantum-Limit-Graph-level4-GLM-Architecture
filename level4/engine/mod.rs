// -*- coding: utf-8 -*-
//! Code Execution Engine
//! 
//! Safe execution of generated code using sandboxed interpreters.

pub mod code_executor;

pub use code_executor::{CodeExecutor, ExecutionResult, ExecutionEnvironment};
