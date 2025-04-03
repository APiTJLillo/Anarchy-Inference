# Symbolic/Minimal Syntax Enhancements Documentation

This document provides details about the enhancements implemented for the Anarchy-Inference language, focusing on the new symbolic/minimal syntax system.

## Overview

The enhancements add a comprehensive standard library (`std`) with various modules that provide functionality for file system operations, shell control, HTTP networking, browser automation, security controls, crypto primitives, and agent memory management. Each module implements both symbolic (emoji) and alphabetic shorthand tokens for maximum flexibility.

## Module Structure

The standard library is organized into the following modules:

- `fs`: File system operations
- `shell`: Shell and OS process control
- `http`: HTTP and networking
- `browser`: Browser automation
- `security`: Security and permissions
- `crypto`: Hashing and crypto primitives
- `mem`: Agent memory management

## File System Operations (High Priority)

| Action | Symbol | Name | Description | Example |
|--------|--------|------|-------------|---------|
| List directory | 📂 | d | List files in a directory | `📂("path")` → `[files...]` |
| Read file | 📖 | r | Read file contents | `📖("file")` → `"contents"` |
| Write file | ✍ | w | Write to file | `✍("file", "contents")` |
| Remove file/dir | ✂ | x | Delete file or directory | `✂("path")` |
| Copy file | ⧉ | c | Copy a file | `⧉("src", "dst")` |
| Move file | ↷ | m | Move a file | `↷("src", "dst")` |
| File exists | ? | e | Check if file exists | `?("path")` → `bool` |

### Implementation Notes

- All functions return `✓` (boolean true) on success, or throw an error object.
- File write supports an optional third parameter for append mode: `✍("file", "data", "a")`

## Shell & OS Process Control (High Priority)

| Action | Symbol | Name | Description | Example |
|--------|--------|------|-------------|---------|
| Execute shell | ! | - | Run shell command | `!("ls -la")` → `{o:stdout, e:stderr, c:code}` |
| Current OS | 🖥 | s | Get current OS | `🖥()` → `"linux"` |
| Env var get | 🌐 | v | Get environment variable | `🌐("VAR_NAME")` → `"value"` |

### Security Note

Shell commands are wrapped with permission checks. The security module can disable or restrict command execution as needed.

## HTTP & Networking (High Priority)

| Action | Symbol | Name | Description | Example |
|--------|--------|------|-------------|---------|
| GET request | ↗ | g | HTTP GET | `↗("https://site")` → `{s:status, b:body}` |
| POST request | ↓ | p | HTTP POST | `↓("url", "body")` → `{s:status, b:body}` |
| JSON parse | ⎋ | j | Parse JSON | `⎋("{...}")` → `{key: val}` |
| WebSocket open | ~ | - | Open WebSocket | `~("ws://...")` → `socket handle` |

### Example Use

```
r = ↗("https://site")
j(r.b)
```

## Browser Automation (Medium Priority)

| Action | Symbol | Name | Description | Example |
|--------|--------|------|-------------|---------|
| Open page | 🌐 | b | Open browser page | `🌐("https://site")` → `browser` |
| Click selector | 🖱 | k | Click element | `🖱(browser, "#btn")` |
| Input text | ⌨ | i | Enter text | `⌨(browser, "#inp", "hello")` |
| Get text | 👁 | t | Get element text | `👁(browser, "#el")` → `"text"` |
| Eval JS | 🧠 | e | Execute JavaScript | `🧠(browser, "return window.title;")` |
| Close browser | ❌ | z | Close browser | `❌(browser)` |

### Implementation Note

The browser is represented as an opaque numeric ID that is passed to all browser-related calls.

## Security Gate (Medium Priority)

| Capability | Config Name | Description |
|------------|-------------|-------------|
| Allow file I/O | 🔓_fs | Enable/disable file operations |
| Allowed paths | 📁_allow | Limit I/O to specific directories |
| Allow shell | 🔓_sh | Enable/disable shell commands |
| Allow network | 🔓_net | Enable/disable network operations |

If a forbidden action is attempted, the interpreter throws a ⚠ with message "Forbidden".

## Crypto Primitives (Low Priority)

| Action | Symbol | Name | Description | Example |
|--------|--------|------|-------------|---------|
| Hash string | # | - | Hash a string | `#("abc", "sha256")` → `"..."` |
| Hash file | #f | h | Hash a file | `h("file", "sha1")` → `"..."` |

Currently supports SHA256 and MD5 hash algorithms.

## Agent Memory (Optional)

| Action | Symbol | Name | Description | Example |
|--------|--------|------|-------------|---------|
| Set memory | 📝 | m | Store value | `📝("key", "val")` |
| Get memory | 📖 | n | Retrieve value | `📖("key")` → `"val"` |
| Forget key | 🗑 | f | Remove value | `🗑("key")` |

Backed by a thread-safe in-memory map.

## Integration

The standard library is integrated into the main codebase through the `src/std.rs` file, which registers all token functions and provides wrapper functions for each operation. The token registration system maps both symbolic and alphabetic tokens to their corresponding functions.

## Testing

Comprehensive tests have been added in `tests/std_tests.rs` to verify the functionality of the implemented modules. The tests cover file system operations, shell operations, crypto operations, and memory operations.

## Usage

To use these enhancements in your Anarchy-Inference code, simply import the standard library and use the symbolic or alphabetic tokens as shown in the examples above.

```
// Example using file operations
📂(".")                  // List current directory
📖("file.txt")           // Read file
✍("output.txt", "data")  // Write to file

// Example using HTTP
r = ↗("https://example.com")
j(r.b)  // Parse JSON response
```

## Security Considerations

The security module provides fine-grained control over which operations are allowed. By default, all potentially dangerous operations (file I/O, shell execution, network access) are disabled and must be explicitly enabled.

To enable file operations:
```
security::set_allow_fs(true)
```

To restrict file operations to specific directories:
```
security::add_allowed_path("/safe/path")
```
