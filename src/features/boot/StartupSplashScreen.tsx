import { useEffect, useState } from "react";
import { BrandMark } from "@/components/BrandMark";

export interface StartupSplashScreenProps {
  /** Total minimum duration to display the startup splash screen in milliseconds (default: 3000ms) */
  durationMs?: number;
  /** Optional callback fired when the minimum display duration has elapsed */
  onComplete?: () => void;
}

const STAGES = [
  { at: 0, text: "Opening local library database…" },
  { at: 0.25, text: "Verifying local schema integrity…" },
  { at: 0.55, text: "Loading circulation rules and calendar…" },
  { at: 0.8, text: "Opening your library…" },
];

/**
 * Branded startup loading screen.
 * Displays when Pusthakam launches and remains visible for at least 3 seconds
 * (or `durationMs`) before transitioning into the main application or sign-in gate.
 */
export function StartupSplashScreen({ durationMs = 3000, onComplete }: StartupSplashScreenProps) {
  const [progress, setProgress] = useState(durationMs <= 0 ? 100 : 0);
  const [stageIndex, setStageIndex] = useState(durationMs <= 0 ? STAGES.length - 1 : 0);

  useEffect(() => {
    if (durationMs <= 0) {
      setProgress(100);
      onComplete?.();
      return;
    }

    let elapsed = 0;
    const tickMs = 50;
    const interval = setInterval(() => {
      elapsed += tickMs;
      const fraction = Math.min(elapsed / durationMs, 1);
      setProgress(Math.round(fraction * 100));

      const newIndex = STAGES.reduce((acc, stage, idx) => {
        return fraction >= stage.at ? idx : acc;
      }, 0);
      setStageIndex(newIndex);

      if (fraction >= 1) {
        clearInterval(interval);
        onComplete?.();
      }
    }, tickMs);

    return () => clearInterval(interval);
  }, [durationMs, onComplete]);

  const currentStageText = STAGES[stageIndex]?.text ?? "Opening your library…";

  return (
    <div
      data-testid="startup-splash-screen"
      className="flex min-h-screen w-full flex-col items-center justify-center bg-ink p-6 select-none"
    >
      <div className="flex flex-col items-center gap-6 text-center">
        {/* Brand Header */}
        <div className="flex items-center gap-3.5">
          <BrandMark />
          <div className="text-left">
            <h1 className="text-2xl font-bold tracking-tight text-white leading-tight">Pusthakam</h1>
            <p className="text-xs font-medium text-rail-mute tracking-wide">A Better Way to Manage Libraries.</p>
          </div>
        </div>

        {/* Progress Container */}
        <div className="mt-4 flex flex-col items-center gap-3">
          <div
            className="h-1.5 w-64 overflow-hidden rounded-full bg-ink-3"
            role="progressbar"
            aria-valuenow={progress}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-label="Application startup progress"
          >
            <div
              className="h-full rounded-full bg-[var(--cls-6)] transition-all duration-150 ease-out"
              style={{ width: `${progress}%` }}
            />
          </div>

          {/* Accessible Live Region & Status Text */}
          <div role="status" aria-live="polite" className="flex flex-col items-center gap-1">
            <span className="text-xs text-rail-text font-mono animate-pulse">{currentStageText}</span>
            {/* Screen reader guidance */}
            <span className="sr-only">Opening your library… Just a moment.</span>
          </div>
        </div>

        <p className="mt-8 max-w-xs text-sm text-rail-text">
          More time for books and readers.
        </p>
      </div>
    </div>
  );
}
