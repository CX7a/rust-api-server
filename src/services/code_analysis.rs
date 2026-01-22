use crate::error::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisResult {
    pub language: String,
    pub complexity: f64,
    pub maintainability: f64,
    pub security_issues: Vec<String>,
    pub performance_issues: Vec<String>,
}

pub struct CodeAnalyzer;

impl CodeAnalyzer {
    pub fn new() -> Self {
        CodeAnalyzer
    }

    pub fn analyze(&self, code: &str, language: &str) -> AppResult<CodeAnalysisResult> {
        let complexity = self.calculate_complexity(code);
        let maintainability = self.calculate_maintainability(code);
        let security_issues = self.detect_security_issues(code, language);
        let performance_issues = self.detect_performance_issues(code, language);

        Ok(CodeAnalysisResult {
            language: language.to_string(),
            complexity,
            maintainability,
            security_issues,
            performance_issues,
        })
    }

    fn calculate_complexity(&self, code: &str) -> f64 {
        // Simple cyclomatic complexity estimation
        let conditions = code.matches("if").count()
            + code.matches("for").count()
            + code.matches("while").count()
            + code.matches("match").count();

        1.0 + (conditions as f64 * 0.5)
    }

    fn calculate_maintainability(&self, code: &str) -> f64 {
        let lines = code.lines().count();
        let comments = code.matches("//").count() + code.matches("/*").count();
        let comment_ratio = if lines > 0 {
            (comments as f64) / (lines as f64) * 100.0
        } else {
            0.0
        };

        // Maintainability score (0-10)
        10.0 - (lines as f64 / 100.0).min(10.0) + (comment_ratio / 10.0).min(2.0)
    }

    fn detect_security_issues(&self, code: &str, _language: &str) -> Vec<String> {
        let mut issues = Vec::new();

        if code.contains("eval(") || code.contains("exec(") {
            issues.push("Dynamic code execution detected".to_string());
        }

        if code.contains("password") && !code.contains("hash") {
            issues.push("Potential plaintext password handling".to_string());
        }

        if code.contains("SQL") && !code.contains("prepared") {
            issues.push("Potential SQL injection vulnerability".to_string());
        }

        issues
    }

    fn detect_performance_issues(&self, code: &str, _language: &str) -> Vec<String> {
        let mut issues = Vec::new();

        if code.contains("nested for") || code.contains("for (") && code.contains("for (") {
            issues.push("Nested loops detected - O(n²) complexity".to_string());
        }

        if code.contains(".clone()") && code.matches(".clone()").count() > 5 {
            issues.push("Excessive cloning detected".to_string());
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_calculation() {
        let analyzer = CodeAnalyzer::new();
        let code = "if x { if y { if z { } } }";
        let complexity = analyzer.calculate_complexity(code);
        assert!(complexity > 1.0);
    }
}
