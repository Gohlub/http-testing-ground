use hyperprocess_macro::hyperprocess;
use hyperware_app_common::{get_http_method, get_path, sleep};
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
        Binding::Http {
            path: "/health",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Ws {
            path: "/ws",
            config: WsBindingConfig::new(false, false, false),
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
        Binding::Http {
            path: "/api",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Http {
            path: "/admin",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Http {
            path: "/test",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Http {
            path: "/api/unknown",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Http {
            path: "/admin/dashboard",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Http {
            path: "/test/something",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Http {
            path: "/api/upload",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Http {
            path: "/other/endpoint",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Http {
            path: "/anything",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Http {
            path: "/whatever",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Http {
            path: "/some/path",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Http {
            path: "/users-slow",
            config: HttpBindingConfig::new(false, false, false, None),
        },
    ],
    save_config = hyperware_app_common::SaveOptions::EveryMessage,
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

    /// Toggle a todo task's completion status
    #[http]
    async fn toggle_task(&mut self, task_id: String) -> Result<TodoItem, String> {
        kiprintln!("Toggling task: {}", task_id);

        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.completed = !task.completed;
            kiprintln!("Task toggled: {:?}", task);
            Ok(task.clone())
        } else {
            Err(format!("Task with id '{}' not found", task_id))
        }
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

    /// Demo handler: POST /users-slow (with 5 second delay)
    #[http(method = "POST", path = "/users-slow")]
    async fn create_user_slow(&mut self, req: ApiRequest) -> Result<ApiResponse, String> {
        kiprintln!("POST /users-slow: {:?} - Starting 5 second delay", req);
        let sleep_res = sleep(5_000).await;
        if sleep_res.is_err() {
            return Err(format!("failed to sleep: {}", sleep_res.unwrap_err()));
        }
        kiprintln!("POST /users-slow: Delay complete, returning response");
        Ok(ApiResponse::new(&format!("Created user slowly: {}", req.message)))
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

/// Fallback handler for API GET requests - NO PATH, uses get_path() internally
#[http(method = "GET")]
fn handle_api_get_fallback(&mut self) -> ApiResponse {
    let path = get_path().unwrap_or_default();

    // Only handle paths we want to handle
    if path.starts_with("/api/") {
        kiprintln!("GET fallback for API: {}", path);
        ApiResponse::new(&format!("API GET fallback for {}", path))
    } else if path.starts_with("/admin/") {
        kiprintln!("GET fallback for admin: {}", path);
        ApiResponse::new(&format!("Admin GET fallback for {}", path))
    } else if path.starts_with("/test/") {
        kiprintln!("GET fallback for test: {}", path);
        ApiResponse::new(&format!("Test GET fallback for {}", path))
    } else {
        // This will let other handlers (like catch-all) handle it
        // But since this is a GET handler without path, it might interfere with UI
        // So we should be more specific about what we handle
        ApiResponse::new(&format!("Unexpected GET fallback for {}", path))
    }
}

/// Fallback handler for POST requests - NO PATH, uses get_path() internally
#[http(method = "POST")]
async fn handle_post_fallback(&mut self, req: ApiRequest) -> Result<ApiResponse, String> {
    let path = get_path().unwrap_or_default();
    kiprintln!("POST fallback for: {} with data: {:?}", path, req);

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

/// Catch-all for non-GET methods - NO PATH, uses get_path() internally
#[http(method = "PUT")]
fn handle_put_fallback(&mut self) -> ApiResponse {
    let path = get_path().unwrap_or_default();
    let method = get_http_method().unwrap_or_default();
    kiprintln!("{} {} catch-all", method, path);
    ApiResponse::new(&format!("Catch-all: {} {}", method, path))
}

#[http(method = "DELETE")]
fn handle_delete_fallback(&mut self) -> ApiResponse {
    let path = get_path().unwrap_or_default();
    let method = get_http_method().unwrap_or_default();
    kiprintln!("{} {} catch-all", method, path);
    ApiResponse::new(&format!("Catch-all: {} {}", method, path))
}

#[http(method = "PATCH")]
fn handle_patch_fallback(&mut self) -> ApiResponse {
    let path = get_path().unwrap_or_default();
    let method = get_http_method().unwrap_or_default();
    kiprintln!("{} {} catch-all", method, path);
    ApiResponse::new(&format!("Catch-all: {} {}", method, path))
}


}
