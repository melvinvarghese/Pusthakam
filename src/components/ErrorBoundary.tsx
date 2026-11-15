import { Component, type ErrorInfo, type ReactNode } from "react";
import { TriangleAlert } from "lucide-react";
import { EmptyState } from "./EmptyState";
import { Button } from "./ui/Button";

interface State { error: Error | null }

/** Catches render errors so one broken page never blanks the whole app. */
export class ErrorBoundary extends Component<{ children: ReactNode }, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    // Local console only; nothing is transmitted anywhere.
    console.error("UI error:", error.message, info.componentStack);
  }

  render() {
    if (this.state.error) {
      return (
        <EmptyState
          icon={TriangleAlert}
          title="Something went wrong on this page"
          description="Your data is safe. Try again, or go to another section and come back."
          action={<Button variant="primary" onClick={() => this.setState({ error: null })}>Try again</Button>}
        />
      );
    }
    return this.props.children;
  }
}
