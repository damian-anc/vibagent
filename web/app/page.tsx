"use client";

import { useState, useRef, useEffect } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

type OutputEvent =
  | { OutputText: string }
  | { OutputToolCall: { id: string; name: string; arguments: string } }
  | { OutputToolCallDelta: { index: number; id?: string; name?: string; arguments?: string } }
  | { OutputToolResult: { id: string; result: string; is_error: boolean } }
  | { Error: string };

type ToolCall = {
  id: string;
  name: string;
  arguments: string;
  result?: string;
  isError?: boolean;
};

type Message = {
  role: "user" | "assistant";
  content: string;
  isError?: boolean;
  toolCalls?: ToolCall[];
};

export default function Home() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [selectedToolCallId, setSelectedToolCallId] = useState<string | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const toolListRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  // Extract all tool calls from messages into a flat list, preserving order
  const allToolCalls = messages.flatMap(msg => msg.toolCalls || []);
  const selectedToolCall = allToolCalls.find(tc => tc.id === selectedToolCallId);

  // Auto-scroll tool list to bottom when new tools are added
  useEffect(() => {
    if (toolListRef.current && !selectedToolCallId) {
      toolListRef.current.scrollTop = toolListRef.current.scrollHeight;
    }
  }, [allToolCalls.length, selectedToolCallId]);


  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isStreaming) return;

    const userMessage: Message = { role: "user", content: input };
    setMessages((prev) => [...prev, userMessage]);
    setInput("");
    setIsStreaming(true);

    try {
      const response = await fetch("http://localhost:3000/agent", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ UserInputEvent: input }),
      });

      if (!response.body) throw new Error("No response body");

      const reader = response.body.getReader();
      const decoder = new TextDecoder();
      let assistantMessage: Message = { role: "assistant", content: "" };

      setMessages((prev) => [...prev, assistantMessage]);

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const chunk = decoder.decode(value);
        const lines = chunk.split("\n");

        for (const line of lines) {
          if (line.startsWith("data: ")) {
            try {
              const event: OutputEvent = JSON.parse(line.slice(6));

              setMessages((prev) => {
                const newMessages = [...prev];
                const lastIdx = newMessages.length - 1;
                const msg = { ...newMessages[lastIdx] };

                if ("OutputText" in event) {
                  msg.content += event.OutputText;
                } else if ("OutputToolCall" in event) {
                  if (!msg.toolCalls) msg.toolCalls = [];
                  const existingIdx = msg.toolCalls.findIndex(tc => tc.id === event.OutputToolCall.id);

                  if (existingIdx !== -1) {
                    msg.toolCalls[existingIdx] = {
                      ...msg.toolCalls[existingIdx],
                      name: event.OutputToolCall.name,
                      arguments: event.OutputToolCall.arguments,
                    };
                  } else {
                    msg.toolCalls.push({
                      id: event.OutputToolCall.id,
                      name: event.OutputToolCall.name,
                      arguments: event.OutputToolCall.arguments,
                    });
                  }
                } else if ("OutputToolCallDelta" in event) {
                  if (!msg.toolCalls) msg.toolCalls = [];
                  const delta = event.OutputToolCallDelta;
                  const index = delta.index;

                  // Ensure tool call exists at index
                  if (!msg.toolCalls[index]) {
                    msg.toolCalls[index] = {
                      id: delta.id || "",
                      name: delta.name || "",
                      arguments: delta.arguments || "",
                    };
                  } else {
                    // Update existing
                    if (delta.id) msg.toolCalls[index].id = delta.id;
                    if (delta.name) msg.toolCalls[index].name += delta.name;
                    if (delta.arguments) msg.toolCalls[index].arguments += delta.arguments;
                  }
                } else if ("OutputToolResult" in event) {
                  const toolCall = msg.toolCalls?.find(tc => tc.id === event.OutputToolResult.id);
                  if (toolCall) {
                    toolCall.result = event.OutputToolResult.result;
                    toolCall.isError = event.OutputToolResult.is_error;
                  }
                } else if ("Error" in event) {
                  msg.content = event.Error;
                  msg.isError = true;
                }

                newMessages[lastIdx] = msg;
                return newMessages;
              });
            } catch (err) {
              console.error("Error parsing event:", err, line);
            }
          }
        }
      }
    } catch (error) {
      console.error("Error streaming:", error);
    } finally {
      setIsStreaming(false);
    }
  };

  return (
    <div className="flex h-screen bg-zinc-50 dark:bg-zinc-950 text-zinc-900 dark:text-zinc-50">
      {/* Left Column: Chat */}
      <div className="flex w-1/2 flex-col border-r border-zinc-200 dark:border-zinc-800">
        <header className="flex h-16 items-center border-b border-zinc-200 bg-white px-6 dark:border-zinc-800 dark:bg-zinc-900">
          <h1 className="text-xl font-bold">VibAgent</h1>
        </header>

        <div ref={scrollRef} className="flex-1 overflow-y-auto p-6">
          <div className="space-y-6">
            {messages.map((msg, i) => (
              <div key={i} className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}>
                <div className={`max-w-[85%] rounded-2xl px-4 py-3 shadow-sm ${msg.role === "user"
                  ? "bg-blue-600 text-white"
                  : msg.isError
                    ? "bg-red-50 text-red-900 border border-red-200 dark:bg-red-900/20 dark:text-red-200 dark:border-red-900/50"
                    : "bg-white text-zinc-900 dark:bg-zinc-900 dark:text-zinc-100"
                  }`}>
                  <div className="prose prose-zinc dark:prose-invert max-w-none">
                    <ReactMarkdown remarkPlugins={[remarkGfm]}>
                      {msg.content}
                    </ReactMarkdown>
                  </div>
                </div>
              </div>
            ))}
            {isStreaming && messages[messages.length - 1]?.content === "" && !messages[messages.length - 1]?.toolCalls && (
              <div className="flex justify-start">
                <div className="rounded-2xl bg-white px-4 py-3 shadow-sm dark:bg-zinc-900">
                  <div className="flex space-x-1">
                    <div className="h-2 w-2 animate-bounce rounded-full bg-zinc-400" />
                    <div className="h-2 w-2 animate-bounce rounded-full bg-zinc-400 [animation-delay:-0.15s]" />
                    <div className="h-2 w-2 animate-bounce rounded-full bg-zinc-400 [animation-delay:-0.3s]" />
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>

        <footer className="p-4 border-t border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900">
          <form onSubmit={handleSubmit} className="flex items-center gap-2 rounded-xl border border-zinc-200 bg-white p-2 shadow-sm focus-within:ring-2 focus-within:ring-blue-500 dark:border-zinc-700 dark:bg-zinc-800">
            <input
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              disabled={isStreaming}
              placeholder="Type your message..."
              className="flex-1 bg-transparent px-3 py-1 outline-none"
            />
            <button
              type="submit"
              disabled={isStreaming || !input.trim()}
              className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-semibold text-white transition hover:bg-blue-700 disabled:opacity-50"
            >
              Send
            </button>
          </form>
        </footer>
      </div>

      {/* Right Column: Tool Calls */}
      <div className="flex w-1/2 flex-col bg-zinc-100 dark:bg-zinc-900/50">
        <header className="flex h-16 items-center justify-between border-b border-zinc-200 px-6 dark:border-zinc-800">
          <h2 className="text-lg font-semibold">Tool Calls</h2>
          {selectedToolCallId && (
            <button
              onClick={() => setSelectedToolCallId(null)}
              className="text-sm text-blue-600 hover:text-blue-700 dark:text-blue-400"
            >
              Back to List
            </button>
          )}
        </header>

        <div ref={toolListRef} className="flex-1 overflow-y-auto p-4">
          {selectedToolCallId && selectedToolCall ? (
            // Detail View
            <div className="space-y-4">
              <div className="rounded-xl border border-zinc-200 bg-white p-6 shadow-sm dark:border-zinc-800 dark:bg-zinc-900">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-xl font-bold font-mono text-zinc-900 dark:text-zinc-100">{selectedToolCall.name}</h3>
                  <span className={`px-2 py-1 rounded text-xs font-semibold ${selectedToolCall.isError
                      ? "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300"
                      : selectedToolCall.result
                        ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300"
                        : "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300"
                    }`}>
                    {selectedToolCall.isError ? "Error" : selectedToolCall.result ? "Completed" : "Running"}
                  </span>
                </div>

                <div className="space-y-4">
                  <div>
                    <h4 className="text-sm font-semibold text-zinc-500 uppercase tracking-wider mb-2">Arguments</h4>
                    <pre className="bg-zinc-50 border border-zinc-100 p-4 rounded-lg overflow-x-auto text-sm font-mono dark:bg-zinc-950 dark:border-zinc-800 whitespace-pre-wrap break-all">
                      {selectedToolCall.arguments}
                    </pre>
                  </div>

                  {selectedToolCall.result && (
                    <div>
                      <h4 className="text-sm font-semibold text-zinc-500 uppercase tracking-wider mb-2">Result</h4>
                      <pre className={`p-4 rounded-lg overflow-x-auto text-sm font-mono whitespace-pre-wrap break-all ${selectedToolCall.isError
                          ? "bg-red-50 border border-red-100 text-red-700 dark:bg-red-900/10 dark:border-red-900/30 dark:text-red-300"
                          : "bg-zinc-50 border border-zinc-100 text-zinc-700 dark:bg-zinc-950 dark:border-zinc-800 dark:text-zinc-300"
                        }`}>
                        {selectedToolCall.result}
                      </pre>
                    </div>
                  )}
                </div>
              </div>
            </div>
          ) : (
            // List View
            <div className="space-y-3">
              {allToolCalls.length === 0 ? (
                <div className="text-center text-zinc-500 py-10">
                  No tool calls yet.
                </div>
              ) : (
                allToolCalls.map((tc, i) => (
                  <div
                    key={tc.id || i}
                    onClick={() => setSelectedToolCallId(tc.id)}
                    className="cursor-pointer group rounded-xl border border-zinc-200 bg-white p-4 shadow-sm transition-all hover:border-blue-300 hover:shadow-md dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-blue-700"
                  >
                    <div className="flex items-center justify-between mb-2">
                      <span className="font-mono font-medium text-blue-600 dark:text-blue-400 group-hover:underline">
                        {tc.name}
                      </span>
                      <span className={`h-2 w-2 rounded-full ${tc.isError
                          ? "bg-red-500"
                          : tc.result
                            ? "bg-green-500"
                            : "bg-blue-500 animate-pulse"
                        }`} />
                    </div>
                    <div className="text-xs text-zinc-500 line-clamp-2 font-mono break-all">
                      {tc.arguments}
                    </div>
                  </div>
                ))
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
