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
}
