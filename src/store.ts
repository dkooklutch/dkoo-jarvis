import { create } from "zustand";
import type { ActionEntry, Health, JarvisState, Memory, Message, Project, Task } from "./types";

type Store = {
  ready: boolean; error?: string; firstRun: boolean; locked: boolean;
  state: JarvisState; health: Health; projects: Project[]; activeProjectId?: string;
  actions: ActionEntry[]; messages: Message[]; tasks: Task[]; memories: Memory[];
  activeView: string; commandPalette: boolean;
  hydrate: (data: Partial<Store>) => void;
  set: (data: Partial<Store>) => void;
};

const emptyHealth: Health = { core: "ONLINE", groq: "UNCONFIGURED", fishAudio: "UNCONFIGURED", wakeWord: "OFF", microphone: "OFF", project: "NONE" };

export const useJarvis = create<Store>((set) => ({
  ready: false, firstRun: true, locked: false, state: "IDLE", health: emptyHealth,
  projects: [], actions: [], messages: [], tasks: [], memories: [], activeView: "Core", commandPalette: false,
  hydrate: (data) => set({ ...data, ready: true, activeProjectId: data.projects?.[0]?.id }),
  set
}));

