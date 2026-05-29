import { create } from "zustand";
import { useSettingsStore } from "./settingsStore";
import { useChatStore } from "./chatStore";
import { DEFAULT_GATEWAY_URL } from "../lib/config";

/** Single workspace directory entry — matches Gateway API response */
interface WorkspaceDir {
  id: string;
  path: string;
  alias: string | null;
  access: "read-only" | "read-write";
  added_at: string;
  /** Deprecated: replaced by session-level workspace selection (sessionWorkspaceMap). */
  is_current?: boolean;
  /** Legacy field for backward compat; frontend reads sessionWorkspaceMap instead. */
  last_active?: boolean;
  select_count: number;
  last_selected_at: string | null;
}

interface WorkspaceState {
  workspaces: WorkspaceDir[];
  /** Per-session current workspace selection. "__agent_home__" = agent home. */
  sessionWorkspaceMap: Record<string, string>;
  loading: boolean;

  // Fetch workspace list for a given agent
  fetchWorkspaces: (agentId: string) => Promise<void>;

  // Set current workspace for a specific session (preferred API)
  setSessionWorkspace: (agentId: string, sessionId: string, workspaceId: string) => Promise<void>;

  // Legacy: set current workspace using the active session (backward compat)
  setCurrentWorkspace: (agentId: string, workspaceId: string) => Promise<void>;

  // Synchronous local-only setter — used by chatStore/sessionStore to keep
  // sessionWorkspaceMap consistent without an API roundtrip.
  setSessionWorkspaceLocal: (sessionId: string, workspaceId: string) => void;

  // Bulk-sync session workspaces from fetchSessions / activate_session.
  // Accepts the raw session list; removes stale entries automatically.
  syncSessionWorkspaces: (sessions: Array<{ session_id: string; workspace_id?: string | null }>) => void;

  // Get current workspace ID for a session (defaults to "__agent_home__")
  getSessionWorkspaceId: (sessionId: string) => string;

  // Clear state on agent switch
  reset: () => void;
}

/** Helper: resolve Gateway URL from settings store, fallback to default */
function getGatewayUrl(): string {
  return useSettingsStore.getState().gatewayUrl || DEFAULT_GATEWAY_URL;
}

/** Monotonic counter to discard stale async responses (race-condition guard) */
let requestSeq = 0;

export const useWorkspaceStore = create<WorkspaceState>((set, get) => ({
  workspaces: [],
  sessionWorkspaceMap: {},
  loading: false,

  fetchWorkspaces: async (agentId: string) => {
    const seq = ++requestSeq;
    set({ loading: true });
    try {
      const baseUrl = getGatewayUrl();
      const resp = await fetch(`${baseUrl}/api/agents/${agentId}/workspaces`);
      if (!resp.ok) {
        console.error("[WorkspaceStore] fetchWorkspaces failed:", resp.status, resp.statusText);
        set({ loading: false });
        return;
      }
      const data = (await resp.json()) as { workspaces: WorkspaceDir[] };
      const workspaces = data.workspaces || [];
      // Discard stale response if a newer request has been issued
      if (seq !== requestSeq) return;
      set({
        workspaces,
        loading: false,
      });
    } catch (e) {
      console.error("[WorkspaceStore] fetchWorkspaces error:", e);
      if (seq !== requestSeq) return;
      set({ loading: false });
    }
  },

  setSessionWorkspace: async (agentId: string, sessionId: string, workspaceId: string) => {
    const seq = ++requestSeq;
    const prevWorkspaces = get().workspaces;
    const prevMap = { ...get().sessionWorkspaceMap };
    try {
      const baseUrl = getGatewayUrl();
      const resp = await fetch(
        `${baseUrl}/api/agents/${agentId}/workspaces/current?session_id=${encodeURIComponent(sessionId)}`,
        {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ workspace_id: workspaceId }),
        },
      );
      if (!resp.ok) {
        console.error("[WorkspaceStore] setSessionWorkspace failed:", resp.status, resp.statusText);
        return;
      }
      // API returns the updated workspace list after switching
      const data = (await resp.json()) as { workspaces: WorkspaceDir[] };
      const workspaces = data.workspaces || [];
      // Discard stale response if a newer request has been issued
      if (seq !== requestSeq) return;
      set({
        workspaces,
        sessionWorkspaceMap: {
          ...get().sessionWorkspaceMap,
          [sessionId]: workspaceId,
        },
      });
    } catch (e) {
      console.error("[WorkspaceStore] setSessionWorkspace error:", e);
      // Revert to previous state on failure (only if still the latest request)
      if (seq !== requestSeq) return;
      set({ workspaces: prevWorkspaces, sessionWorkspaceMap: prevMap });
    }
  },

  setCurrentWorkspace: async (agentId: string, workspaceId: string) => {
    // Legacy wrapper: resolve active session ID and delegate to setSessionWorkspace
    const activeSessionId = useChatStore.getState().getActiveSessionId(agentId);
    if (!activeSessionId) {
      console.warn("[WorkspaceStore] setCurrentWorkspace: no active session for agent", agentId);
      return;
    }
    return get().setSessionWorkspace(agentId, activeSessionId, workspaceId);
  },

  setSessionWorkspaceLocal: (sessionId: string, workspaceId: string) => {
    set((state) => ({
      sessionWorkspaceMap: { ...state.sessionWorkspaceMap, [sessionId]: workspaceId },
    }));
  },

  syncSessionWorkspaces: (sessions) => {
    set((state) => {
      const next = { ...state.sessionWorkspaceMap };
      let changed = false;
      for (const s of sessions) {
        const wsId = s.workspace_id;
        if (wsId && wsId !== "__agent_home__") {
          if (next[s.session_id] !== wsId) {
            next[s.session_id] = wsId;
            changed = true;
          }
        } else if (s.session_id in next) {
          delete next[s.session_id];
          changed = true;
        }
      }
      return changed ? { sessionWorkspaceMap: next } : {};
    });
  },

  getSessionWorkspaceId: (sessionId: string) => {
    return get().sessionWorkspaceMap[sessionId] ?? "__agent_home__";
  },

  reset: () => {
    set({ workspaces: [], sessionWorkspaceMap: {}, loading: false });
  },
}));

export type { WorkspaceDir, WorkspaceState };
