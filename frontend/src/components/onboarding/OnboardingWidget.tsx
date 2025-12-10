'use client';

import * as React from 'react';
import { useOnboarding } from './useOnboarding';
import { OnboardingChecklist } from './OnboardingChecklist';
import { FeatureTooltip } from './FeatureTooltip';
import { CompletionCelebration } from './CompletionCelebration';
import { useRouter } from 'next/navigation';

/**
 * Onboarding widget that combines all onboarding components
 * Place this in the dashboard layout to enable onboarding features
 * Requirements: 5.1, 5.2, 5.4
 */
export function OnboardingWidget() {
  const router = useRouter();
  const {
    // Checklist
    steps,
    completionPercent,
    isChecklistVisible,
    isChecklistDismissed,
    dismissChecklist,
    markStepComplete,

    // Tour
    tourSteps,
    currentTourStep,
    isTourActive,
    isTourCompleted,
    nextTourStep,
    prevTourStep,
    skipTour,
    completeTour,
    startTour,

    // Celebration
    showCelebration,
    celebrationMessage,
    closeCelebration,
  } = useOnboarding();

  // Handle step click - navigate to relevant page
  const handleStepClick = (stepId: string) => {
    const step = steps.find((s) => s.id === stepId);
    if (step?.href && !step.completed) {
      router.push(step.href);
    }
  };

  // Auto-start tour for new users who haven't completed it
  React.useEffect(() => {
    if (!isTourCompleted && !isTourActive && completionPercent <= 25) {
      // Delay tour start to let page load
      const timer = setTimeout(() => {
        startTour();
      }, 1000);
      return () => clearTimeout(timer);
    }
  }, [isTourCompleted, isTourActive, completionPercent, startTour]);

  return (
    <>
      {/* Onboarding Checklist - shown on dashboard */}
      {isChecklistVisible && !isChecklistDismissed && completionPercent < 100 && (
        <div className="mb-6">
          <OnboardingChecklist
            steps={steps}
            completionPercent={completionPercent}
            onDismiss={dismissChecklist}
            onStepClick={handleStepClick}
          />
        </div>
      )}

      {/* Feature Tour Tooltip */}
      {isTourActive && (
        <FeatureTooltip
          steps={tourSteps}
          currentStep={currentTourStep}
          onNext={nextTourStep}
          onPrev={prevTourStep}
          onSkip={skipTour}
          onComplete={completeTour}
          className="bottom-20 left-1/2 -translate-x-1/2 md:bottom-auto md:top-20 md:left-72"
        />
      )}

      {/* Celebration Modal */}
      <CompletionCelebration
        isVisible={showCelebration}
        onClose={closeCelebration}
        message={celebrationMessage}
      />
    </>
  );
}

export default OnboardingWidget;
