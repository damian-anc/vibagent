# VibAgent

VibAgent is a powerful, Rust-based AI agent designed to perform tasks using a suite of tools. It features a high-performance backend processing loop and a modern, real-time web interface.

![VibAgent Chat UI](/Users/damian/.gemini/antigravity/brain/483391f7-d4ff-4bb5-8e5e-0bf093c6b18a/completed_conversation_1770512515549.png)

## 🚀 Features

-   **Rust Core**: Built with Rust for speed, safety, and reliability.
-   **Tool Use**: Equipped with tools to perform real-world actions:
    -   **Calculator**: Perform mathematical calculations.
    -   **Web Search**: Search the internet for real-time information.
    -   **Command Runner**: Execute terminal commands (sandboxed/safe mode recommended).
-   **HTTP API**: Exposes a streaming API (`POST /agent`) using `Axum` and Server-Sent Events (SSE).
-   **Structured Logging**: Comprehensive tracing with `INFO` and `DEBUG` levels to monitor requests and tool execution.
-   **Modern Frontend**: A premium Next.js web interface for interacting with the agent.
    -   Real-time text streaming.
    -   Visualized tool calls and results.
    -   Dark mode support.

## 🛠️ Architecture

### Backend (`/src`)
The backend is written in Rust and uses:
-   `tokio` for asynchronous runtime.
-   `axum` for the HTTP server.
-   `reqwest` for calling LLM APIs (OpenRouter).
-   `tracing` for structured logging.

### Frontend (`/web`)
The frontend is built with:
-   **Next.js 15+** (App Router)
-   **Tailwind CSS** for styling
-   **TypeScript** for type safety
-   **Lucide React** for icons

## 📦 Getting Started

### Prerequisites
-   Rust (latest stable)
-   Node.js (v18+) and npm
-   OpenRouter API Key

### Installation

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/your-username/vibagent.git
    cd vibagent
    ```

2.  **Set up Environment**:
    Create a `.env` file in the root directory:
    ```env
    OPENROUTER_API_KEY=your_api_key_here
    AGENT_MODEL=arcee-ai/trinity-large-preview:free
    ```

### Running the Application

You need to run both the backend and the frontend.

**1. Start the Backend Server**
```bash
# In the project root
cargo run --bin vibagent

# Start with verbose logging
cargo run --bin vibagent -- -v
```
The server will start on `http://localhost:3000`.

**2. Start the Frontend**
```bash
# In a new terminal, navigate to the web directory
cd web
npm install
npm run dev -- -p 3001
```
The web interface will be available at `http://localhost:3001`.

## 🔌 API Usage

You can also interact with the agent directly via the API:

```bash
curl -N -X POST http://localhost:3000/agent \
  -H "Content-Type: application/json" \
  -d '{"UserInputEvent": "Calculate 123 + 456"}'
```

Returns a stream of Server-Sent Events (SSE) containing:
-   `OutputText`: Text segments from the LLM.
-   `OutputToolCall`: Details of tool calls.
-   `OutputToolResult`: Results from executed tools.

## 📝 License

This project is licensed under the MIT License.
