use hyperprocess_macro::hyperprocess;
use hyperware_app_common::{get_http_method, get_path};
use hyperware_process_lib::http::server::{send_ws_push, WsMessageType};
use hyperware_process_lib::{kiprintln, LazyLoadBlob};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid; 

// =============================================================================
// CORE TODO APPLICATION DATA STRUCTURES
// =============================================================================

/// Core todo item with unique ID, text content, and completion status
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct TodoItem {
    id: String,
    text: String,
    completed: bool,
}

/// Legacy response structure (kept for compatibility)
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    pub data: NestedData,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct NestedData {
    pub items: Vec<Item>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
}

// =============================================================================
// TESTING/DEMO DATA STRUCTURES
// =============================================================================

/// Generic request structure for API testing and demos
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRequest {
    pub message: String,
    pub id: Option<u32>,
}

/// Generic response structure for API testing and demos
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: String,
    pub data: String,
    pub path: Option<String>,
    pub method: Option<String>,
}

impl ApiResponse {
    fn new(data: &str) -> Self {
        Self {
            status: "success".to_string(),
            data: data.to_string(),
            path: get_path(),
            method: get_http_method(),
        }
    }
}

// =============================================================================
// APPLICATION STATE
// =============================================================================

/// Main application state containing todo tasks and WebSocket connections
#[derive(PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
pub struct TodoState {
    /// List of todo tasks
    tasks: Vec<TodoItem>,
    /// Active WebSocket channel IDs (not serialized)
    #[serde(skip)]
    ws_channels: HashSet<u32>,
}

// =============================================================================
// HYPERPROCESS CONFIGURATION
// =============================================================================

#[hyperprocess(
    name = "todo",
    ui = Some(HttpBindingConfig::default()),
    endpoints = vec![
        // Core application endpoints
        Binding::Http {
            path: "/",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Http {
            path: "/health",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        
        // Demo/testing endpoints
        Binding::Http {
            path: "/users",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Http {
            path: "/posts",
            config: HttpBindingConfig::new(false, false, false, None),
        },
    ],
    save_config = SaveOptions::EveryMessage,
    wit_world = "todo-template-dot-os-v0"
)]

// =============================================================================
// CORE TODO APPLICATION IMPLEMENTATION
// =============================================================================

impl TodoState {
    /// Initialize the application state
    #[init]
    async fn initialize(&mut self) {
        kiprintln!("Initializing todo list state");
        self.tasks = Vec::new();
        self.ws_channels = HashSet::new();
    }

    // -------------------------------------------------------------------------
    // CORE TODO FUNCTIONALITY
    // -------------------------------------------------------------------------

    /// Add a new todo task
    #[http]
    async fn add_task(&mut self, text: String) -> Result<TodoItem, String> {
        if text.trim().is_empty() {
            return Err("Task text cannot be empty".to_string());
        }
        
        let new_task = TodoItem {
            id: Uuid::new_v4().to_string(),
            text,
            completed: false,
        };
        
        self.tasks.push(new_task.clone());
        kiprintln!("Added task: {:?}", new_task);

        Ok(new_task)
    }

    /// Get all todo tasks
    #[http]
    async fn get_tasks(&self, request: String) -> Result<Vec<TodoItem>, String> {
        kiprintln!("Request: {:?}", request);
        kiprintln!("Fetching tasks");
        Ok(self.tasks.clone())
    }

    #[ws]
    fn websocket(&mut self, channel_id: u32, message_type: WsMessageType, blob: LazyLoadBlob) {
        match message_type {
            WsMessageType::Text => {
                // Get the message from the blob
                if let Ok(message) = String::from_utf8(blob.bytes.clone()) {
                    kiprintln!("Received WebSocket text message: {}", message);
                    // Parse the message as JSON
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&message) {
                        // Handle different message types
                        if let Some(action) = json.get("action").and_then(|v| v.as_str()) {
                            match action {
                                "get_tasks" => {
                                    // Send current tasks to the requesting client
                                    let response = serde_json::json!({
                                        "type": "tasks_overview",
                                        "tasks": self.tasks
                                    });

                                    let response_bytes = response.to_string().into_bytes();

                                    let response_blob = LazyLoadBlob {
                                        mime: Some("application/json".to_string()),
                                        bytes: response_bytes,
                                    };
                                    send_ws_push(channel_id, WsMessageType::Text, response_blob);
                                }
                                "add_task" => {
                                    if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
                                        if !text.trim().is_empty() {
                                            let new_task = TodoItem {
                                                id: Uuid::new_v4().to_string(),
                                                text: text.to_string(),
                                                completed: false,
                                            };
                                            self.tasks.push(new_task.clone());

                                            // Broadcast the update to all connected clients
                                            let broadcast = serde_json::json!({
                                                "type": "task_added",
                                                "task": new_task,
                                                "tasks": self.tasks
                                            });
                                            let response_bytes = broadcast.to_string().into_bytes();

                                            let response_blob = LazyLoadBlob {
                                                mime: Some("application/json".to_string()),
                                                bytes: response_bytes,
                                            };
                                            send_ws_push(
                                                channel_id,
                                                WsMessageType::Text,
                                                response_blob,
                                            );
                                        }
                                    }
                                }
                                "toggle_task" => {
                                    if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                                        if let Some(task) =
                                            self.tasks.iter_mut().find(|t| t.id == id)
                                        {
                                            task.completed = !task.completed;

                                            // Broadcast the update to all connected clients
                                            let broadcast = serde_json::json!({
                                                "type": "task_toggled",
                                                "task": task.clone(),
                                                "tasks": self.tasks
                                            });
                                            let response_bytes = broadcast.to_string().into_bytes();

                                            let response_blob = LazyLoadBlob {
                                                mime: Some("application/json".to_string()),
                                                bytes: response_bytes,
                                            };
                                            send_ws_push(
                                                channel_id,
                                                WsMessageType::Text,
                                                response_blob,
                                            );
                                        }
                                    }
                                }
                                _ => {
                                    println!("Unknown WebSocket action: {}", action);
                                }
                            }
                        }
                    }
                }
            }
            WsMessageType::Binary => {
                println!("Received WebSocket binary message");
            }
            WsMessageType::Ping => {
                println!("Received WebSocket ping message");
            }
            WsMessageType::Pong => {
                println!("Received WebSocket pong message");
            }
            WsMessageType::Close => {
                println!("Received WebSocket close message");
            }
        }
    }


    // =============================================================================
    // TESTING & DEMO HANDLERS (for HTTP routing validation)
    // =============================================================================
    


    /// Demo handler: GET /users
    #[http(method = "GET", path = "/users")]
    fn get_users(&mut self) -> ApiResponse {
        kiprintln!("GET /users");
        ApiResponse::new("List of users")
    }

    /// Demo handler: POST /users (with parameters)
    #[http(method = "POST", path = "/users")]
    async fn create_user(&mut self, req: ApiRequest) -> Result<ApiResponse, String> {
        kiprintln!("POST /users: {:?}", req);
        Ok(ApiResponse::new(&format!("Created user: {}", req.message)))
    }

    /// Demo handler: GET /posts
    #[http(method = "GET", path = "/posts")]
    fn get_posts(&mut self) -> ApiResponse {
        kiprintln!("GET /posts");
        ApiResponse::new("List of posts")
    }

    /// Demo handler: POST /api/data (with parameters)
    #[http(method = "POST", path = "/api/data")]
    async fn process_data(&mut self, req: ApiRequest) -> Result<ApiResponse, String> {
        kiprintln!("POST /api/data: {:?}", req);
        Ok(ApiResponse::new(&format!("Processed: {}", req.message)))
    }

    // -------------------------------------------------------------------------
    // DYNAMIC ROUTING HANDLERS (for testing fallback behavior)
    // -------------------------------------------------------------------------

    /// Fallback handler for all GET requests not matched by specific handlers
    #[http(method = "GET")]
    fn handle_get_fallback(&mut self) -> ApiResponse {
        let path = get_path().unwrap_or_default();
        kiprintln!("GET fallback for: {}", path);

        // Demonstrate path-based routing logic within fallback
        match path.as_str() {
            p if p.starts_with("/api/") => {
                ApiResponse::new(&format!("API GET fallback for {}", p))
            }
            p if p.starts_with("/admin/") => {
                ApiResponse::new(&format!("Admin GET fallback for {}", p))
            }
            _ => ApiResponse::new(&format!("General GET fallback for {}", path)),
        }
    }

    /// Fallback handler for all POST requests with parameters not matched by specific handlers
    #[http(method = "POST")]
    async fn handle_post_fallback(&mut self, req: ApiRequest) -> Result<ApiResponse, String> {
        let path = get_path().unwrap_or_default();
        kiprintln!("POST fallback for: {} with data: {:?}", path, req);

        // Demonstrate path-based routing logic for POST with parameters
        match path.as_str() {
            p if p.starts_with("/api/") => Ok(ApiResponse::new(&format!(
                "API POST to {} with: {}",
                p, req.message
            ))),
            _ => Ok(ApiResponse::new(&format!(
                "General POST to {} with: {}",
                path, req.message
            ))),
        }
    }

    /// Ultimate catch-all handler for any unmatched method/path combination
    #[http]
    fn handle_any_method(&mut self) -> ApiResponse {
        let path = get_path().unwrap_or_default();
        let method = get_http_method().unwrap_or_default();
        kiprintln!("{} {} catch-all", method, path);

        ApiResponse::new(&format!("Catch-all: {} {}", method, path))
    }
 

}
