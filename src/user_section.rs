use anyhow::Context as AnyhowContext;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::code_writer::CodeWriter; // Changed from crate::codegen::
use crate::error::{CodeGenError, Result}; // Changed from crate::codegen::

/// A trait for dynamic content generation
#[allow(dead_code)]
pub trait ContentGenerator {
  fn generate(&self) -> String;
}

/// A simple function-based content generator
pub struct FunctionGenerator<F>
where
  F: Fn() -> String,
{
  generator: F,
}

impl<F> FunctionGenerator<F>
where
  F: Fn() -> String,
{
  #[allow(dead_code)]
  pub fn new(generator: F) -> Self {
    Self { generator }
  }
}

impl<F> ContentGenerator for FunctionGenerator<F>
where
  F: Fn() -> String,
{
  fn generate(&self) -> String {
    (self.generator)()
  }
}

/// A user-modifiable section in generated code
#[derive(Debug, Clone)]
pub struct UserSection {
  /// The name of the section
  pub name: String,
  /// Optional description of the section
  pub description: Option<String>,
  /// Optional default content for the section
  pub default_content: Option<String>,
  /// Whether this section uses dynamic content generation
  pub is_dynamic: bool,
}

impl UserSection {
  /// Create a new user section with a name
  pub fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
      description: None,
      default_content: None,
      is_dynamic: false,
    }
  }

  /// Create a new user section with a name and description
  pub fn with_description(name: &str, description: &str) -> Self {
    Self {
      name: name.to_string(),
      description: Some(description.to_string()),
      default_content: None,
      is_dynamic: false,
    }
  }

  /// Create a new user section with a name, description, and default content
  pub fn with_default(name: &str, description: Option<&str>, default_content: &str) -> Self {
    Self {
      name: name.to_string(),
      description: description.map(|s| s.to_string()),
      default_content: Some(default_content.to_string()),
      is_dynamic: false,
    }
  }

  /// Create a new dynamic user section
  pub fn with_dynamic(name: &str, description: Option<&str>) -> Self {
    Self {
      name: name.to_string(),
      description: description.map(|s| s.to_string()),
      default_content: None,
      is_dynamic: true,
    }
  }
}

/// Manager for user-modifiable sections in generated code
pub struct UserSectionManager {
  /// Map of section name to section definition
  sections: HashMap<String, UserSection>,
  /// Map of section name to captured content
  captured_content: HashMap<String, String>,
  /// Map of partial section number to captured content
  partial_sections: HashMap<u32, String>,
  /// Track which sections have been written to avoid duplicates
  written_sections: std::cell::RefCell<std::collections::HashSet<String>>,
  /// Dynamic content generators
  #[allow(dead_code)]
  dynamic_generators: HashMap<String, Box<dyn ContentGenerator>>,
}

impl UserSectionManager {
  /// Create a new UserSectionManager
  pub fn new() -> Self {
    Self {
      sections: HashMap::new(),
      captured_content: HashMap::new(),
      partial_sections: HashMap::new(),
      written_sections: std::cell::RefCell::new(std::collections::HashSet::new()),
      dynamic_generators: HashMap::new(),
    }
  }

  /// Reset the written sections tracker
  pub fn reset_written_tracker(&self) {
    self.written_sections.borrow_mut().clear();
  }

  /// Check if a section has already been written
  pub fn is_section_written(&self, name: &str) -> bool {
    self.written_sections.borrow().contains(name)
  }

  /// Mark a section as written
  fn mark_section_written(&self, name: &str) {
    self.written_sections.borrow_mut().insert(name.to_string());
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

  /// Define a user section with a dynamic content generator
  pub fn define_section_with_generator<F>(
    &mut self,
    name: &str,
    description: Option<&str>,
    generator: F,
  ) where
    F: Fn() -> String + 'static,
  {
    let content = generator();
    self.sections.insert(
      name.to_string(),
      UserSection::with_default(name, description, &content),
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
    // Patterns for USER CODE sections
    let begin_pattern =
      Regex::new(r"/\* USER CODE BEGIN ([\w]+) \*/").map_err(CodeGenError::Regex)?;
    let end_pattern = Regex::new(r"/\* USER CODE END ([\w]+) \*/").map_err(CodeGenError::Regex)?;
    
    // Patterns for partial update sections
    let partial_begin_pattern = 
      Regex::new(r"//!begin\s+(\d+)").map_err(CodeGenError::Regex)?;
    let partial_end_pattern = 
      Regex::new(r"//!end\s+(\d+)").map_err(CodeGenError::Regex)?;

    let mut current_section: Option<String> = None;
    let mut current_partial: Option<u32> = None;
    let mut section_content = String::new();
    let mut line_number = 0;

    for line in content.lines() {
      line_number += 1;

      // Check for partial section begin
      if let Some(caps) = partial_begin_pattern.captures(line) {
        if current_section.is_some() || current_partial.is_some() {
          return Err(CodeGenError::NestedSection {
            line: line_number,
            section: format!("partial section {}", caps.get(1).unwrap().as_str()),
          });
        }

        let section_num: u32 = caps.get(1).unwrap().as_str().parse().unwrap();
        current_partial = Some(section_num);
        section_content.clear();
        continue;
      }

      // Check for USER CODE section begin
      if let Some(caps) = begin_pattern.captures(line) {
        if current_section.is_some() || current_partial.is_some() {
          return Err(CodeGenError::NestedSection {
            line: line_number,
            section: current_section.unwrap_or_else(|| format!("partial {}", current_partial.unwrap())),
          });
        }

        let section_name = caps.get(1).unwrap().as_str();
        current_section = Some(section_name.to_string());
        section_content.clear();
        continue;
      }

      // Check for partial section end
      if let Some(caps) = partial_end_pattern.captures(line) {
        let section_num: u32 = caps.get(1).unwrap().as_str().parse().unwrap();

        if let Some(current_num) = current_partial {
          if current_num != section_num {
            return Err(CodeGenError::MismatchedSection {
              line: line_number,
              expected: current_num.to_string(),
              found: section_num.to_string(),
            });
          }

          self
            .partial_sections
            .insert(current_num, section_content.clone());
          current_partial = None;
        } else {
          return Err(CodeGenError::InvalidSection(format!(
            "Unexpected partial section end at line {}: no matching begin for '{}'",
            line_number, section_num
          )));
        }

        continue;
      }

      // Check for USER CODE section end
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

      if current_section.is_some() || current_partial.is_some() {
        section_content.push_str(line);
        section_content.push('\n');
      }
    }

    if let Some(section) = current_section {
      return Err(CodeGenError::UnclosedSection(section));
    }

    if let Some(partial_num) = current_partial {
      return Err(CodeGenError::UnclosedSection(format!("partial section {}", partial_num)));
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

    // Check for duplicate writes
    if self.is_section_written(name) {
      return Ok(()); // Silently skip if already written
    }
    self.mark_section_written(name);

    // Write section begin marker
    writer.writeln(&format!("/* USER CODE BEGIN {} */", name))?;

    // Write section content
    let content = self.get_section_content(name).unwrap_or_default();
    if !content.is_empty() {
      writer.write(&content)?;
      // Ensure content ends with newline if it doesn't already
      if !content.ends_with('\n') {
        writer.newline()?;
      }
    }

    // Write section end marker
    writer.writeln(&format!("/* USER CODE END {} */", name))?;
    writer.newline()?;

    Ok(())
  }

  /// Write a user section without description comment
  pub fn write_section_without_description<W: std::io::Write>(
    &self,
    writer: &mut CodeWriter<W>,
    name: &str,
  ) -> Result<()> {
    if !self.sections.contains_key(name) {
      return Err(CodeGenError::UnknownSection(name.to_string()));
    }

    // Write section begin marker
    writer.writeln(&format!("/* USER CODE BEGIN {} */", name))?;

    // Write section content
    let content = self.get_section_content(name).unwrap_or_default();
    if !content.is_empty() {
      writer.write(&content)?;
      if !content.ends_with('\n') {
        writer.newline()?;
      }
    }

    // Write section end marker
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
    self.partial_sections.clear();
    self.reset_written_tracker();
  }

  /// Write a partial section to a CodeWriter
  pub fn write_partial_section<W: std::io::Write>(
    &self,
    writer: &mut CodeWriter<W>,
    number: u32,
    default_content: Option<&str>,
  ) -> Result<()> {
    // Write section begin marker
    writer.writeln(&format!("//!begin {}", number))?;

    // Write section content
    if let Some(content) = self.partial_sections.get(&number) {
      writer.write(content)?;
    } else if let Some(default) = default_content {
      writer.write(default)?;
      if !default.ends_with('\n') {
        writer.newline()?;
      }
    }

    // Write section end marker
    writer.writeln(&format!("//!end {}", number))?;

    Ok(())
  }

  /// Get the content of a partial section
  pub fn get_partial_section_content(&self, number: u32) -> Option<&str> {
    self.partial_sections.get(&number).map(|s| s.as_str())
  }

  /// Check if a partial section exists
  pub fn has_partial_section(&self, number: u32) -> bool {
    self.partial_sections.contains_key(&number)
  }

  /// Get a reference to the sections map
  pub fn sections(&self) -> &HashMap<String, UserSection> {
    &self.sections
  }

  /// Get a reference to the captured content map
  pub fn captured_content(&self) -> &HashMap<String, String> {
    &self.captured_content
  }

  /// Write a simplified section with just content (no USER CODE markers)
  pub fn write_content_only<W: std::io::Write>(
    &self,
    writer: &mut CodeWriter<W>,
    name: &str,
  ) -> Result<()> {
    if !self.sections.contains_key(name) {
      return Err(CodeGenError::UnknownSection(name.to_string()));
    }

    let content = self.get_section_content(name).unwrap_or("");
    if !content.is_empty() {
      writer.write(content)?;
      if !content.ends_with('\n') {
        writer.newline()?;
      }
    }
    Ok(())
  }

  /// Get stats about defined and captured sections
  pub fn get_stats(&self) -> UserSectionStats {
    UserSectionStats {
      total_sections: self.sections.len(),
      captured_sections: self.captured_content.len(),
      partial_sections: self.partial_sections.len(),
      sections_with_default: self.sections.values()
        .filter(|s| s.default_content.is_some())
        .count(),
    }
  }

  /// Validate that all captured sections have corresponding definitions
  pub fn validate(&self) -> Result<()> {
    for captured_name in self.captured_content.keys() {
      if !self.sections.contains_key(captured_name) {
        return Err(CodeGenError::UnknownSection(format!(
          "Captured section '{}' has no definition", captured_name
        )));
      }
    }
    Ok(())
  }
}

/// Statistics about user sections
#[derive(Debug, Clone)]
pub struct UserSectionStats {
  pub total_sections: usize,
  pub captured_sections: usize,
  pub partial_sections: usize,
  pub sections_with_default: usize,
}

impl Default for UserSectionManager {
  fn default() -> Self {
    Self::new()
  }
}
