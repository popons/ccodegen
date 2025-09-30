#[cfg(test)]
mod tests {
  use std::fs;
  use std::io::Cursor;
  use tempfile::tempdir;

  use super::super::code_writer::CodeWriter;
  use super::super::user_section::UserSectionManager;

  #[test]
  fn test_code_writer_basic() {
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = CodeWriter::new(&mut buffer);

    // Test basic writing
    writer.writeln("// This is a test").unwrap();
    writer.writeln("int main() {").unwrap();
    writer.indent();
    writer.writeln("printf(\"Hello, World!\\n\");").unwrap();
    writer.writeln("return 0;").unwrap();
    writer.dedent();
    writer.writeln("}").unwrap();

    let result = String::from_utf8(buffer.into_inner()).unwrap();
    let expected =
      "// This is a test\nint main() {\n    printf(\"Hello, World!\\n\");\n    return 0;\n}\n";
    assert_eq!(result, expected);
  }

  #[test]
  fn test_code_writer_functions() {
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = CodeWriter::new(&mut buffer);

    // Test function writing
    writer.write_include("stdio.h", true).unwrap();
    writer.newline().unwrap();
    writer
      .begin_function("int", "add", &[("int", "a"), ("int", "b")])
      .unwrap();
    writer.indent();
    writer.writeln("return a + b;").unwrap();
    writer.dedent();
    writer.end_function().unwrap();

    let result = String::from_utf8(buffer.into_inner()).unwrap();
    let expected = "#include <stdio.h>\n\nint add(int a, int b) {\n    return a + b;\n}\n";
    assert_eq!(result, expected);
  }

  #[test]
  fn test_user_section_manager() {
    let mut manager = UserSectionManager::new();

    // Define sections
    manager.define_section_with_description("Header", "File header");
    manager.define_section_with_default(
      "Includes",
      Some("System includes"),
      "#include <stdio.h>\n",
    );

    // Test section content retrieval
    assert_eq!(manager.get_section_content("Header"), None);
    assert_eq!(
      manager.get_section_content("Includes"),
      Some("#include <stdio.h>\n")
    );

    // Test section writing
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = CodeWriter::new(&mut buffer);

    manager.write_section(&mut writer, "Header").unwrap();
    manager.write_section(&mut writer, "Includes").unwrap();

    let result = String::from_utf8(buffer.into_inner()).unwrap();
    let expected = "/* File header */\n/* USER CODE BEGIN Header */\n\n/* USER CODE END Header */\n\n/* System includes */\n/* USER CODE BEGIN Includes */\n#include <stdio.h>\n/* USER CODE END Includes */\n\n";
    assert_eq!(result, expected);
  }

  #[test]
  fn test_capture_user_sections() {
    // Create a temporary directory for our test
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.c");

    // Create a test file with user sections
    let content = r#"/* Some header comment */
/* USER CODE BEGIN Header */
// This is a custom header
/* USER CODE END Header */

#include <stdio.h>
/* USER CODE BEGIN Includes */
#include <stdlib.h>
#include <string.h>
/* USER CODE END Includes */

int main() {
/* USER CODE BEGIN Main */
    printf("Hello, World!\n");
    return 0;
/* USER CODE END Main */
}
"#;
    fs::write(&file_path, content).unwrap();

    // Create a UserSectionManager and capture sections
    let mut manager = UserSectionManager::new();
    manager.define_section("Header");
    manager.define_section("Includes");
    manager.define_section("Main");

    manager.capture_from_file(&file_path).unwrap();

    // Verify captured content
    assert_eq!(
      manager.get_section_content("Header"),
      Some("// This is a custom header\n")
    );
    assert_eq!(
      manager.get_section_content("Includes"),
      Some("#include <stdlib.h>\n#include <string.h>\n")
    );
    assert_eq!(
      manager.get_section_content("Main"),
      Some("    printf(\"Hello, World!\\n\");\n    return 0;\n")
    );

    // Test writing captured content to a new file
    let output_path = dir.path().join("output.c");
    let output_file = fs::File::create(&output_path).unwrap();
    let mut writer = CodeWriter::new(output_file);

    writer.writeln("/* Generated file */").unwrap();
    manager.write_section(&mut writer, "Header").unwrap();
    writer.writeln("#include <stdio.h>").unwrap();
    manager.write_section(&mut writer, "Includes").unwrap();
    writer.writeln("int main() {").unwrap();
    manager.write_section(&mut writer, "Main").unwrap();
    writer.writeln("}").unwrap();
    writer.flush().unwrap();

    // Read the generated file and verify
    let generated = fs::read_to_string(&output_path).unwrap();
    let expected = r#"/* Generated file */
/* USER CODE BEGIN Header */
// This is a custom header
/* USER CODE END Header */

#include <stdio.h>
/* USER CODE BEGIN Includes */
#include <stdlib.h>
#include <string.h>
/* USER CODE END Includes */

int main() {
/* USER CODE BEGIN Main */
    printf("Hello, World!\n");
    return 0;
/* USER CODE END Main */

}
"#;
    assert_eq!(generated, expected);
  }

  #[test]
  fn test_partial_section_capture() {
    let content = r#"
// Some initial content
//!begin 1
// This is partial section 1
int counter = 0;
//!end 1

// More content
//!begin 2
void custom_function() {
    // User code here
}
//!end 2
"#;

    let mut manager = UserSectionManager::new();
    let result = manager.capture_from_string(content, std::path::Path::new("test.c"));
    assert!(result.is_ok());

    assert!(manager.has_partial_section(1));
    assert!(manager.has_partial_section(2));
    assert!(!manager.has_partial_section(3));

    let section1 = manager.get_partial_section_content(1).unwrap();
    assert!(section1.contains("int counter = 0;"));

    let section2 = manager.get_partial_section_content(2).unwrap();
    assert!(section2.contains("void custom_function()"));
  }

  #[test]
  fn test_partial_section_write() {
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = CodeWriter::new(&mut buffer);
    let manager = UserSectionManager::new();

    // Write partial section with default content
    let result = manager.write_partial_section(&mut writer, 1, Some("// Default content"));
    assert!(result.is_ok());

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    assert!(output.contains("//!begin 1"));
    assert!(output.contains("// Default content"));
    assert!(output.contains("//!end 1"));
  }

  #[test]
  fn test_partial_section_preserve() {
    // First, capture existing content
    let existing_content = r#"
//!begin 1
// User's custom code
int user_variable = 42;
//!end 1
"#;

    let mut manager = UserSectionManager::new();
    manager
      .capture_from_string(existing_content, std::path::Path::new("test.c"))
      .unwrap();

    // Write it back
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = CodeWriter::new(&mut buffer);
    manager
      .write_partial_section(&mut writer, 1, Some("// Default"))
      .unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    assert!(output.contains("int user_variable = 42;"));
    assert!(!output.contains("// Default"));
  }

  #[test]
  fn test_mixed_section_types() {
    let content = r#"
/* USER CODE BEGIN Header */
// File header
/* USER CODE END Header */

//!begin 1
// Partial section 1
//!end 1

/* USER CODE BEGIN Includes */
#include <stdio.h>
/* USER CODE END Includes */

//!begin 2
// Partial section 2
//!end 2
"#;

    let mut manager = UserSectionManager::new();
    manager.define_section("Header");
    manager.define_section("Includes");

    let result = manager.capture_from_string(content, std::path::Path::new("test.c"));
    assert!(result.is_ok());

    // Check USER CODE sections
    assert_eq!(
      manager.get_section_content("Header").unwrap().trim(),
      "// File header"
    );
    assert_eq!(
      manager.get_section_content("Includes").unwrap().trim(),
      "#include <stdio.h>"
    );

    // Check partial sections
    assert!(manager
      .get_partial_section_content(1)
      .unwrap()
      .contains("// Partial section 1"));
    assert!(manager
      .get_partial_section_content(2)
      .unwrap()
      .contains("// Partial section 2"));
  }

  #[test]
  fn test_nested_section_error() {
    let content = r#"
//!begin 1
//!begin 2
// Nested - should fail
//!end 2
//!end 1
"#;

    let mut manager = UserSectionManager::new();
    let result = manager.capture_from_string(content, std::path::Path::new("test.c"));
    assert!(result.is_err());
  }

  #[test]
  fn test_mismatched_section_error() {
    let content = r#"
//!begin 1
// Content
//!end 2
"#;

    let mut manager = UserSectionManager::new();
    let result = manager.capture_from_string(content, std::path::Path::new("test.c"));
    assert!(result.is_err());
  }

  #[test]
  fn test_unclosed_section_error() {
    let content = r#"
//!begin 1
// Content without end
"#;

    let mut manager = UserSectionManager::new();
    let result = manager.capture_from_string(content, std::path::Path::new("test.c"));
    assert!(result.is_err());
  }
}
