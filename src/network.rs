use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use futures::stream::StreamExt;
use futures::SinkExt;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::str::FromStr;
use async_trait::async_trait;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio::sync::mpsc;
use tokio_tungstenite::WebSocketStream;
use futures::stream::{SplitSink, SplitStream};
use crate::error::LangError;
use tokio::sync::Semaphore;
use tokio::sync::OwnedSemaphorePermit;
use std::pin::Pin;
use std::future::Future;

const MAX_PORT: u16 = 65535;
const DEFAULT_TIMEOUT: u64 = 30;
const DEFAULT_RATE_LIMIT: u32 = 100; 
const DEFAULT_POOL_SIZE: usize = 32;
const PING_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Text,
    Binary,
    Ping,
    Pong,
    Close,
}

pub trait NetworkHandler: Send + Sync {
    fn handle<'life0>(
        &'life0 self,
        data: Vec<u8>,
        msg_type: MessageType,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + Send + 'life0>>;
}

pub struct DefaultHandler;

impl NetworkHandler for DefaultHandler {
    fn handle<'life0>(
        &'life0 self,
        data: Vec<u8>,
        _msg_type: MessageType,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + Send + 'life0>> {
        Box::pin(async move {
            Ok(data)
        })
    }
}

pub struct Network {
    http_client: Client,
    ws_connections: Arc<Mutex<HashMap<String, WebSocketStream<TcpStream>>>>,
    connection_pool: Arc<Semaphore>,
    connections: Arc<Mutex<HashMap<u16, Arc<TcpListener>>>>,
    handlers: Arc<Mutex<Vec<Box<dyn NetworkHandler>>>>
}

impl Network {
    pub fn new() -> Self {
        Network {
            http_client: Client::new(),
            ws_connections: Arc::new(Mutex::new(HashMap::new())),
            connection_pool: Arc::new(Semaphore::new(DEFAULT_POOL_SIZE)),
            connections: Arc::new(Mutex::new(HashMap::new())),
            handlers: Arc::new(Mutex::new(vec![Box::new(DefaultHandler)]))
        }
    }

    pub async fn send_request(&self, url: &str, method: &str, body: Option<String>, headers: Option<HashMap<String, String>>) -> Result<String, LangError> {
        let mut request = match method {
            "GET" => self.http_client.get(url),
            "POST" => self.http_client.post(url),
            "PUT" => self.http_client.put(url),
            "DELETE" => self.http_client.delete(url),
            _ => return Err(LangError::NetworkError("Invalid HTTP method".to_string()))
        };

        if let Some(body) = body {
            request = request.body(body);
        }

        if let Some(headers) = headers {
            let mut header_map = HeaderMap::new();
            for (key, value) in headers {
                if let (Ok(name), Ok(val)) = (HeaderName::from_str(&key), HeaderValue::from_str(&value)) {
                    header_map.insert(name, val);
                }
            }
            request = request.headers(header_map);
        }

        let response = request.send().await.map_err(|e| LangError::NetworkError(e.to_string()))?;
        let text = response.text().await.map_err(|e| LangError::NetworkError(e.to_string()))?;
        
        Ok(text)
    }

    pub async fn connect_websocket(&self, url: &str) -> Result<(), LangError> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(&url)
            .await
            .map_err(|e| LangError::NetworkError(e.to_string()))?;

        let mut connections = self.ws_connections.lock().await;
        connections.insert(url.to_string(), ws_stream);
        
        Ok(())
    }

    pub async fn send_websocket(&self, url: &str, message: &str) -> Result<(), LangError> {
        let mut connections = self.ws_connections.lock().await;
        if let Some(ws_stream) = connections.get_mut(url) {
            ws_stream.send(Message::Text(message.to_string()))
                .await
                .map_err(|e| LangError::NetworkError(e.to_string()))?;
            Ok(())
        } else {
            Err(LangError::NetworkError("WebSocket not connected".to_string()))
        }
    }

    pub async fn close_websocket(&self, url: &str) -> Result<(), LangError> {
        let mut connections = self.ws_connections.lock().await;
        if let Some(mut ws_stream) = connections.remove(url) {
            ws_stream.close(None)
                .await
                .map_err(|e| LangError::NetworkError(e.to_string()))?;
            Ok(())
        } else {
            Err(LangError::NetworkError("WebSocket not connected".to_string()))
        }
    }

    pub async fn listen(&self, port: u16) -> Result<Arc<TcpListener>, LangError> {
        let _permit = self.connection_pool.acquire().await
            .map_err(|e| LangError::NetworkError(e.to_string()))?;

        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .map_err(|e| LangError::NetworkError(e.to_string()))?;

        let listener = Arc::new(listener);
        self.connections.lock().await.insert(port, Arc::clone(&listener));

        Ok(listener)
    }

    pub async fn handle_connection(
        &self,
        stream: TcpStream,
        handlers: Arc<Mutex<Vec<Box<dyn NetworkHandler>>>>,
        _permit: OwnedSemaphorePermit
    ) -> Result<(), LangError> {
        let ws_stream = match tokio_tungstenite::accept_async(stream).await {
            Ok(ws_stream) => ws_stream,
            Err(e) => return Err(LangError::NetworkError(e.to_string()))
        };

        let (mut write, half) = ws_stream.split();

        while let Some(msg) = half.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    break;
                }
            };

            let msg_type = match msg {
                Message::Text(_) => MessageType::Text,
                Message::Binary(_) => MessageType::Binary,
                Message::Ping(_) => MessageType::Ping,
                Message::Pong(_) => MessageType::Pong,
                Message::Close(_) => MessageType::Close,
                _ => continue,
            };

            let data = match msg {
                Message::Text(s) => s.into_bytes(),
                Message::Binary(b) => b,
                Message::Ping(p) => p,
                Message::Pong(p) => p,
                Message::Close(_) => break,
                _ => continue,
            };

            let handlers = handlers.lock().await;
            for handler in handlers.iter() {
                let result = handler.handle(data.clone(), msg_type.clone()).await;
                
                match result {
                    Ok(response) => {
                        if let Err(e) = write.send(Message::Binary(response)).await {
                            eprintln!("Error sending response: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error handling message: {}", e);
                        break;
                    }
                }
            }

            if msg_type == MessageType::Close {
                break;
            }
        }

        Ok(())
    }

    pub async fn close(&self, port: u16) -> Result<(), LangError> {
        let mut connections = self.connections.lock().await;
        if let Some(_) = connections.remove(&port) {
            Ok(())
        } else {
            Err(LangError::NetworkError("Port not found".to_string()))
        }
    }
}
