import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { Message } from "../types";

export function Conversation({ messages }: { messages: Message[] }) {
  return <section className="conversation" aria-label="Conversation">
    {messages.length === 0 ? <div className="initial-brief"><span>LOCAL-FIRST ENGINEERING ASSISTANT</span><h2>Awaiting command.</h2><p>Authorize a project to inspect code, run controlled developer commands, and manage engineering work. No folders are scanned automatically.</p></div> : messages.map(m => <article className={`message ${m.role}`} key={m.id}><header>{m.role === "assistant" ? "JARVIS" : m.role.toUpperCase()}<time>{new Date(m.createdAt).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}</time></header><ReactMarkdown remarkPlugins={[remarkGfm]}>{m.content}</ReactMarkdown></article>)}
  </section>;
}

