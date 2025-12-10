'use client';

import * as React from 'react';
import { useEffect, useState } from 'react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/Button';

export interface CompletionCelebrationProps {
  isVisible: boolean;
  onClose: () => void;
  title?: string;
  message?: string;
  className?: string;
}

/**
 * Celebration animation component for first successful request
 * Requirements: 5.4 - Show celebration animation on first successful request
 */
export function CompletionCelebration({
  isVisible,
  onClose,
  title = '🎉 Selamat!',
  message = 'Request pertamamu berhasil! Kamu sudah siap menggunakan Webrana.',
  className,
}: CompletionCelebrationProps) {
  const [confetti, setConfetti] = useState<ConfettiPiece[]>([]);

  useEffect(() => {
    if (isVisible) {
      // Generate confetti pieces
      const pieces: ConfettiPiece[] = Array.from({ length: 50 }, (_, i) => ({
        id: i,
        x: Math.random() * 100,
        delay: Math.random() * 0.5,
        duration: 2 + Math.random() * 2,
        color: CONFETTI_COLORS[Math.floor(Math.random() * CONFETTI_COLORS.length)],
        rotation: Math.random() * 360,
      }));
      setConfetti(pieces);

      // Auto-close after 5 seconds
      const timer = setTimeout(onClose, 5000);
      return () => clearTimeout(timer);
    }
  }, [isVisible, onClose]);

  if (!isVisible) return null;

  return (
    <div
      className={cn(
        'fixed inset-0 z-50 flex items-center justify-center',
        'bg-black/50 backdrop-blur-sm',
        'animate-in fade-in-0 duration-300',
        className
      )}
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-labelledby="celebration-title"
    >
      {/* Confetti */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        {confetti.map((piece) => (
          <div
            key={piece.id}
            className="absolute w-3 h-3 rounded-sm animate-confetti"
            style={{
              left: `${piece.x}%`,
              backgroundColor: piece.color,
              animationDelay: `${piece.delay}s`,
              animationDuration: `${piece.duration}s`,
              transform: `rotate(${piece.rotation}deg)`,
            }}
          />
        ))}
      </div>

      {/* Modal content */}
      <div
        className={cn(
          'relative bg-white rounded-2xl shadow-2xl p-8 max-w-md mx-4',
          'animate-in zoom-in-95 duration-300',
          'text-center'
        )}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Animated emoji */}
        <div className="text-6xl mb-4 animate-bounce">{title.includes('🎉') ? '🎉' : '✨'}</div>

        <h2 id="celebration-title" className="text-2xl font-bold text-gray-900 mb-2">
          {title.replace(/🎉|✨/g, '').trim() || 'Selamat!'}
        </h2>

        <p className="text-gray-600 mb-6">{message}</p>

        <div className="flex flex-col gap-3">
          <Button onClick={onClose} className="w-full">
            Lanjutkan ke Dashboard
          </Button>
          <Button variant="ghost" onClick={onClose} className="w-full text-sm">
            Tutup
          </Button>
        </div>

        {/* Achievement badge */}
        <div className="mt-6 pt-4 border-t">
          <div className="inline-flex items-center gap-2 px-3 py-1.5 bg-gradient-to-r from-yellow-100 to-orange-100 rounded-full">
            <span className="text-lg">🏆</span>
            <span className="text-sm font-medium text-orange-800">
              Achievement Unlocked: First Request!
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

interface ConfettiPiece {
  id: number;
  x: number;
  delay: number;
  duration: number;
  color: string;
  rotation: number;
}

const CONFETTI_COLORS = [
  '#3B82F6', // blue
  '#10B981', // green
  '#F59E0B', // yellow
  '#EF4444', // red
  '#8B5CF6', // purple
  '#EC4899', // pink
  '#06B6D4', // cyan
];

export { CONFETTI_COLORS };
