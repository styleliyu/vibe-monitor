# Use Tauri, Rust, React, and SQLite for the Desktop Stack

vibe-monitor will use Tauri 2 with a Rust backend, React + TypeScript + Vite frontend, and SQLite for local persistence. This keeps the app local-first and capable of process, filesystem, PTY, Git, and Codex child-process management while avoiding the resource cost of an Electron-first architecture and the deployment weight of a server database.

