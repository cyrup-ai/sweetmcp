//! Prompt templates for memory operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prompt template for memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// Template name
    pub name: String,

    /// Template content with placeholders
    pub template: String,

    /// Variables used in the template
    pub variables: Vec<String>,

    /// Example values for variables
    pub examples: HashMap<String, String>,
}

impl PromptTemplate {
    /// Create a new prompt template
    pub fn new(name: String, template: String) -> Self {
        let variables = extract_variables(&template);

        Self {
            name,
            template,
            variables,
            examples: HashMap::new(),
        }
    }

    /// Add an example value for a variable
    pub fn with_example(mut self, variable: &str, value: &str) -> Self {
        self.examples
            .insert(variable.to_string(), value.to_string());
        self
    }

    /// Format the template with provided values
    pub fn format(&self, values: &HashMap<String, String>) -> Result<String, String> {
        let mut result = self.template.clone();

        for variable in &self.variables {
            if let Some(value) = values.get(variable) {
                result = result.replace(&format!("{{{{{}}}}}", variable), value);
            } else {
                return Err(format!("Missing value for variable: {}", variable));
            }
        }

        Ok(result)
    }
}

/// Extract variables from a template string
fn extract_variables(template: &str) -> Vec<String> {
    let mut variables = Vec::new();
    let mut in_var = false;
    let mut current_var = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' && chars.peek() == Some(&'{') {
            chars.next(); // Skip second {
            in_var = true;
        } else if ch == '}' && chars.peek() == Some(&'}') && in_var {
            chars.next(); // Skip second }
            in_var = false;
            if !current_var.is_empty() {
                variables.push(current_var.clone());
                current_var.clear();
            }
        } else if in_var {
            current_var.push(ch);
        }
    }

    variables
}

/// Default prompt templates
pub struct DefaultPrompts;

impl DefaultPrompts {
    /// Memory extraction prompt
    pub fn memory_extraction() -> PromptTemplate {
        PromptTemplate::new(
            "memory_extraction".to_string(),
            r#"Extract key memories from the following text. Identify:
1. Important facts and information (semantic memory)
2. Specific events and experiences (episodic memory)
3. Skills or procedures mentioned (procedural memory)

Text: {{text}}

Output as JSON with the following structure:
{
  "semantic": ["fact1", "fact2", ...],
  "episodic": ["event1", "event2", ...],
  "procedural": ["skill1", "skill2", ...]
}"#.to_string(),
        )
        .with_example("text", "I learned Python programming last year at university. I remember struggling with recursion at first, but after practicing with fibonacci examples, it clicked.")
    }

    /// Memory summarization prompt
    pub fn memory_summarization() -> PromptTemplate {
        PromptTemplate::new(
            "memory_summarization".to_string(),
            r#"Summarize the following memories into a concise description:

Memories:
{{memories}}

Create a summary that captures the key information while being concise and informative."#
                .to_string(),
        )
    }

    /// Memory relationship extraction prompt
    pub fn relationship_extraction() -> PromptTemplate {
        PromptTemplate::new(
            "relationship_extraction".to_string(),
            r#"Analyze the following two memories and identify any relationships between them:

Memory 1: {{memory1}}
Memory 2: {{memory2}}

Identify relationships such as:
- Causal (one caused the other)
- Temporal (one happened before/after the other)
- Similarity (they share common themes or content)
- Contradiction (they contain conflicting information)
- Support (one provides evidence for the other)

Output as JSON:
{
  "relationship_type": "type",
  "confidence": 0.0-1.0,
  "explanation": "brief explanation"
}"#
            .to_string(),
        )
    }

    /// Query expansion prompt
    pub fn query_expansion() -> PromptTemplate {
        PromptTemplate::new(
            "query_expansion".to_string(),
            r#"Expand the following search query to find relevant memories. Generate alternative phrasings and related terms:

Query: {{query}}

Generate:
1. Alternative phrasings of the same concept
2. Related terms and synonyms
3. Broader and narrower terms

Output as JSON:
{
  "alternatives": ["phrase1", "phrase2", ...],
  "related": ["term1", "term2", ...],
  "broader": ["term1", "term2", ...],
  "narrower": ["term1", "term2", ...]
}"#.to_string(),
        )
    }
}

/// Prompt template manager
pub struct PromptManager {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptManager {
    /// Create a new prompt manager
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };

        // Add default templates
        manager.add_template(DefaultPrompts::memory_extraction());
        manager.add_template(DefaultPrompts::memory_summarization());
        manager.add_template(DefaultPrompts::relationship_extraction());
        manager.add_template(DefaultPrompts::query_expansion());

        manager
    }

    /// Add a template
    pub fn add_template(&mut self, template: PromptTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }

    /// Format a template with values
    pub fn format_template(
        &self,
        template_name: &str,
        values: &HashMap<String, String>,
    ) -> Result<String, String> {
        self.get_template(template_name)
            .ok_or_else(|| format!("Template '{}' not found", template_name))?
            .format(values)
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}
