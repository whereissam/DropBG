import { useState } from "react";
import logoSvg from "../assets/logo.svg";

interface Props {
  onComplete: () => void;
}

const steps = [
  {
    title: "Welcome to DropBG",
    subtitle: "Local AI background remover for macOS",
    content: (
      <div className="onboard-features">
        <div className="onboard-feature">
          <span className="onboard-feature-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
          </span>
          <div>
            <strong>100% Private</strong>
            <p>Your images never leave your Mac. All processing happens locally.</p>
          </div>
        </div>
        <div className="onboard-feature">
          <span className="onboard-feature-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>
          </span>
          <div>
            <strong>Fast on Apple Silicon</strong>
            <p>Uses CoreML and the Neural Engine for hardware-accelerated inference.</p>
          </div>
        </div>
        <div className="onboard-feature">
          <span className="onboard-feature-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
          </span>
          <div>
            <strong>One-Time Setup</strong>
            <p>Download an AI model once, then everything runs offline forever.</p>
          </div>
        </div>
      </div>
    ),
  },
  {
    title: "System Permissions",
    subtitle: "DropBG needs a few permissions to work properly",
    content: (
      <div className="onboard-permissions">
        <div className="onboard-perm">
          <div className="onboard-perm-header">
            <span className="onboard-perm-icon">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
            </span>
            <strong>File Access</strong>
          </div>
          <p>To read your images and save results. macOS may ask you to grant access the first time.</p>
        </div>
        <div className="onboard-perm">
          <div className="onboard-perm-header">
            <span className="onboard-perm-icon">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            </span>
            <strong>Network (first launch only)</strong>
          </div>
          <p>To download the AI model from HuggingFace. After that, the app works fully offline.</p>
        </div>
        <div className="onboard-hint">
          When macOS shows a permission dialog, click <strong>Allow</strong> to continue.
        </div>
      </div>
    ),
  },
  {
    title: "Opening an Unsigned App",
    subtitle: "DropBG is open-source and not signed with an Apple certificate",
    content: (
      <div className="onboard-unsigned">
        <p className="onboard-unsigned-intro">
          If macOS blocks the app with <em>"can't be opened because Apple cannot check it for malicious software"</em>, follow these steps:
        </p>
        <div className="onboard-steps">
          <div className="onboard-step">
            <span className="onboard-step-num">1</span>
            <div>
              <strong>Open System Settings</strong>
              <p>Go to <strong>Privacy & Security</strong></p>
            </div>
          </div>
          <div className="onboard-step">
            <span className="onboard-step-num">2</span>
            <div>
              <strong>Find the DropBG message</strong>
              <p>Scroll down — you'll see <em>"DropBG was blocked"</em></p>
            </div>
          </div>
          <div className="onboard-step">
            <span className="onboard-step-num">3</span>
            <div>
              <strong>Click "Open Anyway"</strong>
              <p>Enter your password, then click Open</p>
            </div>
          </div>
        </div>
        <div className="onboard-hint">
          <strong>Alternative:</strong> Run <code>xattr -cr /Applications/DropBG.app</code> in Terminal.
        </div>
      </div>
    ),
  },
];

export default function Onboarding({ onComplete }: Props) {
  const [step, setStep] = useState(0);
  const current = steps[step];
  const isLast = step === steps.length - 1;

  return (
    <div className="setup-overlay">
      <div className="onboard-card">
        {step === 0 && (
          <div className="onboard-logo">
            <img src={logoSvg} alt="DropBG" width="64" height="64" style={{ borderRadius: 14 }} />
          </div>
        )}

        <h1>{current.title}</h1>
        <p className="onboard-subtitle">{current.subtitle}</p>

        {current.content}

        {/* Progress dots */}
        <div className="onboard-dots">
          {steps.map((_, i) => (
            <span key={i} className={`onboard-dot ${i === step ? "active" : ""}`} />
          ))}
        </div>

        <div className="onboard-actions">
          {step > 0 && (
            <button className="onboard-btn-secondary" onClick={() => setStep(step - 1)}>
              Back
            </button>
          )}
          <button
            className="onboard-btn-primary"
            onClick={() => {
              if (isLast) {
                onComplete();
              } else {
                setStep(step + 1);
              }
            }}
          >
            {isLast ? "Get Started" : "Continue"}
          </button>
        </div>

        {!isLast && (
          <button className="onboard-skip" onClick={onComplete}>
            Skip intro
          </button>
        )}
      </div>
    </div>
  );
}
