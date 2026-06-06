import { useEffect, useRef, useState } from "react";

export type SseStatus = "connecting" | "open" | "closed";

export interface UseEventSourceResult {
  status: SseStatus;
  lastEvent: MessageEvent | null;
}

/**
 * Subscribe to a Server-Sent Events endpoint. Manages the EventSource
 * lifecycle and exposes connection status and the latest received event.
 * The browser's native EventSource handles reconnection automatically; the
 * status returns to "connecting" while a reconnect is in progress.
 *
 * Pass `url = null` to keep the connection closed.
 */
export function useEventSource(
  url: string | null,
  eventName = "message",
): UseEventSourceResult {
  const [status, setStatus] = useState<SseStatus>("closed");
  const [lastEvent, setLastEvent] = useState<MessageEvent | null>(null);
  const sourceRef = useRef<EventSource | null>(null);

  useEffect(() => {
    if (!url) {
      setStatus("closed");
      return;
    }

    setStatus("connecting");
    const source = new EventSource(url);
    sourceRef.current = source;

    source.onopen = () => setStatus("open");
    source.onerror = () => setStatus("connecting");

    const handler = (event: MessageEvent) => setLastEvent(event);
    source.addEventListener(eventName, handler as EventListener);

    return () => {
      source.removeEventListener(eventName, handler as EventListener);
      source.close();
      sourceRef.current = null;
      setStatus("closed");
    };
  }, [url, eventName]);

  return { status, lastEvent };
}
