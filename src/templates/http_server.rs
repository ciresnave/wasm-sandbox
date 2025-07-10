//! HTTP Server template module

use crate::error::Result;
use crate::templates::{TemplateRenderer, SimpleTemplateRenderer};
use std::collections::HashMap;

/// HTTP Server template generator
pub struct HttpServerTemplate {
    renderer: SimpleTemplateRenderer,
}

impl HttpServerTemplate {
    /// Create a new HTTP server template generator
    pub fn new() -> Self {
        let mut renderer = SimpleTemplateRenderer::new();
        
        // Register the HTTP server template
        let template = include_str!("../../fixtures/http_server_template.rs.txt");
        renderer.register_template("http_server", template).unwrap();
        
        Self { renderer }
    }
    
    /// Generate HTTP server wrapper code
    pub fn generate(&self, variables: &HashMap<String, String>) -> Result<String> {
        self.renderer.render("http_server", variables)
    }
}

impl Default for HttpServerTemplate {
    fn default() -> Self {
        Self::new()
    }
}
