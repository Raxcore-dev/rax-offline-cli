# 🚀 Rax CLI - Next-Gen Offline AI

[![CI](https://github.com/rax-cli/rax-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/rax-cli/rax-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/rax-cli/rax-cli)](https://github.com/rax-cli/rax-cli/releases)
[![License](https://img.shields.io/github/license/rax-cli/rax-cli)](LICENSE)

Fast, secure, and context-aware offline AI assistant powered by Llama 3.2.

## Features

- 🤖 **Claude Code-style Interface** - Just type and chat, no need to prefix every message
- 💾 **Persistent Chat History** - All conversations auto-saved locally
- 📁 **Project Context** - Automatically includes relevant files from your project
- 🎨 **Interactive TUI** - Beautiful full-screen chat interface
- 🔒 **100% Offline** - No data leaves your machine
- 🧠 **CPU Optimized** - Runs on CPU with Llama-3.2-1B (~650MB model)

## Installation

### Option 1: Official apt Repository (Recommended for Ubuntu/Debian)

```bash
# Add the repository key
curl -fsSL https://rax-cli.github.io/repo.key | \
    sudo gpg --dearmor -o /usr/share/keyrings/rax-archive-keyring.gpg

# Add the repository
echo "deb [signed-by=/usr/share/keyrings/rax-archive-keyring.gpg] https://rax-cli.github.io/ /" | \
    sudo tee /etc/apt/sources.list.d/rax.list

# Update and install
sudo apt update
sudo apt install rax
```

### Option 2: Download .deb Package

```bash
# Download the latest release
wget https://github.com/rax-cli/rax-cli/releases/latest/download/rax_0.1.0_amd64.deb

# Install
sudo apt install ./rax_0.1.0_amd64.deb
```

### Option 3: Official Installer Script

```bash
# Download and run the installer
curl -fsSL https://raw.githubusercontent.com/rax-cli/rax-cli/main/install-official.sh | bash
```

### Option 4: Build from Source

```bash
# Clone the repository
git clone https://github.com/rax-cli/rax-cli.git
cd rax-cli

# Build
cargo build --release

# Run
./target/release/rax

# Or install system-wide
./install.sh
```

## Quick Start

After installation, just run:

```bash
rax
```

The first run will:
1. Show a welcome screen
2. Download the AI model (~650 MB, one-time)
3. Start the interactive chat

## Usage

### Basic Chat
```bash
rax                          # Start interactive chat
rax "What is Rust?"          # Quick single question
rax -c "Fix this code"       # Include project context
rax -i                       # Full TUI mode
```

### Interactive Commands
```
/help          # Show commands
/clear         # Clear chat history
/list          # List saved conversations
/save          # Save current chat
/quit          # Exit
```

### CLI Options
```bash
rax -i                       # Full TUI mode
rax -c "Question"            # Include project context
rax --list-chats             # List all conversations
rax --resume 3               # Resume conversation #3
rax --delete-chat 2          # Delete conversation #2
rax --status                 # Show install status
rax --uninstall              # Remove model & data
rax --export chat.md         # Export chat to Markdown
```

## Examples

### Chat about code
```
❯ What does this Rust code do?
🤖 This code implements a thread-safe counter using Arc and Mutex...
```

### Get help with errors
```
❯ I'm getting a borrow checker error
🤖 The borrow checker is complaining because you're trying to...
```

### Project context
```bash
# Automatically includes relevant files
rax -c "How can I improve this architecture?"
```

## Data Storage

All data stored locally in `~/.local/share/RaxCli/`:
- `model.gguf` - The Llama model (downloaded once)
- `chat_history.json` - All conversations
- `config.json` - Settings
- `context_cache.json` - Cached project files

## Uninstall

```bash
# Remove package (keeps chat history)
sudo apt remove rax

# Remove completely (including chat history)
sudo apt purge rax

# Or manually
rm -rf ~/.local/share/RaxCli
```

## Requirements

### For apt Installation
- Ubuntu 20.04+ or Debian 11+
- curl or wget

### For Source Installation
- Rust 1.70+
- build-essential, libssl-dev, pkg-config
- ~700MB disk space for model
- 4GB+ RAM recommended

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Enter | Send message |
| Ctrl+C | Exit |
| Esc | Exit (TUI mode) |
| ↑/↓ | Scroll history (TUI) |

## Troubleshooting

### Model download fails
```bash
# Retry by running rax again
rax
```

### Check installation status
```bash
rax --status
```

### Re-download model
```bash
rax --update-model
```

### Permission denied
```bash
# Make sure the binary is executable
chmod +x ~/.local/bin/rax
```

## Development

### Build
```bash
cargo build --release
```

### Test
```bash
cargo test
```

### Build .deb package
```bash
./build-deb.sh
```

### Update apt Repository
```bash
./update-repo.sh
```

## CI/CD

The project uses GitHub Actions for:
- **CI**: Build and test on every push/PR
- **Release**: Create GitHub releases with artifacts
- **Deploy**: Automatically deploy apt repository to GitHub Pages

## Project Structure

```
rax-cli/
├── src/
│   └── main.rs          # Main application code
├── .github/
│   └── workflows/
│       ├── ci.yml       # Continuous integration
│       ├── release.yml  # Release automation
│       └── deploy-repo.yml  # apt repository deployment
├── repo/
│   └── ...              # apt repository structure
├── build-deb.sh         # Debian package builder
├── install-official.sh  # Official installer script
└── README.md            # This file
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

Built with:
- [llama-cpp-2](https://crates.io/crates/llama-cpp-2) - Rust bindings for llama.cpp
- [Llama 3.2](https://ai.meta.com/blog/llama-3-2-connect-2024-vision-edge-mobile-devices/) - Meta's efficient LLM
- [indicatif](https://crates.io/crates/indicatif) - Progress bar library
- [crossterm](https://crates.io/crates/crossterm) - Terminal manipulation
- [clap](https://crates.io/crates/clap) - Command line argument parser

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Support

- **Issues**: [GitHub Issues](https://github.com/rax-cli/rax-cli/issues)
- **Discussions**: [GitHub Discussions](https://github.com/rax-cli/rax-cli/discussions)
- **Documentation**: [Wiki](https://github.com/rax-cli/rax-cli/wiki)
