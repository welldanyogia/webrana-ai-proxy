'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Input } from '@/components/ui/Input';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  DialogTrigger,
} from '@/components/ui/Dialog';

type Provider = 'openai' | 'anthropic' | 'google' | 'qwen';

interface ApiKey {
  id: string;
  provider: Provider;
  maskedKey: string;
  createdAt: string;
  lastUsed?: string;
}

interface ProxyKey {
  id: string;
  name: string;
  key: string;
  createdAt: string;
}

const providerInfo: Record<Provider, { name: string; prefix: string; color: string }> = {
  openai: { name: 'OpenAI', prefix: 'sk-', color: 'bg-green-100 text-green-800' },
  anthropic: { name: 'Anthropic', prefix: 'sk-ant-', color: 'bg-orange-100 text-orange-800' },
  google: { name: 'Google AI', prefix: 'AI', color: 'bg-blue-100 text-blue-800' },
  qwen: { name: 'Qwen', prefix: 'sk-', color: 'bg-purple-100 text-purple-800' },
};

// Mock data
const mockApiKeys: ApiKey[] = [
  { id: '1', provider: 'openai', maskedKey: 'sk-...abc123', createdAt: '2024-12-01', lastUsed: '2024-12-09' },
  { id: '2', provider: 'anthropic', maskedKey: 'sk-ant-...xyz789', createdAt: '2024-12-05' },
];

const mockProxyKeys: ProxyKey[] = [
  { id: '1', name: 'Production', key: 'wrn_live_abc123xyz789', createdAt: '2024-12-01' },
];

export default function ApiKeysPage() {
  const [apiKeys, setApiKeys] = useState<ApiKey[]>(mockApiKeys);
  const [proxyKeys] = useState<ProxyKey[]>(mockProxyKeys);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [keyToDelete, setKeyToDelete] = useState<string | null>(null);


  const handleCopyKey = async (key: string) => {
    await navigator.clipboard.writeText(key);
    // TODO: Show toast notification
  };

  const handleDeleteKey = (id: string) => {
    setApiKeys(apiKeys.filter((k) => k.id !== id));
    setDeleteDialogOpen(false);
    setKeyToDelete(null);
  };

  const handleTestConnection = async (provider: Provider) => {
    // TODO: Implement actual API test
    alert(`Testing ${providerInfo[provider].name} connection...`);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">API Keys</h1>
          <p className="text-gray-500">Kelola API keys untuk provider AI</p>
        </div>
      </div>

      {/* Provider API Keys */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle className="text-lg">Provider API Keys</CardTitle>
          <AddApiKeyDialog />
        </CardHeader>
        <CardContent>
          {apiKeys.length === 0 ? (
            <p className="text-gray-500 text-center py-8">
              Belum ada API key. Tambahkan API key untuk mulai menggunakan proxy.
            </p>
          ) : (
            <div className="space-y-4">
              {apiKeys.map((key) => (
                <div
                  key={key.id}
                  className="flex items-center justify-between p-4 border rounded-lg"
                >
                  <div className="flex items-center gap-4">
                    <span
                      className={`px-2 py-1 rounded text-xs font-medium ${
                        providerInfo[key.provider].color
                      }`}
                    >
                      {providerInfo[key.provider].name}
                    </span>
                    <div>
                      <p className="font-mono text-sm">{key.maskedKey}</p>
                      <p className="text-xs text-gray-400">
                        Added {key.createdAt}
                        {key.lastUsed && ` • Last used ${key.lastUsed}`}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleTestConnection(key.provider)}
                    >
                      Test
                    </Button>
                    <Button
                      variant="destructive"
                      size="sm"
                      onClick={() => {
                        setKeyToDelete(key.id);
                        setDeleteDialogOpen(true);
                      }}
                    >
                      Delete
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Proxy API Keys */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle className="text-lg">Proxy API Keys</CardTitle>
          <Button size="sm">Generate New Key</Button>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-gray-500 mb-4">
            Gunakan proxy key ini untuk mengakses semua provider melalui endpoint Webrana.
          </p>
          {proxyKeys.map((key) => (
            <div
              key={key.id}
              className="flex items-center justify-between p-4 border rounded-lg"
            >
              <div>
                <p className="font-medium">{key.name}</p>
                <p className="font-mono text-sm text-gray-600">{key.key}</p>
                <p className="text-xs text-gray-400">Created {key.createdAt}</p>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleCopyKey(key.key)}
              >
                📋 Copy
              </Button>
            </div>
          ))}
        </CardContent>
      </Card>

      {/* Delete Confirmation Dialog */}
      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Hapus API Key?</DialogTitle>
            <DialogDescription>
              API key ini akan dihapus permanen. Anda tidak akan bisa menggunakan
              provider ini sampai menambahkan key baru.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteDialogOpen(false)}>
              Batal
            </Button>
            <Button
              variant="destructive"
              onClick={() => keyToDelete && handleDeleteKey(keyToDelete)}
            >
              Hapus
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

function AddApiKeyDialog() {
  const [provider, setProvider] = useState<Provider>('openai');
  const [apiKey, setApiKey] = useState('');
  const [open, setOpen] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // TODO: Validate and submit
    console.log('Adding key for', provider, apiKey);
    setOpen(false);
    setApiKey('');
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm">+ Add API Key</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add Provider API Key</DialogTitle>
          <DialogDescription>
            Masukkan API key dari provider AI yang ingin Anda gunakan.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-2">Provider</label>
            <select
              value={provider}
              onChange={(e) => setProvider(e.target.value as Provider)}
              className="w-full border rounded-md px-3 py-2"
            >
              <option value="openai">OpenAI</option>
              <option value="anthropic">Anthropic</option>
              <option value="google">Google AI</option>
              <option value="qwen">Qwen (Alibaba)</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium mb-2">API Key</label>
            <Input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder={`${providerInfo[provider].prefix}...`}
            />
            <p className="text-xs text-gray-400 mt-1">
              Key harus dimulai dengan &quot;{providerInfo[provider].prefix}&quot;
            </p>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => setOpen(false)}>
              Batal
            </Button>
            <Button type="submit" disabled={!apiKey}>
              Simpan
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
