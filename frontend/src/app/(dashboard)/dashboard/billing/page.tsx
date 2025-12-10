'use client';

import { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { formatRupiah, formatNumber } from '@/lib/utils';

/** Plan tier definition */
interface PlanTier {
  id: string;
  name: string;
  price: number;
  priceWithPpn: number;
  requests: number;
  apiKeys: string;
  providers: string;
  features: string[];
  popular?: boolean;
}

/** Subscription from API */
interface Subscription {
  id: string;
  plan_tier: string;
  price_idr: number;
  status: string;
  current_period_start: string;
  current_period_end: string;
}

/** Invoice from API */
interface Invoice {
  id: string;
  invoice_number: string;
  subtotal_idr: number;
  ppn_idr: number;
  total_idr: number;
  payment_method: string;
  status: string;
  paid_at: string;
  created_at: string;
}

const PLANS: PlanTier[] = [
  {
    id: 'free',
    name: 'Free',
    price: 0,
    priceWithPpn: 0,
    requests: 1000,
    apiKeys: '1',
    providers: '1 provider',
    features: ['Basic analytics', 'Community support'],
  },
  {
    id: 'starter',
    name: 'Starter',
    price: 49000,
    priceWithPpn: 54390,
    requests: 10000,
    apiKeys: '5',
    providers: 'Max 2 providers',
    features: ['Full analytics', 'Email support', 'CSV export'],
  },
  {
    id: 'pro',
    name: 'Pro',
    price: 99000,
    priceWithPpn: 109890,
    requests: 50000,
    apiKeys: 'Unlimited',
    providers: 'All providers',
    features: ['Priority support', 'API webhooks', 'Custom alerts'],
    popular: true,
  },
  {
    id: 'team',
    name: 'Team',
    price: 299000,
    priceWithPpn: 331890,
    requests: 200000,
    apiKeys: 'Unlimited',
    providers: 'All + 10 users',
    features: ['Team management', 'SSO', 'Dedicated support'],
  },
];

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';

/**
 * Fetches current subscription from API
 */
async function fetchSubscription(): Promise<Subscription | null> {
  const response = await fetch(`${API_URL}/billing/subscription`, {
    credentials: 'include',
  });
  if (!response.ok) return null;
  return response.json();
}

/**
 * Fetches invoice history from API
 */
async function fetchInvoices(): Promise<Invoice[]> {
  const response = await fetch(`${API_URL}/billing/invoices`, {
    credentials: 'include',
  });
  if (!response.ok) return [];
  return response.json();
}

/**
 * Creates a subscription and returns Midtrans redirect URL
 */
async function createSubscription(planId: string): Promise<{ redirect_url: string }> {
  const response = await fetch(`${API_URL}/billing/subscribe`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify({ plan: planId }),
  });
  if (!response.ok) {
    throw new Error('Failed to create subscription');
  }
  return response.json();
}


/**
 * Billing Page - Subscription management and invoice history
 * Requirements: 4.6 - Billing dashboard with plan selection
 */
export default function BillingPage() {
  const [selectedPlan, setSelectedPlan] = useState<string | null>(null);

  const { data: subscription, isLoading: subLoading } = useQuery({
    queryKey: ['subscription'],
    queryFn: fetchSubscription,
    staleTime: 60_000,
  });

  const { data: invoices = [], isLoading: invLoading } = useQuery({
    queryKey: ['invoices'],
    queryFn: fetchInvoices,
    staleTime: 60_000,
  });

  const subscribeMutation = useMutation({
    mutationFn: createSubscription,
    onSuccess: (data) => {
      window.location.href = data.redirect_url;
    },
  });

  const currentPlan = subscription?.plan_tier || 'free';

  const handleUpgrade = (planId: string) => {
    if (planId === 'free' || planId === currentPlan) return;
    setSelectedPlan(planId);
    subscribeMutation.mutate(planId);
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Billing</h1>
        <p className="text-gray-500">Kelola langganan dan lihat riwayat pembayaran</p>
      </div>

      <CurrentSubscriptionCard subscription={subscription} isLoading={subLoading} />

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Pilih Plan</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-4">
            {PLANS.map((plan) => (
              <PlanCard
                key={plan.id}
                plan={plan}
                isCurrentPlan={currentPlan === plan.id}
                isLoading={subscribeMutation.isPending && selectedPlan === plan.id}
                onSelect={() => handleUpgrade(plan.id)}
              />
            ))}
          </div>
          <p className="text-xs text-gray-400 mt-4 text-center">
            * Harga sudah termasuk PPN 11%
          </p>
        </CardContent>
      </Card>

      <InvoiceHistoryCard invoices={invoices} isLoading={invLoading} />
    </div>
  );
}

/**
 * Displays current subscription status
 */
function CurrentSubscriptionCard({
  subscription,
  isLoading,
}: {
  subscription: Subscription | null | undefined;
  isLoading: boolean;
}) {
  if (isLoading) {
    return (
      <Card>
        <CardContent className="py-6">
          <div className="animate-pulse space-y-2">
            <div className="h-6 bg-gray-200 rounded w-32" />
            <div className="h-4 bg-gray-200 rounded w-48" />
          </div>
        </CardContent>
      </Card>
    );
  }

  const planName = subscription?.plan_tier
    ? subscription.plan_tier.charAt(0).toUpperCase() + subscription.plan_tier.slice(1)
    : 'Free';

  const endDate = subscription?.current_period_end
    ? new Date(subscription.current_period_end).toLocaleDateString('id-ID', {
        day: 'numeric',
        month: 'long',
        year: 'numeric',
      })
    : null;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Langganan Saat Ini</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-2xl font-bold">{planName}</p>
            {subscription?.status === 'active' && endDate && (
              <p className="text-sm text-gray-500">Berlaku hingga {endDate}</p>
            )}
            {!subscription && (
              <p className="text-sm text-gray-500">Upgrade untuk fitur lebih lengkap</p>
            )}
          </div>
          <StatusBadge status={subscription?.status || 'free'} />
        </div>
      </CardContent>
    </Card>
  );
}


/**
 * Individual plan card with pricing and features
 */
function PlanCard({
  plan,
  isCurrentPlan,
  isLoading,
  onSelect,
}: {
  plan: PlanTier;
  isCurrentPlan: boolean;
  isLoading: boolean;
  onSelect: () => void;
}) {
  return (
    <div
      className={`relative border rounded-lg p-4 ${
        plan.popular ? 'border-blue-500 ring-2 ring-blue-100' : 'border-gray-200'
      } ${isCurrentPlan ? 'bg-blue-50' : 'bg-white'}`}
      role="article"
      aria-label={`Plan ${plan.name}`}
    >
      {plan.popular && (
        <span className="absolute -top-2 left-1/2 -translate-x-1/2 px-2 py-0.5 bg-blue-600 text-white text-xs rounded-full">
          Popular
        </span>
      )}

      <div className="text-center mb-4">
        <h3 className="font-semibold text-lg">{plan.name}</h3>
        <p className="text-2xl font-bold mt-2">
          {plan.price === 0 ? 'Gratis' : formatRupiah(plan.priceWithPpn)}
        </p>
        {plan.price > 0 && <p className="text-xs text-gray-400">/bulan</p>}
      </div>

      <ul className="space-y-2 text-sm mb-4" aria-label="Fitur plan">
        <li className="flex items-center gap-2">
          <span className="text-green-500">✓</span>
          {formatNumber(plan.requests)} requests/bulan
        </li>
        <li className="flex items-center gap-2">
          <span className="text-green-500">✓</span>
          {plan.apiKeys} API keys
        </li>
        <li className="flex items-center gap-2">
          <span className="text-green-500">✓</span>
          {plan.providers}
        </li>
        {plan.features.map((feature) => (
          <li key={feature} className="flex items-center gap-2">
            <span className="text-green-500">✓</span>
            {feature}
          </li>
        ))}
      </ul>

      <Button
        className="w-full"
        variant={isCurrentPlan ? 'outline' : plan.popular ? 'default' : 'outline'}
        disabled={isCurrentPlan || plan.id === 'free' || isLoading}
        onClick={onSelect}
        aria-label={isCurrentPlan ? 'Plan saat ini' : `Pilih plan ${plan.name}`}
      >
        {isLoading ? 'Memproses...' : isCurrentPlan ? 'Plan Saat Ini' : 'Pilih Plan'}
      </Button>
    </div>
  );
}

/**
 * Invoice history table
 */
function InvoiceHistoryCard({
  invoices,
  isLoading,
}: {
  invoices: Invoice[];
  isLoading: boolean;
}) {
  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Riwayat Invoice</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="animate-pulse space-y-3">
            {[...Array(3)].map((_, i) => (
              <div key={i} className="h-12 bg-gray-100 rounded" />
            ))}
          </div>
        </CardContent>
      </Card>
    );
  }

  if (invoices.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Riwayat Invoice</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-center text-gray-500 py-8">Belum ada riwayat pembayaran</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Riwayat Invoice</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="overflow-x-auto">
          <table className="w-full text-sm" role="table" aria-label="Riwayat invoice">
            <thead>
              <tr className="border-b">
                <th scope="col" className="text-left py-3 px-2 font-medium text-gray-500">
                  Invoice
                </th>
                <th scope="col" className="text-left py-3 px-2 font-medium text-gray-500">
                  Tanggal
                </th>
                <th scope="col" className="text-right py-3 px-2 font-medium text-gray-500">
                  Total
                </th>
                <th scope="col" className="text-center py-3 px-2 font-medium text-gray-500">
                  Status
                </th>
                <th scope="col" className="text-right py-3 px-2 font-medium text-gray-500">
                  Aksi
                </th>
              </tr>
            </thead>
            <tbody>
              {invoices.map((invoice) => (
                <tr key={invoice.id} className="border-b last:border-0 hover:bg-gray-50">
                  <td className="py-3 px-2 font-mono text-xs">{invoice.invoice_number}</td>
                  <td className="py-3 px-2">
                    {new Date(invoice.created_at).toLocaleDateString('id-ID')}
                  </td>
                  <td className="py-3 px-2 text-right font-medium">
                    {formatRupiah(invoice.total_idr)}
                  </td>
                  <td className="py-3 px-2 text-center">
                    <StatusBadge status={invoice.status} />
                  </td>
                  <td className="py-3 px-2 text-right">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() =>
                        window.open(`${API_URL}/billing/invoices/${invoice.id}/download`, '_blank')
                      }
                      aria-label={`Download invoice ${invoice.invoice_number}`}
                    >
                      📥 Download
                    </Button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </CardContent>
    </Card>
  );
}

/**
 * Status badge component
 */
function StatusBadge({ status }: { status: string }) {
  const statusStyles: Record<string, string> = {
    active: 'bg-green-100 text-green-800',
    paid: 'bg-green-100 text-green-800',
    pending: 'bg-yellow-100 text-yellow-800',
    expired: 'bg-red-100 text-red-800',
    cancelled: 'bg-gray-100 text-gray-800',
    free: 'bg-blue-100 text-blue-800',
  };

  const statusLabels: Record<string, string> = {
    active: 'Aktif',
    paid: 'Lunas',
    pending: 'Pending',
    expired: 'Expired',
    cancelled: 'Dibatalkan',
    free: 'Free',
  };

  return (
    <span
      className={`px-2 py-0.5 rounded text-xs font-medium ${
        statusStyles[status] || 'bg-gray-100 text-gray-800'
      }`}
    >
      {statusLabels[status] || status}
    </span>
  );
}
