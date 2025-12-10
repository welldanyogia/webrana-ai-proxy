'use client';

import { X, ChevronLeft, ChevronRight } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/Button';

export interface TooltipStep {
  id: string;
  title: string;
  description: string;
  targetSelector?: string;
  position?: 'top' | 'bottom' | 'left' | 'right';
}

export interface FeatureTooltipProps {
  steps: TooltipStep[];
  currentStep: number;
  onNext: () => void;
  onPrev: () => void;
  onSkip: () => void;
  onComplete: () => void;
  className?: string;
}

/**
 * Feature tooltip component for highlighting key features
 * Requirements: 5.2 - Tooltips highlighting key features on first dashboard visit
 */
export function FeatureTooltip({
  steps,
  currentStep,
  onNext,
  onPrev,
  onSkip,
  onComplete,
  className,
}: FeatureTooltipProps) {
  const step = steps[currentStep];
  const isFirst = currentStep === 0;
  const isLast = currentStep === steps.length - 1;

  if (!step) return null;

  return (
    <div
      className={cn(
        'fixed z-50 bg-white rounded-xl shadow-2xl border border-gray-200 p-4 max-w-sm',
        'animate-in fade-in-0 zoom-in-95 duration-200',
        className
      )}
      role="tooltip"
      aria-live="polite"
    >
      {/* Close button */}
      <button
        onClick={onSkip}
        className="absolute top-2 right-2 p-1 text-gray-400 hover:text-gray-600 rounded"
        aria-label="Lewati tour"
      >
        <X className="h-4 w-4" />
      </button>

      {/* Step indicator */}
      <div className="flex gap-1 mb-3">
        {steps.map((_, idx) => (
          <div
            key={idx}
            className={cn(
              'h-1 flex-1 rounded-full transition-colors',
              idx <= currentStep ? 'bg-blue-500' : 'bg-gray-200'
            )}
          />
        ))}
      </div>

      {/* Content */}
      <h3 className="font-semibold text-gray-900 mb-1">{step.title}</h3>
      <p className="text-sm text-gray-600 mb-4">{step.description}</p>

      {/* Navigation */}
      <div className="flex items-center justify-between">
        <span className="text-xs text-gray-400">
          {currentStep + 1} / {steps.length}
        </span>
        <div className="flex gap-2">
          {!isFirst && (
            <Button variant="ghost" size="sm" onClick={onPrev}>
              <ChevronLeft className="h-4 w-4 mr-1" />
              Kembali
            </Button>
          )}
          {isLast ? (
            <Button size="sm" onClick={onComplete}>
              Selesai
            </Button>
          ) : (
            <Button size="sm" onClick={onNext}>
              Lanjut
              <ChevronRight className="h-4 w-4 ml-1" />
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}

/**
 * Default feature tour steps
 */
export const defaultFeatureTourSteps: TooltipStep[] = [
  {
    id: 'welcome',
    title: 'Selamat Datang di Webrana! 👋',
    description:
      'Webrana adalah proxy AI yang memungkinkan kamu mengakses berbagai model AI melalui satu API.',
    position: 'bottom',
  },
  {
    id: 'api-keys',
    title: 'Kelola API Keys 🔑',
    description:
      'Tambahkan API key dari provider seperti OpenAI, Anthropic, atau Google untuk mulai menggunakan proxy.',
    targetSelector: '[href="/dashboard/api-keys"]',
    position: 'right',
  },
  {
    id: 'usage',
    title: 'Pantau Penggunaan 📊',
    description:
      'Lihat statistik penggunaan, jumlah request, dan biaya secara real-time.',
    targetSelector: '[href="/dashboard/usage"]',
    position: 'right',
  },
  {
    id: 'billing',
    title: 'Kelola Billing 💳',
    description:
      'Upgrade plan, lihat invoice, dan kelola pembayaran dengan mudah dalam Rupiah.',
    targetSelector: '[href="/dashboard/billing"]',
    position: 'right',
  },
];


