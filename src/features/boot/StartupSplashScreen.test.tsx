import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { StartupSplashScreen } from "./StartupSplashScreen";
import { Gate } from "./Gate";
import { SPLASH_QUOTES } from "./splashQuotes";

const mockApi = vi.hoisted(() => ({
  setupStatus: vi.fn(),
  currentSession: vi.fn(),
  login: vi.fn(),
  logout: vi.fn(),
  completeSetup: vi.fn(),
}));

vi.mock("@/lib/api", async (orig) => ({
  ...(await orig<typeof import("@/lib/api")>()),
  authApi: mockApi,
}));

describe("StartupSplashScreen & Gate Minimum Duration", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    Object.values(mockApi).forEach((fn) => fn.mockReset());
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders branded visual elements, progress bar, and accessibility live region", () => {
    const splashQuote = SPLASH_QUOTES[0]!;
    render(<StartupSplashScreen durationMs={3000} quote={splashQuote} />);

    expect(screen.getByTestId("startup-splash-screen")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Pusthakam" })).toBeInTheDocument();
    expect(screen.getByText("A Better Way to Manage Libraries.")).toBeInTheDocument();
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "0");
    expect(screen.getByRole("status")).toHaveTextContent("Opening your library…");
    expect(screen.getByText(`“${splashQuote.text}”`)).toBeInTheDocument();
    expect(screen.getByText(splashQuote.author)).toBeInTheDocument();
    expect(screen.queryByText(/Local Sovereign Architecture|Offline Ready/)).not.toBeInTheDocument();
  });

  it("keeps its quotation through progress updates and parent rerenders", () => {
    const { rerender } = render(<StartupSplashScreen durationMs={3000} />);
    const original = screen.getByRole("blockquote").textContent;
    act(() => { vi.advanceTimersByTime(1500); });
    rerender(<StartupSplashScreen durationMs={3000} />);
    expect(screen.getByRole("blockquote")).toHaveTextContent(original!);
  });

  it("identifies a fictional speaker alongside the author", () => {
    const quote = SPLASH_QUOTES.find((entry) => entry.speaker)!;
    render(<StartupSplashScreen quote={quote} />);
    expect(screen.getByText(`${quote.speaker}, ${quote.work}`)).toBeInTheDocument();
    expect(screen.getByText(quote.author)).toBeInTheDocument();
  });

  it("advances progress stages and triggers onComplete when durationMs elapses", () => {
    const onComplete = vi.fn();
    render(<StartupSplashScreen durationMs={3000} onComplete={onComplete} />);

    expect(onComplete).not.toHaveBeenCalled();

    // Advance to 1000ms (~33%)
    act(() => {
      vi.advanceTimersByTime(1000);
    });
    expect(screen.getByRole("progressbar").getAttribute("aria-valuenow")).not.toBe("0");
    expect(onComplete).not.toHaveBeenCalled();

    // Advance to 2000ms (~66%)
    act(() => {
      vi.advanceTimersByTime(1000);
    });
    expect(onComplete).not.toHaveBeenCalled();

    // Advance past 3000ms
    act(() => {
      vi.advanceTimersByTime(1100);
    });
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "100");
    expect(onComplete).toHaveBeenCalledTimes(1);
  });

  it("keeps the startup deadline when the completion callback changes", () => {
    const original = vi.fn();
    const latest = vi.fn();
    const { rerender } = render(<StartupSplashScreen durationMs={3000} onComplete={original} />);
    act(() => { vi.advanceTimersByTime(1500); });
    rerender(<StartupSplashScreen durationMs={3000} onComplete={latest} />);
    act(() => { vi.advanceTimersByTime(1500); });
    expect(original).not.toHaveBeenCalled();
    expect(latest).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("status")).toHaveTextContent("Just a moment…");
  });

  it("immediately finishes if durationMs is 0 or negative", () => {
    const onComplete = vi.fn();
    render(<StartupSplashScreen durationMs={0} onComplete={onComplete} />);
    expect(onComplete).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "100");
  });

  it("keeps the splash visible after three seconds while startup queries are pending", () => {
    mockApi.setupStatus.mockReturnValue(new Promise(() => {}));
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(<QueryClientProvider client={qc}><Gate minSplashDurationMs={3000}><div>Inside App</div></Gate></QueryClientProvider>);
    act(() => { vi.advanceTimersByTime(4000); });
    expect(screen.getByTestId("startup-splash-screen")).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("Just a moment…");
    expect(screen.queryByRole("heading", { name: "Sign in" })).not.toBeInTheDocument();
  });

  it("enforces 3-second minimum splash screen in Gate before revealing sign-in screen", async () => {
    mockApi.setupStatus.mockResolvedValue({ configured: true, profiles: [] });
    mockApi.currentSession.mockResolvedValue(null);

    const qc = new QueryClient({ defaultOptions: { queries: { retry: false, staleTime: Infinity } } });
    qc.setQueryData(["setup-status"], { configured: true, profiles: [] });
    qc.setQueryData(["session"], null);

    render(
      <QueryClientProvider client={qc}>
        <Gate minSplashDurationMs={3000}>
          <div>Inside App</div>
        </Gate>
      </QueryClientProvider>,
    );

    // Initial render shows splash screen
    expect(screen.getByTestId("startup-splash-screen")).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Sign in" })).not.toBeInTheDocument();

    // Advance 1500ms - still shows splash screen
    act(() => {
      vi.advanceTimersByTime(1500);
    });
    expect(screen.getByTestId("startup-splash-screen")).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Sign in" })).not.toBeInTheDocument();

    // Advance past 3000ms - completes 3 seconds and reveals sign-in
    act(() => {
      vi.advanceTimersByTime(1600);
    });
    expect(screen.queryByTestId("startup-splash-screen")).not.toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Sign in" })).toBeInTheDocument();
  });
});
