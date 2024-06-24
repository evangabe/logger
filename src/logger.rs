use std::cell::RefCell;

use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};

use crate::fmt;
use crate::fmt::writer::{self, Writer};
use crate::fmt::{FormatFn, Formatter};

#[derive(Default)]
pub struct Builder {
    filter: env_filter::Builder,
    writer: writer::Builder,
    format: fmt::Builder,
    built: bool,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn format_level(&mut self, write: bool) -> &mut Self {
        self.format.format_level = write;
        self
    }

    pub fn format_module_path(&mut self, write: bool) -> &mut Self {
        self.format.format_module_path = write;
        self
    }

    pub fn format_target(&mut self, write: bool) -> &mut Self {
        self.format.format_target = write;
        self
    }

    /// Configures the end of line suffix.
    pub fn format_suffix(&mut self, suffix: &'static str) -> &mut Self {
        self.format.format_suffix = suffix;
        self
    }

    pub fn filter_module(&mut self, module: &str, level: LevelFilter) -> &mut Self {
        self.filter.filter_module(module, level);
        self
    }

    pub fn filter_level(&mut self, level: LevelFilter) -> &mut Self {
        self.filter.filter_level(level);
        self
    }

    pub fn filter(&mut self, module: Option<&str>, level: LevelFilter) -> &mut Self {
        self.filter.filter(module, level);
        self
    }

    pub fn parse_filters(&mut self, filters: &str) -> &mut Self {
        self.filter.parse(filters);
        self
    }

    pub fn is_test(&mut self, is_test: bool) -> &mut Self {
        self.writer.is_test(is_test);
        self
    }

    pub fn try_init(&mut self) -> Result<(), SetLoggerError> {
        let logger = self.build();

        let max_level = logger.filter();
        let r = log::set_boxed_logger(Box::new(logger));

        if r.is_ok() {
            log::set_max_level(max_level);
        }

        r
    }

    pub fn init(&mut self) {
        self.try_init()
            .expect("Builder::init should not be called after logger initialized");
    }

    pub fn build(&mut self) -> Logger {
        assert!(!self.built, "attempt to re-use consumed builder");
        self.built = true;

        Logger {
            writer: self.writer.build(),
            filter: self.filter.build(),
            format: self.format.build(),
        }
    }
}

pub struct Logger {
    writer: Writer,
    filter: env_filter::Filter,
    format: FormatFn,
}

impl Logger {
    pub fn filter(&self) -> LevelFilter {
        self.filter.filter()
    }

    pub fn matches(&self, record: &Record) -> bool {
        self.filter.matches(record)
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.filter.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if self.matches(record) {
            thread_local! {
                static FORMATTER: RefCell<Option<Formatter>> = RefCell::new(None);
            }

            let print = |formatter: &mut Formatter, record: &Record| {
                let _ =
                    (self.format)(formatter, record).and_then(|_| formatter.print(&self.writer));
                formatter.clear();
            };

            let printed = FORMATTER
                .try_with(|tl_buf| {
                    match tl_buf.try_borrow_mut() {
                        Ok(mut tl_buf) => match *tl_buf {
                            Some(ref mut formatter) => {
                                // Check the buffer style. If it's different from the logger's
                                // style then drop the buffer and recreate it.
                                if formatter.write_style() != self.writer.write_style() {
                                    *formatter = Formatter::new(&self.writer);
                                }

                                print(formatter, record);
                            }
                            // We don't have a previously set formatter
                            None => {
                                let mut formatter = Formatter::new(&self.writer);
                                print(&mut formatter, record);

                                *tl_buf = Some(formatter);
                            }
                        },
                        // There's already an active borrow of the buffer (due to re-entrancy)
                        Err(_) => {
                            print(&mut Formatter::new(&self.writer), record);
                        }
                    }
                })
                .is_ok();

            if !printed {
                // The thread-local storage was not available (because its
                // destructor has already run). Create a new single-use
                // Formatter on the stack for this call.
                print(&mut Formatter::new(&self.writer), record);
            }
        }
    }

    fn flush(&self) {}
}
