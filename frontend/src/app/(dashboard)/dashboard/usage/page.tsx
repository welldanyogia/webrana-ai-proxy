'use client';

import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
  Legend,
} from 'recharts';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { formatRupiah, formatNumber } from '@/lib/utils';

/** Date range preset options */
type DatePreset = '7d' | '30d' | '90d';

/** Usage stats from API */
interface UsageStats {
  total_requests: number;
  total_input_tokens: number;
  total_output_tokens: number;
  total_tokens: number;
  total_cost_idr: number;
  avg_latency_ms: number;
}

/** Provider usage breakdown */
interface ProviderUsage {
  provider: string;
  request_count: number;
  total_tokens: number;
  total_cost_idr: number;
}

/** Model usage breakdown */
interface ModelUsage {
  model: string;
  provider: string;
  request_count: number;
  total_tokens: number;
  total_cost_idr: number;
}

/** Daily usage for charts */
interface DailyUsage {
  date: string;
  request_count: number;
  total_tokens: number;
  total_cost_idr: number;
}

/** Combined usage response */
interface UsageResponse {
  stats: UsageStats;
  by_provider: ProviderUsage[];
  by_model: ModelUsage[];
  daily: DailyUsage[];
}

const PROVIDER_COLORS: Record<string, string> = {
  openai: '#10B981',
  anthropic: '#F97316',
  google: '#3B82F6',
  qwen: '#8B5CF6',
};

const PIE_COLORS = ['#10B981', '#F97316', '#3B82F6', '#8B5CF6', '#EC4899'];

/**
 * Fetches usage data from the API
 * @param preset - Date range preset (7d, 30d, 90d)
 */
async function fetchUsageData(preset: DatePreset): Promise<UsageResponse> {
  const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';
  const response = await fetch(`${apiUrl}/usage?preset=${preset}`, {
    credentials: 'include',
  });

  if (!response.ok) {
    throw new Error('Failed to fetch usage data');
  }

  return response.json();
}

/**
 * Usage Dashboard Page
 * Requirements: 1.1, 1.2, 1.3, 1.5 - Usage analytics with charts and export
 */
export default function UsagePage() {
  const [preset, setPreset] = useState<DatePreset>('30d');

  const { data, isLoading, error } = useQuery({
    queryKey: ['usage', preset],
    queryFn: () => fetchUsageData(preset),
    staleTime: 60_000,
  });

  const handleExportCSV = () => {
    const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';
    window.open(`${apiUrl}/usage/export?preset=${preset}`, '_blank');
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold">Usage Analytics</h1>
          <p className="text-gray-500">Monitor penggunaan API dan biaya</p>
        </div>
        <div className="flex items-center gap-2">
          <DateRangeSelector value={preset} onChange={setPreset} />
          <Button
            variant="outline"
            onClick={handleExportCSV}
            aria-label="Export data ke CSV"
          >
            📥 Export CSV
          </Button>
        </div>
      </div>

      {/* Loading State */}
      {isLoading && <LoadingSkeleton />}

      {/* Error State */}
      {error && (
        <Card>
          <CardContent className="py-8 text-center">
            <div className="text-4xl mb-4" role="img" aria-label="Error">
              ⚠️
            </div>
            <p className="text-red-500 font-medium">Gagal memuat data usage</p>
            <p className="text-sm text-gray-500 mt-1">
              Pastikan backend berjalan dan coba lagi
            </p>
          </CardContent>
        </Card>
      )}

      {/* Data Display */}
      {data && data.stats.total_requests > 0 && (
        <>
          {/* Summary Cards */}
          <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-4">
            <StatCard
              title="Total Requests"
              value={formatNumber(data.stats.total_requests)}
              icon="📊"
            />
            <StatCard
              title="Total Tokens"
              value={formatNumber(data.stats.total_tokens)}
              subtitle={`In: ${formatNumber(data.stats.total_input_tokens)} | Out: ${formatNumber(data.stats.total_output_tokens)}`}
              icon="🔤"
            />
            <StatCard
              title="Estimated Cost"
              value={formatRupiah(data.stats.total_cost_idr)}
              icon="💰"
            />
            <StatCard
              title="Avg Latency"
              value={`${Math.round(data.stats.avg_latency_ms)}ms`}
              icon="⚡"
            />
          </div>

          {/* Charts Row */}
          <div className="grid lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Daily Usage</CardTitle>
              </CardHeader>
              <CardContent>
                <DailyUsageChart data={data.daily} />
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Usage by Provider</CardTitle>
              </CardHeader>
              <CardContent>
                <ProviderPieChart data={data.by_provider} />
              </CardContent>
            </Card>
          </div>

          {/* Model Usage Table */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Usage by Model</CardTitle>
            </CardHeader>
            <CardContent>
              <ModelUsageTable data={data.by_model} />
            </CardContent>
          </Card>
        </>
      )}

      {/* Empty State */}
      {data && data.stats.total_requests === 0 && (
        <Card>
          <CardContent className="py-12 text-center">
            <div className="text-4xl mb-4" role="img" aria-label="Empty">
              📊
            </div>
            <h3 className="text-lg font-medium mb-2">Belum ada data usage</h3>
            <p className="text-gray-500 text-sm max-w-md mx-auto">
              Mulai gunakan API proxy untuk melihat statistik penggunaan di
              sini. Data akan diperbarui secara real-time.
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}


/**
 * Loading skeleton for perceived performance
 */
function LoadingSkeleton() {
  return (
    <div className="space-y-6 animate-pulse">
      <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {[...Array(4)].map((_, i) => (
          <Card key={i}>
            <CardContent className="pt-6">
              <div className="h-4 bg-gray-200 rounded w-24 mb-2" />
              <div className="h-8 bg-gray-200 rounded w-32" />
            </CardContent>
          </Card>
        ))}
      </div>
      <div className="grid lg:grid-cols-2 gap-6">
        <Card>
          <CardContent className="pt-6">
            <div className="h-64 bg-gray-100 rounded" />
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="h-64 bg-gray-100 rounded" />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

/**
 * Date range selector component
 * @param value - Current selected preset
 * @param onChange - Callback when preset changes
 */
function DateRangeSelector({
  value,
  onChange,
}: {
  value: DatePreset;
  onChange: (preset: DatePreset) => void;
}) {
  const presets: { value: DatePreset; label: string }[] = [
    { value: '7d', label: '7 Hari' },
    { value: '30d', label: '30 Hari' },
    { value: '90d', label: '90 Hari' },
  ];

  return (
    <div
      className="flex rounded-lg border overflow-hidden"
      role="group"
      aria-label="Pilih rentang waktu"
    >
      {presets.map((preset) => (
        <button
          key={preset.value}
          onClick={() => onChange(preset.value)}
          className={`px-3 py-1.5 text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-inset ${
            value === preset.value
              ? 'bg-blue-600 text-white'
              : 'bg-white text-gray-600 hover:bg-gray-50'
          }`}
          aria-pressed={value === preset.value}
        >
          {preset.label}
        </button>
      ))}
    </div>
  );
}

/**
 * Statistics card component
 */
function StatCard({
  title,
  value,
  subtitle,
  icon,
}: {
  title: string;
  value: string;
  subtitle?: string;
  icon: string;
}) {
  return (
    <Card>
      <CardContent className="pt-6">
        <div className="flex items-start justify-between">
          <div>
            <p className="text-sm text-gray-500">{title}</p>
            <p className="text-2xl font-bold mt-1">{value}</p>
            {subtitle && (
              <p className="text-xs text-gray-400 mt-1">{subtitle}</p>
            )}
          </div>
          <span className="text-2xl" role="img" aria-hidden="true">
            {icon}
          </span>
        </div>
      </CardContent>
    </Card>
  );
}

/**
 * Daily usage line chart component
 */
function DailyUsageChart({ data }: { data: DailyUsage[] }) {
  if (data.length === 0) {
    return (
      <div className="h-64 flex items-center justify-center text-gray-400">
        Tidak ada data untuk periode ini
      </div>
    );
  }

  return (
    <div className="h-64" role="img" aria-label="Grafik penggunaan harian">
      <ResponsiveContainer width="100%" height="100%">
        <LineChart
          data={data}
          margin={{ top: 5, right: 20, bottom: 5, left: 0 }}
        >
          <CartesianGrid strokeDasharray="3 3" stroke="#E5E7EB" />
          <XAxis
            dataKey="date"
            tick={{ fontSize: 12 }}
            tickFormatter={(date: string) => {
              const d = new Date(date);
              return `${d.getDate()}/${d.getMonth() + 1}`;
            }}
          />
          <YAxis tick={{ fontSize: 12 }} />
          <Tooltip
            formatter={(val: number, name: string) => {
              if (name === 'request_count')
                return [formatNumber(val), 'Requests'];
              if (name === 'total_tokens') return [formatNumber(val), 'Tokens'];
              return [val, name];
            }}
            labelFormatter={(label: string) => {
              const d = new Date(label);
              return d.toLocaleDateString('id-ID', {
                weekday: 'long',
                year: 'numeric',
                month: 'long',
                day: 'numeric',
              });
            }}
          />
          <Legend />
          <Line
            type="monotone"
            dataKey="request_count"
            stroke="#3B82F6"
            strokeWidth={2}
            dot={false}
            name="Requests"
          />
          <Line
            type="monotone"
            dataKey="total_tokens"
            stroke="#10B981"
            strokeWidth={2}
            dot={false}
            name="Tokens"
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}

/**
 * Provider usage pie chart component
 */
function ProviderPieChart({ data }: { data: ProviderUsage[] }) {
  if (data.length === 0) {
    return (
      <div className="h-64 flex items-center justify-center text-gray-400">
        Tidak ada data untuk periode ini
      </div>
    );
  }

  const chartData = data.map((item) => ({
    name: item.provider.charAt(0).toUpperCase() + item.provider.slice(1),
    value: item.request_count,
    color: PROVIDER_COLORS[item.provider] || PIE_COLORS[0],
  }));

  return (
    <div className="h-64" role="img" aria-label="Grafik penggunaan per provider">
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          <Pie
            data={chartData}
            cx="50%"
            cy="50%"
            innerRadius={60}
            outerRadius={80}
            paddingAngle={2}
            dataKey="value"
            label={({ name, percent }: { name: string; percent: number }) =>
              `${name} ${(percent * 100).toFixed(0)}%`
            }
            labelLine={false}
          >
            {chartData.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={entry.color} />
            ))}
          </Pie>
          <Tooltip
            formatter={(val: number) => [formatNumber(val), 'Requests']}
          />
          <Legend />
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
}

/**
 * Model usage table component with accessibility
 */
function ModelUsageTable({ data }: { data: ModelUsage[] }) {
  if (data.length === 0) {
    return (
      <div className="py-8 text-center text-gray-400">
        Tidak ada data untuk periode ini
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm" role="table" aria-label="Usage by model">
        <thead>
          <tr className="border-b">
            <th scope="col" className="text-left py-3 px-2 font-medium text-gray-500">
              Model
            </th>
            <th scope="col" className="text-left py-3 px-2 font-medium text-gray-500">
              Provider
            </th>
            <th scope="col" className="text-right py-3 px-2 font-medium text-gray-500">
              Requests
            </th>
            <th scope="col" className="text-right py-3 px-2 font-medium text-gray-500">
              Tokens
            </th>
            <th scope="col" className="text-right py-3 px-2 font-medium text-gray-500">
              Cost
            </th>
          </tr>
        </thead>
        <tbody>
          {data.map((row, index) => (
            <tr
              key={`${row.model}-${index}`}
              className="border-b last:border-0 hover:bg-gray-50"
            >
              <td className="py-3 px-2 font-mono text-xs">{row.model}</td>
              <td className="py-3 px-2">
                <ProviderBadge provider={row.provider} />
              </td>
              <td className="py-3 px-2 text-right">
                {formatNumber(row.request_count)}
              </td>
              <td className="py-3 px-2 text-right">
                {formatNumber(row.total_tokens)}
              </td>
              <td className="py-3 px-2 text-right font-medium">
                {formatRupiah(row.total_cost_idr)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/**
 * Provider badge with consistent colors
 */
function ProviderBadge({ provider }: { provider: string }) {
  const colorClasses: Record<string, string> = {
    openai: 'bg-green-100 text-green-800',
    anthropic: 'bg-orange-100 text-orange-800',
    google: 'bg-blue-100 text-blue-800',
    qwen: 'bg-purple-100 text-purple-800',
  };

  return (
    <span
      className={`px-2 py-0.5 rounded text-xs font-medium ${
        colorClasses[provider.toLowerCase()] || 'bg-gray-100 text-gray-800'
      }`}
    >
      {provider}
    </span>
  );
}
