use std::{
    env, io,
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use stuk_platform::{PlatformEvent, SingleInstanceActivation, SingleInstancePolicy};

type EventQueue = Arc<Mutex<Vec<PlatformEvent>>>;

pub(super) enum SingleInstanceSetup {
    Primary(SingleInstanceGuard),
    AlreadyRunning,
    Failed,
}

pub(super) struct SingleInstanceGuard {
    port: u16,
    running: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

impl SingleInstanceGuard {
    pub(super) fn acquire(policy: SingleInstancePolicy, events: EventQueue) -> SingleInstanceSetup {
        let Ok(port) = single_instance_port() else {
            return SingleInstanceSetup::Failed;
        };
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(listener) => {
                let _ = listener.set_nonblocking(true);
                let running = Arc::new(AtomicBool::new(true));
                let thread_running = Arc::clone(&running);
                let thread = thread::spawn(move || {
                    while thread_running.load(Ordering::Relaxed) {
                        match listener.accept() {
                            Ok((stream, _)) => {
                                if let Some(activation) =
                                    read_single_instance_activation(policy, stream)
                                    && let Ok(mut events) = events.lock()
                                {
                                    events.push(PlatformEvent::SingleInstance(activation));
                                }
                            }
                            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                                thread::sleep(Duration::from_millis(50));
                            }
                            Err(_) => break,
                        }
                    }
                });
                SingleInstanceSetup::Primary(Self {
                    port,
                    running,
                    thread,
                })
            }
            Err(_) if send_single_instance_activation(port, policy).is_ok() => {
                SingleInstanceSetup::AlreadyRunning
            }
            Err(_) => SingleInstanceSetup::Failed,
        }
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = TcpStream::connect(("127.0.0.1", self.port));
        let _ = self.thread.thread().id();
    }
}

fn read_single_instance_activation(
    policy: SingleInstancePolicy,
    mut stream: TcpStream,
) -> Option<SingleInstanceActivation> {
    let mut body = String::new();
    stream.read_to_string(&mut body).ok()?;
    let value: serde_json::Value = serde_json::from_str(&body).ok()?;
    if value.get("magic").and_then(|value| value.as_str()) != Some("stuk-single-instance") {
        return None;
    }
    let arguments = value
        .get("arguments")
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut activation = SingleInstanceActivation::new(policy, arguments);
    if let Some(cwd) = value.get("cwd").and_then(|value| value.as_str()) {
        activation = activation.working_directory(cwd);
    }
    if let Some(token) = value
        .get("activationToken")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        activation = activation.activation_token(token);
    }
    let _ = stream.write_all(b"stuk-ok");
    Some(activation)
}

fn send_single_instance_activation(port: u16, policy: SingleInstancePolicy) -> io::Result<()> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(Some(Duration::from_millis(250)))?;
    let cwd = env::current_dir()
        .ok()
        .map(|path| path.display().to_string());
    let body = serde_json::json!({
        "magic": "stuk-single-instance",
        "policy": format!("{policy:?}"),
        "arguments": env::args().collect::<Vec<_>>(),
        "cwd": cwd,
        "activationToken": startup_activation_token(),
    });
    stream.write_all(body.to_string().as_bytes())?;
    stream.shutdown(Shutdown::Write)?;
    let mut ack = [0; 7];
    stream.read_exact(&mut ack)?;
    if ack == *b"stuk-ok" {
        Ok(())
    } else {
        Err(io::Error::other(
            "single-instance listener did not acknowledge activation",
        ))
    }
}

fn startup_activation_token() -> Option<String> {
    env::var("XDG_ACTIVATION_TOKEN")
        .or_else(|_| env::var("DESKTOP_STARTUP_ID"))
        .ok()
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
}

fn single_instance_port() -> io::Result<u16> {
    let exe = env::current_exe()?;
    let hash = stable_hash(&exe.display().to_string());
    Ok(49152 + (hash % 15000) as u16)
}

fn stable_hash(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}
