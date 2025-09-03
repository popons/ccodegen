# ccodegen

A Rust crate for generating C/C++ code while preserving user-defined sections.

## Overview

`ccodegen` helps automate the generation of C or C++ code, particularly useful for scenarios where parts of the generated code need to be manually edited and preserved across regeneration cycles. It allows defining specific sections within the code that users can modify. When the code is regenerated, `ccodegen` can capture the content of these user sections from the existing file and re-insert them into the newly generated code, effectively merging generated code with manual modifications.

## Features

*   **C/C++ Code Generation:** Provides helpers (`CodeWriter`) for generating common C/C++ constructs like includes, defines, structs, enums, functions, and comments with proper indentation.
*   **User-Defined Sections:** Define named sections (`UserSectionManager`) within your code templates. These sections act as placeholders for user modifications.
*   **Preserve User Code:** Automatically capture and re-apply content from user-defined sections when regenerating code from existing files.
*   **Default Content:** Provide default content for user sections, which is used if the section doesn't exist in the captured file.

## Usage Example

Here's a basic example demonstrating how to generate a C header file:

```rust
use ccodegen::{CodeWriter, UserSectionManager, Result}; // Use the actual crate name
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use anyhow::Context; // For error handling context

fn generate_my_header(output_path: &Path, capture_path: Option<&Path>) -> Result<()> {
    // 1. Define user sections
    let mut user_sections = UserSectionManager::new();
    user_sections.define_section_with_default(
        "Includes",
        Some("Additional user includes"),
        "// Add your includes here\n",
    );
    user_sections.define_section("Declarations"); // Section for user declarations

    // 2. (Optional) Capture existing user sections
    if let Some(path) = capture_path {
        user_sections.capture_from_file(path)
            .context("Failed to capture user sections")?;
    }

    // 3. Setup CodeWriter
    let file = File::create(output_path)
        .context("Failed to create output file")?;
    let mut writer = CodeWriter::new(BufWriter::new(file));

    // 4. Generate code, writing user sections where needed
    writer.write_comment("Auto-generated header file")?;
    writer.write_ifndef("MY_HEADER_H")?;
    writer.write_define("MY_HEADER_H", None)?;
    writer.newline()?;

    // Write the user-defined Includes section
    user_sections.write_section(&mut writer, "Includes")?;
    writer.newline()?;

    // Write some generated code
    writer.write_separator("Generated Declarations", 80)?;
    writer.write_function_declaration("void", "generated_function", &[])?;
    writer.newline()?;

    // Write the user-defined Declarations section
    user_sections.write_section(&mut writer, "Declarations")?;
    writer.newline()?;

    writer.write_endif(Some("MY_HEADER_H"))?;

    // 5. Flush the writer
    writer.flush()?;

    Ok(())
}

// Example usage (assuming the function is called from an async context or error handling is set up):
// if let Err(e) = generate_my_header(Path::new("my_header.h"), Some(Path::new("my_header.h"))) {
//     eprintln!("Error generating header: {:?}", e);
// }
```

This example defines two user sections: `Includes` and `Declarations`. When `generate_my_header` is run, it will:
1.  Define the sections.
2.  Optionally read `my_header.h` (if it exists) to capture the content within `/* BEGIN USER SECTION: Includes */` and `/* END USER SECTION: Includes */` (and similarly for `Declarations`). The exact marker format might depend on `UserSectionManager`'s implementation details.
3.  Generate the new `my_header.h`, inserting the captured (or default) content for the user sections.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under the MIT license.
