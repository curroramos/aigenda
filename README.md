# 🤖 aigenda

> AI-ready daily notes CLI - your digital journal for the modern age

A fast, simple command-line tool for capturing daily thoughts, ideas, and notes. Built in Rust for speed and reliability, with AI integration ready for the future.

## ✨ Features

- **📝 Quick Note Taking**: Add notes instantly from anywhere in your terminal
- **📅 Daily Organization**: Automatically organizes notes by date
- **🔍 Flexible Viewing**: List today's notes, specific dates, or everything
- **💾 Local Storage**: Your data stays on your machine as simple JSON files
- **🚀 Lightning Fast**: Built in Rust for maximum performance
- **🤖 AI-Ready**: Designed with future AI integrations in mind

## 🚀 Quick Start

```bash
# Add a note to today's log
cargo run -- add "Ship v0.1 MVP"

# View today's notes
cargo run -- list

# View all notes across all days
cargo run -- list --all

# View notes for a specific date
cargo run -- list --date 2025-01-15
```

## 📦 Installation

```bash
# Clone and build
git clone <repository-url>
cd aigenda
cargo build --release

# Add to your PATH (optional)
cp target/release/aigenda ~/.local/bin/
```

## 💡 Usage Examples

```bash
# Quick thoughts
aigenda add "Great idea for the new feature"

# Meeting notes
aigenda add "Team standup: discussed API redesign"

# Daily reflection
aigenda add "Learned about Rust error handling today"

# Review your day
aigenda list

# Check what you did last week
aigenda list --date 2025-01-20
```

## 📊 Architecture

```mermaid
graph TD
    %% Entry Point
    User[👤 User] --> CLI[🖥️ CLI Commands]
    CLI --> |"cargo run -- add 'text'"| AddCmd[📝 Add Command]
    CLI --> |"cargo run -- list [--all] [--date]"| ListCmd[📋 List Command]

    %% Core Application Flow
    Main[🚀 main.rs] --> |"clap::Parser"| CliParser[📋 cli.rs]
    CliParser --> |"Commands enum"| AppBuilder[🏗️ app::build_default]
    AppBuilder --> |"creates"| AppInstance[🎯 App<FsStorage>]
    AppInstance --> |"app.run()"| CommandRouter[🔀 Command Router]

    %% Command Processing
    CommandRouter --> |"Commands::Add"| AddHandler[commands/add.rs]
    CommandRouter --> |"Commands::List"| ListHandler[commands/list.rs]

    %% Storage Layer
    AddHandler --> |"store.load_day()"| Storage[💾 Storage Trait]
    AddHandler --> |"store.save_day()"| Storage
    ListHandler --> |"store.load_day()"| Storage
    ListHandler --> |"store.iter_days()"| Storage

    Storage --> |"implemented by"| FsStorage[📁 FsStorage]
    Storage --> |"future: sqlite"| SqliteStorage[🗄️ SqliteStorage]

    %% File System Storage Details
    FsStorage --> |"reads/writes"| JsonFiles[📄 JSON Files]
    JsonFiles --> |"format: YYYY-MM-DD.json"| DataDir[📂 ~/.local/share/aigenda/]

    %% Data Models
    AddHandler --> |"creates"| Note[📝 Note]
    ListHandler --> |"displays"| DayLog[📅 DayLog]
    Note --> |"part of"| DayLog
    FsStorage --> |"serializes/deserializes"| DayLog

    %% Model Structure
    DayLog --> |"contains"| NotesList[📝 Vec<Note>]
    DayLog --> |"contains"| DateField[📅 NaiveDate]
    Note --> |"contains"| Timestamp[⏰ RFC3339 timestamp]
    Note --> |"contains"| TextContent[📄 text content]
    Note --> |"contains"| TagsList[🏷️ Vec<String>]

    %% Error Handling
    AddHandler --> |"AppResult"| ErrorTypes[⚠️ AppError]
    ListHandler --> |"AppResult"| ErrorTypes
    FsStorage --> |"AppResult"| ErrorTypes
    ErrorTypes --> |"variants"| IoError[💥 IO Error]
    ErrorTypes --> |"variants"| JsonError[💥 JSON Error]
    ErrorTypes --> |"variants"| DateParseError[💥 Date Parse Error]
    ErrorTypes --> |"variants"| StorageError[💥 Storage Error]

    %% Future Extensions (Phase 2)
    AppInstance -.-> |"future"| AIFeatures[🤖 AI Features]
    AIFeatures -.-> |"claude.rs"| ClaudeAPI[🧠 Claude API]
    CommandRouter -.-> |"future"| ExportCmd[📤 Export Command]
    CommandRouter -.-> |"future"| SearchCmd[🔍 Search Command]

    %% Configuration
    FsStorage --> |"uses"| ProjectDirs[📁 ProjectDirs]
    ProjectDirs --> |"determines"| DataDir

    %% Testing
    TestSuite[🧪 Tests] --> |"integration"| CliSmoke[CLI Smoke Tests]
    TestSuite --> |"unit"| StorageTests[Storage Tests]

    %% Style the diagram
    classDef userClass fill:#e1f5fe
    classDef coreClass fill:#f3e5f5
    classDef storageClass fill:#e8f5e8
    classDef modelClass fill:#fff3e0
    classDef errorClass fill:#ffebee
    classDef futureClass fill:#f5f5f5,stroke-dasharray: 5 5

    class User,CLI userClass
    class Main,CliParser,AppInstance,CommandRouter,AddHandler,ListHandler coreClass
    class Storage,FsStorage,JsonFiles,DataDir storageClass
    class DayLog,Note,NotesList,DateField,Timestamp,TextContent,TagsList modelClass
    class ErrorTypes,IoError,JsonError,DateParseError,StorageError errorClass
    class AIFeatures,ClaudeAPI,ExportCmd,SearchCmd,SqliteStorage futureClass
```

## 🗃️ Data Storage

Your notes are stored locally as JSON files:

- **Location**: `~/.local/share/aigenda/` (Linux/macOS)
- **Format**: `YYYY-MM-DD.json` per day
- **Structure**: Each file contains a `DayLog` with an array of timestamped notes

Example data file:
```json
{
  "date": "2025-01-15",
  "notes": [
    {
      "when": "2025-01-15T10:30:00Z",
      "text": "Ship v0.1 MVP",
      "tags": []
    }
  ]
}
```

## 🧩 Development

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Run with logging
RUST_LOG=debug cargo run -- add "Debug message"

# Build release version
cargo build --release
```

## 🔮 Roadmap

- **Phase 1** (Current): ✅ Basic note-taking and listing
- **Phase 2**: 🔄 AI integration, search, export features
- **Phase 3**: 📱 Cross-platform sync, mobile companion

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 🧠 Learning & Discussion

### Why Another Note-Taking Tool?

**The honest answer?** I was frustrated with existing solutions.

Most note-taking apps either:
- Lock you into proprietary formats 📱➡️🔒
- Require internet connectivity ☁️❌
- Are bloated with features I never use 🎛️😵
- Don't integrate well with my terminal workflow 💻⚡

I wanted something **dead simple** that just works:
```bash
aigenda add "Had a brilliant idea during coffee"
# Done. It's saved. Forever. In plain JSON.
```

### Design Philosophy

**Local-first, AI-second.** Your thoughts belong to you, on your machine. The AI features (coming in Phase 2) will be *assistive*, not *dependent*. Think of it as having a conversation partner who remembers everything you've written, but never judges you for the random 3am thoughts.

### What I Learned Building This

**Rust Error Handling is Chef's Kiss** 👨‍🍳💋
Coming from JavaScript/Python, Rust's `Result<T, E>` felt verbose at first. Now? I can't imagine building CLI tools any other way. Every error is handled explicitly, no silent failures.

**Traits > Inheritance**
The `Storage` trait makes this incredibly flexible. Want SQLite? Implement the trait. Want cloud sync? Implement the trait. Want to store notes as carrier pigeons? ...please don't, but you could implement the trait.

**CLI Design is UX Design**
Every command should feel natural to type. `aigenda add` flows better than `aigenda create-note` or `aigenda new`. Small details matter when you're typing fast.

### Philosophical Rambling (Feel Free to Skip)

We're in this weird era where our thoughts are scattered across Slack, Discord, Apple Notes, random text files, and forgotten browser tabs. **aigenda** is my attempt at creating a single, reliable place for daily brain dumps.

It's not trying to be Notion. It's not trying to be Obsidian. It's trying to be the digital equivalent of that notebook you always carry but in a way that feels native to developers.

The AI integration (Phase 2) will be about *enhancing* your existing thoughts, not replacing them:
- "Show me patterns in what I've been thinking about"
- "What questions could I explore based on this note?"
- "Help me connect this idea to something I wrote last month"

Think of it as having a conversation with your past self, facilitated by AI.

### Community & Discussion

Found a bug? Have a feature idea? Just want to chat about note-taking philosophy?

- 🐛 **Issues**: Use GitHub issues for bugs and feature requests
- 💬 **Discussions**: GitHub discussions for questions and ideas
- 📧 **Email**: `your-email@domain.com` for longer conversations
- 🐦 **Twitter**: `@your-handle` for quick thoughts and updates

**Hot take**: The best productivity system is the one you actually use. If aigenda helps you capture more thoughts, awesome. If not, that's okay too - use what works for you.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

> Built with ❤️ and 🦀 Rust
> *"The best camera is the one you have with you. The best note-taking app is the one in your terminal."*
