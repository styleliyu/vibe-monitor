# vibe-monitor Context

This context defines the product language for vibe-monitor. It keeps product terms stable while the implementation evolves from a Codex-focused MVP into a broader local development cockpit.

## Language

**Workspace**:
A local project root that vibe-monitor can open, monitor, and associate with sessions, Git state, terminals, and tool layout.
_Avoid_: Project folder, repository, codebase

**Session**:
A user-visible work surface inside a workspace, such as a Codex conversation, terminal, browser preview, or external tool panel.
_Avoid_: Tab, window, process

**Codex Thread**:
A Codex conversation managed through Codex app-server and associated with a workspace or worktree.
_Avoid_: Chat, conversation, agent session

**Attention Item**:
A prioritized item that requires or may soon require user awareness, such as an approval, failure, blocker, completion, or unread external-tool signal.
_Avoid_: Notification, alert, task

**Attention Queue**:
The ordered collection of unresolved Attention Items shown to help the user decide where to intervene next.
_Avoid_: Inbox, notification center, Kanban board

**External Tool**:
A user-configured application or URL outside vibe-monitor, such as VS Code, Zed, GitHub, ChatGPT, WeChat, a browser, or a music app.
_Avoid_: Plugin, integration, app

**Launcher**:
A lightweight External Tool action that opens, focuses, or deep-links into another application without reading private content from it.
_Avoid_: Automation, connector

**Plugin**:
A packaged extension point that can contribute commands, panels, or Attention Items without coupling external-tool logic into the core application.
_Avoid_: Tool, integration, extension

**MVP**:
The first usable version of vibe-monitor, limited to Workspace management, Codex control, terminal, Git status/diff, and the Attention Queue.
_Avoid_: Prototype, beta, v1

**Cockpit**:
The overall product metaphor for a local-first control surface that helps the user observe, approve, route, and resume AI-assisted development work.
_Avoid_: Dashboard, IDE, control panel

