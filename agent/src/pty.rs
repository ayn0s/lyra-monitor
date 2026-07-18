use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use shared::pb::terminal_input::Payload;
use shared::pb::TerminalOutput;
use std::io::{Read, Write};
use tokio::sync::mpsc;

pub struct PtySession {
    pub stdin_tx: mpsc::UnboundedSender<Vec<u8>>,
    pub resize_tx: mpsc::UnboundedSender<(u16, u16)>,
    pub stdout_rx: mpsc::UnboundedReceiver<TerminalOutput>,
}

impl PtySession {
    pub fn spawn(username: &str) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("opening PTY")?;

        let mut cmd = CommandBuilder::new("su");
        cmd.arg("-l");
        cmd.arg(username);
        pair.slave
            .spawn_command(cmd)
            .context("starting login shell in PTY")?;
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .context("cloning PTY reader")?;
        let mut writer = pair.master.take_writer().context("acquiring PTY writer")?;

        let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (resize_tx, mut resize_rx) = mpsc::unbounded_channel::<(u16, u16)>();
        let (stdout_tx, stdout_rx) = mpsc::unbounded_channel::<TerminalOutput>();

        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = TerminalOutput {
                            stdout: buf[..n].to_vec(),
                        };
                        if stdout_tx.send(chunk).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        tokio::spawn(async move {
            let master = pair.master;
            loop {
                tokio::select! {
                    Some(bytes) = stdin_rx.recv() => {
                        if writer.write_all(&bytes).is_err() {
                            break;
                        }
                    }
                    Some((cols, rows)) = resize_rx.recv() => {
                        let _ = master.resize(PtySize {
                            rows,
                            cols,
                            pixel_width: 0,
                            pixel_height: 0,
                        });
                    }
                    else => break,
                }
            }
        });

        Ok(Self {
            stdin_tx,
            resize_tx,
            stdout_rx,
        })
    }

    #[allow(dead_code)]
    pub fn handle_input(&self, payload: Payload) {
        match payload {
            Payload::Stdin(bytes) => {
                let _ = self.stdin_tx.send(bytes);
            }
            Payload::Resize(resize) => {
                let _ = self
                    .resize_tx
                    .send((resize.cols as u16, resize.rows as u16));
            }
            Payload::Auth(_) => {}
        }
    }
}
