use std::{io, sync::Mutex};

#[derive(Debug)]
pub(in crate::fmt::writer) struct BufferWriter {
    target: WritableTarget,
    write_style: anstream::ColorChoice,
}

impl BufferWriter {
    pub(in crate::fmt::writer) fn stderr(
        is_test: bool,
        write_style: anstream::ColorChoice,
    ) -> Self {
        BufferWriter {
            target: if is_test {
                WritableTarget::PrintStderr
            } else {
                WritableTarget::WriteStderr
            },
            write_style,
        }
    }

    pub(in crate::fmt::writer) fn stdout(
        is_test: bool,
        write_style: anstream::ColorChoice,
    ) -> Self {
        BufferWriter {
            target: if is_test {
                WritableTarget::PrintStdout
            } else {
                WritableTarget::WriteStdout
            },
            write_style,
        }
    }

    pub(in crate::fmt::writer) fn pipe(
        pipe: Box<Mutex<dyn io::Write + Send + 'static>>,
        write_style: anstream::ColorChoice,
    ) -> Self {
        BufferWriter {
            target: WritableTarget::Pipe(pipe),
            write_style,
        }
    }

    pub(in crate::fmt::writer) fn write_style(&self) -> anstream::ColorChoice {
        self.write_style
    }

    pub(in crate::fmt::writer) fn buffer(&self) -> Buffer {
        Buffer(Vec::new())
    }

    pub(in crate::fmt::writer) fn print(&self, buf: &Buffer) -> io::Result<()> {
        use std::io::Write as _;

        let buf = buf.as_bytes();
        match &self.target {
            WritableTarget::WriteStdout => {
                let stream = std::io::stdout();
                let stream = anstream::AutoStream::new(stream, self.write_style.into());
                let mut stream = stream.lock();
                stream.write_all(buf)?;
                stream.flush()?;
            }
            WritableTarget::PrintStdout => {
                let buf = String::from_utf8_lossy(buf);
                print!("{}", buf);
            }
            WritableTarget::WriteStderr => {
                let stream = std::io::stderr();
                let stream = anstream::AutoStream::new(stream, self.write_style.into());
                let mut stream = stream.lock();
                stream.write_all(buf)?;
                stream.flush()?;
            }
            WritableTarget::PrintStderr => {
                let buf = String::from_utf8_lossy(buf);
                eprint!("{}", buf);
            }
            WritableTarget::Pipe(pipe) => {
                let mut stream = pipe.lock().unwrap();
                stream.write_all(buf)?;
                stream.flush()?;
            }
        }

        Ok(())
    }
}

pub(in crate::fmt) struct Buffer(Vec<u8>);
impl Buffer {
    pub(in crate::fmt) fn clear(&mut self) {
        self.0.clear();
    }
    pub(in crate::fmt) fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.extend(buf);
        Ok(buf.len())
    }
    pub(in crate::fmt) fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
    pub(in crate::fmt) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

pub(super) enum WritableTarget {
    /// Logs will be written to standard output.
    WriteStdout,
    /// Logs will be printed to standard output.
    PrintStdout,
    /// Logs will be written to standard error.
    WriteStderr,
    /// Logs will be printed to standard error.
    PrintStderr,
    /// Logs will be sent to a custom pipe.
    Pipe(Box<std::sync::Mutex<dyn std::io::Write + Send + 'static>>),
}

impl std::fmt::Debug for WritableTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::WriteStdout => "stdout",
                Self::PrintStdout => "stdout",
                Self::WriteStderr => "stderr",
                Self::PrintStderr => "stderr",
                Self::Pipe(_) => "pipe",
            }
        )
    }
}
