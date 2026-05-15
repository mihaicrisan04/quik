mod backends;
mod cli;
mod render;

use anyhow::{Context, Result};
use clap::Parser;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::io::{BufRead, BufReader, Write};
use std::process::Stdio;

use backends::Event;
use cli::{Cli, Cmd};
use render::Renderer;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Stream(args) => stream(args),
    }
}

fn stream(args: cli::StreamArgs) -> Result<()> {
    let mut backend = backends::resolve(args.backend)?;
    let mut cmd = backend.build_command(&args);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let mut child = cmd.spawn().with_context(|| {
        format!(
            "failed to spawn `{}` — is it installed and on PATH?",
            backend.name()
        )
    })?;

    let stdout = child.stdout.take().context("backend has no stdout pipe")?;
    let reader = BufReader::new(stdout);

    let mut renderer = Renderer::new(args.width);
    let mut got_text = false;
    let mut stopped = false;

    for line_res in reader.lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.is_empty() {
            continue;
        }
        for event in backend.parse_line(&line) {
            match event {
                Event::TextDelta(text) => {
                    if !got_text {
                        kill_shimmer(args.shimmer_pid);
                        write_divider();
                        got_text = true;
                    }
                    for c in text.chars() {
                        renderer.feed(c);
                    }
                }
                Event::MessageStop => {
                    renderer.finish();
                    stopped = true;
                    break;
                }
            }
        }
        if stopped {
            break;
        }
    }

    if !stopped {
        renderer.finish();
    }

    let _ = child.kill();
    let _ = child.wait();

    // Make sure the shimmer is dead even on a no-token failure path so the
    // caller's prompt doesn't keep animating.
    if !got_text {
        kill_shimmer(args.shimmer_pid);
        std::process::exit(1);
    }

    Ok(())
}

fn kill_shimmer(pid: Option<i32>) {
    if let Some(p) = pid {
        if p > 0 {
            let _ = kill(Pid::from_raw(p), Signal::SIGTERM);
        }
    }
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(b"\r\x1b[2K\x1b[?25h");
    let _ = stdout.flush();
}

fn write_divider() {
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(b"\x1b[38;5;240m\xe2\x86\xb3\x1b[0m\n");
    let _ = stdout.flush();
}
