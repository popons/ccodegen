use anyhow::Context as AnyhowContext;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::code_writer::CodeWriter; // Changed from crate::codegen::
use crate::error::Result;
use crate::user_section::UserSectionManager; // Changed from crate::codegen:: // Changed from crate::codegen::

/// Example of generating a C header file with user-modifiable sections
pub fn generate_example_header(output_path: &Path, capture_path: Option<&Path>) -> Result<()> {
  // Create a UserSectionManager and define sections
  let mut user_sections = UserSectionManager::new();

  // Define sections with descriptions and default content
  user_sections.define_section_with_description("Header", "File header comment");
  user_sections.define_section_with_default(
    "Includes",
    Some("Additional includes"),
    "#include <stdio.h>\n#include <stdlib.h>\n",
  );
  user_sections.define_section_with_default(
    "Typedefs",
    Some("User-defined types"),
    "typedef unsigned int uint32_t;\ntypedef unsigned char uint8_t;\n",
  );
  user_sections.define_section_with_default(
    "Constants",
    Some("User-defined constants"),
    "#define MAX_BUFFER_SIZE 1024\n#define VERSION \"1.0.0\"\n",
  );
  user_sections.define_section("Functions");

  // Capture existing user sections if a capture path is provided
  if let Some(path) = capture_path {
    user_sections
      .capture_from_file(path)
      .with_context(|| format!("Failed to capture user sections from {}", path.display()))?;
  }

  // Create a CodeWriter
  let file = File::create(output_path)
    .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
  let mut writer = CodeWriter::new(BufWriter::new(file));

  // Write the header file

  // Header section
  user_sections.write_section(&mut writer, "Header")?;

  // Include guards
  let guard_name = "EXAMPLE_H";
  writer.write_ifndef(guard_name)?;
  writer.write_define(guard_name, None)?;
  writer.newline()?;

  // Includes section
  user_sections.write_section(&mut writer, "Includes")?;
  writer.newline()?;

  // Typedefs section
  user_sections.write_section(&mut writer, "Typedefs")?;
  writer.newline()?;

  // Constants section
  user_sections.write_section(&mut writer, "Constants")?;
  writer.newline()?;

  // Struct definition
  writer.write_separator("Struct definitions", 80)?;
  writer.write_typedef_struct("ExampleStruct")?;
  writer.begin_struct("ExampleStruct")?;
  writer.indent();
  writer.write_variable("int", "id", Some("Unique identifier"))?;
  writer.write_variable("char*", "name", Some("Name string"))?;
  writer.write_variable("uint32_t", "flags", Some("Bit flags"))?;
  writer.dedent();
  writer.end_struct()?;
  writer.newline()?;

  // Function declarations
  writer.write_separator("Function declarations", 80)?;
  writer.write_function_declaration("void", "example_init", &[])?;
  writer.write_function_declaration(
    "int",
    "example_process",
    &[("ExampleStruct*", "data"), ("uint32_t", "size")],
  )?;
  writer.write_function_declaration("void", "example_cleanup", &[])?;
  writer.newline()?;

  // User-defined functions section
  user_sections.write_section(&mut writer, "Functions")?;

  // End include guard
  writer.write_endif(Some(guard_name))?;

  // Flush the writer
  writer.flush()?;

  Ok(())
}

/// Example of generating a C source file with user-modifiable sections
pub fn generate_example_source(
  output_path: &Path,
  header_name: &str,
  capture_path: Option<&Path>,
) -> Result<()> {
  // Create a UserSectionManager and define sections
  let mut user_sections = UserSectionManager::new();

  // Define sections with descriptions and default content
  user_sections.define_section_with_description("Header", "File header comment");
  user_sections.define_section_with_default("Includes", Some("Additional includes"), "");
  user_sections.define_section_with_default(
    "Globals",
    Some("Global variables"),
    "static ExampleStruct g_examples[MAX_BUFFER_SIZE];\nstatic int g_count = 0;\n",
  );
  user_sections.define_section_with_default(
        "InitFunction",
        Some("Initialization function implementation"),
        "    // Initialize the example system\n    g_count = 0;\n    memset(g_examples, 0, sizeof(g_examples));\n"
    );
  user_sections.define_section_with_default(
        "ProcessFunction",
        Some("Processing function implementation"),
        "    // Process the data\n    if (data == NULL || size == 0) {\n        return -1;\n    }\n    \n    // Copy data to global storage\n    if (g_count < MAX_BUFFER_SIZE) {\n        g_examples[g_count++] = *data;\n        return 0;\n    }\n    \n    return -1;\n"
    );
  user_sections.define_section_with_default(
    "CleanupFunction",
    Some("Cleanup function implementation"),
    "    // Clean up resources\n    g_count = 0;\n",
  );

  // Capture existing user sections if a capture path is provided
  if let Some(path) = capture_path {
    user_sections
      .capture_from_file(path)
      .with_context(|| format!("Failed to capture user sections from {}", path.display()))?;
  }

  // Create a CodeWriter
  let file = File::create(output_path)
    .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
  let mut writer = CodeWriter::new(BufWriter::new(file));

  // Write the source file

  // Header section
  user_sections.write_section(&mut writer, "Header")?;

  // Include the header file
  writer.write_include(header_name, false)?;
  writer.write_include("string.h", true)?;

  // Additional includes section
  user_sections.write_section(&mut writer, "Includes")?;
  writer.newline()?;

  // Global variables section
  user_sections.write_section(&mut writer, "Globals")?;
  writer.newline()?;

  // Function implementations
  writer.write_separator("Function implementations", 80)?;

  // Init function
  writer.begin_function("void", "example_init", &[])?;
  user_sections.write_section(&mut writer, "InitFunction")?;
  writer.end_function()?;
  writer.newline()?;

  // Process function
  writer.begin_function(
    "int",
    "example_process",
    &[("ExampleStruct*", "data"), ("uint32_t", "size")],
  )?;
  user_sections.write_section(&mut writer, "ProcessFunction")?;
  writer.end_function()?;
  writer.newline()?;

  // Cleanup function
  writer.begin_function("void", "example_cleanup", &[])?;
  user_sections.write_section(&mut writer, "CleanupFunction")?;
  writer.end_function()?;

  // Flush the writer
  writer.flush()?;

  Ok(())
}
