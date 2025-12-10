/**
 * Onboarding components for user guidance
 * Requirements: 5.1, 5.2, 5.4 - Onboarding checklist, tooltips, celebration
 */

export {
  OnboardingChecklist,
  OnboardingStepItem,
  defaultOnboardingSteps,
  type OnboardingStep,
  type OnboardingChecklistProps,
} from './OnboardingChecklist';

export {
  FeatureTooltip,
  defaultFeatureTourSteps,
  type TooltipStep,
  type FeatureTooltipProps,
} from './FeatureTooltip';

export {
  CompletionCelebration,
  CONFETTI_COLORS,
  type CompletionCelebrationProps,
} from './CompletionCelebration';

export { useOnboarding, OnboardingProvider } from './useOnboarding';

export { OnboardingWidget } from './OnboardingWidget';
