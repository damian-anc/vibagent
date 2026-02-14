use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use super::Tool;

pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Perform basic mathematical calculations. Supports addition, subtraction, multiplication, and division."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "The mathematical expression to evaluate, e.g., '2 + 2' or '10 / 2'."
                }
            },
            "required": ["expression"]
        })
    }

    async fn call(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let expression = args["expression"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'expression' argument"))?;

        let result = eval_arithmetic(expression)?;
        Ok(result.to_string())
    }
}

fn eval_arithmetic(expr: &str) -> Result<f64> {
    // Very naive parser for demonstration
    let expr = expr.replace(' ', "");
    
    if let Some(pos) = expr.find('+') {
        let left: f64 = expr[..pos].parse()?;
        let right: f64 = expr[pos + 1..].parse()?;
        return Ok(left + right);
    }
    if let Some(pos) = expr.find('-') {
        let left: f64 = expr[..pos].parse()?;
        let right: f64 = expr[pos + 1..].parse()?;
        return Ok(left - right);
    }
    if let Some(pos) = expr.find('*') {
        let left: f64 = expr[..pos].parse()?;
        let right: f64 = expr[pos + 1..].parse()?;
        return Ok(left * right);
    }
    if let Some(pos) = expr.find('/') {
        let left: f64 = expr[..pos].parse()?;
        let right: f64 = expr[pos + 1..].parse()?;
        if right == 0.0 {
            return Err(anyhow::anyhow!("Division by zero"));
        }
        return Ok(left / right);
    }

    expr.parse::<f64>().map_err(|e| anyhow::anyhow!("Failed to parse expression {}: {}", expr, e))
}
