# External Tool Integration Documentation

This document provides comprehensive documentation for the external tool integration system in Anarchy Inference. The system enables AI agents to interact with external systems such as web services, search engines, and the file system while maintaining the token efficiency benefits of Anarchy Inference.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Tool Interfaces](#tool-interfaces)
   - [Web Interface](#web-interface)
   - [Search Interface](#search-interface)
   - [File System Interface](#file-system-interface)
4. [Tool Manager](#tool-manager)
5. [Integration with Anarchy Inference](#integration-with-anarchy-inference)
6. [Error Handling](#error-handling)
7. [Security Considerations](#security-considerations)
8. [Examples](#examples)
9. [Best Practices](#best-practices)

## Overview

The external tool integration system provides a modular and extensible framework for integrating external tools into Anarchy Inference. It enables AI agents to:

- Make HTTP requests and parse HTML content
- Search the web, local content, and knowledge bases
- Read, write, and manipulate files and directories
- Manage and execute tools through a unified interface

The system is designed with security, efficiency, and ease of use in mind, making it suitable for a wide range of agent applications.

## Architecture

The external tool integration system follows a modular design with four main components:

1. **Common Interfaces**: Define common behavior and data structures for all tools
2. **Tool-Specific Implementations**: Concrete implementations for web, search, and file system
3. **Tool Manager**: Central registry for managing and accessing tools
4. **Interpreter Integration**: Extensions to the Anarchy Inference language for using tools

This architecture allows for:
- Consistent interface across different tool types
- Easy extension with new tool types
- Centralized error handling and resource management
- Integration with the existing interpreter and string dictionary system

## Tool Interfaces

### Web Interface

The web interface provides capabilities for HTTP requests, WebSocket communication, and HTML parsing.

#### HTTP Requests

```
// Syntax
⚙️🌐 "http" {
  "method": "GET",
  "url": "https://example.com",
  "headers": {"Content-Type": "application/json"},
  "body": "{\"key\": \"value\"}"
} → response

// Example
⚙️🌐 "http" {
  "method": "GET",
  "url": "https://api.example.com/data"
} → response

// Access response data
response.status → 200
response.headers["Content-Type"] → "application/json"
response.body → "{\"result\": \"success\"}"
```

#### WebSocket Communication

```
// Connect to WebSocket
⚙️🌐 "websocket_connect" {
  "url": "wss://echo.websocket.org"
} → connection_id

// Send message
⚙️🌐 "websocket_send" {
  "connection_id": connection_id,
  "message": "Hello, WebSocket!"
} → success

// Close connection
⚙️🌐 "websocket_close" {
  "connection_id": connection_id
} → success
```

#### HTML Parsing

```
// Parse HTML
⚙️🌐 "parse_html" {
  "html": "<html><body><h1>Title</h1><p>Content</p></body></html>"
} → document

// Access parsed data
document.title → "Title"
document.body_text → "Title Content"
document.links[0].url → "https://example.com"
document.images[0].alt → "Image description"
```

### Search Interface

The search interface provides capabilities for searching the web, local content, and knowledge bases.

#### Web Search

```
// Syntax
⚙️🔍 "web" {
  "query": "search query",
  "max_results": 10,
  "time_range": "day",
  "site": "example.com",
  "file_type": "pdf",
  "language": "en",
  "safe_search": true
} → results

// Example
⚙️🔍 "web" {
  "query": "Anarchy Inference token efficiency",
  "max_results": 5
} → results

// Access results
results.total_count → 42
results.results[0].title → "Anarchy Inference: A Token-Efficient Language"
results.results[0].url → "https://example.com/article"
results.results[0].snippet → "Anarchy Inference provides significant token efficiency..."
```

#### Local Search

```
// Search local files
⚙️🔍 "local" {
  "query": "important document",
  "max_results": 10
} → results

// Access results
results.total_count → 3
results.results[0].title → "document.txt"
results.results[0].url → "file:///path/to/document.txt"
results.results[0].snippet → "This is an important document..."
```

#### Knowledge Base Search

```
// Search knowledge base
⚙️🔍 "knowledge_base" {
  "query": "token efficiency",
  "kb_id": "anarchy_inference_docs",
  "max_results": 5
} → results

// Access results
results.total_count → 7
results.results[0].title → "Token Efficiency Guide"
results.results[0].url → "kb://anarchy_inference_docs/token_efficiency"
results.results[0].snippet → "Token efficiency is a measure of..."
```

### File System Interface

The file system interface provides capabilities for file and directory operations.

#### File Operations

```
// Read file
⚙️📁 "read" {
  "path": "/path/to/file.txt"
} → content

// Write file
⚙️📁 "write" {
  "path": "/path/to/file.txt",
  "content": "Hello, world!",
  "append": false
} → success

// Delete file
⚙️📁 "delete" {
  "path": "/path/to/file.txt"
} → success

// Copy file
⚙️📁 "copy" {
  "src": "/path/to/source.txt",
  "dst": "/path/to/destination.txt",
  "overwrite": false
} → success

// Move file
⚙️📁 "move" {
  "src": "/path/to/source.txt",
  "dst": "/path/to/destination.txt",
  "overwrite": false
} → success
```

#### Directory Operations

```
// Create directory
⚙️📁 "mkdir" {
  "path": "/path/to/directory",
  "recursive": true
} → success

// List directory contents
⚙️📁 "list" {
  "path": "/path/to/directory"
} → entries

// Access directory entries
entries[0].name → "file.txt"
entries[0].path → "/path/to/directory/file.txt"
entries[0].size → 1024
entries[0].is_dir → false
entries[0].modified → "2025-04-25T12:34:56Z"
```

#### File Information

```
// Get file information
⚙️📁 "info" {
  "path": "/path/to/file.txt"
} → info

// Access file information
info.name → "file.txt"
info.path → "/path/to/file.txt"
info.size → 1024
info.is_dir → false
info.modified → "2025-04-25T12:34:56Z"
info.accessed → "2025-04-25T12:34:56Z"
info.created → "2025-04-24T12:34:56Z"
```

## Tool Manager

The tool manager provides a central registry for managing and accessing tools.

```
// Get tool manager
tool_manager → manager

// List available tools
manager.list_tools() → ["web", "search", "filesystem"]

// Get tool descriptions
manager.get_tool_descriptions() → {
  "web": "Web tool for HTTP requests, WebSocket communication, and HTML parsing",
  "search": "Search tool for web, local content, and knowledge base search",
  "filesystem": "File system tool for file and directory operations"
}

// Execute tool
manager.execute_tool("web", {
  "command": "http",
  "args": {
    "method": "GET",
    "url": "https://example.com"
  }
}) → result

// Get execution log
manager.get_log() → [
  {
    "tool_name": "web",
    "command": "http",
    "timestamp": "2025-04-25T12:34:56Z",
    "duration_ms": 123,
    "status": "success"
  }
]
```

## Integration with Anarchy Inference

The external tool integration system is integrated with the Anarchy Inference language through new syntax and built-in functions.

### Tool Operation Syntax

```
// General syntax
⚙️<tool_emoji> <command> <parameters> → <result>

// Web tool (🌐)
⚙️🌐 "http" {"method": "GET", "url": "https://example.com"} → response

// Search tool (🔍)
⚙️🔍 "web" {"query": "Anarchy Inference"} → results

// File system tool (📁)
⚙️📁 "read" {"path": "/path/to/file.txt"} → content
```

### Tool Manager Access

```
// Get tool manager
tool_manager → manager

// Execute tool through manager
manager.execute_tool("web", {
  "command": "http",
  "args": {
    "method": "GET",
    "url": "https://example.com"
  }
}) → result
```

## Error Handling

The external tool integration system provides comprehensive error handling.

```
// Try-catch for tool operations
try {
  ⚙️🌐 "http" {"method": "GET", "url": "https://invalid-url"} → response
} catch (error) {
  error.code → 500
  error.message → "Failed to send request: invalid URL"
}

// Error handling with tool manager
try {
  manager.execute_tool("unknown_tool", {"command": "test"}) → result
} catch (error) {
  error.code → 404
  error.message → "Tool not found: unknown_tool"
}
```

## Security Considerations

The external tool integration system includes several security measures:

1. **Path Sanitization**: File system operations validate and sanitize paths to prevent directory traversal attacks.
2. **Rate Limiting**: Web requests are rate-limited to prevent abuse.
3. **Configurable Permissions**: Each tool type has configurable permissions to restrict operations.
4. **Sandboxing**: Tools operate within a security sandbox to prevent unauthorized access.
5. **Resource Limits**: Memory and CPU usage are limited to prevent resource exhaustion.

Example of configuring security settings:

```
// Configure file system security
filesystem_tool.allowed_operations = {
  allow_read: true,
  allow_write: false,
  allow_delete: false,
  allow_dir_ops: true,
  allow_outside_base: false
}

filesystem_tool.security_sandbox = {
  allowed_read_extensions: ["txt", "md", "json"],
  max_read_size: 10 * 1024 * 1024, // 10 MB
  disallowed_paths: ["**/.*", "**/node_modules/**"]
}

// Configure web tool rate limiting
web_tool.rate_limiter = new RateLimiter(100) // 100 requests per minute
```

## Examples

### Example 1: Web Scraping

```
// Fetch webpage
⚙️🌐 "http" {
  "method": "GET",
  "url": "https://example.com/articles"
} → response

// Parse HTML
⚙️🌐 "parse_html" {
  "html": response.body
} → document

// Extract article links
articles = []
for (link in document.links) {
  if (link.url.contains("/article/")) {
    articles.push(link)
  }
}

// Fetch and process each article
for (article in articles) {
  ⚙️🌐 "http" {
    "method": "GET",
    "url": article.url
  } → article_response
  
  ⚙️🌐 "parse_html" {
    "html": article_response.body
  } → article_document
  
  // Save article content
  ⚙️📁 "write" {
    "path": "/articles/" + article.url.split("/").last() + ".txt",
    "content": article_document.body_text,
    "append": false
  } → _
}
```

### Example 2: Research Assistant

```
// Search for information
⚙️🔍 "web" {
  "query": "Anarchy Inference token efficiency",
  "max_results": 5
} → search_results

// Process search results
findings = []
for (result in search_results.results) {
  // Fetch webpage
  ⚙️🌐 "http" {
    "method": "GET",
    "url": result.url
  } → response
  
  // Parse HTML
  ⚙️🌐 "parse_html" {
    "html": response.body
  } → document
  
  // Extract relevant information
  if (document.body_text.contains("token efficiency")) {
    findings.push({
      "title": result.title,
      "url": result.url,
      "content": document.body_text.substring(0, 500) + "..."
    })
  }
}

// Save findings to file
⚙️📁 "write" {
  "path": "/research/token_efficiency_findings.json",
  "content": JSON.stringify(findings, null, 2),
  "append": false
} → _
```

### Example 3: File System Operations

```
// Create directory structure
⚙️📁 "mkdir" {
  "path": "/project/src",
  "recursive": true
} → _

⚙️📁 "mkdir" {
  "path": "/project/docs",
  "recursive": true
} → _

// Create source file
⚙️📁 "write" {
  "path": "/project/src/main.ai",
  "content": "// Anarchy Inference source code\n\nλ main() {\n  print(\"Hello, world!\")\n}",
  "append": false
} → _

// Create documentation file
⚙️📁 "write" {
  "path": "/project/docs/README.md",
  "content": "# Project Documentation\n\nThis is a sample project using Anarchy Inference.",
  "append": false
} → _

// List project contents
⚙️📁 "list" {
  "path": "/project"
} → project_contents

// Process each entry
for (entry in project_contents) {
  if (entry.is_dir) {
    ⚙️📁 "list" {
      "path": entry.path
    } → dir_contents
    
    print("Directory: " + entry.name + " contains " + dir_contents.length + " items")
  } else {
    ⚙️📁 "info" {
      "path": entry.path
    } → file_info
    
    print("File: " + entry.name + " (" + file_info.size + " bytes)")
  }
}
```

## Best Practices

1. **Error Handling**: Always use try-catch blocks when executing tool operations to handle potential errors.

2. **Resource Management**: Close resources like WebSocket connections when they are no longer needed.

3. **Security**: Configure appropriate security settings for each tool to prevent unauthorized access.

4. **Rate Limiting**: Be mindful of rate limits when making web requests to avoid being blocked.

5. **Path Handling**: Use absolute paths when possible and ensure paths are properly sanitized.

6. **Asynchronous Operations**: Use asynchronous operations for long-running tasks to avoid blocking the interpreter.

7. **Caching**: Cache results when appropriate to reduce external requests and improve performance.

8. **Logging**: Enable logging to track tool usage and diagnose issues.

9. **Validation**: Validate input parameters before executing tool operations to prevent errors.

10. **Modularity**: Organize code into modules to improve maintainability and reusability.
