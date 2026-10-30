import { BrowserRouter as Router, Routes, Route, useLocation } from "react-router-dom";
import Navigation from "./components/Navigation";
import Footer from "./components/Footer";
import ErrorBoundary from "./components/ErrorBoundary";

import Home from "./pages/Home";
import HowItWorks from "./pages/HowItWorks";
import Features from "./pages/Features";
import UseCases from "./pages/UseCases";
import Developers from "./pages/Developers";
import DocsHome from "./pages/docs/DocsHome";
import GettingStarted from "./pages/docs/GettingStarted";
import Concepts from "./pages/docs/Concepts";
import SmartContracts from "./pages/docs/SmartContracts";
import SDK from "./pages/docs/SDK";
import Examples from "./pages/docs/Examples";
import Security from "./pages/docs/Security";
import Roadmap from "./pages/Roadmap";
import About from "./pages/About";
import Community from "./pages/Community";
import NotFound from "./pages/NotFound";

import Dashboard from "./pages/app/Dashboard";
import Streams from "./pages/app/Streams";
import CreateStream from "./pages/app/CreateStream";
import StreamDetail from "./pages/app/StreamDetail";
import Activity from "./pages/app/Activity";
import Withdraw from "./pages/app/Withdraw";
import Settings from "./pages/app/Settings";

export default function App() {
  return (
    <Router>
      <AppLayout />
    </Router>
  );
}

function AppLayout() {
  const location = useLocation();

  return (
    <div className="flex flex-col min-h-screen">
      <Navigation />
      <main className="flex-1">
        <ErrorBoundary key={location.pathname}>
          <Routes>
            {/* Public Pages */}
            <Route path="/" element={<Home />} />
            <Route path="/how-it-works" element={<HowItWorks />} />
            <Route path="/features" element={<Features />} />
            <Route path="/use-cases" element={<UseCases />} />
            <Route path="/developers" element={<Developers />} />

            {/* Documentation */}
            <Route path="/docs" element={<DocsHome />} />
            <Route path="/docs/getting-started" element={<GettingStarted />} />
            <Route path="/docs/concepts" element={<Concepts />} />
            <Route path="/docs/smart-contracts" element={<SmartContracts />} />
            <Route path="/docs/sdk" element={<SDK />} />
            <Route path="/docs/examples" element={<Examples />} />
            <Route path="/docs/security" element={<Security />} />

            {/* More Pages */}
            <Route path="/roadmap" element={<Roadmap />} />
            <Route path="/about" element={<About />} />
            <Route path="/community" element={<Community />} />

            {/* Application Routes */}
            <Route path="/app" element={<Dashboard />} />
            <Route path="/app/streams" element={<Streams />} />
            <Route path="/app/streams/create" element={<CreateStream />} />
            <Route path="/app/streams/:id" element={<StreamDetail />} />
            <Route path="/app/streams/:id/withdraw" element={<Withdraw />} />
            <Route path="/app/activity" element={<Activity />} />
            <Route path="/app/settings" element={<Settings />} />

            {/* 404 */}
            <Route path="*" element={<NotFound />} />
          </Routes>
        </ErrorBoundary>
      </main>
      <Footer />
    </div>
  );
}
