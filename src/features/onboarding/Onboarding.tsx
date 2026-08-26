import React, { useState } from 'react';
import { Sparkles, Command, CheckCircle2, ArrowRight } from 'lucide-react';
import './Onboarding.css';

interface OnboardingProps {
  onComplete: () => void;
}

export const Onboarding: React.FC<OnboardingProps> = ({ onComplete }) => {
  const [step, setStep] = useState(0);

  const steps = [
    {
      title: 'Welcome to Spotlight',
      subtitle: 'A Windows-first, keyboard-first, offline-first application launcher.',
      icon: <Sparkles size={36} className="onboarding-icon" />,
      content: (
        <p className="onboarding-desc">
          Instant, minimal, and completely local. Spotlight is designed to disappear into your
          workflow.
        </p>
      ),
    },
    {
      title: 'Press Alt + Space',
      subtitle: 'The universal shortcut to open Spotlight anywhere.',
      icon: <Command size={36} className="onboarding-icon" />,
      content: (
        <div className="onboarding-shortcut-display">
          <kbd>Alt</kbd>
          <span>+</span>
          <kbd>Space</kbd>
        </div>
      ),
    },
    {
      title: 'Ready to Go',
      subtitle: 'Search applications, files, commands, and math expressions instantly.',
      icon: <CheckCircle2 size={36} className="onboarding-icon" />,
      content: (
        <ul className="onboarding-tips">
          <li>
            Type <code>notepad</code> to find and launch apps
          </li>
          <li>
            Type <code>= 25 * 4</code> for instant calculator
          </li>
          <li>
            Type <code>&gt; lock</code> for built-in system commands
          </li>
        </ul>
      ),
    },
  ];

  const current = steps[step];

  const handleNext = () => {
    if (step < steps.length - 1) {
      setStep(step + 1);
    } else {
      localStorage.setItem('spotlight_onboarding_completed', 'true');
      onComplete();
    }
  };

  const handleSkip = () => {
    localStorage.setItem('spotlight_onboarding_completed', 'true');
    onComplete();
  };

  if (!current) return null;

  return (
    <div className="onboarding-card anim-fade-in">
      <div className="onboarding-header">
        <div className="onboarding-icon-container">{current.icon}</div>
        <h2>{current.title}</h2>
        <p className="onboarding-subtitle">{current.subtitle}</p>
      </div>

      <div className="onboarding-body">{current.content}</div>

      <div className="onboarding-footer">
        <button className="onboarding-skip" onClick={handleSkip}>
          Skip
        </button>
        <div className="onboarding-dots">
          {steps.map((_, idx) => (
            <span
              key={idx}
              className={`onboarding-dot ${idx === step ? 'active' : ''}`}
              onClick={() => {
                setStep(idx);
              }}
            />
          ))}
        </div>
        <button className="onboarding-next" onClick={handleNext}>
          {step === steps.length - 1 ? 'Get Started' : 'Next'} <ArrowRight size={14} />
        </button>
      </div>
    </div>
  );
};

export default Onboarding;
