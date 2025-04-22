use std::io::Write;

use crate::error::{CodeGenError, Result}; // Changed from crate::codegen::
use crate::utils::repeat_str; // Changed from crate::codegen::

/// A writer for generating code with proper indentation and formatting
pub struct CodeWriter<W: Write> {
  /// The underlying writer
  writer: W,
  /// Current indentation level
  indent_level: usize,
  /// Number of spaces per indentation level
  indent_size: usize,
  /// Whether to add a newline after each write
  with_newline: bool,
}

impl<W: Write> CodeWriter<W> {
  /// Create a new CodeWriter with default settings
  pub fn new(writer: W) -> Self {
    Self {
      writer,
      indent_level: 0,
      indent_size: 4,
      with_newline: true,
    }
  }

  /// Create a new CodeWriter with custom settings
  pub fn with_options(writer: W, indent_size: usize, with_newline: bool) -> Self {
    Self {
      writer,
      indent_level: 0,
      indent_size,
      with_newline,
    }
  }

  /// Set whether to add a newline after each write
  pub fn set_with_newline(&mut self, with_newline: bool) {
    self.with_newline = with_newline;
  }

  /// Get whether to add a newline after each write
  pub fn with_newline(&self) -> bool {
    self.with_newline
  }

  /// Set the indentation size
  pub fn set_indent_size(&mut self, indent_size: usize) {
    self.indent_size = indent_size;
  }

  /// Get the indentation size
  pub fn indent_size(&self) -> usize {
    self.indent_size
  }

  /// Increase the indentation level
  pub fn indent(&mut self) {
    self.indent_level += 1;
  }

  /// Decrease the indentation level
  pub fn dedent(&mut self) {
    if self.indent_level > 0 {
      self.indent_level -= 1;
    }
  }

  /// Get the current indentation level
  pub fn indent_level(&self) -> usize {
    self.indent_level
  }

  /// Write a string with the current indentation
  pub fn write(&mut self, content: &str) -> Result<()> {
    if content.is_empty() {
      if self.with_newline {
        self
          .writer
          .write_all(b"\n")
          .map_err(|e| CodeGenError::Io(e))
      } else {
        Ok(())
      }
    } else {
      let indent = repeat_str(" ", self.indent_level * self.indent_size);

      for (i, line) in content.lines().enumerate() {
        if i > 0 {
          self
            .writer
            .write_all(b"\n")
            .map_err(|e| CodeGenError::Io(e))?;
        }

        if !line.is_empty() {
          self
            .writer
            .write_all(indent.as_bytes())
            .map_err(|e| CodeGenError::Io(e))?;

          self
            .writer
            .write_all(line.as_bytes())
            .map_err(|e| CodeGenError::Io(e))?;
        }
      }

      if self.with_newline && !content.ends_with('\n') {
        self
          .writer
          .write_all(b"\n")
          .map_err(|e| CodeGenError::Io(e))
      } else {
        Ok(())
      }
    }
  }

  /// Write a string with the current indentation and a newline
  pub fn writeln(&mut self, content: &str) -> Result<()> {
    let prev_newline = self.with_newline;
    self.with_newline = true;
    let result = self.write(content);
    self.with_newline = prev_newline;
    result
  }

  /// Write a newline
  pub fn newline(&mut self) -> Result<()> {
    self
      .writer
      .write_all(b"\n")
      .map_err(|e| CodeGenError::Io(e))
  }

  /// Write a line comment
  pub fn write_comment(&mut self, comment: &str) -> Result<()> {
    if comment.contains('\n') {
      self.writeln("/*")?;
      for line in comment.lines() {
        self.writeln(&format!(" * {}", line))?;
      }
      self.writeln(" */")
    } else {
      self.writeln(&format!("// {}", comment))
    }
  }

  /// Write a separator comment
  pub fn write_separator(&mut self, title: &str, _width: usize) -> Result<()> {
    self.writeln(&format!("/* {} */", title))
  }

  /// Begin a struct definition
  pub fn begin_struct(&mut self, name: &str) -> Result<()> {
    self.writeln(&format!("struct {} {{", name))
  }

  /// End a struct definition
  pub fn end_struct(&mut self) -> Result<()> {
    self.writeln("};")
  }

  /// Begin an enum definition
  pub fn begin_enum(&mut self, name: &str) -> Result<()> {
    self.writeln(&format!("enum {} {{", name))
  }

  /// End an enum definition
  pub fn end_enum(&mut self) -> Result<()> {
    self.writeln("};")
  }

  /// Write an enum member
  pub fn write_enum_member(&mut self, name: &str, value: Option<&str>) -> Result<()> {
    match value {
      Some(val) => self.writeln(&format!("    {} = {},", name, val)),
      None => self.writeln(&format!("    {},", name)),
    }
  }

  /// Begin a function definition
  pub fn begin_function(
    &mut self,
    ret_type: &str,
    name: &str,
    args: &[(&str, &str)],
  ) -> Result<()> {
    let args_str = if args.is_empty() {
      "(void)".to_string()
    } else {
      let args_formatted: Vec<String> = args
        .iter()
        .map(|(type_name, arg_name)| format!("{} {}", type_name, arg_name))
        .collect();

      format!("({})", args_formatted.join(", "))
    };

    self.writeln(&format!("{} {}{} {{", ret_type, name, args_str))
  }

  /// End a function definition
  pub fn end_function(&mut self) -> Result<()> {
    self.writeln("}")
  }

  /// Write a variable declaration
  pub fn write_variable(
    &mut self,
    type_name: &str,
    var_name: &str,
    comment: Option<&str>,
  ) -> Result<()> {
    if let Some(cmt) = comment {
      self.write_comment(cmt)?;
    }
    self.writeln(&format!("{} {};", type_name, var_name))
  }

  /// Write a #include directive
  pub fn write_include(&mut self, header: &str, is_system: bool) -> Result<()> {
    if is_system {
      self.writeln(&format!("#include <{}>", header))
    } else {
      self.writeln(&format!("#include \"{}\"", header))
    }
  }

  /// Write a #define directive
  pub fn write_define(&mut self, name: &str, value: Option<&str>) -> Result<()> {
    match value {
      Some(val) => self.writeln(&format!("#define {} {}", name, val)),
      None => self.writeln(&format!("#define {}", name)),
    }
  }

  /// Write a #ifdef directive
  pub fn write_ifdef(&mut self, name: &str) -> Result<()> {
    self.writeln(&format!("#ifdef {}", name))
  }

  /// Write a #ifndef directive
  pub fn write_ifndef(&mut self, name: &str) -> Result<()> {
    self.writeln(&format!("#ifndef {}", name))
  }

  /// Write a #endif directive
  pub fn write_endif(&mut self, comment: Option<&str>) -> Result<()> {
    match comment {
      Some(cmt) => self.writeln(&format!("#endif // {}", cmt)),
      None => self.writeln("#endif"),
    }
  }

  /// Write a typedef for a struct
  pub fn write_typedef_struct(&mut self, name: &str) -> Result<()> {
    self.writeln(&format!("typedef struct {} {};", name, name))
  }

  /// Write a function declaration
  pub fn write_function_declaration(
    &mut self,
    ret_type: &str,
    name: &str,
    args: &[(&str, &str)],
  ) -> Result<()> {
    let args_str = if args.is_empty() {
      "(void)".to_string()
    } else {
      let args_formatted: Vec<String> = args
        .iter()
        .map(|(type_name, arg_name)| format!("{} {}", type_name, arg_name))
        .collect();

      format!("({})", args_formatted.join(", "))
    };

    self.writeln(&format!("{} {}{};", ret_type, name, args_str))
  }

  /// Flush the underlying writer
  pub fn flush(&mut self) -> Result<()> {
    self.writer.flush().map_err(|e| CodeGenError::Io(e))
  }
}
