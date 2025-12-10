'use client';

import Link from 'next/link';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';

/**
 * Forgot Password placeholder page
 * Requirements: 9.5 - Add "Forgot Password" link (implementation in Week 3)
 */
export default function ForgotPasswordPage() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 px-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <Link href="/" className="text-2xl font-bold text-blue-600 mb-2 block">
            Webrana
          </Link>
          <CardTitle>Lupa Password</CardTitle>
          <CardDescription>
            Fitur reset password akan segera hadir
          </CardDescription>
        </CardHeader>
        <CardContent className="text-center">
          <div className="py-8">
            <span className="text-6xl">🔐</span>
            <p className="text-gray-500 mt-4">
              Fitur ini sedang dalam pengembangan dan akan tersedia di update berikutnya.
            </p>
          </div>
          <Button asChild className="w-full">
            <Link href="/login">Kembali ke Login</Link>
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
