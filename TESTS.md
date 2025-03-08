# Test Documentation

## Network Tests (`network_tests.a.i`)

### TCP Tests
1. `test_tcp` - Basic TCP server and connection forwarding
   - Creates server on port 8080
   - Forwards connections to localhost:8081
   - Verifies connection handling

2. `test_tcp_invalid_port` - Invalid port handling
   - Attempts to bind to invalid port (99999)
   - Verifies error handling
   - Expects failure

3. `test_tcp_concurrent` - Concurrent connection handling
   - Creates multiple handlers on same port
   - Tests simultaneous connections
   - Verifies concurrent processing

### HTTP Tests
1. `test_http_get` - Basic HTTP GET
   - Performs GET request to test endpoint
   - Verifies response handling
   - Tests successful case

2. `test_http_get_invalid` - Invalid HTTP GET
   - Attempts request to invalid domain
   - Tests error handling
   - Expects failure

3. `test_http_post` - Basic HTTP POST
   - Sends POST with plain text data
   - Verifies response
   - Tests data transmission

4. `test_http_post_json` - JSON HTTP POST
   - Sends POST with JSON data
   - Tests content type handling
   - Verifies JSON processing

### WebSocket Tests
1. `test_websocket` - Basic WebSocket
   - Establishes WebSocket connection
   - Tests message handling
   - Verifies basic functionality

2. `test_websocket_binary` - Binary WebSocket
   - Tests binary message format
   - Verifies binary data handling
   - Tests message type detection

3. `test_websocket_reconnect` - Connection Recovery
   - Tests connection drop handling
   - Verifies reconnection logic
   - Tests connection stability

## Core Language Tests (`tests.a.i`)

### Arithmetic Operations
1. `t1` - Addition
2. `t2` - Subtraction
3. `t3` - Multiplication
4. `t4` - Division

### String Operations
1. `t5` - String concatenation
2. `t6` - String equality

### Collection Operations
1. `t7` - Collection manipulation and sum

### Error Handling
1. `t8` - Division by zero
2. `t9` - Try-catch with no error

### Variable Scoping
1. `t10` - Nested expressions
2. `t11` - Variable equality (true case)
3. `t12` - Variable equality (false case)

## Running Tests

### Network Tests
```bash
cargo run network_tests.a.i
```

### Core Tests
```bash
cargo run tests.a.i
```

## Test Coverage

- ✓ TCP/IP Networking
- ✓ HTTP Requests
- ✓ WebSocket Communication
- ✓ Basic Arithmetic
- ✓ String Operations
- ✓ Collections
- ✓ Error Handling
- ✓ Variable Scoping
- ✓ Type System
- ✓ Function Calls 