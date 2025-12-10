'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';

type Language = 'id' | 'en';

const translations = {
  id: {
    heroTitle: 'Unified API untuk Semua AI Models',
    heroSubtitle: 'Akses GPT, Claude, Gemini, dan Qwen melalui satu endpoint. Analytics, team management, dan billing dalam Rupiah.',
    ctaStart: 'Mulai Gratis',
    ctaPricing: 'Lihat Harga',
    whyTitle: 'Kenapa Webrana?',
    feature1Title: 'Unified API',
    feature1Desc: 'Satu endpoint untuk semua AI models. Tidak perlu manage multiple API keys.',
    feature2Title: 'Analytics Dashboard',
    feature2Desc: 'Track usage, costs, dan performance real-time. Export ke CSV.',
    feature3Title: 'Rupiah Billing',
    feature3Desc: 'Bayar dengan QRIS, Transfer Bank, atau Kartu. Harga dalam Rupiah.',
    providersTitle: 'Didukung oleh Provider Terbaik',
    pricingTitle: 'Harga',
    perMonth: 'per bulan',
    requests: 'requests/bulan',
    apiKeys: 'API keys',
    providers: 'providers',
    teamMembers: 'team members',
    unlimited: 'Unlimited',
    allProviders: 'Semua providers',
  },
  en: {
    heroTitle: 'Unified API for All AI Models',
    heroSubtitle: 'Access GPT, Claude, Gemini, and Qwen through a single endpoint. Analytics, team management, and Rupiah billing included.',
    ctaStart: 'Start Free',
    ctaPricing: 'View Pricing',
    whyTitle: 'Why Webrana?',
    feature1Title: 'Unified API',
    feature1Desc: 'One endpoint for all AI models. No need to manage multiple API keys.',
    feature2Title: 'Analytics Dashboard',
    feature2Desc: 'Track usage, costs, and performance in real-time. Export to CSV.',
    feature3Title: 'Rupiah Billing',
    feature3Desc: 'Pay with QRIS, Bank Transfer, or Card. Prices in Rupiah.',
    providersTitle: 'Powered by Top Providers',
    pricingTitle: 'Pricing',
    perMonth: 'per month',
    requests: 'requests/month',
    apiKeys: 'API keys',
    providers: 'providers',
    teamMembers: 'team members',
    unlimited: 'Unlimited',
    allProviders: 'All providers',
  },
};


export default function HomePage() {
  const [lang, setLang] = useState<Language>('id');
  const t = translations[lang];

  return (
    <main className="min-h-screen">
      {/* Language Toggle */}
      <nav className="fixed top-0 right-0 p-4 z-50">
        <div className="flex gap-2 bg-white/80 backdrop-blur rounded-lg p-1 shadow-sm">
          <button
            onClick={() => setLang('id')}
            className={`px-3 py-1 rounded text-sm font-medium transition-colors ${
              lang === 'id' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-100'
            }`}
          >
            ID
          </button>
          <button
            onClick={() => setLang('en')}
            className={`px-3 py-1 rounded text-sm font-medium transition-colors ${
              lang === 'en' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-100'
            }`}
          >
            EN
          </button>
        </div>
      </nav>

      {/* Hero Section */}
      <section className="py-16 md:py-24 px-4 text-center">
        <div className="max-w-4xl mx-auto">
          <h1 className="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-bold mb-6 leading-tight">
            {t.heroTitle}
          </h1>
          <p className="text-lg md:text-xl text-gray-600 mb-8 max-w-2xl mx-auto">
            {t.heroSubtitle}
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <a
              href="/register"
              className="bg-blue-600 text-white px-8 py-3 rounded-lg font-semibold hover:bg-blue-700 transition-colors"
            >
              {t.ctaStart}
            </a>
            <a
              href="#pricing"
              className="border border-gray-300 px-8 py-3 rounded-lg font-semibold hover:bg-gray-50 transition-colors"
            >
              {t.ctaPricing}
            </a>
          </div>
        </div>
      </section>

      {/* Provider Logos Section */}
      <section className="py-12 px-4 bg-gray-50">
        <div className="max-w-6xl mx-auto">
          <p className="text-center text-gray-500 mb-8">{t.providersTitle}</p>
          <div className="flex flex-wrap justify-center items-center gap-8 md:gap-12">
            <ProviderLogo name="OpenAI" />
            <ProviderLogo name="Anthropic" />
            <ProviderLogo name="Google" />
            <ProviderLogo name="Alibaba" />
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-16 md:py-20 px-4">
        <div className="max-w-6xl mx-auto">
          <h2 className="text-2xl md:text-3xl font-bold text-center mb-12">
            {t.whyTitle}
          </h2>
          <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6 md:gap-8">
            <FeatureCard
              icon="🔗"
              title={t.feature1Title}
              description={t.feature1Desc}
            />
            <FeatureCard
              icon="📊"
              title={t.feature2Title}
              description={t.feature2Desc}
            />
            <FeatureCard
              icon="💰"
              title={t.feature3Title}
              description={t.feature3Desc}
            />
          </div>
        </div>
      </section>

      {/* Pricing Section */}
      <section id="pricing" className="py-16 md:py-20 px-4 bg-gray-50">
        <div className="max-w-6xl mx-auto">
          <h2 className="text-2xl md:text-3xl font-bold text-center mb-12">
            {t.pricingTitle}
          </h2>
          <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-6">
            <PricingCard
              tier="Free"
              price="Rp 0"
              features={[
                `1.000 ${t.requests}`,
                `1 ${t.apiKeys}`,
                `1 ${t.providers}`,
              ]}
              perMonth={t.perMonth}
            />
            <PricingCard
              tier="Starter"
              price="Rp 49.000"
              features={[
                `10.000 ${t.requests}`,
                `5 ${t.apiKeys}`,
                `2 ${t.providers}`,
              ]}
              perMonth={t.perMonth}
              highlighted
            />
            <PricingCard
              tier="Pro"
              price="Rp 99.000"
              features={[
                `50.000 ${t.requests}`,
                `${t.unlimited} ${t.apiKeys}`,
                t.allProviders,
              ]}
              perMonth={t.perMonth}
            />
            <PricingCard
              tier="Team"
              price="Rp 299.000"
              features={[
                `200.000 ${t.requests}`,
                `${t.unlimited} ${t.apiKeys}`,
                `10 ${t.teamMembers}`,
              ]}
              perMonth={t.perMonth}
            />
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-8 px-4 border-t">
        <div className="max-w-6xl mx-auto text-center text-gray-500 text-sm">
          © 2024 Webrana. All rights reserved.
        </div>
      </footer>
    </main>
  );
}

function ProviderLogo({ name }: { name: string }) {
  const logos: Record<string, { bg: string; text: string }> = {
    OpenAI: { bg: 'bg-black', text: 'text-white' },
    Anthropic: { bg: 'bg-orange-100', text: 'text-orange-800' },
    Google: { bg: 'bg-blue-100', text: 'text-blue-800' },
    Alibaba: { bg: 'bg-orange-500', text: 'text-white' },
  };

  const style = logos[name] || { bg: 'bg-gray-100', text: 'text-gray-800' };

  return (
    <div
      className={`${style.bg} ${style.text} px-4 py-2 rounded-lg font-semibold text-sm md:text-base`}
    >
      {name}
    </div>
  );
}

function FeatureCard({
  icon,
  title,
  description,
}: {
  icon: string;
  title: string;
  description: string;
}) {
  return (
    <Card>
      <CardHeader>
        <div className="text-3xl mb-2">{icon}</div>
        <CardTitle>{title}</CardTitle>
      </CardHeader>
      <CardContent>
        <p className="text-gray-600">{description}</p>
      </CardContent>
    </Card>
  );
}

function PricingCard({
  tier,
  price,
  features,
  perMonth,
  highlighted = false,
}: {
  tier: string;
  price: string;
  features: string[];
  perMonth: string;
  highlighted?: boolean;
}) {
  return (
    <Card
      className={highlighted ? 'border-blue-600 border-2 relative' : ''}
    >
      {highlighted && (
        <div className="absolute -top-3 left-1/2 -translate-x-1/2 bg-blue-600 text-white text-xs px-3 py-1 rounded-full">
          Popular
        </div>
      )}
      <CardHeader>
        <CardTitle className="text-lg">{tier}</CardTitle>
        <div className="mt-2">
          <span className="text-3xl font-bold">{price}</span>
          <span className="text-gray-500 text-sm ml-1">/{perMonth}</span>
        </div>
      </CardHeader>
      <CardContent>
        <ul className="space-y-3">
          {features.map((feature, i) => (
            <li key={i} className="flex items-center gap-2 text-sm">
              <span className="text-green-500">✓</span>
              {feature}
            </li>
          ))}
        </ul>
        <button
          className={`w-full mt-6 px-4 py-2 rounded-lg font-medium transition-colors ${
            highlighted
              ? 'bg-blue-600 text-white hover:bg-blue-700'
              : 'border border-gray-300 hover:bg-gray-50'
          }`}
        >
          {tier === 'Free' ? 'Mulai Gratis' : 'Pilih Plan'}
        </button>
      </CardContent>
    </Card>
  );
}
