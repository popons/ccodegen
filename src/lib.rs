// Code generation module for generating code with user-modifiable sections

mod code_writer;
mod error;
pub mod examples;
mod generated_code;
#[cfg(test)]
mod tests;
mod user_section;
mod utils;

pub use code_writer::CodeWriter;
pub use error::{CodeGenError, Result};
pub use examples::{generate_example_header, generate_example_source};
pub use generated_code::GeneratedCodeManager;
pub use user_section::{UserSection, UserSectionManager};
