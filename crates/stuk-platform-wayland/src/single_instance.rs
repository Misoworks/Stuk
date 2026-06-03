use std::{
    env, fs, io,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    thread::{self, JoinHandle},
};

use stuk_platform::{PlatformEvent, SingleInstanceActivation, SingleInstancePolicy};

use crate::desktop_files::sanitize_desktop_id;

type EventQueue = std::sync::Arc<std::sync::Mutex<Vec<PlatformEvent>>>;

pub(super) enum SingleInstanceSetup {
    Primary(SingleInstanceGuard),
    AlreadyRunning,
    Failed,
}

pub(super) struct SingleInstanceGuard {
    socket_path: PathBuf,
    thread: JoinHandle<()>,
}

impl SingleInstanceGuard {
    pub(super) fn acquire(policy: SingleInstancePolicy, events: EventQueue) -> SingleInstanceSetup {
        let Ok(socket_path) = single_instance_socket_path() else {
            return SingleInstanceSetup::Failed;
        };
        if let Some(parent) = socket_path.parent()
            && fs::create_dir_all(parent).is_err()
        {
            return SingleInstanceSetup::Failed;
        }
        match bind_listener(&socket_path, policy, events.clone()) {
            Some(guard) => SingleInstanceSetup::Primary(guard),
            None => {
                if send_single_instance_activation(&socket_path, policy).is_ok() {
                    SingleInstanceSetup::AlreadyRunning
                } else if fs::remove_file(&socket_path).is_ok() {
                    bind_listener(&socket_path, policy, events)
                        .map(SingleInstanceSetup::Primary)
                        .unwrap_or(SingleInstanceSetup::Failed)
                } else {
                    SingleInstanceSetup::Failed
                }
            }
        }
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        let _ = self.thread.thread().id();
        let _ = fs::remove_file(&self.socket_path);
    }
}

fn bind_listener(
    socket_path: &PathBuf,
    policy: SingleInstancePolicy,
    events: EventQueue,
) -> Option<SingleInstanceGuard> {
    let listener = UnixListener::bind(socket_path).ok()?;
    let socket_path = socket_path.clone();
    let thread = thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            if let Some(activation) = read_single_instance_activation(policy, stream)
                && let Ok(mut events) = events.lock()
            {
                events.push(PlatformEvent::SingleInstance(activation));
            }
        }
    });
    Some(SingleInstanceGuard {
        socket_path,
        thread,
    })
}

fn read_single_instance_activation(
    policy: SingleInstancePolicy,
    mut stream: UnixStream,
) -> Option<SingleInstanceActivation> {
    let mut body = String::new();
    stream.read_to_string(&mut body).ok()?;
    let value: serde_json::Value = serde_json::from_str(&body).ok()?;
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
    Some(activation)
}

fn send_single_instance_activation(
    socket_path: &PathBuf,
    policy: SingleInstancePolicy,
) -> io::Result<()> {
    let mut stream = UnixStream::connect(socket_path)?;
    let cwd = env::current_dir()
        .ok()
        .map(|path| path.display().to_string());
    let body = serde_json::json!({
        "policy": format!("{policy:?}"),
        "arguments": env::args().collect::<Vec<_>>(),
        "cwd": cwd,
    });
    stream.write_all(body.to_string().as_bytes())
}

fn single_instance_socket_path() -> io::Result<PathBuf> {
    let runtime = env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir());
    let exe = env::current_exe()?;
    let id = exe
        .file_stem()
        .and_then(|name| name.to_str())
        .map(sanitize_desktop_id)
        .unwrap_or_else(|| "app".to_string());
    Ok(runtime.join("stuk").join(format!("{id}.sock")))
}
