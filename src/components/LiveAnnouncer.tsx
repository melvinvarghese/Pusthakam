import { useEffect, useState } from "react";
import { subscribeAnnouncer } from "@/lib/announcer";

/**
 * Renders visually hidden ARIA live regions for assistive technologies (screen readers like Windows Narrator).
 * Maintains separate polite and assertive regions so critical alerts can interrupt while normal state
 * changes queue politely.
 */
export function LiveAnnouncer() {
  const [politeText, setPoliteText] = useState("");
  const [assertiveText, setAssertiveText] = useState("");

  useEffect(() => {
    return subscribeAnnouncer((message, politeness) => {
      if (politeness === "assertive") {
        setAssertiveText("");
        setTimeout(() => setAssertiveText(message), 50);
      } else {
        setPoliteText("");
        setTimeout(() => setPoliteText(message), 50);
      }
    });
  }, []);

  return (
    <div className="sr-only" aria-hidden="false">
      <div
        role="status"
        aria-live="polite"
        aria-atomic="true"
        data-testid="live-region-polite"
      >
        {politeText}
      </div>
      {assertiveText ? (
        <div
          role="alert"
          aria-live="assertive"
          aria-atomic="true"
          data-testid="live-region-assertive"
        >
          {assertiveText}
        </div>
      ) : null}
    </div>
  );
}
