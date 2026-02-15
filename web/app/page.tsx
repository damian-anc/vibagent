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

type Message = {
  role: "user" | "assistant";
  content: string;
  isError?: boolean;
  toolCalls?: Array<{
    id: string;
    name: string;
    arguments: string;
    result?: string;
    isError?: boolean;
  }>;
};

export default function Home() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

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
    <div className="flex h-screen flex-col bg-zinc-50 dark:bg-zinc-950">
      <header className="flex h-16 items-center border-b border-zinc-200 bg-white px-6 dark:border-zinc-800 dark:bg-zinc-900">
        <h1 className="text-xl font-bold text-zinc-900 dark:text-zinc-50">VibAgent</h1>
      </header>

      <div ref={scrollRef} className="flex-1 overflow-y-auto p-6 transition-all duration-300">
        <div className="mx-auto max-w-3xl space-y-6">
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

                {msg.toolCalls && msg.toolCalls.length > 0 && (
                  <div className="mt-3 space-y-3 border-t border-zinc-100 pt-3 dark:border-zinc-800">
                    {msg.toolCalls.map((tc, j) => (
                      <div key={j} className="rounded-lg bg-zinc-50 p-3 text-sm dark:bg-zinc-800/50">
                        <div className={`flex items-center gap-2 font-mono font-medium ${tc.isError ? "text-red-600 dark:text-red-400" : "text-blue-600 dark:text-blue-400"}`}>
                          <span className={`h-2 w-2 rounded-full animate-pulse ${tc.isError ? "bg-red-600" : "bg-blue-600"}`} />
                          Tool Call: {tc.name} {tc.isError && "(Failed)"}
                        </div>
                        <div className="mt-1 text-zinc-500 line-clamp-2">{tc.arguments}</div>
                        {tc.result && (
                          <div className={`mt-2 border-t pt-2 ${tc.isError ? "border-red-200 dark:border-red-800" : "border-zinc-200 dark:border-zinc-700"}`}>
                            <div className={`font-semibold ${tc.isError ? "text-red-700 dark:text-red-300" : "text-zinc-700 dark:text-zinc-300"}`}>Result:</div>
                            <pre className={`mt-1 max-h-40 overflow-auto whitespace-pre-wrap font-mono text-xs ${tc.isError ? "text-red-600 dark:text-red-400" : "text-zinc-600 dark:text-zinc-400"}`}>
                              {tc.result}
                            </pre>
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                )}
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

      <footer className="p-6">
        <form onSubmit={handleSubmit} className="mx-auto flex max-w-3xl items-center gap-2 overflow-hidden rounded-2xl border border-zinc-200 bg-white p-2 shadow-lg transition-focus focus-within:ring-2 focus-within:ring-blue-500 dark:border-zinc-800 dark:bg-zinc-900">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            disabled={isStreaming}
            placeholder="Type your message..."
            className="flex-1 bg-transparent px-4 py-2 outline-none text-zinc-900 dark:text-zinc-100"
          />
          <button
            type="submit"
            disabled={isStreaming || !input.trim()}
            className="rounded-xl bg-blue-600 px-6 py-2 text-sm font-semibold text-white transition hover:bg-blue-700 disabled:bg-zinc-400 dark:disabled:bg-zinc-800"
          >
            Send
          </button>
        </form>
      </footer>
    </div>
  );
}
