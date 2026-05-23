use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use std::io::{IsTerminal, Read, Seek, SeekFrom, Write, stderr};
use std::time::Instant;

pub(crate) fn byte_progress_bar(total: u64, message: &str) -> ProgressBar {
    if !stderr().is_terminal() {
        return ProgressBar::hidden();
    }

    let bar = ProgressBar::new(total);
    let style = ProgressStyle::with_template(
        "{msg:>10} [{bar:40.cyan/blue}] {bytes}/{total_bytes} {binary_bytes_per_sec} ({eta})",
    )
    .unwrap()
    .progress_chars("=> ");
    bar.set_style(style);
    bar.set_message(message.to_string());
    bar
}

pub(crate) fn pack_progress_bar(total: u64, message: &str) -> ProgressBar {
    if !stderr().is_terminal() {
        return ProgressBar::hidden();
    }

    let bar = ProgressBar::new(total);
    let style = ProgressStyle::with_template(
        "{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
    )
    .unwrap()
    .progress_chars("=> ");
    bar.set_style(style);
    bar.set_message(format!("{message} out:0 B 0 B/s"));
    bar
}

pub(crate) fn spinner(message: &str) -> ProgressBar {
    if !stderr().is_terminal() {
        return ProgressBar::hidden();
    }

    let spinner = ProgressBar::new_spinner();
    let style = ProgressStyle::with_template("{msg:>10} {spinner}").unwrap();
    spinner.set_style(style);
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner
}

pub(crate) struct ProgressReader<R> {
    inner: R,
    progress: ProgressBar,
}

impl<R> ProgressReader<R> {
    pub(crate) fn new(inner: R, progress: ProgressBar) -> Self {
        Self { inner, progress }
    }
}

impl<R: Read> Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buf)?;
        if read != 0 {
            self.progress.inc(read as u64);
        }
        Ok(read)
    }
}

pub(crate) struct ProgressWriter<W> {
    inner: W,
    progress: ProgressBar,
    label: &'static str,
    written: u64,
    started_at: Instant,
}

impl<W> ProgressWriter<W> {
    pub(crate) fn new(inner: W, progress: ProgressBar, label: &'static str) -> Self {
        Self {
            inner,
            progress,
            label,
            written: 0,
            started_at: Instant::now(),
        }
    }

    pub(crate) fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> Write for ProgressWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written = self.inner.write(buf)?;
        if written != 0 {
            self.written = self.written.saturating_add(written as u64);
            let elapsed = self.started_at.elapsed().as_secs_f64().max(0.001);
            let rate = (self.written as f64 / elapsed) as u64;
            self.progress.set_message(format!(
                "{} out:{} {}/s",
                self.label,
                HumanBytes(self.written),
                HumanBytes(rate),
            ));
        }
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<W: Seek> Seek for ProgressWriter<W> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}
