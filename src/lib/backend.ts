import { invoke } from "@tauri-apps/api/core";
import type { ActionEntry, Health, Memory, Message, Project, Task } from "../types";

const isTauri = () => "__TAURI_INTERNALS__" in window;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) throw new Error("Desktop backend is unavailable. Run with npm run desktop:dev.");
  return invoke<T>(command, args);
}

export const backend = {
  bootstrap: () => call<{ projects: Project[]; actions: ActionEntry[]; tasks: Task[]; memories: Memory[]; messages: Message[]; health: Health; locked: boolean; firstRun: boolean }>("bootstrap"),
  previewProject: (root: string) => call<{ canonicalRoot: string; name: string; isGit: boolean }>("preview_project", { root }),
  authorizeProject: (root: string, cloudCodeAllowed: boolean) => call<Project>("authorize_project", { root, cloudCodeAllowed }),
  revokeProject: (id: string) => call<void>("revoke_project", { id }),
  setLocked: (locked: boolean) => call<Health>("set_locked", { locked }),
  emergencyStop: () => call<void>("emergency_stop"),
  saveCredential: (provider: "groq" | "fish_audio", value: string) => call<string>("save_credential", { provider, value }),
  removeCredential: (provider: "groq" | "fish_audio") => call<void>("remove_credential", { provider }),
  credentialStatus: () => call<{ groq?: string; fishAudio?: string }>("credential_status"),
  sendMessage: (projectId: string | null, content: string) => call<Message>("send_message", { projectId, content }),
  listActions: () => call<ActionEntry[]>("list_actions"),
  listDirectory: (projectId: string, relativePath: string) => call<Array<{ name: string; relativePath: string; directory: boolean; modified: boolean }>>("list_directory", { projectId, relativePath }),
  readFile: (projectId: string, relativePath: string) => call<{ content: string; language: string; sha256: string }>("read_project_file", { projectId, relativePath }),
  writeFile: (projectId: string, relativePath: string, content: string, expectedSha256: string) => call<{ content: string; language: string; sha256: string }>("write_project_file", { projectId, relativePath, content, expectedSha256, userApproved: true }),
  searchProject: (projectId: string, query: string) => call<Array<{ relativePath: string; line: number; preview: string }>>("search_project", { projectId, query }),
  gitStatus: (projectId: string) => call<{ branch: string; modified: string[]; staged: string[]; untracked: string[] }>("git_status", { projectId }),
  gitDiff: (projectId: string) => call<string>("git_diff", { projectId }),
  runCommand: (projectId: string, program: string, args: string[]) => call<{ stdout: string; stderr: string; exitCode: number; durationMs: number }>("run_controlled_command", { projectId, program, args }),
  setVoiceListening: (enabled: boolean) => call<Health>("set_voice_listening", { enabled }),
  stopSpeaking: () => call<void>("stop_speaking"),
  addTask: (task: Omit<Task, "id" | "updatedAt">) => call<Task>("add_task", { task }),
  updateTaskStatus: (id: string, status: Task["status"]) => call<Task>("update_task_status", { id, status }),
  addMemory: (memory: Omit<Memory, "id" | "updatedAt">) => call<Memory>("add_memory", { memory }),
  deleteMemory: (id: string) => call<void>("delete_memory", { id }),
  completeOnboarding: () => call<void>("complete_onboarding"),
  getStartAtLogin: () => call<boolean>("get_start_at_login"),
  setStartAtLogin: (enabled: boolean) => call<boolean>("set_start_at_login", { enabled }),
  setFishVoiceId: (referenceId: string) => call<void>("set_fish_voice_id", { referenceId }),
  resetDataConfirmed: () => call<void>("reset_jarvis_data", { confirmation: "RESET JARVIS DATA" })
};
