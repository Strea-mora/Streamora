import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "../App";
import { WalletProvider } from "../contexts/WalletContext";
import TxStatus from "../components/TxStatus";

const freighter = vi.hoisted(() => ({
  isConnected: vi.fn(),
  isAllowed: vi.fn(),
  requestAccess: vi.fn(),
  getAddress: vi.fn(),
  getNetworkDetails: vi.fn(),
  signTransaction: vi.fn(),
}));
vi.mock("@stellar/freighter-api", () => freighter);

const ADDR = "GDHLHKGQFWANAUZF4Z7LI7IJQF4IEUFPMLFVIF7OSQC5K335K5FLKYQE";
const TESTNET = "Test SDF Network ; September 2015";

function renderAt(path: string) {
  window.history.pushState({}, "", path);
  return render(<WalletProvider><App /></WalletProvider>);
}

beforeEach(() => {
  Object.values(freighter).forEach((f) => f.mockReset());
  freighter.isConnected.mockResolvedValue({ isConnected: false });
  freighter.isAllowed.mockResolvedValue({ isAllowed: false });
  freighter.getNetworkDetails.mockResolvedValue({ networkPassphrase: TESTNET });
});

describe("routes", () => {
  const cases: [string, RegExp][] = [
    ["/", /Money that moves with time/i],
    ["/how-it-works", /How Streaming Payments Work/i],
    ["/features", /Features/],
    ["/use-cases", /Use Cases/i],
    ["/developers", /Build with Streamora/i],
    ["/docs", /Documentation/],
    ["/roadmap", /Roadmap/],
    ["/about", /About Streamora/],
    ["/community", /Community/],
    ["/app", /Dashboard/],
    ["/definitely-missing", /Page Not Found/i],
  ];
  it.each(cases)("renders %s", async (path, text) => {
    renderAt(path);
    expect(await screen.findAllByText(text)).not.toHaveLength(0);
  });
});

describe("wallet states", () => {
  it("shows no-wallet error when Freighter is missing", async () => {
    renderAt("/app/streams/create");
    await userEvent.click((await screen.findAllByRole("button", { name: /connect wallet/i }))[0]);
    expect(await screen.findAllByText(/Freighter wallet not detected/i)).not.toHaveLength(0);
  });

  it("shows rejection message when user rejects connection", async () => {
    freighter.isConnected.mockResolvedValue({ isConnected: true });
    freighter.requestAccess.mockResolvedValue({ address: "", error: { message: "User declined access" } });
    renderAt("/app/streams/create");
    await userEvent.click((await screen.findAllByRole("button", { name: /connect wallet/i }))[0]);
    expect(await screen.findAllByText(/User declined access/i)).not.toHaveLength(0);
  });

  it("connects and shows the create form", async () => {
    freighter.isConnected.mockResolvedValue({ isConnected: true });
    freighter.requestAccess.mockResolvedValue({ address: ADDR });
    renderAt("/app/streams/create");
    await userEvent.click((await screen.findAllByRole("button", { name: /connect wallet/i }))[0]);
    expect(await screen.findByLabelText(/total amount/i)).toBeInTheDocument();
  });

  it("blocks the form on the wrong network", async () => {
    freighter.isConnected.mockResolvedValue({ isConnected: true });
    freighter.requestAccess.mockResolvedValue({ address: ADDR });
    freighter.getNetworkDetails.mockResolvedValue({ networkPassphrase: "Public Global Stellar Network ; September 2015" });
    renderAt("/app/streams/create");
    await userEvent.click((await screen.findAllByRole("button", { name: /connect wallet/i }))[0]);
    expect(await screen.findByText(/Wrong network/i, { selector: "p" })).toBeInTheDocument();
  });
});

describe("create form validation", () => {
  it("keeps Review disabled until inputs are valid", async () => {
    freighter.isConnected.mockResolvedValue({ isConnected: true });
    freighter.requestAccess.mockResolvedValue({ address: ADDR });
    renderAt("/app/streams/create");
    await userEvent.click((await screen.findAllByRole("button", { name: /connect wallet/i }))[0]);
    const review = await screen.findByRole("button", { name: "Review" });
    expect(review).toBeDisabled();
    await userEvent.type(screen.getByLabelText(/total amount/i), "5");
    await userEvent.type(screen.getByLabelText(/recipient 1 address/i), ADDR);
    await waitFor(() => expect(review).toBeEnabled());
    await userEvent.clear(screen.getByLabelText(/recipient 1 share/i));
    await userEvent.type(screen.getByLabelText(/recipient 1 share/i), "60");
    await waitFor(() => expect(review).toBeDisabled());
  });
});

describe("transaction states", () => {
  const cases: [any, RegExp][] = [
    [{ phase: "simulating" }, /Simulating/],
    [{ phase: "signing" }, /Waiting for your signature/],
    [{ phase: "pending", hash: "abc" }, /waiting for confirmation/],
    [{ phase: "confirmed", hash: "abc" }, /Confirmed on-chain/],
    [{ phase: "failed", error: "boom" }, /Failed: boom/],
    [{ phase: "rejected" }, /rejected the signature/],
  ];
  it.each(cases)("renders %o", (state, text) => {
    render(<TxStatus state={state} />);
    const role = state.phase === "failed" || state.phase === "rejected" ? "alert" : "status";
    expect(screen.getByRole(role)).toHaveTextContent(text);
  });
  it("renders nothing when idle", () => {
    const { container } = render(<TxStatus state={{ phase: "idle" }} />);
    expect(container).toBeEmptyDOMElement();
  });
});
