import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

/**
 * Middleware for authentication redirect
 * Requirements: 7.4 - Redirect to /login if not authenticated
 */
export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // Check if accessing dashboard routes
  if (pathname.startsWith('/dashboard')) {
    // Check for auth token (simplified - in production use NextAuth session)
    const token = request.cookies.get('auth-token')?.value;

    if (!token) {
      // Preserve the original URL for redirect after login
      const loginUrl = new URL('/login', request.url);
      loginUrl.searchParams.set('callbackUrl', pathname);
      return NextResponse.redirect(loginUrl);
    }
  }

  return NextResponse.next();
}

export const config = {
  matcher: ['/dashboard/:path*'],
};
