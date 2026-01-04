use rustyline_async::{Readline, ReadlineError, ReadlineEvent, SharedWriter};
use std::io::{IsTerminal, Write};
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

/// Exit command.
const EXIT_COMMANDS: &[&str] = &["exit", "quit", "q"];

pub struct Cli {
    kill_rx: KillReceiver,
    printer: Printer,
    handles: Option<Handles>,
}

#[derive(Clone)]
pub struct KillReceiver(watch::Receiver<bool>);

pub enum CliEvent {
    /// Input(command, args)
    Input(String, Vec<String>),
    /// exit
    Exit,
}

#[derive(Clone)]
pub struct Printer(SharedWriter, bool);

struct Handles {
    kill_tx: watch::Sender<bool>,
    cli_handle: JoinHandle<()>,
}

pub type CliEventReceiver = mpsc::Receiver<CliEvent>;

#[allow(dead_code)]
impl Cli {
    pub fn new(
        prompt: impl ToString,
    ) -> Result<(Self, CliEventReceiver), Box<dyn std::error::Error>> {
        let (rl, stdout) = Readline::new(prompt.to_string())?;

        let printer = Printer(stdout, std::io::stdout().is_terminal());

        let (kill_tx, kill_rx) = watch::channel(false);
        let kill_rx = KillReceiver(kill_rx);

        let (tx, rx) = mpsc::channel(1);
        let cli_handle = tokio::spawn(cli_main_loop(rl, printer.clone(), kill_rx.clone(), tx));
        Ok((
            Self {
                kill_rx,
                printer,
                handles: Some(Handles {
                    kill_tx,
                    cli_handle,
                }),
            },
            rx,
        ))
    }

    pub async fn kill(&mut self) {
        if let Some(x) = self.handles.take() {
            let _ = x.kill_tx.send(true);
            let _ = x.cli_handle.await;
        }
    }

    pub fn get_kill_receiver(&self) -> KillReceiver {
        self.kill_rx.clone()
    }

    pub fn get_printer(&self) -> Printer {
        self.printer.clone()
    }
}

impl KillReceiver {
    #[inline]
    pub async fn recv(&mut self) {
        if !*self.0.borrow() {
            let _ = self.0.changed().await;
        }
    }
}

#[allow(dead_code)]
impl Printer {
    #[inline]
    pub async fn println(&mut self, msg: impl AsRef<str>) {
        let _ = writeln!(self.0, "{}", msg.as_ref());
    }
    #[inline]
    pub async fn print(&mut self, msg: impl AsRef<str>) {
        let _ = write!(self.0, "{}", msg.as_ref());
    }
    #[inline]
    pub async fn errln(&mut self, msg: impl AsRef<str>) {
        if self.1 {
            let _ = writeln!(self.0, "\x1b[31m{}\x1b[0m", msg.as_ref());
        } else {
            let _ = writeln!(self.0, "{}", msg.as_ref());
        }
    }
    #[inline]
    pub async fn err(&mut self, msg: impl AsRef<str>) {
        if self.1 {
            let _ = write!(self.0, "\x1b[31m{}\x1b[0m", msg.as_ref());
        } else {
            let _ = write!(self.0, "{}", msg.as_ref());
        }
    }
    #[inline]
    pub async fn warnln(&mut self, msg: impl AsRef<str>) {
        if self.1 {
            let _ = writeln!(self.0, "\x1b[33m{}\x1b[0m", msg.as_ref());
        } else {
            let _ = writeln!(self.0, "{}", msg.as_ref());
        }
    }
    #[inline]
    pub async fn warn(&mut self, msg: impl AsRef<str>) {
        if self.1 {
            let _ = write!(self.0, "\x1b[33m{}\x1b[0m", msg.as_ref());
        } else {
            let _ = write!(self.0, "{}", msg.as_ref());
        }
    }
}

async fn cli_main_loop(
    mut rx: Readline,
    mut printer: Printer,
    mut kill: KillReceiver,
    tx: mpsc::Sender<CliEvent>,
) {
    loop {
        tokio::select! {
          _ = kill.recv() => break,
          cmd = rx.readline() => if cli_main(cmd, &mut printer, &tx).await { break },
        }
    }
    let _ = rx.flush();
    let _ = tx.send(CliEvent::Exit).await;
}
type DoExit = bool;
const EXIT: DoExit = true;
const CONTINUE: DoExit = false;

async fn cli_main(
    cmd: std::result::Result<ReadlineEvent, ReadlineError>,
    p: &mut Printer,
    tx: &mpsc::Sender<CliEvent>,
) -> DoExit {
    let mut args = match cmd {
        Ok(ReadlineEvent::Line(ref l)) => match shell_words::split(l) {
            Ok(x) => x,
            Err(e) => {
                p.errln(format!("[Input Error] {e}")).await;
                return CONTINUE;
            }
        },
        Ok(ReadlineEvent::Eof) => return EXIT,
        Ok(ReadlineEvent::Interrupted) => return EXIT,
        Err(e) => {
            p.errln(format!("[Input Error] {e}")).await;
            return CONTINUE;
        }
    };

    if args.is_empty() {
        return CONTINUE;
    }
    let cmd = args.remove(0);

    // Exit commands
    let c = cmd.as_str();
    if EXIT_COMMANDS.contains(&c) {
        return EXIT;
    }

    // * Exit when send error occured
    tx.send(CliEvent::Input(cmd, args)).await.is_err()
}
