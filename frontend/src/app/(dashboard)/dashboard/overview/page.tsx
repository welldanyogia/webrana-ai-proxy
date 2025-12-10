'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';

export default function OverviewPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Overview</h1>
        <p className="text-gray-500">Selamat datang di dashboard Webrana</p>
      </div>

      {/* Plan Info */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Plan Saat Ini</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-2xl font-bold">Free</p>
              <p className="text-gray-500 text-sm">Rp 0/bulan</p>
            </div>
            <button className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700">
              Upgrade Plan
            </button>
          </div>
        </CardContent>
      </Card>

      {/* Usage Summary */}
      <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          title="Total Requests"
          value="523"
          subtitle="dari 1.000 limit"
          percentage={52}
        />
        <StatCard
          title="Tokens Used"
          value="45.2K"
          subtitle="bulan ini"
        />
        <StatCard
          title="Estimated Cost"
          value="Rp 12.500"
          subtitle="bulan ini"
        />
        <StatCard
          title="Avg Latency"
          value="245ms"
          subtitle="p95"
        />
      </div>

      {/* Provider Usage */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Usage by Provider</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <ProviderUsageBar provider="OpenAI" usage={320} color="bg-green-500" />
            <ProviderUsageBar provider="Anthropic" usage={150} color="bg-orange-500" />
            <ProviderUsageBar provider="Google" usage={40} color="bg-blue-500" />
            <ProviderUsageBar provider="Qwen" usage={13} color="bg-purple-500" />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function StatCard({
  title,
  value,
  subtitle,
  percentage,
}: {
  title: string;
  value: string;
  subtitle: string;
  percentage?: number;
}) {
  return (
    <Card>
      <CardContent className="pt-6">
        <p className="text-sm text-gray-500">{title}</p>
        <p className="text-2xl font-bold mt-1">{value}</p>
        <p className="text-xs text-gray-400 mt-1">{subtitle}</p>
        {percentage !== undefined && (
          <div className="mt-3">
            <div className="h-2 bg-gray-100 rounded-full overflow-hidden">
              <div
                className="h-full bg-blue-500 rounded-full"
                style={{ width: `${percentage}%` }}
              />
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function ProviderUsageBar({
  provider,
  usage,
  color,
}: {
  provider: string;
  usage: number;
  color: string;
}) {
  const maxUsage = 523; // Total for percentage calculation
  const percentage = (usage / maxUsage) * 100;

  return (
    <div className="flex items-center gap-4">
      <div className="w-24 text-sm font-medium">{provider}</div>
      <div className="flex-1">
        <div className="h-4 bg-gray-100 rounded-full overflow-hidden">
          <div
            className={`h-full ${color} rounded-full`}
            style={{ width: `${percentage}%` }}
          />
        </div>
      </div>
      <div className="w-16 text-right text-sm text-gray-600">{usage}</div>
    </div>
  );
}
