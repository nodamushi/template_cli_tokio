use cli::KillReceiver;

mod cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{IsTerminal, stdout};
    let prompt = if stdout().is_terminal() {
        "\x1b[33m>\x1b[0m "
    } else {
        "> "
    };

    let (mut cli, rx) = cli::Cli::new(prompt)?;

    // --------------  Example: Send/Recv TCP ----------------------------------------------
    let stream = tokio::net::TcpStream::connect("127.0.0.1:5555").await?;
    let (srx, stx) = stream.into_split();
    let recv_handle = tokio::spawn(recv_loop(cli.get_kill_receiver(), srx, cli.get_printer()));

    send_loop(rx, stx, cli.get_printer()).await;

    // ---- exit ---------
    cli.kill().await;
    let _ = recv_handle.await;

    Ok(())
}


async fn send_loop(
    mut rx: cli::CliEventReceiver,
    mut stream: tokio::net::tcp::OwnedWriteHalf,
    mut printer: cli::Printer,
) {
    use tokio::io::AsyncWriteExt;

    while let Some(x) = rx.recv().await {
        match x {
            cli::CliEvent::Input(cmd, _args) => {
                if cmd == "foo" {
                    match stream.write_all(b"foo\n").await {
                        Ok(_) => {
                            printer.println("[Send] foo").await;
                        }
                        Err(e) => {
                            printer.errln(format!("[Send Error] {e}")).await;
                            break;
                        }
                    }
                }
            }
            cli::CliEvent::Exit => break,
        }
    }
}

async fn recv_loop(
    mut kill: KillReceiver,
    stream: tokio::net::tcp::OwnedReadHalf,
    mut printer: cli::Printer,
) {
    use tokio::io::AsyncBufReadExt;
    let mut buf_reader = tokio::io::BufReader::new(stream);
    let mut line = String::new();
    loop {
        tokio::select! {
            _ = kill.recv() => break,
            x = buf_reader.read_line(&mut line) => match x {
                Ok(0) => break,
                Ok(l) =>
                    printer.println(format!("[Recv] size: {l}, {line}")).await,
                Err(e) => printer.errln(format!("[Recv Err] {e}")).await,
            }
        }
        line.clear();
    }
}
