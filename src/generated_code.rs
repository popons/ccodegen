use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context as AnyhowContext, Result};

/// Manager for generated code sections in user files
pub struct GeneratedCodeManager {
  /// Map of (tool_name, purpose) to generated content
  sections: HashMap<(String, String), String>,
}

impl GeneratedCodeManager {
  /// Create a new GeneratedCodeManager
  pub fn new() -> Self {
    Self {
      sections: HashMap::new(),
    }
  }

  /// Set the content for a generated code section
  pub fn set_section(&mut self, tool_name: &str, purpose: &str, content: String) {
    self
      .sections
      .insert((tool_name.to_string(), purpose.to_string()), content);
  }

  /// Embed all registered generated code sections into a file
  pub fn embed_to_file(&self, path: &Path) -> Result<()> {
    if !path.exists() {
      // If file doesn't exist, create it with all sections
      let mut content = String::new();
      for ((tool_name, purpose), code) in &self.sections {
        content.push_str(&format!(
          "/* GENERATED CODE BEGIN {} {} */\n",
          tool_name, purpose
        ));
        content.push_str(code);
        content.push_str(&format!(
          "\n/* GENERATED CODE END {} {} */\n\n",
          tool_name, purpose
        ));
      }
      fs::write(path, content)
        .with_context(|| format!("Failed to write to file: {}", path.display()))?;
      return Ok(());
    }

    // Read existing file content
    let mut content = fs::read_to_string(path)
      .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Process each section
    for ((tool_name, purpose), code) in &self.sections {
      let begin_marker = format!("/* GENERATED CODE BEGIN {} {} */", tool_name, purpose);
      let end_marker = format!("/* GENERATED CODE END {} {} */", tool_name, purpose);

      if let (Some(begin_pos), Some(end_pos)) =
        (content.find(&begin_marker), content.rfind(&end_marker))
      {
        // Section exists, replace content between markers
        let begin_marker_end = begin_pos + begin_marker.len();
        let replacement = format!("{}\n{}", begin_marker, code);
        content.replace_range(begin_pos..end_pos, &replacement);
      } else {
        // Section doesn't exist, append to end of file
        if !content.ends_with('\n') {
          content.push('\n');
        }
        content.push_str(&format!(
          "\n/* GENERATED CODE BEGIN {} {} */\n",
          tool_name, purpose
        ));
        content.push_str(code);
        content.push_str(&format!(
          "\n/* GENERATED CODE END {} {} */\n",
          tool_name, purpose
        ));
      }
    }

    // Write updated content back to file
    fs::write(path, content)
      .with_context(|| format!("Failed to write to file: {}", path.display()))?;

    Ok(())
  }
}

impl Default for GeneratedCodeManager {
  fn default() -> Self {
    Self::new()
  }
}
