export type Politeness = "polite" | "assertive";

export type AnnouncerListener = (message: string, politeness: Politeness) => void;

const listeners = new Set<AnnouncerListener>();

/**
 * Broadcast an accessible spoken message to active screen readers (e.g. Windows Narrator, NVDA, VoiceOver).
 * - "polite": Waited until current speech finishes (status updates, search results counts, successful save).
 * - "assertive": Interrupts speech immediately (blocking errors, permission denials, critical alerts).
 */
export function announce(message: string, politeness: Politeness = "polite"): void {
  for (const listener of listeners) {
    listener(message, politeness);
  }
}

/**
 * Subscribe a live region component to broadcasted announcements.
 */
export function subscribeAnnouncer(listener: AnnouncerListener): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/**
 * React hook to dispatch screen reader announcements.
 */
export function useAnnounce() {
  return { announce };
}
