import { Link, useLocation } from "react-router-dom";
import { useEffect, useId, useRef, useState } from "react";
import { useWallet } from "../contexts/WalletContext";

const short = (a: string) => `${a.slice(0, 6)}…${a.slice(-4)}`;

export default function Navigation() {
  const { pathname } = useLocation();
  const { status, address, connect, disconnect, error, wrongNetwork } = useWallet();
  const [walletMenuOpen, setWalletMenuOpen] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const walletMenuId = useId();
  const mobileMenuId = useId();
  const walletRegionRef = useRef<HTMLDivElement>(null);
  const walletTriggerRef = useRef<HTMLButtonElement>(null);
  const disconnectRef = useRef<HTMLButtonElement>(null);
  const mobileTriggerRef = useRef<HTMLButtonElement>(null);
  const inApp = pathname.startsWith("/app");
  const link = "rounded text-gray-700 hover:text-teal-600 transition focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-teal-600";
  const links = inApp
    ? [["/app", "Dashboard"], ["/app/streams", "Streams"], ["/app/activity", "Activity"], ["/app/settings", "Settings"]]
    : [["/", "Home"], ["/how-it-works", "How It Works"], ["/features", "Features"], ["/docs", "Docs"], ["/community", "Community"]];

  useEffect(() => {
    setWalletMenuOpen(false);
    setMobileMenuOpen(false);
  }, [pathname]);

  useEffect(() => {
    if (walletMenuOpen) disconnectRef.current?.focus();
  }, [walletMenuOpen]);

  useEffect(() => {
    if (!walletMenuOpen && !mobileMenuOpen) return;

    const onEscape = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      if (walletMenuOpen) {
        setWalletMenuOpen(false);
        walletTriggerRef.current?.focus();
      } else {
        setMobileMenuOpen(false);
        mobileTriggerRef.current?.focus();
      }
    };
    const onOutsidePointer = (event: PointerEvent) => {
      if (walletMenuOpen && !walletRegionRef.current?.contains(event.target as Node)) {
        setWalletMenuOpen(false);
      }
    };
    document.addEventListener("keydown", onEscape);
    document.addEventListener("pointerdown", onOutsidePointer);
    return () => {
      document.removeEventListener("keydown", onEscape);
      document.removeEventListener("pointerdown", onOutsidePointer);
    };
  }, [walletMenuOpen, mobileMenuOpen]);

  const closeMobileMenu = () => {
    setMobileMenuOpen(false);
    setWalletMenuOpen(false);
  };

  return (
    <nav aria-label="Primary" className="bg-white border-b border-gray-200">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 py-4 md:flex md:items-center md:justify-between md:gap-4">
        <div className="flex items-center justify-between gap-3">
          <Link to="/" onClick={closeMobileMenu} className="text-2xl font-bold gradient-text rounded focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-teal-600">
            Streamora
          </Link>
          <button
            ref={mobileTriggerRef}
            type="button"
            aria-label="Toggle navigation menu"
            aria-controls={mobileMenuId}
            aria-expanded={mobileMenuOpen}
            onClick={() => setMobileMenuOpen((open) => !open)}
            className="md:hidden inline-flex items-center justify-center rounded-lg border border-gray-300 p-2 text-gray-700 hover:bg-gray-50"
          >
            <svg aria-hidden="true" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
              {mobileMenuOpen ? <path d="M5 5l14 14M19 5L5 19" /> : <path d="M4 6h16M4 12h16M4 18h16" />}
            </svg>
          </button>
        </div>

        <div
          id={mobileMenuId}
          className={`${mobileMenuOpen ? "flex" : "hidden"} flex-col items-stretch gap-4 pt-4 min-w-0 md:flex md:flex-row md:items-center md:gap-3 lg:gap-6 md:pt-0`}
        >
          {links.map(([to, label]) => (
            <Link key={to} to={to} onClick={closeMobileMenu} className={`${link} block md:inline-block`}>
              {label}
            </Link>
          ))}
          {!inApp && (
            <Link to="/app" onClick={closeMobileMenu} className="btn-outline text-center">
              Launch App
            </Link>
          )}
          <div ref={walletRegionRef} className="relative min-w-0">
            {status === "connected" && address ? (
              <>
                <button
                  ref={walletTriggerRef}
                  type="button"
                  onClick={() => setWalletMenuOpen((open) => !open)}
                  className="btn-primary w-full md:w-auto"
                  aria-label="Wallet options"
                  aria-haspopup="dialog"
                  aria-controls={walletMenuId}
                  aria-expanded={walletMenuOpen}
                >
                  {wrongNetwork ? "Wrong network" : short(address)}
                </button>
                <div
                  id={walletMenuId}
                  role="dialog"
                  aria-label="Wallet actions"
                  hidden={!walletMenuOpen}
                  className="absolute right-0 mt-2 w-64 max-w-full md:max-w-none bg-white border border-gray-200 rounded-lg shadow-lg z-50 p-4"
                >
                  <p className="font-mono text-xs break-all mb-3">{address}</p>
                  <button
                    ref={disconnectRef}
                    type="button"
                    onClick={() => {
                      setWalletMenuOpen(false);
                      disconnect();
                    }}
                    className="rounded text-red-600 hover:text-red-700"
                  >
                    Disconnect
                  </button>
                </div>
              </>
            ) : (
              <button
                type="button"
                onClick={connect}
                className="btn-primary w-full md:w-auto"
                disabled={status === "connecting"}
              >
                {status === "connecting" ? "Connecting…" : "Connect Wallet"}
              </button>
            )}
            {error && status !== "connected" && (
              <div role="alert" className="absolute right-0 mt-2 w-72 max-w-full md:max-w-none bg-red-50 border border-red-200 rounded-lg p-3 text-sm text-red-700 z-50">
                {error}
              </div>
            )}
          </div>
        </div>
      </div>
    </nav>
  );
}
