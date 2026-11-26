import { useTranslation } from "@/lib/translations";
import { useEffect, useEffectEvent, useState } from "react";
import { BrandMark } from "@/components/BrandMark";
import { getLaunchSplashQuote, type SplashQuote } from "./splashQuotes";

export interface StartupSplashScreenProps {
  /** Total minimum duration to display the startup splash screen in milliseconds (default: 3000ms) */
  durationMs?: number;
  /** Optional callback fired when the minimum display duration has elapsed */
  onComplete?: () => void;
  /** A supplied quote is useful for previews and tests; normal startup chooses locally. */
  quote?: SplashQuote;
}


/**
 * Branded startup loading screen.
 * Displays when Pusthakam launches and remains visible for at least 3 seconds
 * (or `durationMs`) before transitioning into the main application or sign-in gate.
 */
export function StartupSplashScreen({ durationMs = 3000, onComplete, quote: suppliedQuote }: StartupSplashScreenProps) {
  const { t } = useTranslation();
  const [progress, setProgress] = useState(durationMs <= 0 ? 100 : 0);
  const [splashQuote] = useState(() => suppliedQuote ?? getLaunchSplashQuote());

  const complete = useEffectEvent(() => onComplete?.());

  useEffect(() => {
    if (durationMs <= 0) {
      setProgress(100);
      complete();
      return;
    }

    let elapsed = 0;
    const tickMs = 50;
    const interval = setInterval(() => {
      elapsed += tickMs;
      const fraction = Math.min(elapsed / durationMs, 1);
      setProgress(Math.round(fraction * 100));

      if (fraction >= 1) {
        clearInterval(interval);
        complete();
      }
    }, tickMs);

    return () => clearInterval(interval);
  }, [durationMs]);

  const statusText = progress >= 100 ? "Just a moment…" : "Opening your library…";

  return (
    <div data-testid="startup-splash-screen" className="startup-splash">
      <div className="startup-presentation">
        {/* Brand Header */}
        <header className="startup-brand">
          <BrandMark size="splash" />
          <h1>Pusthakam</h1>
          <p>{t("A Better Way to Manage Libraries.")}</p>
        </header>

        <figure className="startup-quote">
          <blockquote>“{splashQuote.text}”</blockquote>
          <figcaption>
            {splashQuote.author}
            {splashQuote.speaker && <span>{splashQuote.speaker}, {splashQuote.work}</span>}
          </figcaption>
        </figure>

        {/* Progress Container */}
        <div className="startup-loading">
          <div
            className="startup-progress"
            role="progressbar"
            aria-valuenow={progress}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-label={t("Application startup progress")}
          >
            <div className="startup-progress-fill" style={{ width: `${progress}%` }} />
          </div>
          {/* Accessible Live Region & Status Text */}
          <p role="status" aria-live="polite">{t(statusText)}</p>
        </div>
      </div>
    </div>
  );
}
