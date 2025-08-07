use crossterm::event;
#[cfg(not(feature = "no-tty"))]
use crossterm::execute;
#[cfg(feature = "no-tty")]
use crossterm::{async_execute, event::SenderWriter};

/// Helper managing proper setup and teardown of bracketed paste mode
///
/// <https://en.wikipedia.org/wiki/Bracketed-paste>
#[derive(Default)]
pub(crate) struct BracketedPasteGuard {
    enabled: bool,
    active: bool,
    #[cfg(feature = "no-tty")]
    pub writer: Option<SenderWriter>,
}

impl BracketedPasteGuard {
    pub fn set(&mut self, enable: bool) {
        self.enabled = enable;
    }
    #[cfg(feature = "no-tty")]
    pub fn with_writer(mut self, writer: SenderWriter) -> Self {
        self.writer = Some(writer);
        self
    }
    #[maybe_async_cfg::maybe(
        sync(cfg(not(feature = "no-tty")), keep_self),
        async(cfg(feature = "no-tty"), keep_self)
    )]
    pub async fn enter(&mut self) {
        if self.enabled && !self.active {
            #[cfg(not(feature = "no-tty"))]
            let _ = execute!(std::io::stdout(), event::EnableBracketedPaste);
            #[cfg(feature = "no-tty")]
            if let Some(writer) = &self.writer {
                let _ = async_execute!(writer, event::EnableBracketedPaste).await;
            }
            self.active = true;
        }
    }
    #[maybe_async_cfg::maybe(
        sync(cfg(not(feature = "no-tty")), keep_self),
        async(cfg(feature = "no-tty"), keep_self)
    )]
    pub async fn exit(&mut self) {
        if self.active {
            #[cfg(not(feature = "no-tty"))]
            let _ = execute!(std::io::stdout(), event::DisableBracketedPaste);
            #[cfg(feature = "no-tty")]
            if let Some(writer) = &self.writer {
                let _ = async_execute!(writer, event::DisableBracketedPaste).await;
            }
            self.active = false;
        }
    }
}

impl Drop for BracketedPasteGuard {
    fn drop(&mut self) {
        if self.active {
            #[cfg(not(feature = "no-tty"))]
            let _ = execute!(std::io::stdout(), event::DisableBracketedPaste);
            #[cfg(feature = "no-tty")]
            if let Some(writer) = &self.writer {
                async_drop(writer.clone());
            }
        }
    }
}

#[cfg(feature = "no-tty")]
fn async_drop(writer: SenderWriter) {
    std::thread::spawn(move || {
        let mut buf = ::std::string::String::new();
        let mut ok = true;
        if crossterm::Command::write_ansi(&event::DisableBracketedPaste, &mut buf).is_err() {
            ok = false;
        }
        if ok {
            let _ = writer.blocking_write_all(buf.as_bytes());
        }
    });
}
