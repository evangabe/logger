mod buffer;
mod target;

pub(super) use buffer::Buffer;
use buffer::BufferWriter;
use std::{io, mem, sync::Mutex};
pub use target::Target;

#[derive(Debug)]
pub(crate) struct Writer {
    inner: BufferWriter,
}

impl Writer {
    pub fn write_style(&self) -> anstream::ColorChoice {
        self.inner.write_style()
    }

    pub(super) fn buffer(&self) -> Buffer {
        self.inner.buffer()
    }

    pub(super) fn print(&self, buf: &Buffer) -> io::Result<()> {
        self.inner.print(buf)
    }
}

#[derive(Debug)]
pub(crate) struct Builder {
    target: Target,
    built: bool,
    is_test: bool,
}

impl Builder {
    pub(crate) fn new() -> Self {
        Builder {
            target: Default::default(),
            is_test: false,
            built: false,
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn is_test(&mut self, is_test: bool) -> &mut Self {
        self.is_test = is_test;
        self
    }

    pub(crate) fn build(&mut self) -> Writer {
        assert!(!self.built, "attempt to re-use consumed builder");
        self.built = true;

        let color_choice = match &self.target {
            Target::Stdout => anstream::AutoStream::choice(&std::io::stdout()).into(),
            Target::Stderr => anstream::AutoStream::choice(&std::io::stderr()).into(),
            Target::Pipe(_) => anstream::ColorChoice::Never,
        };

        let writer = match mem::take(&mut self.target) {
            Target::Stdout => BufferWriter::stdout(self.is_test, color_choice),
            Target::Stderr => BufferWriter::stderr(self.is_test, color_choice),
            Target::Pipe(pipe) => BufferWriter::pipe(Box::new(Mutex::new(pipe)), color_choice),
        };

        Writer { inner: writer }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}
