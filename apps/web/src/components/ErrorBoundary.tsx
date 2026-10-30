import { Component, type ErrorInfo, type ReactNode } from "react";
import { Link } from "react-router-dom";

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
}

export default class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false };

  static getDerivedStateFromError(): ErrorBoundaryState {
    return { hasError: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Uncaught error in routed page:", error, info.componentStack);
  }

  render() {
    if (this.state.hasError) {
      return (
        <section className="section-container" role="alert">
          <div className="card mx-auto max-w-2xl text-center">
            <h1>Something went wrong</h1>
            <p className="mt-3 text-gray-600">This page ran into a problem. You can reload it or return home.</p>
            <div className="mt-6 flex flex-wrap justify-center gap-3">
              <button className="btn-primary" onClick={() => window.location.reload()}>
                Reload page
              </button>
              <Link className="btn-secondary" to="/">
                Back to home
              </Link>
            </div>
          </div>
        </section>
      );
    }

    return this.props.children;
  }
}
