//! Code template system

use std::collections::HashMap;
use crate::error::{Error, Result};

/// Template renderer
pub trait TemplateRenderer {
    /// Render a template with variables
    fn render(&self, template_name: &str, variables: &HashMap<String, String>) -> Result<String>;
    
    /// Register a template
    fn register_template(&mut self, name: &str, template: &str) -> Result<()>;
    
    /// Check if a template exists
    fn has_template(&self, name: &str) -> bool;
    
    /// Get a template by name
    fn get_template(&self, name: &str) -> Option<&str>;
}

/// Simple template renderer
pub struct SimpleTemplateRenderer {
    /// Templates
    templates: HashMap<String, String>,
}

impl SimpleTemplateRenderer {
    /// Create a new simple template renderer
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }
    
    /// Create a new simple template renderer with built-in templates
    pub fn with_builtin_templates() -> Result<Self> {
        let mut renderer = Self::new();
        
        // Register HTTP server template
        renderer.register_template("http_server", include_str!("../../fixtures/http_server_template.rs.txt"))?;
        
        // Register MCP server template
        renderer.register_template("mcp_server", include_str!("../../fixtures/mcp_server_template.rs.txt"))?;
        
        // Register CLI tool template
        renderer.register_template("cli_tool", include_str!("../../fixtures/cli_tool_template.rs.txt"))?;
        
        // Register generic template
        renderer.register_template("generic", include_str!("../../fixtures/generic_template.rs.txt"))?;
        
        Ok(renderer)
    }
    
    /// Render a template with simple variable substitution
    fn render_simple(&self, template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        
        for (key, value) in variables {
            let var = format!("{{{{{}}}}}", key);
            result = result.replace(&var, value);
        }
        
        result
    }
}

impl TemplateRenderer for SimpleTemplateRenderer {
    fn render(&self, template_name: &str, variables: &HashMap<String, String>) -> Result<String> {
        let template = self.get_template(template_name)
            .ok_or_else(|| Error::Generic { message: format!("Template not found: {}", template_name) })?;
        
        Ok(self.render_simple(template, variables))
    }
    
    fn register_template(&mut self, name: &str, template: &str) -> Result<()> {
        self.templates.insert(name.to_string(), template.to_string());
        Ok(())
    }
    
    fn has_template(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }
    
    fn get_template(&self, name: &str) -> Option<&str> {
        self.templates.get(name).map(|s| s.as_str())
    }
}

pub mod http_server;
pub mod mcp_server;
pub mod cli;
pub mod cli_tool;
pub mod generic;
pub mod http_server_template;

// Re-export HTTP server template
pub use http_server_template::HTTP_SERVER_TEMPLATE;
