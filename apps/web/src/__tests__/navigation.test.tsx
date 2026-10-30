import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import Navigation from "../components/Navigation";

const wallet = vi.hoisted(() => ({
  status: "connected",
  address: "GDHLHKGQFWANAUZF4Z7LI7IJQF4IEUFPMLFVIF7OSQC5K335K5FLKYQE",
  wrongNetwork: false,
  error: null as string | null,
  connect: vi.fn(),
  disconnect: vi.fn(),
}));
vi.mock("../contexts/WalletContext", () => ({ useWallet: () => wallet }));

function renderNav(path = "/") {
  return render(<MemoryRouter initialEntries={[path]}><Navigation /></MemoryRouter>);
}

beforeEach(() => {
  wallet.status = "connected";
  wallet.error = null;
  wallet.disconnect.mockReset();
  wallet.connect.mockReset();
});

describe("accessible navigation", () => {
  it("opens the wallet popover, focuses Disconnect, then returns focus on Escape", async () => {
    const user = userEvent.setup();
    renderNav();

    const trigger = screen.getByRole("button", { name: "Wallet options" });
    expect(trigger).toHaveAttribute("aria-expanded", "false");
    expect(trigger).toHaveAttribute("aria-haspopup", "dialog");
    expect(document.getElementById(trigger.getAttribute("aria-controls")!)).toBeInTheDocument();

    await user.click(trigger);
    expect(trigger).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByRole("button", { name: "Disconnect" })).toHaveFocus();

    await user.keyboard("{Escape}");
    expect(trigger).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByRole("button", { name: "Disconnect" })).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
  });

  it("closes the wallet popover on an outside pointer click", async () => {
    const user = userEvent.setup();
    renderNav();
    const trigger = screen.getByRole("button", { name: "Wallet options" });
    await user.click(trigger);
    await user.click(document.body);
    expect(trigger).toHaveAttribute("aria-expanded", "false");
  });

  it("toggles mobile navigation and closes it when a link is activated", async () => {
    const user = userEvent.setup();
    renderNav();
    const trigger = screen.getByRole("button", { name: "Toggle navigation menu" });
    const panel = document.getElementById(trigger.getAttribute("aria-controls")!);
    expect(panel).toBeInTheDocument();
    expect(trigger).toHaveAttribute("aria-expanded", "false");
    expect(panel).toHaveClass("hidden");

    await user.click(trigger);
    expect(trigger).toHaveAttribute("aria-expanded", "true");
    expect(panel).toHaveClass("flex");

    await user.click(screen.getByRole("link", { name: "Features" }));
    expect(trigger).toHaveAttribute("aria-expanded", "false");
    expect(panel).toHaveClass("hidden");
  });

  it("closes the mobile navigation on Escape and restores toggle focus", async () => {
    const user = userEvent.setup();
    renderNav();
    const trigger = screen.getByRole("button", { name: "Toggle navigation menu" });
    await user.click(trigger);
    await user.keyboard("{Escape}");
    expect(trigger).toHaveAttribute("aria-expanded", "false");
    expect(trigger).toHaveFocus();
  });

  it("supports keyboard activation of the mobile toggle", async () => {
    const user = userEvent.setup();
    renderNav();
    const trigger = screen.getByRole("button", { name: "Toggle navigation menu" });
    trigger.focus();
    await user.keyboard("{Enter}");
    expect(trigger).toHaveAttribute("aria-expanded", "true");
    await user.keyboard(" ");
    expect(trigger).toHaveAttribute("aria-expanded", "false");
  });
});
