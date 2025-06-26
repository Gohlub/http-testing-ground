import { useState, useEffect, useCallback, useRef } from "react";
import "./App.css";
import useTodoStore from "./store/todo";
import { 
  TodoItem,
  AddTaskRequest, 
  GetTasksRequest,
  ToggleTaskRequest,
  AddTaskResponse,
  GetTasksResponse,
  ToggleTaskResponse
} from "./types/todo";

const BASE_URL = import.meta.env.BASE_URL;
if (window.our) window.our.process = BASE_URL?.replace("/", "");

const PROXY_TARGET = `${(import.meta.env.VITE_NODE_URL || "http://localhost:8080")}${BASE_URL}`;

// WebSocket URL for raw connection
const WEBSOCKET_URL = `ws://localhost:8080${BASE_URL}/ws`;

console.log('BASE_URL:', BASE_URL);
console.log('PROXY_TARGET:', PROXY_TARGET);
console.log('WEBSOCKET_URL:', WEBSOCKET_URL);

function App() {
  const { tasks, setTasks } = useTodoStore();
  const [nodeConnected, setNodeConnected] = useState(true);
  const [wsConnected, setWsConnected] = useState(false);
  const [newTaskText, setNewTaskText] = useState("");
  const wsRef = useRef<WebSocket | null>(null);

  // Send message via WebSocket
  const sendWsMessage = useCallback((message: any) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    }
  }, []);

  const fetchTasks = useCallback(async () => {
    // If WebSocket is connected, request tasks via WebSocket
    if (wsConnected && wsRef.current) {
      sendWsMessage({ action: "get_tasks" });
      return;
    }

    // Otherwise fall back to HTTP
    const requestData: GetTasksRequest = { GetTasks: "" };

    try {
      const result = await fetch(`${BASE_URL}/api`, {
        method: "POST",
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(requestData), 
      });

      if (!result.ok) {
        const errorText = await result.text();
        console.error(`HTTP request failed: ${result.status} ${result.statusText}. Response:`, errorText);
        throw new Error(`HTTP request failed: ${result.statusText}`);
      }
      
      const responseData = await result.json() as GetTasksResponse; 
      
      if (responseData.Ok) {
        console.log("Fetched tasks:", responseData.Ok); 
        setTasks(responseData.Ok); 
      } else {
        console.error("Error fetching tasks:", responseData.Err || "Unknown error"); 
        setTasks([]);
      }
    } catch (error) {
      console.error("Failed to fetch tasks:", error);
      setTasks([]);
    }
  }, [wsConnected, sendWsMessage, setTasks]);

  const handleAddTask = useCallback(async () => {
    if (!newTaskText.trim()) return;

    // If WebSocket is connected, send via WebSocket
    if (wsConnected && wsRef.current) {
      sendWsMessage({
        action: "add_task",
        text: newTaskText 
      });
      setNewTaskText("");
      return;
    }

    // Otherwise fall back to HTTP
    const requestData: AddTaskRequest = { AddTask: newTaskText };

    try {
      const result = await fetch(`${BASE_URL}/api`, {
        method: "POST",
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(requestData),
      });

      if (!result.ok) {
        const errorText = await result.text();
        console.error(`HTTP request failed: ${result.status} ${result.statusText}. Response:`, errorText);
        throw new Error(`HTTP request failed: ${result.statusText}`);
      }

      const responseData = await result.json() as AddTaskResponse;

      if (responseData.Ok) { 
        console.log("Task added successfully:", responseData.Ok);
        setNewTaskText("");
        fetchTasks();
      } else {
        console.error("Error adding task:", responseData.Err || "Unknown error");
      }
    } catch (error) {
      console.error("Failed to add task:", error);
    }
  }, [newTaskText, wsConnected, sendWsMessage, fetchTasks]);

  const handleToggleComplete = useCallback(async (taskId: string) => {
    // If WebSocket is connected, send via WebSocket
    if (wsConnected && wsRef.current) {
      sendWsMessage({
        action: "toggle_task",
        id: taskId 
      });
      return;
    }

    // Otherwise fall back to HTTP
    const requestData: ToggleTaskRequest = { ToggleTask: taskId };

    try {
        const result = await fetch(`${BASE_URL}/api`, {
            method: "POST",
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(requestData),
        });

        if (!result.ok) {
            const errorText = await result.text();
            console.error(`HTTP request failed: ${result.status} ${result.statusText}. Response:`, errorText);
            throw new Error(`HTTP request failed: ${result.statusText}`);
        }

        const responseData = await result.json() as ToggleTaskResponse;

        if (responseData.Ok) {
            console.log("Task toggled successfully:", responseData.Ok);
            fetchTasks();
        } else {
            console.error("Error toggling task:", responseData.Err || "Unknown error");
        }
    } catch (error) {
        console.error("Failed to toggle task:", error);
    }
  }, [wsConnected, sendWsMessage, fetchTasks]);

  // Setup WebSocket connection
  useEffect(() => {
    // Initial fetch only if not using WebSocket
    if (!wsConnected) {
      fetchTasks(); 
    }

    // Create WebSocket connection
    const ws = new WebSocket(WEBSOCKET_URL);
    wsRef.current = ws;

    ws.onopen = (event) => {
      console.log("WebSocket connection opened:", event);
      setWsConnected(true);
      // Subscribe to task updates
      sendWsMessage({ action: "subscribe" });
    };

    ws.onmessage = (event) => {
      console.log('WebSocket message received:', event.data);
      try {
        const data = JSON.parse(event.data);
        console.log("Parsed WebSocket message:", data);
        
        // Handle different message types
        if (data.type === "tasks_update" || data.type === "task_added" || data.type === "task_toggled") {
          if (data.tasks) {
            console.log("Updating tasks from WebSocket:", data.tasks);
            setTasks(data.tasks);
          }
        }
      } catch (error) {
        console.error("Error parsing WebSocket message:", error);
      }
    };

    ws.onclose = (event) => {
      console.log("WebSocket connection closed:", event);
      setWsConnected(false);
    };

    ws.onerror = (error) => {
      console.error("WebSocket error:", error);
      setWsConnected(false);
    };

    // Cleanup
    return () => {
      console.log("Closing WebSocket connection");
      if (ws.readyState === WebSocket.OPEN) {
        sendWsMessage({ action: "unsubscribe" });
        ws.close();
      }
    };
  }, []); // Empty dependencies to run only once

  return (
    <div style={{ width: "100%" }}>
      <div style={{ position: "absolute", top: 4, left: 8 }}>
        ID: <strong>{window.our?.node}</strong>
        {wsConnected && (
          <span style={{ marginLeft: '10px', color: 'green' }}>
            ● WebSocket Connected
          </span>
        )}
      </div>
      {!nodeConnected && (
        <div className="node-not-connected">
          <h2 style={{ color: "red" }}>Node not connected</h2>
          <h4>
            Check console. Connection to {PROXY_TARGET} might be needed.
          </h4>
        </div>
      )}
      <h2>Todo List</h2>
      <div className="card">
        <div className="input-row" style={{ marginBottom: '1em' }}>
          <input 
            type="text" 
            value={newTaskText} 
            onChange={(e) => setNewTaskText(e.target.value)} 
            placeholder="Enter new task..."
            onKeyDown={(e) => e.key === 'Enter' && handleAddTask()}
          />
          <button onClick={handleAddTask}>Add Task</button>
        </div>
        <div style={{ border: "1px solid #ccc", padding: "1em", borderRadius: '0.25em' }}>
          <h3 style={{ marginTop: 0, textAlign: 'left' }}>Tasks</h3>
          <div>
            {tasks.length > 0 ? (
              <ul className="task-list"> 
                {tasks.map((task) => (
                  <li key={task.id} className={`task-item ${task.completed ? 'completed' : ''}`}>
                    <input 
                      type="checkbox"
                      checked={task.completed}
                      onChange={() => handleToggleComplete(task.id)}
                      style={{ marginRight: '0.5em' }}
                    />
                    <span className="task-text">{task.text}</span>
                  </li>
                ))}
              </ul>
            ) : (
              <p>No tasks yet. Add one above!</p>
            )}
          </div>
          <button onClick={fetchTasks} style={{ marginTop: '1em' }}>Refresh Tasks</button> 
        </div>
      </div>
    </div>
  );
}

export default App;
