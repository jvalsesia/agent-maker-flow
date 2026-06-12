import { useEffect, useState } from "react";

import { sseUrlWithToken } from "../lib/authToken";
import { fetchRunSnapshot, RUN_EVENT_NAMES } from "../lib/runs";
import {
  initialRunState,
  parseRunEvent,
  reduceRunEvent,
  type RunState,
} from "../lib/runStream";

export type RunConnection = "idle" | "connecting" | "open" | "closed";

export interface UseRunStreamResult {
  /** The live run state folded from the event stream. */
  state: RunState;
  /** Connection status; "connecting" drives the "Reconnecting…" indicator. */
  connection: RunConnection;
}

/**
 * Subscribe to a run's F09 SSE event stream and fold it into a {@link RunState}
 * (F10). Given a `runId`, it opens the authenticated `/runs/{id}/events` stream
 * (the session token is carried as a query param via `sseUrlWithToken`, since a
 * native `EventSource` cannot set an `Authorization` header), registers a
 * listener for every F09 event name, and reduces each event through the pure
 * `reduceRunEvent` — whose `seq` guard makes replay-on-reconnect idempotent.
 *
 * The browser's `EventSource` auto-reconnects and resends `Last-Event-ID`, so
 * F09 replays only newer events. If the stream errors before `run.finished`
 * arrived (e.g. the run already finished server-side and the stream closed),
 * the hook fetches `GET /runs/{id}` once and folds its buffered log to recover
 * the terminal state.
 *
 * Pass `runId = null` to stay disconnected.
 */
export function useRunStream(runId: string | null): UseRunStreamResult {
  const [state, setState] = useState<RunState>(initialRunState);
  const [connection, setConnection] = useState<RunConnection>("idle");

  useEffect(() => {
    if (!runId) {
      setConnection("idle");
      return;
    }

    let cancelled = false;
    let source: EventSource | null = null;
    let finished = false;
    let recovering = false;

    setState(initialRunState());
    setConnection("connecting");

    const apply = (name: string, data: string) => {
      const event = parseRunEvent(name, data);
      if (!event) return;
      setState((prev) => reduceRunEvent(prev, event));
      if (event.event === "run.finished") {
        finished = true;
        source?.close();
        setConnection("closed");
      }
    };

    const recoverViaSnapshot = async () => {
      if (recovering || finished) return;
      recovering = true;
      try {
        const snap = await fetchRunSnapshot(runId);
        if (cancelled) return;
        // The reducer's seq guard de-dupes already-applied events on fold.
        setState((prev) => snap.events.reduce(reduceRunEvent, prev));
        if (snap.status !== "running") {
          finished = true;
          source?.close();
          setConnection("closed");
        }
      } catch {
        // Recovery failed; the EventSource keeps retrying on its own.
      } finally {
        recovering = false;
      }
    };

    const handlers = RUN_EVENT_NAMES.map(
      (name) => [name, (e: MessageEvent) => apply(name, e.data)] as const,
    );

    void sseUrlWithToken(`/api/v1/runs/${runId}/events`).then((url) => {
      if (cancelled) return;
      const es = new EventSource(url);
      source = es;
      es.onopen = () => setConnection("open");
      es.onerror = () => {
        if (finished) return;
        setConnection("connecting");
        void recoverViaSnapshot();
      };
      for (const [name, handler] of handlers) {
        es.addEventListener(name, handler as EventListener);
      }
    });

    return () => {
      cancelled = true;
      if (source) {
        for (const [name, handler] of handlers) {
          source.removeEventListener(name, handler as EventListener);
        }
        source.close();
      }
      setConnection("closed");
    };
  }, [runId]);

  return { state, connection };
}
