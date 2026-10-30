import { config } from "../config";
import { TxState } from "../hooks/useTx";

export default function TxStatus({ state }: { state: TxState }) {
  if (state.phase === "idle") return null;
  const map = {
    simulating: { t: "Simulating transaction…", c: "bg-blue-50 border-blue-200 text-blue-800" },
    signing: { t: "Waiting for your signature in Freighter…", c: "bg-blue-50 border-blue-200 text-blue-800" },
    pending: { t: "Transaction submitted, waiting for confirmation…", c: "bg-yellow-50 border-yellow-200 text-yellow-800" },
    confirmed: { t: "Confirmed on-chain.", c: "bg-green-50 border-green-200 text-green-800" },
    failed: { t: `Failed: ${state.error ?? "unknown error"}`, c: "bg-red-50 border-red-200 text-red-800" },
    rejected: { t: "You rejected the signature request.", c: "bg-red-50 border-red-200 text-red-800" },
  } as const;
  const m = map[state.phase];
  return (
    <div role={state.phase === "failed" || state.phase === "rejected" ? "alert" : "status"} className={`p-4 border rounded-lg ${m.c}`}>
      <p>{m.t}</p>
      {state.hash && (
        <a href={`${config.explorer}/tx/${state.hash}`} target="_blank" rel="noopener noreferrer" className="text-sm underline">
          View transaction
        </a>
      )}
    </div>
  );
}
