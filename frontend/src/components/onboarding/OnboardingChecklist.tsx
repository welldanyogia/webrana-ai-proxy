'use client';

import * as React from 'react';
import { Check, Circle, Key, Zap, BarChart3, ChevronRight, X } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';

export interface OnboardingStep {
  id: string;
  title: string;
  description: string;
  completed: boolean;
  href?: string;
  icon: React.ReactNode;
}

export interface OnboardingChecklistProps {
  steps: OnboardingStep[];
  completionPercent: number;
  onDismiss?: () => void;
  onStepClick?: (stepId: string) => void;
  className?: string;
}

/**
 * Onboarding checklist component
 * Displays on first dashboard visit to guide new users
 * Requirements: 5.1 - Display onboarding checklist
 */
export function OnboardingChecklist({
  steps,
  completionPercent,
  onDismiss,
  onStepClick,
  className,
}: OnboardingChecklistProps) {
  const completedSteps = steps.filter((s) => s.completed).length;
  const isComplete = completionPercent >= 100;

  return (
    <Card className={cn('relative overflow-hidden', className)}>
      {/* Progress bar at top */}
      <div className="absolute top-0 left-0 right-0 h-1 bg-gray-100">
        <div
          className="h-full bg-gradient-to-r from-blue-500 to-blue-600 transition-all duration-500"
          style={{ width: `${completionPercent}%` }}
        />
      </div>

      <CardHeader className="flex flex-row items-start justify-between pt-5">
        <div>
          <CardTitle className="text-lg">
            {isComplete ? '🎉 Selamat!' : 'Mulai dengan Webrana'}
          </CardTitle>
          <p className="text-sm text-gray-500 mt-1">
            {isComplete
              ? 'Kamu sudah siap menggunakan Webrana!'
              : `${completedSteps} dari ${steps.length} langkah selesai`}
          </p>
        </div>
        {onDismiss && (
          <Button
            variant="ghost"
            size="icon"
            onClick={onDismiss}
            className="h-8 w-8 text-gray-400 hover:text-gray-600"
            aria-label="Tutup checklist"
          >
            <X className="h-4 w-4" />
          </Button>
        )}
      </CardHeader>

      <CardContent className="space-y-2">
        {steps.map((step, index) => (
          <OnboardingStepItem
            key={step.id}
            step={step}
            index={index + 1}
            onClick={() => onStepClick?.(step.id)}
          />
        ))}
      </CardContent>
    </Card>
  );
}


interface OnboardingStepItemProps {
  step: OnboardingStep;
  index: number;
  onClick?: () => void;
}

function OnboardingStepItem({ step, index, onClick }: OnboardingStepItemProps) {
  return (
    <button
      onClick={onClick}
      disabled={step.completed}
      className={cn(
        'w-full flex items-center gap-3 p-3 rounded-lg text-left transition-all',
        step.completed
          ? 'bg-green-50 cursor-default'
          : 'bg-gray-50 hover:bg-blue-50 hover:border-blue-200 cursor-pointer'
      )}
    >
      {/* Step indicator */}
      <div
        className={cn(
          'flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center',
          step.completed
            ? 'bg-green-500 text-white'
            : 'bg-white border-2 border-gray-300 text-gray-400'
        )}
      >
        {step.completed ? (
          <Check className="h-4 w-4" />
        ) : (
          <span className="text-sm font-medium">{index}</span>
        )}
      </div>

      {/* Step content */}
      <div className="flex-1 min-w-0">
        <p
          className={cn(
            'font-medium text-sm',
            step.completed ? 'text-green-700' : 'text-gray-900'
          )}
        >
          {step.title}
        </p>
        <p className="text-xs text-gray-500 truncate">{step.description}</p>
      </div>

      {/* Icon */}
      <div
        className={cn(
          'flex-shrink-0',
          step.completed ? 'text-green-500' : 'text-gray-400'
        )}
      >
        {step.completed ? step.icon : <ChevronRight className="h-4 w-4" />}
      </div>
    </button>
  );
}

/**
 * Default onboarding steps configuration
 */
export const defaultOnboardingSteps: Omit<OnboardingStep, 'completed'>[] = [
  {
    id: 'account_created',
    title: 'Buat Akun',
    description: 'Daftar dan verifikasi email',
    href: '/dashboard/settings',
    icon: <Check className="h-4 w-4" />,
  },
  {
    id: 'api_key_added',
    title: 'Tambah API Key',
    description: 'Tambahkan API key dari provider AI',
    href: '/dashboard/api-keys',
    icon: <Key className="h-4 w-4" />,
  },
  {
    id: 'first_request_made',
    title: 'Buat Request Pertama',
    description: 'Kirim request pertama melalui proxy',
    href: '/dashboard/overview',
    icon: <Zap className="h-4 w-4" />,
  },
  {
    id: 'dashboard_viewed',
    title: 'Lihat Dashboard',
    description: 'Cek statistik penggunaan',
    href: '/dashboard/usage',
    icon: <BarChart3 className="h-4 w-4" />,
  },
];

export { OnboardingStepItem };
