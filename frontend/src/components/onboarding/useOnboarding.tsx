'use client';

import * as React from 'react';
import { createContext, useContext, useState, useCallback, useEffect } from 'react';
import { OnboardingStep, defaultOnboardingSteps } from './OnboardingChecklist';
import { TooltipStep, defaultFeatureTourSteps } from './FeatureTooltip';

interface OnboardingState {
  // Checklist state
  steps: OnboardingStep[];
  completionPercent: number;
  isChecklistVisible: boolean;
  isChecklistDismissed: boolean;

  // Feature tour state
  tourSteps: TooltipStep[];
  currentTourStep: number;
  isTourActive: boolean;
  isTourCompleted: boolean;

  // Celebration state
  showCelebration: boolean;
  celebrationMessage: string;
}

interface OnboardingActions {
  // Checklist actions
  markStepComplete: (stepId: string) => void;
  dismissChecklist: () => void;
  showChecklist: () => void;

  // Feature tour actions
  startTour: () => void;
  nextTourStep: () => void;
  prevTourStep: () => void;
  skipTour: () => void;
  completeTour: () => void;

  // Celebration actions
  triggerCelebration: (message?: string) => void;
  closeCelebration: () => void;

  // Data sync
  syncWithBackend: (data: BackendOnboardingData) => void;
}

interface BackendOnboardingData {
  steps_completed: string[];
  completion_percent: number;
}

type OnboardingContextType = OnboardingState & OnboardingActions;

const OnboardingContext = createContext<OnboardingContextType | null>(null);

const STORAGE_KEY = 'webrana_onboarding';

interface StoredOnboardingState {
  isChecklistDismissed: boolean;
  isTourCompleted: boolean;
  completedSteps: string[];
}

/**
 * Onboarding Provider component
 * Manages all onboarding state and persistence
 */
export function OnboardingProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<OnboardingState>(() => ({
    steps: defaultOnboardingSteps.map((s) => ({ ...s, completed: s.id === 'account_created' })),
    completionPercent: 25,
    isChecklistVisible: true,
    isChecklistDismissed: false,
    tourSteps: defaultFeatureTourSteps,
    currentTourStep: 0,
    isTourActive: false,
    isTourCompleted: false,
    showCelebration: false,
    celebrationMessage: '',
  }));

  // Load persisted state on mount
  useEffect(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const parsed: StoredOnboardingState = JSON.parse(stored);
        setState((prev) => ({
          ...prev,
          isChecklistDismissed: parsed.isChecklistDismissed,
          isChecklistVisible: !parsed.isChecklistDismissed,
          isTourCompleted: parsed.isTourCompleted,
          steps: prev.steps.map((s) => ({
            ...s,
            completed: parsed.completedSteps.includes(s.id),
          })),
        }));
      }
    } catch {
      // Ignore localStorage errors
    }
  }, []);

  // Persist state changes
  const persistState = useCallback((newState: Partial<StoredOnboardingState>) => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      const current: StoredOnboardingState = stored
        ? JSON.parse(stored)
        : { isChecklistDismissed: false, isTourCompleted: false, completedSteps: [] };
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ ...current, ...newState }));
    } catch {
      // Ignore localStorage errors
    }
  }, []);

  // Checklist actions
  const markStepComplete = useCallback(
    (stepId: string) => {
      setState((prev) => {
        const newSteps = prev.steps.map((s) =>
          s.id === stepId ? { ...s, completed: true } : s
        );
        const completedCount = newSteps.filter((s) => s.completed).length;
        const completionPercent = Math.round((completedCount / newSteps.length) * 100);

        persistState({ completedSteps: newSteps.filter((s) => s.completed).map((s) => s.id) });

        return {
          ...prev,
          steps: newSteps,
          completionPercent,
        };
      });
    },
    [persistState]
  );


  const dismissChecklist = useCallback(() => {
    setState((prev) => ({
      ...prev,
      isChecklistVisible: false,
      isChecklistDismissed: true,
    }));
    persistState({ isChecklistDismissed: true });
  }, [persistState]);

  const showChecklist = useCallback(() => {
    setState((prev) => ({
      ...prev,
      isChecklistVisible: true,
    }));
  }, []);

  // Feature tour actions
  const startTour = useCallback(() => {
    setState((prev) => ({
      ...prev,
      isTourActive: true,
      currentTourStep: 0,
    }));
  }, []);

  const nextTourStep = useCallback(() => {
    setState((prev) => {
      if (prev.currentTourStep >= prev.tourSteps.length - 1) {
        return prev;
      }
      return {
        ...prev,
        currentTourStep: prev.currentTourStep + 1,
      };
    });
  }, []);

  const prevTourStep = useCallback(() => {
    setState((prev) => ({
      ...prev,
      currentTourStep: Math.max(0, prev.currentTourStep - 1),
    }));
  }, []);

  const skipTour = useCallback(() => {
    setState((prev) => ({
      ...prev,
      isTourActive: false,
      isTourCompleted: true,
    }));
    persistState({ isTourCompleted: true });
  }, [persistState]);

  const completeTour = useCallback(() => {
    setState((prev) => ({
      ...prev,
      isTourActive: false,
      isTourCompleted: true,
    }));
    persistState({ isTourCompleted: true });
  }, [persistState]);

  // Celebration actions
  const triggerCelebration = useCallback((message?: string) => {
    setState((prev) => ({
      ...prev,
      showCelebration: true,
      celebrationMessage: message || 'Request pertamamu berhasil! Kamu sudah siap menggunakan Webrana.',
    }));
  }, []);

  const closeCelebration = useCallback(() => {
    setState((prev) => ({
      ...prev,
      showCelebration: false,
    }));
  }, []);

  // Sync with backend data
  const syncWithBackend = useCallback(
    (data: BackendOnboardingData) => {
      setState((prev) => {
        const newSteps = prev.steps.map((s) => ({
          ...s,
          completed: data.steps_completed.includes(s.id),
        }));

        persistState({ completedSteps: data.steps_completed });

        return {
          ...prev,
          steps: newSteps,
          completionPercent: data.completion_percent,
        };
      });
    },
    [persistState]
  );

  const value: OnboardingContextType = {
    ...state,
    markStepComplete,
    dismissChecklist,
    showChecklist,
    startTour,
    nextTourStep,
    prevTourStep,
    skipTour,
    completeTour,
    triggerCelebration,
    closeCelebration,
    syncWithBackend,
  };

  return <OnboardingContext.Provider value={value}>{children}</OnboardingContext.Provider>;
}

/**
 * Hook to access onboarding state and actions
 */
export function useOnboarding(): OnboardingContextType {
  const context = useContext(OnboardingContext);
  if (!context) {
    throw new Error('useOnboarding must be used within OnboardingProvider');
  }
  return context;
}
