# Minimal LLM Language

A token-minimal, interpreted programming language optimized exclusively for LLM-generated code and not for humans, sorry humans! This language prioritizes AI token efficiency and compression, disregarding human readability.

## Features

- **Single-byte or minimal-byte encoding** for key constructs
- **Mathematical or symbolic representations** for logic and structure
- **Built-in compression mechanisms** for common code patterns
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

### File I/O
- `↯` - Read file
- `↱` - Write file
- `⌸` - File operations library

### Security
- `#` - Hash
- `🔒` - Encrypt
- `🔑` - Decrypt
- `⚿` - Security library

### Timers and Environment
- `⏰` - Set timeout
- `ε` - Get environment variable

### Type Conversions
- `🔢` - String to integer
- `🔤` - Integer to string

### Module Management
- `⇪` - Import module

## Libraries

### Networking Library (`⚡`)
- `⊲(port, handler)` - Listen on port
- `⇉(connection, address, port)` - Forward connection
- `⇓(url)` - HTTP GET
- `⇑(url, data)` - HTTP POST
- `⥮(url, handler)` - WebSocket

### Concurrency Library (`⚯`)
- `⟿(size)` - Create channel with buffer size
- `⇢(channel, value)` - Send value to channel
- `⇠(channel)` - Receive value from channel
- `⟰(name)` - Create shared state
- `⇡(state, key, value)` - Set shared state value
- `⇣(state, key)` - Get shared state value

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

## Implementation Status

### Core Features ✅
- Single-byte token encoding
- Mathematical and symbolic representations
- Built-in compression mechanisms
- Native async support
- Comprehensive error handling with stack traces
- Type system with inference
- Collection operations
- Variable scoping with closure support

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

### In Progress 🚧
- Garbage collection
- Module system improvements
- Performance profiling
- Custom UI components
- Event system
- Developer tools

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
