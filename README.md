# Minimal LLM Language

A token-minimal, interpreted programming language optimized exclusively for LLM-generated code and not for humans, sorry humans! This language prioritizes AI token efficiency and compression, disregarding human readability.

## Features

- **Single-byte or minimal-byte encoding** for key constructs
- **Mathematical or symbolic representations** for logic and structure
- **Built-in compression mechanisms** for common code patterns
- **String dictionary system** for reusing text across projects
- **Native async support** for efficient concurrency and I/O operations
- **Comprehensive networking library** for TCP/IP, HTTP, and WebSocket communication
- **UI library** for creating simple user interfaces
- **File I/O** for reading and writing files
- **Security primitives** for hashing and encryption
- **Error handling** with try-catch mechanism
- **Type system** with type inference and validation
- **Collection operations** for data manipulation
- **Variable scoping** with proper closure support

## Symbolic Tokens

### Core Language Symbols
- `λ` - Library definition
- `ƒ` - Function definition
- `ι` - Integer type
- `σ` - String type
- `ξ` - Generic type (connection/list/value)
- `⟼` - Return
- `⌽` - Print
- `∞` - Infinite loop
- `⊤` - Boolean true
- `⊥` - Boolean false
- `∇` - Core library
- `⚠` - Error library
- `⟑` - Type library
- `:` - String dictionary reference prefix (e.g., `:key`)

### Greek Letters (Variable Prefixes)
- `α`, `β`, `γ` - Generic variables
- `δ`, `ε`, `ζ` - Environment variables
- `η`, `θ`, `κ` - Constants
- `ν`, `ξ`, `ο` - Objects
- `π` - Mathematical constant
- `ρ`, `τ`, `υ` - Type variables
- `φ`, `χ`, `ψ` - Function variables
- `ω` - Loop variables

### Data Structures and Operations
- `∅` - Empty collection
- `＋` - Add to collection
- `∑` - Sum
- `∀` - ForEach
- `⊳` - Left operation
- `⊢` - Push
- `⊣` - Pop

### Error Handling
- `÷` - Try-catch
- `⚠` - Error handling

### File System Operations
- `📂` or `d` - List directory (`📂("path")` → `[files...]`)
- `📖` or `r` - Read file (`📖("file")` → `"contents"`)
- `✍` or `w` - Write file (`✍("file", "contents")`)
- `✂` or `x` - Remove file/dir (`✂("path")`)
- `⧉` or `c` - Copy file (`⧉("src", "dst")`)
- `↷` or `m` - Move file (`↷("src", "dst")`)
- `?` or `e` - File exists (`?("path")` → `bool`)
- `↯` - Read file (legacy)
- `↱` - Write file (legacy)
- `⌸` - File operations library

### Shell & OS Process Control
- `!` - Execute shell (`!("ls -la")` → `{o:stdout, e:stderr, c:code}`)
- `🖥` or `s` - Current OS (`🖥()` → `"linux"`)
- `🌐` or `v` - Env var get (`🌐("VAR_NAME")` → `"value"`)

### Security
- `#` - Hash string (`#("abc", "sha256")` → `"..."`)
- `#f` or `h` - Hash file (`h("file", "sha1")` → `"..."`)
- `🔒` - Encrypt
- `🔑` - Decrypt
- `⚿` - Security library
- `🔓_fs` - Allow file I/O
- `📁_allow` - Limit I/O to specific directories
- `🔓_sh` - Enable/disable shell commands
- `🔓_net` - Enable/disable network operations

### String Dictionary Operations
- `📝` - Set string in dictionary (`📝("key", "value")`)
- `📖` - Get string from dictionary (`📖("key")` → `"value"`)
- `🔠` - Load string dictionary from file (`🔠("path")`)
- `💾` - Save string dictionary to file (`💾("dict_name", "path")`)
- `🔄` - Switch active dictionary (`🔄("dict_name")`)

### Timers and Environment
- `⏰` - Set timeout
- `ε` - Get environment variable

### Type Conversions
- `🔢` - String to integer
- `🔤` - Integer to string

### Module Management
- `⇪` - Import module

## Libraries

### HTTP & Networking
- `↗` or `g` - HTTP GET (`↗("https://site")` → `{s:status, b:body}`)
- `↓` or `p` - HTTP POST (`↓("url", "body")` → `{s:status, b:body}`)
- `⎋` or `j` - JSON parse (`⎋("{...}")` → `{key: val}`)
- `~` - WebSocket open (`~("ws://...")` → `socket handle`)

### Networking Library (`⚡`)
- `⊲(port, handler)` - Listen on port
- `⇉(connection, address, port)` - Forward connection
- `⇓(url)` - HTTP GET (legacy)
- `⇑(url, data)` - HTTP POST (legacy)
- `⥮(url, handler)` - WebSocket

### Concurrency Library (`⚯`)
- `⟿(size)` - Create channel with buffer size
- `⇢(channel, value)` - Send value to channel
- `⇠(channel)` - Receive value from channel
- `⟰(name)` - Create shared state
- `⇡(state, key, value)` - Set shared state value
- `⇣(state, key)` - Get shared state value

### Browser Automation
- `🌐` or `b` - Open page (`🌐("https://site")` → `browser`)
- `🖱` or `k` - Click selector (`🖱(browser, "#btn")`)
- `⌨` or `i` - Input text (`⌨(browser, "#inp", "hello")`)
- `👁` or `t` - Get text (`👁(browser, "#el")` → `"text"`)
- `🧠` or `e` - Eval JS (`🧠(browser, "return window.title;")`)
- `❌` or `z` - Close browser (`❌(browser)`)

### UI Library (`⬢`)
- `□(title, width, height)` - Create window
- `⬚(title, handler)` - Create button
- `✎(content)` - Create text
- `⌨(placeholder, handler)` - Create input

### Core Library (`∇`)
- `⌽(message)` - Print
- `⟼(value)` - Return
- `∑(list)` - Sum
- `∀(list, handler)` - ForEach

### File Library (`⌸`)
- `↯(path)` - Read file
- `↱(path, data)` - Write file

### Security Library (`⚿`)
- `#(data)` - Hash
- `🔒(data, key)` - Encrypt
- `🔑(data, key)` - Decrypt

### Error Library (`⚠`)
- `÷(try, catch)` - Try-catch

### Type Library (`⟑`)
- `🔢(string)` - String to integer
- `🔤(integer)` - Integer to string

### Agent Memory
- `📝` or `m` - Set memory (`📝("key", "val")`)
- `📖` or `n` - Get memory (`📖("key")` → `"val"`)
- `🗑` or `f` - Forget key (`🗑("key")`)

## Examples

### Basic Function and Library
```
λc{
    ƒ⌽(σ,m){
        ⌽(m)
    }
    
    ƒt1(){
        ιx=10;
        ιy=2;
        ⌽("t1");
        ⟼(x+y)
    }
}
```

### Error Handling
```
ƒtest_error(){
    ÷{
        ιx=42;
        ιy=0;
        ⟼(x/y)
    }{
        ⟼("Error caught!")
    }
}
```

### Concurrency with Channels
```
ƒtest_channel(){
    ιchan=⟿(5);
    ⇢(chan,42);
    ιval=⇠(chan);
    ⟼(val=42)
}
```

### Shared State
```
ƒtest_shared_state(){
    ιstate=⟰("test_state");
    ⇡(state,"key",42);
    ιval=⇣(state,"key");
    ⟼(val=42)
}
```

### Collection Operations
```
ƒtest_collection(){
    ιcoll=∅;
    ＋(coll,1);
    ＋(coll,2);
    ＋(coll,3);
    ⟼(∑(coll))
}
```

### String Dictionary Usage
```
// Define strings in dictionary
📝("greeting", "Hello, {}!");
📝("farewell", "Goodbye, {}!");

// Use string references with formatting
ƒgreet(σname){
    // Use :key syntax to reference strings from dictionary
    ⌽(:greeting, name);
}

ƒsayGoodbye(σname){
    ⌽(:farewell, name);
}

ƒmain(){
    greet("World");
    sayGoodbye("World");
}
```

## Implementation Status

### Core Features ✅
- Single-byte token encoding
- Mathematical and symbolic representations
- Built-in compression mechanisms
- String dictionary system for text reuse
- Native async support
- Comprehensive error handling with stack traces
- Type system with inference
- Collection operations
- Variable scoping with closure support
- REPL mode for interactive development

### Networking Features ✅
- TCP server/client with async support
- HTTP client with GET/POST
- WebSocket support with auto-reconnection
- Binary and text message types
- Rate limiting and timeout handling
- Connection pooling
- HTTPS support
- WebSocket ping/pong

### Concurrency Features ✅
- Channel-based communication
- Shared state management
- Async/await patterns
- Thread-safe operations
- Rate limiting
- Connection pooling

### Testing Infrastructure ✅
- Comprehensive test suite
- Network testing utilities
- Concurrency tests
- Error handling tests
- Type system tests
- Coverage reporting

### UI Components ✅
- Window management
- Button components
- Text display
- Input fields
- Basic layouts
- Styling support

### Documentation 📝
- [x] Basic README
- [x] Test Documentation
- [x] Language Specification
- [x] Example Projects
- [x] Tutorial Series
- [ ] API Reference
- [ ] Contributing Guide
- [ ] Style Guide

### Recent Improvements (v0.3.0) 🆕
- Warning suppression for clean builds
- Enhanced interpreter with support for more node types
- Improved string dictionary functionality
- Better error handling and debugging output
- Fixed emoji character recognition in lexer

### In Progress 🚧
- Garbage collection
- Module system improvements
- Performance profiling
- Custom UI components
- Event system
- Developer tools
- Agent integration capabilities

## Implementation Details

### Type System
- Static type checking with inference
- Support for Number, String, Boolean, Collection types
- Generic type support for collections
- Function type validation

### Error Handling
- Comprehensive try-catch mechanism
- Network error handling
- Type error detection
- Runtime error management

### Networking Features
- Async TCP server/client
- HTTP client with GET/POST
- WebSocket support with reconnection
- Binary and text message types
- Timeout handling
- Concurrent connection support

### Memory Management
- Efficient symbol encoding
- Minimal token usage
- Optimized AST representation
- String dictionary for text reuse

### String Dictionary System
- Centralized string storage to minimize token usage
- String references using `:key` syntax
- String formatting with placeholder support
- Multiple dictionaries with switching capability
- File-based dictionary loading and saving
- Significant token reduction for text-heavy applications

## Running the Language

```bash
# Run a program
cargo run program.a.i

# Run tests
cargo run tests.a.i
cargo run network_tests.a.i

# Start REPL
cargo run repl

# Build Desktop Application
cd backend && cargo tauri build
```

The desktop application can be built into:
- A .deb package for Debian-based Linux distributions
- An AppImage that runs on most Linux distributions
- A native binary

Built artifacts will be located in:
- DEB: target/release/bundle/deb/
- AppImage: target/release/bundle/appimage/
- Binary: target/release/

## Documentation
- [Test Documentation](TESTS.md) - Comprehensive test coverage
- [TODO List](TODO.md) - Development roadmap

## License

MIT
