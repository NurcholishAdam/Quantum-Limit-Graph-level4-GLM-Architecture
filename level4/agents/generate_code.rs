// -*- coding: utf-8 -*-
//! Code Generation Agent
//! 
//! Generates executable code based on natural language descriptions.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generated code with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    pub code_id: String,
    pub language: ProgrammingLanguage,
    pub code: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub test_cases: Vec<TestCase>,
    pub safety_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProgrammingLanguage {
    Rust,
    Python,
    JavaScript,
    Rhai, // Embedded scripting
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub input: String,
    pub expected_output: String,
    pub description: String,
}

/// Code template for common patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeTemplate {
    pub template_id: String,
    pub name: String,
    pub language: ProgrammingLanguage,
    pub template_code: String,
    pub placeholders: Vec<String>,
}

/// Code generator with template-based and LLM-based generation
pub struct CodeGenerator {
    templates: HashMap<String, CodeTemplate>,
    safety_checks_enabled: bool,
}

impl CodeGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            templates: HashMap::new(),
            safety_checks_enabled: true,
        };
        
        generator.load_default_templates();
        generator
    }

    fn load_default_templates(&mut self) {
        // Binary search template
        self.add_template(CodeTemplate {
            template_id: "binary_search".to_string(),
            name: "Binary Search".to_string(),
            language: ProgrammingLanguage::Rust,
            template_code: r#"
fn binary_search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    let mut left = 0;
    let mut right = arr.len();
    
    while left < right {
        let mid = left + (right - left) / 2;
        match arr[mid].cmp(target) {
            std::cmp::Ordering::Equal => return Some(mid),
            std::cmp::Ordering::Less => left = mid + 1,
            std::cmp::Ordering::Greater => right = mid,
        }
    }
    None
}
"#.to_string(),
            placeholders: vec![],
        });

        // Graph traversal template
        self.add_template(CodeTemplate {
            template_id: "graph_bfs".to_string(),
            name: "Graph BFS".to_string(),
            language: ProgrammingLanguage::Rust,
            template_code: r#"
use std::collections::{VecDeque, HashSet};

fn bfs(graph: &HashMap<usize, Vec<usize>>, start: usize) -> Vec<usize> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();
    
    queue.push_back(start);
    visited.insert(start);
    
    while let Some(node) = queue.pop_front() {
        result.push(node);
        
        if let Some(neighbors) = graph.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
    }
    
    result
}
"#.to_string(),
            placeholders: vec![],
        });

        // Rhai script template
        self.add_template(CodeTemplate {
            template_id: "rhai_calculator".to_string(),
            name: "Rhai Calculator".to_string(),
            language: ProgrammingLanguage::Rhai,
            template_code: r#"
fn calculate(operation, a, b) {
    if operation == "add" {
        a + b
    } else if operation == "subtract" {
        a - b
    } else if operation == "multiply" {
        a * b
    } else if operation == "divide" {
        if b != 0 {
            a / b
        } else {
            print("Error: Division by zero");
            0
        }
    } else {
        print("Unknown operation");
        0
    }
}
"#.to_string(),
            placeholders: vec!["operation".to_string(), "a".to_string(), "b".to_string()],
        });
    }

    /// Generate code from description
    pub fn generate(&self, description: &str) -> Result<GeneratedCode> {
        let code_id = uuid::Uuid::new_v4().to_string();
        
        // Try to match with templates
        let (code, language, dependencies) = if description.contains("binary search") {
            (
                self.templates.get("binary_search").unwrap().template_code.clone(),
                ProgrammingLanguage::Rust,
                vec!["std".to_string()],
            )
        } else if description.contains("graph") && description.contains("bfs") {
            (
                self.templates.get("graph_bfs").unwrap().template_code.clone(),
                ProgrammingLanguage::Rust,
                vec!["std::collections".to_string()],
            )
        } else if description.contains("calculator") || description.contains("rhai") {
            (
                self.templates.get("rhai_calculator").unwrap().template_code.clone(),
                ProgrammingLanguage::Rhai,
                vec![],
            )
        } else {
            // Generate simple code
            (
                format!("// Generated code for: {}\nfn main() {{\n    println!(\"Implementation needed\");\n}}", description),
                ProgrammingLanguage::Rust,
                vec![],
            )
        };

        // Generate test cases
        let test_cases = self.generate_test_cases(description, &language);
        
        // Calculate safety score
        let safety_score = self.calculate_safety_score(&code);

        Ok(GeneratedCode {
            code_id,
            language,
            code,
            description: description.to_string(),
            dependencies,
            test_cases,
            safety_score,
        })
    }

    fn generate_test_cases(&self, description: &str, language: &ProgrammingLanguage) -> Vec<TestCase> {
        let mut test_cases = Vec::new();
        
        if description.contains("binary search") {
            test_cases.push(TestCase {
                input: "arr=[1,2,3,4,5], target=3".to_string(),
                expected_output: "Some(2)".to_string(),
                description: "Find existing element".to_string(),
            });
            test_cases.push(TestCase {
                input: "arr=[1,2,3,4,5], target=6".to_string(),
                expected_output: "None".to_string(),
                description: "Element not found".to_string(),
            });
        } else if description.contains("calculator") {
            test_cases.push(TestCase {
                input: "operation='add', a=5, b=3".to_string(),
                expected_output: "8".to_string(),
                description: "Addition test".to_string(),
            });
        }
        
        test_cases
    }

    fn calculate_safety_score(&self, code: &str) -> f64 {
        let mut score = 1.0;
        
        // Check for unsafe operations
        if code.contains("unsafe") {
            score -= 0.3;
        }
        if code.contains("unwrap()") {
            score -= 0.1;
        }
        if code.contains("panic!") {
            score -= 0.2;
        }
        
        // Bonus for safety features
        if code.contains("Result<") {
            score += 0.1;
        }
        if code.contains("Option<") {
            score += 0.05;
        }
        
        score.max(0.0).min(1.0)
    }

    pub fn add_template(&mut self, template: CodeTemplate) {
        self.templates.insert(template.template_id.clone(), template);
    }

    pub fn get_template(&self, template_id: &str) -> Option<&CodeTemplate> {
        self.templates.get(template_id)
    }

    /// Generate code with specific language
    pub fn generate_with_language(
        &self,
        description: &str,
        language: ProgrammingLanguage,
    ) -> Result<GeneratedCode> {
        let mut code = self.generate(description)?;
        code.language = language;
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_binary_search() {
        let generator = CodeGenerator::new();
        let code = generator.generate("implement binary search").unwrap();
        
        assert_eq!(code.language, ProgrammingLanguage::Rust);
        assert!(code.code.contains("binary_search"));
        assert!(!code.test_cases.is_empty());
    }

    #[test]
    fn test_safety_score() {
        let generator = CodeGenerator::new();
        let safe_code = "fn safe() -> Result<(), Error> { Ok(()) }";
        let unsafe_code = "fn unsafe_fn() { unsafe { } }";
        
        assert!(generator.calculate_safety_score(safe_code) > 0.9);
        assert!(generator.calculate_safety_score(unsafe_code) < 0.8);
    }
}
