use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::io::{Read, Write};

pub struct Pty {
    pair: PtyPair,
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
}

impl Pty {
    pub fn new(cols: u16, rows: u16) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system.openpty(size)?;

        // Spawn shell
        let shell = Self::get_shell();
        let mut cmd = CommandBuilder::new(&shell.0);
        for arg in &shell.1 {
            cmd.arg(arg);
        }

        pair.slave.spawn_command(cmd)?;

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        Ok(Self {
            pair,
            reader,
            writer,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn read(&mut self) -> anyhow::Result<Option<Vec<u8>>> {
        let mut buf = vec![0u8; 8192];

        // Try non-blocking read
        match self.reader.read(&mut buf) {
            Ok(0) => Ok(None),
            Ok(n) => {
                buf.truncate(n);
                Ok(Some(buf))
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> anyhow::Result<()> {
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        self.pair.master.resize(size)?;
        Ok(())
    }

    fn get_shell() -> (String, Vec<String>) {
        // Try to get shell from environment
        if let Ok(shell) = std::env::var("SHELL") {
            return (shell, vec![]);
        }

        // Default to common shells
        #[cfg(unix)]
        {
            for shell in &["/bin/bash", "/bin/zsh", "/bin/sh"] {
                if std::path::Path::new(shell).exists() {
                    return (shell.to_string(), vec![]);
                }
            }
            return ("/bin/sh".to_string(), vec![]);
        }

        #[cfg(windows)]
        {
            ("powershell.exe".to_string(), vec![])
        }

        #[cfg(not(any(unix, windows)))]
        {
            ("sh".to_string(), vec![])
        }
    }
}
