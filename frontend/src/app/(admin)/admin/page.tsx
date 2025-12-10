'use client';

import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { formatRupiah, formatNumber } from '@/lib/utils';

interface AdminStats {
  total_users: number;
  active_subscriptions: number;
  mrr_idr: number;
  requests_today: number;
  requests_this_month: number;
}

interface UserItem {
  id: string;
  email: string;
  name: string | null;
  plan_tier: string;
  requests_this_month: number;
  created_at: string;
}

interface UserListResponse {
  users: UserItem[];
  total: number;
  page: number;
  per_page: number;
}

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';

async function fetchAdminStats(): Promise<AdminStats> {
  const res = await fetch(`${API_URL}/admin/stats`, { credentials: 'include' });
  if (!res.ok) throw new Error('Failed to fetch stats');
  return res.json();
}

async function fetchUsers(page: number, search: string): Promise<UserListResponse> {
  const params = new URLSearchParams({ page: String(page), per_page: '10' });
  if (search) params.set('search', search);
  const res = await fetch(`${API_URL}/admin/users?${params}`, { credentials: 'include' });
  if (!res.ok) throw new Error('Failed to fetch users');
  return res.json();
}

export default function AdminPage() {
  const [page, setPage] = useState(1);
  const [search, setSearch] = useState('');
  const { data: stats, isLoading: statsLoading } = useQuery({ queryKey: ['admin-stats'], queryFn: fetchAdminStats });
  const { data: usersData, isLoading: usersLoading } = useQuery({ queryKey: ['admin-users', page, search], queryFn: () => fetchUsers(page, search) });

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white border-b"><div className="px-6 py-4"><h1 className="text-2xl font-bold">Admin Dashboard</h1><p className="text-gray-500">Kelola pengguna dan monitor sistem</p></div></header>
      <main className="p-6 space-y-6">
        <div className="grid sm:grid-cols-2 lg:grid-cols-5 gap-4">
          <StatCard title="Total Users" value={stats ? formatNumber(stats.total_users) : '-'} icon="👥" loading={statsLoading} />
          <StatCard title="Active Subs" value={stats ? formatNumber(stats.active_subscriptions) : '-'} icon="✅" loading={statsLoading} />
          <StatCard title="MRR" value={stats ? formatRupiah(stats.mrr_idr) : '-'} icon="💰" loading={statsLoading} />
          <StatCard title="Today" value={stats ? formatNumber(stats.requests_today) : '-'} icon="📊" loading={statsLoading} />
          <StatCard title="This Month" value={stats ? formatNumber(stats.requests_this_month) : '-'} icon="📈" loading={statsLoading} />
        </div>
        <Card>
          <CardHeader><div className="flex items-center justify-between"><CardTitle className="text-lg">User Management</CardTitle>
            <input type="text" placeholder="Search..." value={search} onChange={(e) => { setSearch(e.target.value); setPage(1); }} className="px-3 py-1.5 border rounded-lg text-sm w-64" /></div></CardHeader>
          <CardContent>
            {usersLoading ? <div className="animate-pulse space-y-2">{[1,2,3].map(i => <div key={i} className="h-12 bg-gray-100 rounded" />)}</div>
             : usersData ? <><UserTable users={usersData.users} /><Pagination page={page} total={usersData.total} perPage={usersData.per_page} onPageChange={setPage} /></>
             : <p className="text-gray-400 text-center py-8">Failed to load</p>}
          </CardContent>
        </Card>
      </main>
    </div>
  );
}

function StatCard({ title, value, icon, loading }: { title: string; value: string; icon: string; loading: boolean }) {
  return (<Card><CardContent className="pt-6"><div className="flex items-start justify-between"><div><p className="text-sm text-gray-500">{title}</p>{loading ? <div className="h-8 w-20 bg-gray-100 rounded animate-pulse mt-1" /> : <p className="text-xl font-bold mt-1">{value}</p>}</div><span className="text-2xl">{icon}</span></div></CardContent></Card>);
}

function UserTable({ users }: { users: UserItem[] }) {
  const colors: Record<string, string> = { free: 'bg-gray-100 text-gray-800', starter: 'bg-blue-100 text-blue-800', pro: 'bg-purple-100 text-purple-800', team: 'bg-green-100 text-green-800' };
  if (!users.length) return <p className="text-gray-400 text-center py-8">No users</p>;
  return (<div className="overflow-x-auto"><table className="w-full text-sm"><thead><tr className="border-b"><th className="text-left py-3 px-2 font-medium text-gray-500">Email</th><th className="text-left py-3 px-2 font-medium text-gray-500">Plan</th><th className="text-right py-3 px-2 font-medium text-gray-500">Requests</th><th className="text-left py-3 px-2 font-medium text-gray-500">Joined</th></tr></thead>
    <tbody>{users.map(u => (<tr key={u.id} className="border-b hover:bg-gray-50"><td className="py-3 px-2">{u.email}</td><td className="py-3 px-2"><span className={`px-2 py-0.5 rounded text-xs font-medium ${colors[u.plan_tier] || colors.free}`}>{u.plan_tier}</span></td><td className="py-3 px-2 text-right">{formatNumber(u.requests_this_month)}</td><td className="py-3 px-2 text-gray-500">{new Date(u.created_at).toLocaleDateString('id-ID')}</td></tr>))}</tbody></table></div>);
}

function Pagination({ page, total, perPage, onPageChange }: { page: number; total: number; perPage: number; onPageChange: (p: number) => void }) {
  const totalPages = Math.ceil(total / perPage);
  if (totalPages <= 1) return null;
  return (<div className="flex items-center justify-between mt-4 pt-4 border-t"><p className="text-sm text-gray-500">Page {page} of {totalPages}</p><div className="flex gap-2"><Button variant="outline" size="sm" disabled={page === 1} onClick={() => onPageChange(page - 1)}>Prev</Button><Button variant="outline" size="sm" disabled={page === totalPages} onClick={() => onPageChange(page + 1)}>Next</Button></div></div>);
}
