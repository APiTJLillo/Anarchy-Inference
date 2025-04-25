# External Tool Integration Tests

This file contains tests for the external tool integration system, including the web, search, and file system interfaces.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::external_tools::{
        ExternalTool, ToolParams, ToolResult, ToolStatus, ToolError, ToolContext,
        WebTool, SearchTool, FileSystemTool, ToolManager
    };
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_tool_manager_registration() {
        let mut manager = ToolManager::new();
        
        // Register tools
        let web_tool = WebTool::new();
        let search_tool = SearchTool::new();
        let filesystem_tool = FileSystemTool::new(PathBuf::from("."));
        
        assert!(manager.register_tool(web_tool).is_ok());
        assert!(manager.register_tool(search_tool).is_ok());
        assert!(manager.register_tool(filesystem_tool).is_ok());
        
        // Check registered tools
        let tools = manager.list_tools();
        assert_eq!(tools.len(), 3);
        assert!(tools.contains(&"web".to_string()));
        assert!(tools.contains(&"search".to_string()));
        assert!(tools.contains(&"filesystem".to_string()));
        
        // Get tool descriptions
        let descriptions = manager.get_tool_descriptions();
        assert_eq!(descriptions.len(), 3);
        assert!(descriptions.contains_key("web"));
        assert!(descriptions.contains_key("search"));
        assert!(descriptions.contains_key("filesystem"));
    }
    
    #[test]
    fn test_web_tool_http_request() {
        let web_tool = WebTool::new();
        
        // Create parameters for HTTP request
        let mut params = ToolParams::new("http".to_string());
        params = params.with_arg("method", "GET");
        params = params.with_arg("url", "https://example.com");
        
        // Execute tool
        let result = web_tool.execute(&params);
        
        // Check result
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status, ToolStatus::Success);
        
        // Check result data
        if let Value::Object(data) = result.data {
            assert!(data.contains_key("status"));
            assert!(data.contains_key("headers"));
            assert!(data.contains_key("body"));
            
            if let Value::Number(status) = &data["status"] {
                assert_eq!(*status, 200.0);
            } else {
                panic!("Status is not a number");
            }
            
            if let Value::String(body) = &data["body"] {
                assert!(body.contains("Example Domain"));
            } else {
                panic!("Body is not a string");
            }
        } else {
            panic!("Result data is not an object");
        }
    }
    
    #[test]
    fn test_web_tool_parse_html() {
        let web_tool = WebTool::new();
        
        // Create HTML content
        let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Page</title>
        </head>
        <body>
            <h1>Hello, World!</h1>
            <p>This is a test page.</p>
            <a href="https://example.com">Example Link</a>
            <img src="image.jpg" alt="Test Image">
        </body>
        </html>
        "#;
        
        // Create parameters for HTML parsing
        let mut params = ToolParams::new("parse_html".to_string());
        params = params.with_arg("html", html);
        
        // Execute tool
        let result = web_tool.execute(&params);
        
        // Check result
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status, ToolStatus::Success);
        
        // Check result data
        if let Value::Object(data) = result.data {
            assert!(data.contains_key("title"));
            assert!(data.contains_key("body_text"));
            assert!(data.contains_key("links"));
            assert!(data.contains_key("images"));
            
            if let Value::String(title) = &data["title"] {
                assert_eq!(*title, "Test Page");
            } else {
                panic!("Title is not a string");
            }
            
            if let Value::String(body_text) = &data["body_text"] {
                assert!(body_text.contains("Hello, World!"));
                assert!(body_text.contains("This is a test page."));
            } else {
                panic!("Body text is not a string");
            }
            
            if let Value::Array(links) = &data["links"] {
                assert_eq!(links.len(), 1);
                if let Value::Object(link) = &links[0] {
                    if let Value::String(url) = &link["url"] {
                        assert_eq!(*url, "https://example.com");
                    } else {
                        panic!("Link URL is not a string");
                    }
                    if let Value::String(text) = &link["text"] {
                        assert_eq!(*text, "Example Link");
                    } else {
                        panic!("Link text is not a string");
                    }
                } else {
                    panic!("Link is not an object");
                }
            } else {
                panic!("Links is not an array");
            }
            
            if let Value::Array(images) = &data["images"] {
                assert_eq!(images.len(), 1);
                if let Value::Object(image) = &images[0] {
                    if let Value::String(url) = &image["url"] {
                        assert_eq!(*url, "image.jpg");
                    } else {
                        panic!("Image URL is not a string");
                    }
                    if let Value::String(alt) = &image["alt"] {
                        assert_eq!(*alt, "Test Image");
                    } else {
                        panic!("Image alt is not a string");
                    }
                } else {
                    panic!("Image is not an object");
                }
            } else {
                panic!("Images is not an array");
            }
        } else {
            panic!("Result data is not an object");
        }
    }
    
    #[test]
    fn test_search_tool_local_search() {
        // Create temporary directory
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        // Create test files
        let file1_path = temp_path.join("file1.txt");
        let file2_path = temp_path.join("file2.txt");
        
        std::fs::write(&file1_path, "This is a test file with some content about Rust programming.").unwrap();
        std::fs::write(&file2_path, "This file contains information about Python and JavaScript.").unwrap();
        
        // Create search tool with local index
        let mut search_tool = SearchTool::new().with_local_search_index(temp_path);
        
        // Index files
        if let Some(index) = search_tool.local_search_index.as_mut() {
            index.index_file(&file1_path).unwrap();
            index.index_file(&file2_path).unwrap();
        }
        
        // Create parameters for local search
        let mut params = ToolParams::new("local".to_string());
        params = params.with_arg("query", "Rust programming");
        params = params.with_arg("max_results", 10);
        
        // Execute tool
        let result = search_tool.execute(&params);
        
        // Check result
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.status, ToolStatus::Success);
        
        // Check result data
        if let Value::Object(data) = result.data {
            assert!(data.contains_key("query"));
            assert!(data.contains_key("total_count"));
            assert!(data.contains_key("results"));
            
            if let Value::String(query) = &data["query"] {
                assert_eq!(*query, "rust programming");
            } else {
                panic!("Query is not a string");
            }
            
            if let Value::Number(total_count) = &data["total_count"] {
                assert_eq!(*total_count, 1.0);
            } else {
                panic!("Total count is not a number");
            }
            
            if let Value::Array(results) = &data["results"] {
                assert_eq!(results.len(), 1);
                if let Value::Object(result) = &results[0] {
                    if let Value::String(title) = &result["title"] {
                        assert_eq!(*title, "file1.txt");
                    } else {
                        panic!("Result title is not a string");
                    }
                    if let Value::String(snippet) = &result["snippet"] {
                        assert!(snippet.contains("Rust programming"));
                    } else {
                        panic!("Result snippet is not a string");
                    }
                } else {
                    panic!("Result is not an object");
                }
            } else {
                panic!("Results is not an array");
            }
        } else {
            panic!("Result data is not an object");
        }
    }
    
    #[test]
    fn test_filesystem_tool_operations() {
        // Create temporary directory
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        // Create file system tool
        let filesystem_tool = FileSystemTool::new(temp_path);
        
        // Test write operation
        let write_params = ToolParams::new("write".to_string())
            .with_arg("path", "test.txt")
            .with_arg("content", "Hello, World!")
            .with_arg("append", false);
        
        let write_result = filesystem_tool.execute(&write_params);
        assert!(write_result.is_ok());
        
        // Test read operation
        let read_params = ToolParams::new("read".to_string())
            .with_arg("path", "test.txt");
        
        let read_result = filesystem_tool.execute(&read_params);
        assert!(read_result.is_ok());
        
        let read_result = read_result.unwrap();
        if let Value::String(content) = read_result.data {
            assert_eq!(content, "Hello, World!");
        } else {
            panic!("Read result is not a string");
        }
        
        // Test info operation
        let info_params = ToolParams::new("info".to_string())
            .with_arg("path", "test.txt");
        
        let info_result = filesystem_tool.execute(&info_params);
        assert!(info_result.is_ok());
        
        let info_result = info_result.unwrap();
        if let Value::Object(info) = info_result.data {
            assert!(info.contains_key("name"));
            assert!(info.contains_key("size"));
            assert!(info.contains_key("is_dir"));
            
            if let Value::String(name) = &info["name"] {
                assert_eq!(*name, "test.txt");
            } else {
                panic!("Info name is not a string");
            }
            
            if let Value::Number(size) = &info["size"] {
                assert_eq!(*size, 13.0); // "Hello, World!" is 13 bytes
            } else {
                panic!("Info size is not a number");
            }
            
            if let Value::Boolean(is_dir) = &info["is_dir"] {
                assert_eq!(*is_dir, false);
            } else {
                panic!("Info is_dir is not a boolean");
            }
        } else {
            panic!("Info result is not an object");
        }
        
        // Test mkdir operation
        let mkdir_params = ToolParams::new("mkdir".to_string())
            .with_arg("path", "testdir")
            .with_arg("recursive", true);
        
        let mkdir_result = filesystem_tool.execute(&mkdir_params);
        assert!(mkdir_result.is_ok());
        
        // Test list operation
        let list_params = ToolParams::new("list".to_string())
            .with_arg("path", ".");
        
        let list_result = filesystem_tool.execute(&list_params);
        assert!(list_result.is_ok());
        
        let list_result = list_result.unwrap();
        if let Value::Array(entries) = list_result.data {
            assert_eq!(entries.len(), 2); // test.txt and testdir
            
            // Find test.txt entry
            let test_txt = entries.iter().find(|entry| {
                if let Value::Object(entry) = entry {
                    if let Value::String(name) = &entry["name"] {
                        return name == "test.txt";
                    }
                }
                false
            });
            assert!(test_txt.is_some());
            
            // Find testdir entry
            let testdir = entries.iter().find(|entry| {
                if let Value::Object(entry) = entry {
                    if let Value::String(name) = &entry["name"] {
                        return name == "testdir";
                    }
                }
                false
            });
            assert!(testdir.is_some());
            
            if let Value::Object(testdir) = testdir.unwrap() {
                if let Value::Boolean(is_dir) = &testdir["is_dir"] {
                    assert_eq!(*is_dir, true);
                } else {
                    panic!("Testdir is_dir is not a boolean");
                }
            } else {
                panic!("Testdir is not an object");
            }
        } else {
            panic!("List result is not an array");
        }
        
        // Test copy operation
        let copy_params = ToolParams::new("copy".to_string())
            .with_arg("src", "test.txt")
            .with_arg("dst", "testdir/test_copy.txt")
            .with_arg("overwrite", false);
        
        let copy_result = filesystem_tool.execute(&copy_params);
        assert!(copy_result.is_ok());
        
        // Test move operation
        let move_params = ToolParams::new("move".to_string())
            .with_arg("src", "test.txt")
            .with_arg("dst", "testdir/test_move.txt")
            .with_arg("overwrite", false);
        
        let move_result = filesystem_tool.execute(&move_params);
        assert!(move_result.is_ok());
        
        // Test delete operation
        let delete_params = ToolParams::new("delete".to_string())
            .with_arg("path", "testdir/test_copy.txt");
        
        let delete_result = filesystem_tool.execute(&delete_params);
        assert!(delete_result.is_ok());
    }
    
    #[test]
    fn test_tool_manager_execution() {
        // Create temporary directory
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        // Create tool manager
        let mut manager = ToolManager::new();
        
        // Register tools
        let web_tool = WebTool::new();
        let filesystem_tool = FileSystemTool::new(temp_path);
        
        manager.register_tool(web_tool).unwrap();
        manager.register_tool(filesystem_tool).unwrap();
        
        // Execute filesystem tool
        let write_params = ToolParams::new("write".to_string())
            .with_arg("path", "test.txt")
            .with_arg("content", "Hello from tool manager!")
            .with_arg("append", false);
        
        let write_result = manager.execute_tool("filesystem", &write_params);
        assert!(write_result.is_ok());
        
        // Execute web tool
        let parse_params = ToolParams::new("parse_html".to_string())
            .with_arg("html", "<html><body><h1>Test</h1></body></html>");
        
        let parse_result = manager.execute_tool("web", &parse_params);
        assert!(parse_result.is_ok());
        
        // Check execution log
        let log = manager.get_log();
        assert_eq!(log.len(), 2);
        
        // Check first log entry
        let first_entry = &log[0];
        assert_eq!(first_entry["tool_name"], Value::string("filesystem"));
        assert_eq!(first_entry["command"], Value::string("write"));
        assert_eq!(first_entry["status"], Value::string("success"));
        
        // Check second log entry
        let second_entry = &log[1];
        assert_eq!(second_entry["tool_name"], Value::string("web"));
        assert_eq!(second_entry["command"], Value::string("parse_html"));
        assert_eq!(second_entry["status"], Value::string("success"));
    }
}
```
