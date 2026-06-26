# Integrate Codex Through app-server

vibe-monitor will integrate Codex through `codex app-server` rather than scraping CLI text output or embedding the existing Codex UI. app-server provides structured JSON-RPC access to threads, turns, approvals, streamed events, models, skills, and related state, which is required for the Attention Queue and reliable long-task control.

