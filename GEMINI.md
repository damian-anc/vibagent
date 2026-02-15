# Running VibAgent

This guide explains how to start the backend and frontend services.

## Backend (Rust)

The backend is a Rust application located in the root directory.

1.  Navigate to the root directory:
    ```bash
    cd /Users/damian/git/vibagent
    ```
2.  Run the application:
    ```bash
    cargo run --bin vibagent
    ```
    The server will start at [http://localhost:3000](http://localhost:3000).

## Frontend (Next.js)

The frontend is a Next.js application located in the `web` directory.

1.  Navigate to the `web` directory:
    ```bash
    cd /Users/damian/git/vibagent/web
    ```
2.  Install dependencies (if not already done):
    ```bash
    npm install
    ```
3.  Start the development server:
    ```bash
    npm run dev
    ```
    The frontend will typically be available at [http://localhost:3001](http://localhost:3001) (or port 3000 if available).
