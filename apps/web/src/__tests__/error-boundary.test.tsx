import { useState } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import ErrorBoundary from "../components/ErrorBoundary";

function BrokenChild(): JSX.Element {
  throw new Error("private diagnostic details");
}

function RouteHarness() {
  const [route, setRoute] = useState("broken");
  return (
    <MemoryRouter>
      <button onClick={() => setRoute("healthy")}>Navigate</button>
      <ErrorBoundary key={route}>
        {route === "broken" ? <BrokenChild /> : <p>Healthy route</p>}
      </ErrorBoundary>
    </MemoryRouter>
  );
}

describe("ErrorBoundary", () => {
  afterEach(() => vi.restoreAllMocks());

  it("shows a friendly fallback and recovers when the route changes", () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    render(<RouteHarness />);

    expect(screen.getByRole("alert")).toHaveTextContent("Something went wrong");
    expect(screen.getByRole("button", { name: "Reload page" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Back to home" })).toHaveAttribute("href", "/");
    expect(screen.queryByText("private diagnostic details")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Navigate" }));
    expect(screen.getByText("Healthy route")).toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});
