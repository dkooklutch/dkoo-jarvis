export type JarvisState = "IDLE" | "WAKE WORD READY" | "LISTENING" | "TRANSCRIBING" | "THINKING" | "SEARCHING" | "CODING" | "RUNNING TESTS" | "SPEAKING" | "SUCCESS" | "ERROR" | "LOCKED";

export type Project = {
  id: string;
  name: string;
  root: string;
  cloudCodeAllowed: boolean;
  authorizedAt: string;
  branch?: string;
};

export type ActionEntry = {
  id: number;
  timestamp: string;
  actor: "USER" | "JARVIS" | "READ" | "EDIT" | "COMMAND" | "RESULT" | "SECURITY";
  actionType: string;
  project?: string;
  target?: string;
  result: string;
  success: boolean;
};

export type Health = {
  core: "ONLINE" | "LOCKED" | "ERROR";
  groq: "CONFIGURED" | "CONNECTED" | "UNCONFIGURED" | "ERROR";
  fishAudio: "CONFIGURED" | "CONNECTED" | "UNCONFIGURED" | "ERROR";
  wakeWord: "READY" | "OFF" | "ERROR";
  microphone: "OFF" | "READY" | "LISTENING";
  project: "READY" | "INDEXING" | "NONE";
};

export type Message = { id: string; role: "user" | "assistant" | "system"; content: string; createdAt: string };
export type Task = { id: string; title: string; projectId?: string; description: string; status: "TODO" | "IN PROGRESS" | "BLOCKED" | "DONE"; priority: "LOW" | "MEDIUM" | "HIGH"; updatedAt: string };
export type Memory = { id: string; projectId?: string; category: "PROJECT FACT" | "PREFERENCE" | "DECISION" | "CONSTRAINT" | "TODO"; content: string; updatedAt: string };
