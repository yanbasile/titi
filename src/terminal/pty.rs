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
            // Security: Validate the shell path before using it
            if Self::is_valid_shell(&shell) {
                return (shell, vec![]);
            }
            // If invalid, fall through to defaults
            log::warn!("Shell from $SHELL '{}' is not valid, using default", shell);
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

    /// Validate a shell path for security
    ///
    /// Checks that the shell:
    /// 1. Exists as a file
    /// 2. Is an absolute path
    /// 3. Matches known safe shell patterns
    fn is_valid_shell(shell: &str) -> bool {
        use std::path::Path;

        let path = Path::new(shell);

        // Must be an absolute path
        if !path.is_absolute() {
            return false;
        }

        // Must exist as a file
        if !path.exists() || !path.is_file() {
            return false;
        }

        // On Unix, validate against known safe shells
        #[cfg(unix)]
        {
            // Check if shell is in /etc/shells (if the file exists)
            if let Ok(shells_content) = std::fs::read_to_string("/etc/shells") {
                if shells_content.lines().any(|line| {
                    let line = line.trim();
                    !line.is_empty() && !line.starts_with('#') && line == shell
                }) {
                    return true;
                }
            }

            // Fallback: Check against common known shells
            let known_shells = [
                "/bin/sh", "/bin/bash", "/bin/zsh", "/bin/dash", "/bin/ksh",
                "/bin/fish", "/bin/tcsh", "/bin/csh",
                "/usr/bin/sh", "/usr/bin/bash", "/usr/bin/zsh", "/usr/bin/dash",
                "/usr/bin/ksh", "/usr/bin/fish", "/usr/bin/tcsh", "/usr/bin/csh",
            ];

            if known_shells.contains(&shell) {
                return true;
            }

            // Additional validation: Check if the shell name looks reasonable
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let valid_shell_names = ["sh", "bash", "zsh", "dash", "ksh", "fish", "tcsh", "csh"];
                if valid_shell_names.contains(&name) {
                    return true;
                }
            }

            false
        }

        #[cfg(windows)]
        {
            // On Windows, accept common shells
            let lower_shell = shell.to_lowercase();
            lower_shell.contains("powershell")
                || lower_shell.contains("cmd.exe")
                || lower_shell.contains("pwsh")
        }

        #[cfg(not(any(unix, windows)))]
        {
            // On other platforms, just check existence (already done above)
            true
        }
    }
}
