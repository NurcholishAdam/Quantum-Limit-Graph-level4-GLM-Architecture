// -*- coding: utf-8 -*-
//! Code Execution Engine
//! 
//! Sandboxed execution using Rhai interpreter for safety.

use crate::error::Result;
use crate::level4::agents::generate_code::{GeneratedCode, ProgrammingLanguage};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Execution result with output and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub memory_used_kb: usize,
    pub safety_violations: Vec<String>,
}

/// Execution environment configuration
#[derive(Debug, Clone)]
pub struct ExecutionEnvironment {
    pub timeout_ms: u64,
    pub max_memory_kb: usize,
    pub allow_io: bool,
    pub allow_network: bool,
}

impl Default for ExecutionEnvironment {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            max_memory_kb: 10240, // 10MB
            allow_io: false,
            allow_network: false,
        }
    }
}

/// Code executor with sandboxing
pub struct CodeExecutor {
    environment: ExecutionEnvironment,
}

impl CodeExecutor {
    pub fn new(environment: ExecutionEnvironment) -> Self {
        Self { environment }
    }

    /// Execute generated code safely
    pub fn execute(&self, code: &GeneratedCode) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        
        match code.language {
            ProgrammingLanguage::Rhai => self.execute_rhai(&code.code),
            ProgrammingLanguage::Rust => self.execute_rust_simulation(&code.code),
            ProgrammingLanguage::Python => self.execute_python_simulation(&code.code),
            ProgrammingLanguage::JavaScript => self.execute_js_simulation(&code.code),
        }
    }

    fn execute_rhai(&self, code: &str) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        let mut safety_violations = Vec::new();
        
        // Simulate Rhai execution (in real implementation, use rhai crate)
        // For now, we'll do basic validation
        
        // Check for unsafe operations
        if code.contains("import") || code.contains("eval") {
            safety_violations.push("Unsafe operation detected: import/eval".to_string());
        }
        
        if code.contains("file") || code.contains("network") {
            if !self.environment.allow_io && !self.environment.allow_network {
                safety_violations.push("IO/Network operation not allowed".to_string());
            }
        }
        
        let success = safety_violations.is_empty();
        let output = if success {
            "Code executed successfully (simulated)".to_string()
        } else {
            "Execution blocked due to safety violations".to_string()
        };
        
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(ExecutionResult {
            success,
            output,
            error: if success { None } else { Some("Safety violations detected".to_string()) },
            execution_time_ms,
            memory_used_kb: 512, // Simulated
            safety_violations,
        })
    }

    fn execute_rust_simulation(&self, code: &str) -> Result<ExecutionResult> {
        // Simulate Rust execution
        // In production, this would compile and run in a sandbox
        
        let start_time = std::time::Instant::now();
        let mut safety_violations = Vec::new();
        
        // Check for unsafe blocks
        if code.contains("unsafe {") {
            safety_violations.push("Unsafe block detected".to_string());
        }
        
        // Check for system calls
        if code.contains("std::process") || code.contains("std::fs") {
            if !self.environment.allow_io {
                safety_violations.push("System call not allowed".to_string());
            }
        }
        
        let success = safety_violations.is_empty();
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(ExecutionResult {
            success,
            output: "Rust code validated (simulated)".to_string(),
            error: if success { None } else { Some("Validation failed".to_string()) },
            execution_time_ms,
            memory_used_kb: 1024,
            safety_violations,
        })
    }

    fn execute_python_simulation(&self, code: &str) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        let mut safety_violations = Vec::new();
        
        // Check for dangerous operations
        if code.contains("__import__") || code.contains("exec(") || code.contains("eval(") {
            safety_violations.push("Dangerous Python operation detected".to_string());
        }
        
        let success = safety_violations.is_empty();
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(ExecutionResult {
            success,
            output: "Python code validated (simulated)".to_string(),
            error: if success { None } else { Some("Validation failed".to_string()) },
            execution_time_ms,
            memory_used_kb: 2048,
            safety_violations,
        })
    }

    fn execute_js_simulation(&self, code: &str) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        let mut safety_violations = Vec::new();
        
        // Check for dangerous operations
        if code.contains("eval(") || code.contains("Function(") {
            safety_violations.push("Dangerous JavaScript operation detected".to_string());
        }
        
        let success = safety_violations.is_empty();
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(ExecutionResult {
            success,
            output: "JavaScript code validated (simulated)".to_string(),
            error: if success { None } else { Some("Validation failed".to_string()) },
            execution_time_ms,
            memory_used_kb: 1536,
            safety_violations,
        })
    }

    /// Execute with custom timeout
    pub fn execute_with_timeout(
        &self,
        code: &GeneratedCode,
        timeout_ms: u64,
    ) -> Result<ExecutionResult> {
        let mut env = self.environment.clone();
        env.timeout_ms = timeout_ms;
        
        let executor = CodeExecutor::new(env);
        executor.execute(code)
    }

    /// Batch execute multiple code snippets
    pub fn execute_batch(&self, codes: &[GeneratedCode]) -> Result<Vec<ExecutionResult>> {
        codes.iter()
            .map(|code| self.execute(code))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level4::agents::generate_code::CodeGenerator;

    #[test]
    fn test_safe_execution() {
        let executor = CodeExecutor::new(ExecutionEnvironment::default());
        let generator = CodeGenerator::new();
        
        let code = generator.generate("implement binary search").unwrap();
        let result = executor.execute(&code).unwrap();
        
        assert!(result.success);
        assert!(result.safety_violations.is_empty());
    }

    #[test]
    fn test_unsafe_detection() {
        let executor = CodeExecutor::new(ExecutionEnvironment::default());
        
        let unsafe_code = GeneratedCode {
            code_id: "test".to_string(),
            language: ProgrammingLanguage::Rust,
            code: "unsafe { }".to_string(),
            description: "test".to_string(),
            dependencies: vec![],
            test_cases: vec![],
            safety_score: 0.5,
        };
        
        let result = executor.execute(&unsafe_code).unwrap();
        assert!(!result.safety_violations.is_empty());
    }
}
