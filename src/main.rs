use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::Clear,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use directories::ProjectDirs;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use llama_cpp_2::token::LlamaToken;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};
use termimad::MadSkin;
use walkdir::WalkDir;

const MODEL_URL: &str = "https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/resolve/main/Llama-3.2-1B-Instruct-Q4_K_M.gguf";
const MODEL_FILENAME: &str = "Llama-3.2-1B-Instruct-Q4_K_M.gguf";

#[derive(Parser, Debug)]
#[command(
    name = "rax",
    author = "RaxCore",
    version,
    about = "🚀 Rax CLI - Next-Gen Offline AI",
    long_about = "Fast, secure, and context-aware offline AI assistant powered by Llama 3.2"
)]
struct Args {
    /// Message to send to Rax
    #[arg(index = 1)]
    message: Option<String>,

    /// Interactive chat mode with TUI
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,

    /// Add context from current directory
    #[arg(short = 'c', long = "context")]
    context: bool,

    /// System prompt
    #[arg(
        short = 's',
        long = "system",
        default_value = "You are Rax, a highly intelligent, concise, and proactive AI developer assistant. You provide expert guidance on software architecture, performance, and best practices. Always give high-signal, direct answers with beautifully formatted markdown."
    )]
    system: String,

    /// Force re-download the model
    #[arg(long = "update-model")]
    update_model: bool,

    /// Show installation status
    #[arg(long = "status")]
    status: bool,

    /// Remove downloaded model
    #[arg(long = "uninstall")]
    uninstall: bool,

    /// List saved conversations
    #[arg(long = "list-chats")]
    list_chats: bool,

    /// Resume a conversation by ID
    #[arg(long = "resume")]
    resume: Option<usize>,

    /// Delete a conversation by ID
    #[arg(long = "delete-chat")]
    delete_chat: Option<usize>,

    /// Export chat history to file
    #[arg(long = "export")]
    export: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum InstallationStatus {
    NotInstalled,
    Installed,
    Corrupted,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChatMessage {
    role: String,
    content: String,
    timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatSession {
    id: usize,
    name: String,
    created_at: u64,
    updated_at: u64,
    messages: Vec<ChatMessage>,
    system_prompt: String,
}

#[derive(Serialize, Deserialize, Default)]
struct ChatStorage {
    sessions: HashMap<usize, ChatSession>,
    next_id: usize,
}

impl ChatStorage {
    fn load() -> Self {
        let chat_path = get_chat_history_path();
        if chat_path.exists() {
            if let Ok(content) = fs::read_to_string(&chat_path) {
                if let Ok(storage) = serde_json::from_str(&content) {
                    return storage;
                }
            }
        }
        ChatStorage::default()
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let chat_path = get_chat_history_path();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&chat_path, content)?;
        Ok(())
    }

    fn create_session(&mut self, system_prompt: String) -> &mut ChatSession {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = self.next_id;
        self.next_id += 1;

        let session = ChatSession {
            id,
            name: format!("Chat #{}", id),
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
            system_prompt,
        };

        self.sessions.insert(id, session);
        self.sessions.get_mut(&id).unwrap()
    }

    fn get_session(&mut self, id: usize) -> Option<&mut ChatSession> {
        self.sessions.get_mut(&id)
    }

    fn delete_session(&mut self, id: usize) -> bool {
        self.sessions.remove(&id).is_some()
    }

    fn list_sessions(&self) -> Vec<&ChatSession> {
        let mut sessions: Vec<&ChatSession> = self.sessions.values().collect();
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        sessions
    }
}

fn get_project_dirs() -> ProjectDirs {
    ProjectDirs::from("com", "RaxCore", "RaxCli").expect("Failed to get project directories")
}

fn get_model_path() -> PathBuf {
    let dirs = get_project_dirs();
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir).unwrap();
    data_dir.join(MODEL_FILENAME)
}

fn get_config_path() -> PathBuf {
    let dirs = get_project_dirs();
    let config_dir = dirs.config_dir();
    fs::create_dir_all(config_dir).unwrap();
    config_dir.join("config.json")
}

fn get_chat_history_path() -> PathBuf {
    let dirs = get_project_dirs();
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir).unwrap();
    data_dir.join("chat_history.json")
}

fn get_context_cache_path() -> PathBuf {
    let dirs = get_project_dirs();
    let cache_dir = dirs.cache_dir();
    fs::create_dir_all(cache_dir).unwrap();
    cache_dir.join("context_cache.json")
}

#[derive(Serialize, Deserialize, Default)]
struct ContextCache {
    file_hashes: HashMap<String, (String, u64)>, // path -> (hash, timestamp)
    cached_content: HashMap<String, String>,
}

impl ContextCache {
    fn load() -> Self {
        let cache_path = get_context_cache_path();
        if cache_path.exists() {
            if let Ok(content) = fs::read_to_string(&cache_path) {
                if let Ok(cache) = serde_json::from_str(&content) {
                    return cache;
                }
            }
        }
        ContextCache::default()
    }

    fn save(&self) {
        let cache_path = get_context_cache_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&cache_path, content);
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Config {
    model_downloaded: bool,
    last_updated: Option<String>,
}

impl Config {
    fn load() -> Self {
        let config_path = get_config_path();
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        Config::default()
    }

    fn save(&self) {
        let config_path = get_config_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&config_path, content);
        }
    }
}

fn check_installation_status(model_path: &PathBuf) -> InstallationStatus {
    if !model_path.exists() {
        return InstallationStatus::NotInstalled;
    }

    let metadata = match fs::metadata(model_path) {
        Ok(m) => m,
        Err(_) => return InstallationStatus::Corrupted,
    };

    const MIN_SIZE: u64 = 500 * 1024 * 1024;
    if metadata.len() < MIN_SIZE {
        return InstallationStatus::Corrupted;
    }

    if fs::File::open(model_path).is_err() {
        return InstallationStatus::Corrupted;
    }

    InstallationStatus::Installed
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn format_timestamp(secs: u64) -> String {
    use chrono::{DateTime, Local};
    let dt = DateTime::from_timestamp(secs as i64, 0)
        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
        .with_timezone(&Local);
    dt.format("%Y-%m-%d %H:%M").to_string()
}

fn download_model_with_progress(model_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!();

    let m = MultiProgress::new();

    let spin_pb = m.add(ProgressBar::new_spinner());
    spin_pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spin_pb.set_message("Connecting to model server...");

    let start = Instant::now();

    // Use 40 minute timeout for large files
    let response = match ureq::get(MODEL_URL)
        .timeout(Duration::from_secs(2400))
        .call()
    {
        Ok(r) => r,
        Err(e) => {
            spin_pb.finish_with_message(format!("❌ Connection failed: {}", e));
            return Err(format!("Failed to connect: {}", e).into());
        }
    };

    let total_size = response
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    spin_pb.set_message(format!(
        "Starting download ({})...",
        format_bytes(total_size)
    ));

    let pb = m.add(ProgressBar::new(total_size));
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
        )
        .unwrap()
        .progress_chars("█▓▒░ ")
    );

    let mut reader = response.into_reader();
    let mut dest = match fs::File::create(model_path) {
        Ok(f) => f,
        Err(e) => {
            spin_pb.finish_with_message(format!("❌ Failed to create file: {}", e));
            return Err(format!("File creation failed: {}", e).into());
        }
    };

    let mut buffer = vec![0u8; 65536];
    let mut downloaded: u64 = 0;
    let mut last_progress = Instant::now();
    let mut consecutive_errors = 0;

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break, // EOF - download complete
            Ok(n) => {
                consecutive_errors = 0; // Reset error counter on successful read

                if let Err(e) = dest.write_all(&buffer[..n]) {
                    pb.finish_with_message(format!("❌ Write error: {}", e));
                    return Err(format!("Write failed: {}", e).into());
                }

                downloaded += n as u64;
                pb.set_position(downloaded);

                // Throttle UI updates for performance
                if last_progress.elapsed() > Duration::from_millis(100) {
                    last_progress = Instant::now();
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                consecutive_errors += 1;
                if consecutive_errors >= 3 {
                    pb.finish_with_message(format!("❌ Download timed out after 3 retries"));
                    return Err(format!("Download timed out").into());
                }
                // Continue reading on timeout - might be transient
                continue;
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Download completed but stream ended unexpectedly
                // Check if we got all the data
                if downloaded >= total_size {
                    break; // We have all the data, consider it success
                }
                consecutive_errors += 1;
                if consecutive_errors >= 3 {
                    pb.finish_with_message(format!("❌ Download incomplete: {}", e));
                    return Err(format!("Download incomplete: {}", e).into());
                }
                continue;
            }
            Err(e) => {
                consecutive_errors += 1;
                if consecutive_errors >= 3 {
                    pb.finish_with_message(format!("❌ Download error: {}", e));
                    return Err(format!("Download failed: {}", e).into());
                }
                continue;
            }
        }
    }

    // Verify download completed successfully
    if total_size > 0 && downloaded < total_size {
        let missing = total_size - downloaded;
        pb.finish_with_message(format!(
            "⚠️  Download incomplete (missing {})",
            format_bytes(missing)
        ));
        return Err(format!("Download incomplete: expected {}, got {}", total_size, downloaded).into());
    }

    let elapsed = start.elapsed();

    pb.finish_and_clear();
    spin_pb.finish_with_message(format!(
        "✅ Model downloaded successfully in {:.2}s ({})",
        elapsed.as_secs_f64(),
        format_bytes(total_size)
    ));

    let mut config = Config::load();
    config.model_downloaded = true;
    config.last_updated = Some(chrono::Utc::now().to_rfc3339());
    config.save();

    Ok(())
}

fn show_installation_wizard() -> Result<bool, Box<dyn std::error::Error>> {
    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           🚀 Welcome to Rax CLI!                        ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("  Rax is your offline AI assistant powered by Llama 3.2");
    println!();
    println!("  📦 Model size: ~650 MB (one-time download)");
    println!("  🧠 Runs entirely on your CPU");
    println!("  🔒 No data leaves your machine");
    println!("  💾 Chat history saved locally");
    println!();
    println!("  The model will be downloaded now.");
    println!("  This may take a few minutes depending on your connection.");
    println!();

    print!("  Start download? [Y/n] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let confirm = input.trim().to_lowercase();
    if confirm.is_empty() || confirm == "y" || confirm == "yes" {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn compute_file_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn load_context_with_cache() -> String {
    let mut context_str = String::from("\n### Project Context:\n");
    let mut cache = ContextCache::load();
    let mut count = 0;
    let mut modified_files = Vec::new();

    for entry in WalkDir::new(".")
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let relevant_exts = [
                "rs", "py", "js", "ts", "md", "txt", "toml", "json", "c", "cpp", "h", "go", "sh",
                "yaml", "yml", "html", "css",
            ];

            if relevant_exts.contains(&extension)
                && !path.to_str().unwrap().contains("target/")
                && !path.to_str().unwrap().contains("node_modules/")
                && !path.to_str().unwrap().contains(".git/")
                && !path.to_str().unwrap().contains("dist/")
                && !path.to_str().unwrap().contains("build/")
            {
                let path_str = path.to_string_lossy().to_string();

                if let Ok(content) = fs::read_to_string(path) {
                    let current_hash = compute_file_hash(&content);
                    let needs_update = cache
                        .cached_content
                        .get(&path_str)
                        .map(|cached| compute_file_hash(cached) != current_hash)
                        .unwrap_or(true);

                    if needs_update {
                        cache
                            .cached_content
                            .insert(path_str.clone(), content.clone());
                        cache.file_hashes.insert(
                            path_str.clone(),
                            (
                                current_hash,
                                SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            ),
                        );
                        modified_files.push(path_str.clone());
                    }

                    let cached_content = cache.cached_content.get(&path_str).unwrap_or(&content);
                    let truncated = if cached_content.len() > 4000 {
                        format!(
                            "{}...(truncated)",
                            &cached_content.chars().take(4000).collect::<String>()
                        )
                    } else {
                        cached_content.clone()
                    };

                    context_str.push_str(&format!(
                        "\n### {}\n```\n{}\n```\n",
                        path.display(),
                        truncated
                    ));
                    count += 1;
                    if count >= 20 {
                        break;
                    }
                }
            }
        }
    }

    cache.save();

    if !modified_files.is_empty() {
        println!("  📝 Context cached ({} files)", modified_files.len());
    }

    context_str
}

fn print_help() -> String {
    let mut help = String::new();
    help.push_str("\nCommands:\n");
    help.push_str("  /help     Show this help\n");
    help.push_str("  /clear    Clear chat history\n");
    help.push_str("  /list     List saved conversations\n");
    help.push_str("  /save     Save current chat\n");
    help.push_str("  /quit     Exit Rax\n");
    help.push_str("\nTips:\n");
    help.push_str("  - Just type your message and press Enter\n");
    help.push_str("  - Use -c flag to include project context\n");
    help.push_str("  - Conversations auto-save\n");
    help
}

fn list_chats(storage: &ChatStorage) {
    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║              Saved Conversations                         ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    let sessions = storage.list_sessions();
    if sessions.is_empty() {
        println!("  No saved conversations yet.");
        println!("  Use /save <name> to save the current conversation.");
    } else {
        println!("  {:<6} {:<30} {:<20} Messages", "ID", "Name", "Updated");
        println!("  {}", "─".repeat(70));
        for session in sessions {
            let name = if session.name.len() > 28 {
                format!("{}...", &session.name[..28])
            } else {
                session.name.clone()
            };
            println!(
                "  {:<6} {:<30} {:<20} {}",
                session.id,
                name,
                format_timestamp(session.updated_at),
                session.messages.len()
            );
        }
    }
    println!();
}

fn export_chat_to_markdown(
    session: &ChatSession,
    filepath: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut content = format!("# {}\n\n", session.name);
    content.push_str(&format!(
        "*Created: {} | Messages: {}*\n\n",
        format_timestamp(session.created_at),
        session.messages.len()
    ));
    content.push_str("---\n\n");

    for msg in &session.messages {
        let role = if msg.role == "user" {
            "💬 You"
        } else {
            "🤖 Rax"
        };
        content.push_str(&format!("### {}\n\n{}\n\n---\n\n", role, msg.content));
    }

    fs::write(filepath, content)?;
    Ok(())
}

fn run_interactive_tui(
    model: &LlamaModel,
    ctx: &mut llama_cpp_2::context::LlamaContext,
    mut system_prompt: String,
    mut storage: ChatStorage,
    session_id: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let mut messages: Vec<String> = Vec::new();
    let mut current_input = String::new();
    let mut scroll_offset = 0;
    let mut use_context = false;
    let mut current_session_id: Option<usize> = session_id;

    // Load existing session or create new one
    if let Some(id) = session_id {
        if let Some(sess) = storage.get_session(id) {
            messages = sess
                .messages
                .iter()
                .map(|m| format!("{}: {}", m.role, m.content))
                .collect();
            system_prompt = sess.system_prompt.clone();
            current_session_id = Some(id);
        }
    }

    if current_session_id.is_none() {
        let new_session = storage.create_session(system_prompt.clone());
        current_session_id = Some(new_session.id);
    }

    loop {
        execute!(stdout, Clear(crossterm::terminal::ClearType::All))?;

        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("╔══════════════════════════════════════════════════════════╗\n"),
            Print("║  "),
            SetForegroundColor(Color::Green),
            Print("Rax CLI"),
            SetForegroundColor(Color::Cyan),
            Print(" - Interactive Mode                        ║\n"),
            Print("╚══════════════════════════════════════════════════════════╝\n"),
            ResetColor
        )?;

        let (_, height) = crossterm::terminal::size()?;
        let available_lines = height as usize - 10;

        for msg in messages.iter().skip(scroll_offset).take(available_lines) {
            if msg.starts_with("user: ") {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Blue),
                    Print("\n❯ "),
                    ResetColor,
                    Print(&msg[6..]),
                    Print("\n")
                )?;
            } else if let Some(content) = msg.strip_prefix("rax: ") {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Green),
                    Print("\n🤖 "),
                    ResetColor
                )?;
                let width = 68;
                for line in content.chars().collect::<Vec<_>>().chunks(width) {
                    execute!(
                        stdout,
                        Print("   "),
                        Print(&String::from_iter(line.iter())),
                        Print("\n")
                    )?;
                }
            } else {
                execute!(stdout, Print("\n"), Print(msg), Print("\n"))?;
            }
        }

        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("\n"),
            Print("╭"),
            Print("─".repeat(62)),
            Print("╮\n"),
            Print("│"),
            ResetColor,
            Print(" "),
            SetForegroundColor(Color::Cyan),
            Print("You"),
            ResetColor,
            Print(": "),
            Print(&current_input),
            ResetColor
        )?;

        execute!(stdout, Print("▌"))?;

        execute!(
            stdout,
            ResetColor,
            Print("\n"),
            Print("╰"),
            Print("─".repeat(62)),
            Print("╯\n")
        )?;

        let session_info = format!(
            "Session #{} | Context: {} | /help for commands",
            current_session_id.unwrap_or(0),
            if use_context { "ON" } else { "OFF" }
        );
        execute!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print(&format!("  {}\n", session_info)),
            ResetColor
        )?;

        stdout.flush()?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('c')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        KeyCode::Enter => {
                            if !current_input.is_empty() {
                                let input = current_input.clone();

                                if input.starts_with('/') {
                                    match input.as_str() {
                                        "/help" => {
                                            messages.push(format!("rax: {}", print_help()));
                                        }
                                        "/clear" => {
                                            messages.clear();
                                            if let Some(id) = current_session_id {
                                                if let Some(sess) = storage.get_session(id) {
                                                    sess.messages.clear();
                                                    sess.updated_at = SystemTime::now()
                                                        .duration_since(SystemTime::UNIX_EPOCH)
                                                        .unwrap()
                                                        .as_secs();
                                                }
                                            }
                                        }
                                        "/quit" | "/exit" => {
                                            break;
                                        }
                                        "/context" => {
                                            use_context = !use_context;
                                            if use_context {
                                                messages.push(
                                                    "rax: ✓ Context loading enabled".to_string(),
                                                );
                                            } else {
                                                messages.push(
                                                    "rax: ✗ Context loading disabled".to_string(),
                                                );
                                            }
                                        }
                                        "/list" => {
                                            list_chats(&storage);
                                            messages.push(
                                                "rax: Use /load <id> to resume a conversation"
                                                    .to_string(),
                                            );
                                        }
                                        "/save" => {
                                            if let Some(id) = current_session_id {
                                                if let Some(sess) = storage.get_session(id) {
                                                    let name = format!("Chat #{}", id);
                                                    sess.name = name.clone();
                                                    let _ = storage.save();
                                                    messages.push(format!(
                                                        "rax: ✓ Saved as '{}'",
                                                        name
                                                    ));
                                                }
                                            }
                                        }
                                        cmd if cmd.starts_with("/save ") => {
                                            if let Some(id) = current_session_id {
                                                if let Some(sess) = storage.get_session(id) {
                                                    let name = input[6..].to_string();
                                                    sess.name = name.clone();
                                                    let _ = storage.save();
                                                    messages.push(format!(
                                                        "rax: ✓ Saved as '{}'",
                                                        name
                                                    ));
                                                }
                                            }
                                        }
                                        cmd if cmd.starts_with("/load ") => {
                                            if let Ok(id) = input[6..].parse::<usize>() {
                                                if let Some(sess) = storage.get_session(id) {
                                                    messages = sess
                                                        .messages
                                                        .iter()
                                                        .map(|m| {
                                                            format!("{}: {}", m.role, m.content)
                                                        })
                                                        .collect();
                                                    system_prompt = sess.system_prompt.clone();
                                                    current_session_id = Some(id);
                                                    messages.push(format!(
                                                        "rax: ✓ Loaded conversation #{}",
                                                        id
                                                    ));
                                                } else {
                                                    messages.push(format!(
                                                        "rax: ✗ Conversation #{} not found",
                                                        id
                                                    ));
                                                }
                                            } else {
                                                messages.push(
                                                    "rax: ✗ Invalid ID. Use /load <number>"
                                                        .to_string(),
                                                );
                                            }
                                        }
                                        cmd if cmd.starts_with("/export ") => {
                                            if let Some(id) = current_session_id {
                                                if let Some(sess) = storage.get_session(id) {
                                                    let filepath = input[8..].trim();
                                                    match export_chat_to_markdown(sess, filepath) {
                                                        Ok(_) => messages.push(format!(
                                                            "rax: ✓ Exported to '{}'",
                                                            filepath
                                                        )),
                                                        Err(e) => messages.push(format!(
                                                            "rax: ✗ Export failed: {}",
                                                            e
                                                        )),
                                                    }
                                                }
                                            }
                                        }
                                        "/system" => {
                                            messages.push(format!(
                                                "rax: Current system prompt:\n{}",
                                                system_prompt
                                            ));
                                        }
                                        _ => {
                                            messages.push(format!(
                                                "rax: Unknown command: {}. Type /help",
                                                input
                                            ));
                                        }
                                    }
                                } else {
                                    messages.push(format!("user: {}", input));

                                    let mut prompt = system_prompt.clone();
                                    if use_context {
                                        prompt.push_str(&load_context_with_cache());
                                    }

                                    let full_prompt = format!(
                                        "<|start_header_id|>system<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>user<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n",
                                        prompt, input
                                    );

                                    execute!(
                                        stdout,
                                        SetForegroundColor(Color::Green),
                                        Print("\n🤖 Rax: "),
                                        ResetColor
                                    )?;
                                    stdout.flush()?;

                                    let response = generate_response_streaming(
                                        model,
                                        ctx,
                                        &full_prompt,
                                        true,
                                    )?;
                                    messages.push(format!("rax: {}", response));

                                    // Save to session
                                    if let Some(id) = current_session_id {
                                        if let Some(sess) = storage.get_session(id) {
                                            let now = SystemTime::now()
                                                .duration_since(SystemTime::UNIX_EPOCH)
                                                .unwrap()
                                                .as_secs();

                                            sess.messages.push(ChatMessage {
                                                role: "user".to_string(),
                                                content: input.clone(),
                                                timestamp: now,
                                            });
                                            sess.messages.push(ChatMessage {
                                                role: "assistant".to_string(),
                                                content: response,
                                                timestamp: now,
                                            });
                                            sess.updated_at = now;
                                            let _ = storage.save();
                                        }
                                    }
                                }

                                current_input.clear();
                                scroll_offset = messages.len().saturating_sub(available_lines);
                            }
                        }
                        KeyCode::Backspace => {
                            current_input.pop();
                        }
                        KeyCode::Char(c) => {
                            current_input.push(c);
                        }
                        KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Up => {
                            scroll_offset = scroll_offset.saturating_sub(1);
                        }
                        KeyCode::Down => {
                            let max_scroll = messages.len().saturating_sub(available_lines);
                            if scroll_offset < max_scroll {
                                scroll_offset += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}

fn generate_response_streaming(
    model: &LlamaModel,
    ctx: &mut llama_cpp_2::context::LlamaContext,
    prompt: &str,
    use_tui: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let tokens_list = model
        .str_to_token(prompt, llama_cpp_2::model::AddBos::Always)
        .expect("Tokenization error");
    let mut batch = LlamaBatch::new(2048, 1);

    let last_index = tokens_list.len() - 1;
    for (i, token) in tokens_list.into_iter().enumerate() {
        batch.add(token, i as i32, &[0], i == last_index).unwrap();
    }

    ctx.decode(&mut batch).expect("Initial decode failed");

    let mut n_cur = batch.n_tokens();
    let mut full_response = String::new();

    while n_cur <= ctx.n_ctx() as i32 {
        let logits = ctx.candidates_ith(batch.n_tokens() - 1);
        let mut candidates = LlamaTokenDataArray::from_iter(logits, false);
        let new_token_id = candidates.sample_token_greedy();

        if new_token_id == model.token_eos() || new_token_id == LlamaToken::new(128009) {
            break;
        }

        if let Ok(token_bytes) = model.token_to_piece_bytes(new_token_id, 0, false, None) {
            let token_str = String::from_utf8_lossy(&token_bytes).to_string();

            if !use_tui {
                print!("{}", token_str);
                io::stdout().flush()?;
            }

            full_response.push_str(&token_str);
        }

        batch.clear();
        batch.add(new_token_id, n_cur, &[0], true).unwrap();
        n_cur += 1;
        ctx.decode(&mut batch).expect("Token decode failed");
    }

    if !use_tui {
        println!();
    }

    Ok(full_response)
}

fn generate_response(
    model: &LlamaModel,
    ctx: &mut llama_cpp_2::context::LlamaContext,
    prompt: &str,
) -> String {
    let tokens_list = model
        .str_to_token(prompt, llama_cpp_2::model::AddBos::Always)
        .expect("Tokenization error");
    let mut batch = LlamaBatch::new(2048, 1);

    let last_index = tokens_list.len() - 1;
    for (i, token) in tokens_list.into_iter().enumerate() {
        batch.add(token, i as i32, &[0], i == last_index).unwrap();
    }

    ctx.decode(&mut batch).expect("Initial decode failed");

    let mut n_cur = batch.n_tokens();
    let mut full_response = String::new();
    let skin = MadSkin::default();

    skin.print_text("\n**Rax**:\n");

    while n_cur <= ctx.n_ctx() as i32 {
        let logits = ctx.candidates_ith(batch.n_tokens() - 1);
        let mut candidates = LlamaTokenDataArray::from_iter(logits, false);
        let new_token_id = candidates.sample_token_greedy();

        if new_token_id == model.token_eos() || new_token_id == LlamaToken::new(128009) {
            break;
        }

        if let Ok(token_bytes) = model.token_to_piece_bytes(new_token_id, 0, false, None) {
            let token_str = String::from_utf8_lossy(&token_bytes).to_string();
            print!("{}", token_str);
            io::stdout().flush().unwrap();
            full_response.push_str(&token_str);
        }

        batch.clear();
        batch.add(new_token_id, n_cur, &[0], true).unwrap();
        n_cur += 1;
        ctx.decode(&mut batch).expect("Token decode failed");
    }
    println!();

    full_response
}

fn run_interactive_cli(
    model: &LlamaModel,
    ctx: &mut llama_cpp_2::context::LlamaContext,
    system_prompt: String,
    mut storage: ChatStorage,
) {
    // Create initial session
    let new_session = storage.create_session(system_prompt.clone());
    let current_session_id = new_session.id;

    let mut history = vec![format!(
        "<|start_header_id|>system<|end_header_id|>\n\n{}<|eot_id|>",
        system_prompt
    )];

    loop {
        print!("\n{}", SetForegroundColor(Color::Cyan));
        print!("❯ ");
        print!("{}", ResetColor);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("quit")
            || input.eq_ignore_ascii_case("exit")
            || input == "/quit"
        {
            println!("\n👋 Goodbye!\n");
            break;
        }

        if input == "/help" {
            println!("\n{}", print_help());
            continue;
        }

        if input == "/list" {
            list_chats(&storage);
            continue;
        }

        if input == "/save" {
            if let Some(sess) = storage.get_session(current_session_id) {
                println!(
                    "{}✓ Chat saved as '{}'",
                    SetForegroundColor(Color::Green),
                    sess.name
                );
                print!("{}", ResetColor);
            }
            continue;
        }

        if input == "/clear" {
            history = vec![format!(
                "<|start_header_id|>system<|end_header_id|>\n\n{}<|eot_id|>",
                storage
                    .get_session(current_session_id)
                    .map(|s| s.system_prompt.clone())
                    .unwrap_or_default()
            )];
            if let Some(sess) = storage.get_session(current_session_id) {
                sess.messages.clear();
                let _ = storage.save();
            }
            println!("{} Chat history cleared", SetForegroundColor(Color::Green));
            print!("{}", ResetColor);
            continue;
        }

        if input.is_empty() {
            continue;
        }

        history.push(format!(
            "<|start_header_id|>user<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n",
            input
        ));
        let prompt = history.join("");

        let response = generate_response(model, ctx, &prompt);
        history.push(format!("{}<|eot_id|>", response));

        // Save to session
        if let Some(sess) = storage.get_session(current_session_id) {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Generate conversation name from first message
            if sess.messages.is_empty() {
                let name = if input.len() > 40 {
                    format!("{}...", &input[..40])
                } else {
                    input.to_string()
                };
                sess.name = name;
            }

            sess.messages.push(ChatMessage {
                role: "user".to_string(),
                content: input.to_string(),
                timestamp: now,
            });
            sess.messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: response,
                timestamp: now,
            });
            sess.updated_at = now;
            let _ = storage.save();
        }
    }
}

fn main() {
    let args = Args::parse();
    let model_path = get_model_path();

    // Handle list chats
    if args.list_chats {
        let storage = ChatStorage::load();
        list_chats(&storage);
        return;
    }

    // Handle delete chat
    if let Some(id) = args.delete_chat {
        let mut storage = ChatStorage::load();
        if storage.delete_session(id) {
            let _ = storage.save();
            println!("✅ Deleted conversation #{}", id);
        } else {
            println!("❌ Conversation #{} not found", id);
        }
        return;
    }

    // Handle uninstall
    if args.uninstall {
        print!("🗑️  Removing downloaded model... ");
        io::stdout().flush().unwrap();

        if model_path.exists() {
            if let Err(e) = fs::remove_file(&model_path) {
                println!("❌ Failed: {}", e);
                std::process::exit(1);
            }
        }

        let config_path = get_config_path();
        if config_path.exists() {
            let _ = fs::remove_file(&config_path);
        }

        let chat_path = get_chat_history_path();
        if chat_path.exists() {
            let _ = fs::remove_file(&chat_path);
        }

        println!("✅ Uninstalled successfully");
        println!("   Run 'rax' again to reinstall");
        return;
    }

    // Handle status
    if args.status {
        let status = check_installation_status(&model_path);
        match status {
            InstallationStatus::NotInstalled => {
                println!("📦 Status: Not installed");
                println!("   Run 'rax' to install");
            }
            InstallationStatus::Installed => {
                let config = Config::load();
                println!("✅ Status: Installed");
                println!("   Location: {}", model_path.display());
                if let Some(size) = fs::metadata(&model_path).ok().map(|m| m.len()) {
                    println!("   Size: {}", format_bytes(size));
                }
                if let Some(updated) = config.last_updated {
                    println!("   Last updated: {}", updated);
                }

                // Show chat stats
                let storage = ChatStorage::load();
                let sessions = storage.list_sessions();
                println!("\n📝 Saved conversations: {}", sessions.len());
                if !sessions.is_empty() {
                    let total_messages: usize = sessions.iter().map(|s| s.messages.len()).sum();
                    println!("   Total messages: {}", total_messages);
                }
            }
            InstallationStatus::Corrupted => {
                println!("⚠️  Status: Corrupted or incomplete");
                println!("   Run 'rax --update-model' to re-download");
            }
        }
        return;
    }

    // Check installation status
    let status = check_installation_status(&model_path);

    match status {
        InstallationStatus::NotInstalled => {
            // First run - download model
            if let Ok(confirmed) = show_installation_wizard() {
                if !confirmed {
                    println!("\n👋 No problem! Run 'rax' anytime to initialize.");
                    println!("   Or run 'rax --uninstall' to remove completely.");
                    return;
                }
            } else {
                println!("\n👋 Run 'rax' anytime to initialize.");
                return;
            }

            if let Err(e) = download_model_with_progress(&model_path) {
                eprintln!("\n❌ Installation error: {}", e);
                println!("\n💡 You can retry by running 'rax' again");
                std::process::exit(1);
            }

            println!("\n🎉 Setup complete! Loading Rax...\n");
        }
        InstallationStatus::Corrupted => {
            println!("⚠️  Model appears corrupted. Re-downloading...\n");
            let _ = fs::remove_file(&model_path);
            if let Err(e) = download_model_with_progress(&model_path) {
                eprintln!("\n❌ Installation error: {}", e);
                std::process::exit(1);
            }
        }
        InstallationStatus::Installed => {
            // All good, continue
        }
    }

    // Initialize model
    print!("🧠 Loading model... ");
    io::stdout().flush().unwrap();
    let load_start = Instant::now();

    let backend = LlamaBackend::init().unwrap();
    let model_params = LlamaModelParams::default();
    let model = LlamaModel::load_from_file(&backend, model_path.to_str().unwrap(), &model_params)
        .expect("Failed to load model");

    let ctx_params = LlamaContextParams::default().with_n_ctx(Some(NonZeroU32::new(8192).unwrap()));
    let mut ctx = model
        .new_context(&backend, ctx_params)
        .expect("Failed to create context");

    println!("✅ Ready ({:.2}s)", load_start.elapsed().as_secs_f64());

    let mut system_prompt = args.system.clone();
    if args.context {
        println!("📂 Loading project context...");
        system_prompt.push_str(&load_context_with_cache());
    }

    let storage = ChatStorage::load();

    if args.interactive {
        let session_id = args.resume;
        if let Err(e) = run_interactive_tui(&model, &mut ctx, system_prompt, storage, session_id) {
            eprintln!("\nTUI error: {}", e);
            disable_raw_mode().ok();
            execute!(io::stdout(), LeaveAlternateScreen).ok();
        }
    } else if let Some(msg) = args.message {
        let prompt = format!(
            "<|start_header_id|>system<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>user<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n",
            system_prompt, msg
        );
        generate_response(&model, &mut ctx, &prompt);
    } else {
        // Default: Interactive chat mode (Claude Code style)
        println!("\n{}", SetForegroundColor(Color::Green));
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║            🤖 Rax CLI - Ready to Chat                   ║");
        println!("╚══════════════════════════════════════════════════════════╝");
        print!("{}", ResetColor);
        println!("\n💡 Type your message, or /help for commands, /quit to exit\n");
        run_interactive_cli(&model, &mut ctx, system_prompt, storage);
    }
}
