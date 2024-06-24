pub(crate) mod writer;

use std::cell::RefCell;
use std::fmt::Display;
use std::io::prelude::*;
use std::rc::Rc;
use std::{io, mem};

use writer::Buffer;
pub(super) use writer::Writer;

pub use writer::Target;

// Formatting
pub use anstyle as style;
use log::Level;
use log::Record;

pub struct Formatter {
    buf: Rc<RefCell<Buffer>>,
    write_style: anstream::ColorChoice,
}

impl Formatter {
    pub(crate) fn new(writer: &Writer) -> Self {
        Formatter {
            buf: Rc::new(RefCell::new(writer.buffer())),
            write_style: writer.write_style(),
        }
    }

    pub(crate) fn write_style(&self) -> anstream::ColorChoice {
        self.write_style
    }

    pub(crate) fn print(&self, writer: &Writer) -> io::Result<()> {
        writer.print(&self.buf.borrow())
    }

    pub(crate) fn clear(&mut self) {
        self.buf.borrow_mut().clear()
    }
}

impl Formatter {
    pub fn default_level_style(&self, level: Level) -> style::Style {
        if self.write_style == anstream::ColorChoice::Never {
            style::Style::new()
        } else {
            match level {
                Level::Trace => style::AnsiColor::Cyan.on_default(),
                Level::Debug => style::AnsiColor::Blue.on_default(),
                Level::Info => style::AnsiColor::Green.on_default(),
                Level::Warn => style::AnsiColor::Yellow.on_default(),
                Level::Error => style::AnsiColor::Red
                    .on_default()
                    .effects(style::Effects::BOLD),
            }
        }
    }
}

impl Write for Formatter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buf.borrow_mut().flush()
    }
}

pub(crate) type FormatFn = Box<dyn Fn(&mut Formatter, &Record) -> io::Result<()> + Sync + Send>;

pub(crate) struct Builder {
    pub format_module_path: bool,
    pub format_target: bool,
    pub format_level: bool,
    pub format_suffix: &'static str,
    built: bool,
}

impl Builder {
    pub fn build(&mut self) -> FormatFn {
        assert!(!self.built, "attempt to re-use consumed builder");

        let built = mem::replace(
            self,
            Builder {
                built: true,
                ..Default::default()
            },
        );

        Box::new(move |buf, record| {
            let fmt = DefaultFormat {
                module_path: built.format_module_path,
                target: built.format_target,
                level: built.format_level,
                written_header_value: false,
                suffix: built.format_suffix,
                buf,
            };

            fmt.write(record)
        })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            format_module_path: false,
            format_target: true,
            format_level: true,
            format_suffix: "\n",
            built: false,
        }
    }
}

type SubtleStyle = StyledValue<&'static str>;
struct StyledValue<T> {
    style: style::Style,
    value: T,
}

impl<T: std::fmt::Display> std::fmt::Display for StyledValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let style = self.style;

        // We need to make sure `f`s settings don't get passed onto the styling but do get passed
        // to the value
        write!(f, "{style}")?;
        self.value.fmt(f)?;
        write!(f, "{style:#}")?;
        Ok(())
    }
}

struct DefaultFormat<'a> {
    module_path: bool,
    target: bool,
    level: bool,
    written_header_value: bool,
    buf: &'a mut Formatter,
    suffix: &'a str,
}

impl<'a> DefaultFormat<'a> {
    fn write(mut self, record: &Record) -> io::Result<()> {
        self.write_level(record)?;
        self.write_module_path(record)?;
        self.write_target(record)?;
        self.finish_header()?;

        self.write_args(record)?;
        write!(self.buf, "{}", self.suffix)
    }

    fn subtle_style(&self, text: &'static str) -> SubtleStyle {
        StyledValue {
            style: if self.buf.write_style == anstream::ColorChoice::Never {
                style::Style::new()
            } else {
                style::AnsiColor::BrightBlack.on_default()
            },
            value: text,
        }
    }

    fn write_header_value<T>(&mut self, value: T) -> io::Result<()>
    where
        T: Display,
    {
        if !self.written_header_value {
            self.written_header_value = true;

            let open_brace = self.subtle_style("[");
            write!(self.buf, "{}{}", open_brace, value)
        } else {
            write!(self.buf, " {}", value)
        }
    }

    fn write_level(&mut self, record: &Record) -> io::Result<()> {
        if !self.level {
            return Ok(());
        }

        let level = {
            let level = record.level();
            StyledValue {
                style: self.buf.default_level_style(level),
                value: level,
            }
        };

        self.write_header_value(format_args!("{:<5}", level))
    }

    fn write_module_path(&mut self, record: &Record) -> io::Result<()> {
        if !self.module_path {
            return Ok(());
        }

        if let Some(module_path) = record.module_path() {
            self.write_header_value(module_path)
        } else {
            Ok(())
        }
    }

    fn write_target(&mut self, record: &Record) -> io::Result<()> {
        if !self.target {
            return Ok(());
        }

        match record.target() {
            "" => Ok(()),
            target => self.write_header_value(target),
        }
    }

    fn finish_header(&mut self) -> io::Result<()> {
        if self.written_header_value {
            let close_brace = self.subtle_style("]");
            write!(self.buf, "{} ", close_brace)
        } else {
            Ok(())
        }
    }

    fn write_args(&mut self, record: &Record) -> io::Result<()> {
        write!(self.buf, "{}", record.args())
    }
}
