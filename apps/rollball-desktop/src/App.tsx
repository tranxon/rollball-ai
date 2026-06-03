import { useState, useEffect } from "react";
import { emit } from "@tauri-apps/api/event";
import { AppLayout } from "./components/layout/AppLayout";
import { SplashScreen } from "./components/layout/SplashScreen";
import { OnboardingFlow } from "./components/onboarding/OnboardingFlow";
import { ToastProvider } from "./components/common/ToastProvider";
import { ErrorBoundary } from "./components/common/ErrorBoundary";

function App() {
  const [onboardingDone, setOnboardingDone] = useState(() => {
    return localStorage.getItem("rollball_onboarding") === "completed";
  });

  const [gatewayReady, setGatewayReady] = useState(false);
  const [splashShown, setSplashShown] = useState(false);

  // Signal Rust to show the native window after the first React render.
  // The window starts hidden (visible: false in tauri.conf.json) to prevent
  // the white/transparent flash before the splash screen is ready.
  // Rust listens for "splash-ready" and calls window.show() from the native side.
  useEffect(() => {
    if (!splashShown) {
      setSplashShown(true);
      requestAnimationFrame(() => {
        emit("splash-ready").catch((err) => {
          console.warn("Failed to emit splash-ready:", err);
        });
      });
    }
  }, [splashShown]);

  if (!gatewayReady && onboardingDone) {
    return (
      <div className="h-screen w-screen overflow-hidden">
        <SplashScreen onReady={() => setGatewayReady(true)} />
      </div>
    );
  }

  return (
    <ErrorBoundary>
      <ToastProvider>
        {!onboardingDone ? (
          <OnboardingFlow onComplete={() => setOnboardingDone(true)} />
        ) : (
          <AppLayout />
        )}
      </ToastProvider>
    </ErrorBoundary>
  );
}

export default App;
