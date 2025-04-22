use anyhow::Context as AnyhowContext;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::code_writer::CodeWriter; // Changed from crate::codegen::
use crate::error::{CodeGenError, Result}; // Changed from crate::codegen::

/// A user-modifiable section in generated code
#[derive(Debug, Clone)]
pub struct UserSection {
  /// The name of the section
  pub name: String,
  /// Optional description of the section
  pub description: Option<String>,
  /// Optional default content for the section
  pub default_content: Option<String>,
}

impl UserSection {
  /// Create a new user section with a name
  pub fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
      description: None,
      default_content: None,
    }
  }

  /// Create a new user section with a name and description
  pub fn with_description(name: &str, description: &str) -> Self {
    Self {
      name: name.to_string(),
      description: Some(description.to_string()),
      default_content: None,
    }
  }

  /// Create a new user section with a name, description, and default content
  pub fn with_default(name: &str, description: Option<&str>, default_content: &str) -> Self {
    Self {
      name: name.to_string(),
      description: description.map(|s| s.to_string()),
      default_content: Some(default_content.to_string()),
    }
  }
}

/// Manager for user-modifiable sections in generated code
pub struct UserSectionManager {
  /// Map of section name to section definition
  sections: HashMap<String, UserSection>,
  /// Map of section name to captured content
  captured_content: HashMap<String, String>,
}

impl UserSectionManager {
  /// Create a new UserSectionManager
  pub fn new() -> Self {
    Self {
      sections: HashMap::new(),
      captured_content: HashMap::new(),
    }
  }

  /// Define a new user section with a name
  pub fn define_section(&mut self, name: &str) {
    self
      .sections
      .insert(name.to_string(), UserSection::new(name));
  }

  /// Define a new user section with a name and description
  pub fn define_section_with_description(&mut self, name: &str, description: &str) {
    self.sections.insert(
      name.to_string(),
      UserSection::with_description(name, description),
    );
  }

  /// Define a new user section with a name, description, and default content
  pub fn define_section_with_default(
    &mut self,
    name: &str,
    description: Option<&str>,
    default_content: &str,
  ) {
    self.sections.insert(
      name.to_string(),
      UserSection::with_default(name, description, default_content),
    );
  }

  /// Check if a section with the given name exists
  pub fn has_section(&self, name: &str) -> bool {
    self.sections.contains_key(name)
  }

  /// Get the content of a section
  pub fn get_section_content(&self, name: &str) -> Option<&str> {
    self
      .captured_content
      .get(name)
      .map(|s| s.as_str())
      .or_else(|| {
        self
          .sections
          .get(name)
          .and_then(|s| s.default_content.as_deref())
      })
  }

  /// Capture user sections from a file
  pub fn capture_from_file(&mut self, path: &Path) -> Result<()> {
    if !path.exists() {
      return Ok(());
    }

    let content = fs::read_to_string(path)
      .with_context(|| format!("Failed to read file: {}", path.display()))
      .map_err(|e| CodeGenError::CaptureFailed {
        path: path.to_path_buf(),
        source: e.into(),
      })?;

    self.capture_from_string(&content, path)
  }

  /// Capture user sections from a string
  pub fn capture_from_string(&mut self, content: &str, _path: &Path) -> Result<()> {
    let begin_pattern =
      Regex::new(r"/\* USER CODE BEGIN ([\w]+) \*/").map_err(CodeGenError::Regex)?;
    let end_pattern = Regex::new(r"/\* USER CODE END ([\w]+) \*/").map_err(CodeGenError::Regex)?;

    let mut current_section: Option<String> = None;
    let mut section_content = String::new();
    let mut line_number = 0;

    for line in content.lines() {
      line_number += 1;

      if let Some(caps) = begin_pattern.captures(line) {
        if current_section.is_some() {
          return Err(CodeGenError::NestedSection {
            line: line_number,
            section: current_section.unwrap().to_string(),
          });
        }

        let section_name = caps.get(1).unwrap().as_str();
        current_section = Some(section_name.to_string());
        section_content.clear();
        continue;
      }

      if let Some(caps) = end_pattern.captures(line) {
        let section_name = caps.get(1).unwrap().as_str();

        if let Some(ref current) = current_section {
          if current != section_name {
            return Err(CodeGenError::MismatchedSection {
              line: line_number,
              expected: current.clone(),
              found: section_name.to_string(),
            });
          }

          self
            .captured_content
            .insert(current.clone(), section_content.clone());
          current_section = None;
        } else {
          return Err(CodeGenError::InvalidSection(format!(
            "Unexpected user section end at line {}: no matching begin for '{}'",
            line_number, section_name
          )));
        }

        continue;
      }

      if let Some(_) = current_section {
        section_content.push_str(line);
        section_content.push('\n');
      }
    }

    if let Some(section) = current_section {
      return Err(CodeGenError::UnclosedSection(section));
    }

    Ok(())
  }

  /// Write a user section to a CodeWriter
  pub fn write_section<W: std::io::Write>(
    &self,
    writer: &mut CodeWriter<W>,
    name: &str,
  ) -> Result<()> {
    if !self.sections.contains_key(name) {
      return Err(CodeGenError::UnknownSection(name.to_string()));
    }

    let section = &self.sections[name];

    // Write section description if available
    if let Some(ref desc) = section.description {
      writer.write_separator(desc, 80)?;
    }

    // Write section begin marker
    writer.writeln(&format!("/* USER CODE BEGIN {} */", name))?;

    // Write section content
    let content = self.get_section_content(name).unwrap_or("");
    if !content.is_empty() {
      writer.write(content)?;
    }

    // Write section end marker
    writer.newline()?;
    writer.writeln(&format!("/* USER CODE END {} */", name))?;
    writer.newline()?;

    Ok(())
  }

  /// Get all defined section names
  pub fn section_names(&self) -> Vec<String> {
    self.sections.keys().cloned().collect()
  }

  /// Get all captured section names
  pub fn captured_section_names(&self) -> Vec<String> {
    self.captured_content.keys().cloned().collect()
  }

  /// Clear all captured content
  pub fn clear_captured_content(&mut self) {
    self.captured_content.clear();
  }

  /// Get a reference to the sections map
  pub fn sections(&self) -> &HashMap<String, UserSection> {
    &self.sections
  }

  /// Get a reference to the captured content map
  pub fn captured_content(&self) -> &HashMap<String, String> {
    &self.captured_content
  }
}

impl Default for UserSectionManager {
  fn default() -> Self {
    Self::new()
  }
}
