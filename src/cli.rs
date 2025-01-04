use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Command {
    Pause,
    Resume,
    Cancel,
    ShowProgress,
    SetRateLimit(u64),
}

#[derive(Debug, Clone)]
pub enum DownloadStatus {
    Running,
    Paused,
    Completed,
    Failed(String),
}

pub struct InteractiveMode {
    command_tx: mpsc::Sender<Command>,
    status_rx: mpsc::Receiver<DownloadStatus>,
}

impl InteractiveMode {
    pub fn new() -> (Self, mpsc::Sender<DownloadStatus>, mpsc::Receiver<Command>) {
        let (command_tx, command_rx) = mpsc::channel(32);
        let (status_tx, status_rx) = mpsc::channel(32);
        
        (
            Self {
                command_tx,
                status_rx,
            },
            status_tx,
            command_rx,
        )
    }

    pub async fn run(&mut self) {
        use tokio::io::AsyncBufReadExt;
        let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
        let mut line = String::new();

        loop {
            tokio::select! {
                result = stdin.read_line(&mut line) => {
                    if let Ok(n) = result {
                        if n == 0 {
                            break;
                        }
                        self.handle_command(&line.trim()).await;
                        line.clear();
                    }
                }
                Some(status) = self.status_rx.recv() => {
                    self.handle_status(status).await;
                }
            }
        }
    }

    async fn handle_command(&self, cmd: &str) {
        let command = match cmd.to_lowercase().as_str() {
            "pause" => Some(Command::Pause),
            "resume" => Some(Command::Resume),
            "cancel" => Some(Command::Cancel),
            "progress" => Some(Command::ShowProgress),
            _ if cmd.starts_with("limit ") => {
                cmd.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .map(Command::SetRateLimit)
            }
            _ => {
                println!("Unknown command: {}", cmd);
                None
            }
        };

        if let Some(cmd) = command {
            if let Err(e) = self.command_tx.send(cmd).await {
                println!("Failed to send command: {}", e);
            }
        }
    }

    async fn handle_status(&self, status: DownloadStatus) {
        match status {
            DownloadStatus::Running => println!("Download is running"),
            DownloadStatus::Paused => println!("Download is paused"),
            DownloadStatus::Completed => println!("Download completed"),
            DownloadStatus::Failed(err) => println!("Download failed: {}", err),
        }
    }
} 