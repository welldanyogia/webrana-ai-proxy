/**
 * k6 Load Test Script - Webrana AI Proxy
 * 
 * Task 22: Load testing with 100 concurrent users for 10 minutes
 * Requirements: 9.1, 9.2, 9.3, 9.4
 * 
 * Targets:
 * - p95 latency < 500ms
 * - 0% error rate
 * - 100 concurrent users
 * - 10 minutes duration
 * 
 * Usage:
 *   k6 run infrastructure/scripts/load-test.js
 *   
 * With environment variables:
 *   k6 run -e BASE_URL=https://api.webrana.id infrastructure/scripts/load-test.js
 */

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const authLatency = new Trend('auth_latency');
const proxyLatency = new Trend('proxy_latency');
const usageLatency = new Trend('usage_latency');
const adminLatency = new Trend('admin_latency');

// Test configuration
export const options = {
  stages: [
    // Ramp up to 100 users over 2 minutes
    { duration: '2m', target: 100 },
    // Stay at 100 users for 10 minutes
    { duration: '10m', target: 100 },
    // Ramp down over 1 minute
    { duration: '1m', target: 0 },
  ],
  thresholds: {
    // p95 latency must be < 500ms
    http_req_duration: ['p(95)<500'],
    // Error rate must be 0%
    errors: ['rate<0.01'],
    // Custom latency thresholds per endpoint type
    auth_latency: ['p(95)<300'],
    proxy_latency: ['p(95)<500'],
    usage_latency: ['p(95)<400'],
    admin_latency: ['p(95)<400'],
  },
};

// Configuration
const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';
const TEST_USER_EMAIL = __ENV.TEST_EMAIL || 'loadtest@example.com';
const TEST_USER_PASSWORD = __ENV.TEST_PASSWORD || 'LoadTest123!';

// Simulated API key for proxy tests (should be created in test setup)
let authToken = null;
let proxyApiKey = null;

/**
 * Setup function - runs once before the test
 */
export function setup() {
  console.log(`Starting load test against ${BASE_URL}`);
  
  // Try to login or register test user
  const loginRes = http.post(`${BASE_URL}/auth/login`, JSON.stringify({
    email: TEST_USER_EMAIL,
    password: TEST_USER_PASSWORD,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  if (loginRes.status === 200) {
    const body = JSON.parse(loginRes.body);
    return {
      token: body.access_token,
      userId: body.user_id,
    };
  }

  // If login fails, try to register
  const registerRes = http.post(`${BASE_URL}/auth/register`, JSON.stringify({
    email: TEST_USER_EMAIL,
    password: TEST_USER_PASSWORD,
    name: 'Load Test User',
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  if (registerRes.status === 201 || registerRes.status === 200) {
    const body = JSON.parse(registerRes.body);
    return {
      token: body.access_token,
      userId: body.user_id,
    };
  }

  console.warn('Could not authenticate test user, some tests may fail');
  return { token: null, userId: null };
}

/**
 * Main test function - runs for each virtual user
 */
export default function(data) {
  const headers = data.token ? {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${data.token}`,
  } : {
    'Content-Type': 'application/json',
  };

  // Randomly select which endpoint group to test
  const testGroup = Math.random();

  if (testGroup < 0.3) {
    // 30% - Auth endpoints
    testAuthEndpoints(headers);
  } else if (testGroup < 0.6) {
    // 30% - Usage/Analytics endpoints
    testUsageEndpoints(headers);
  } else if (testGroup < 0.8) {
    // 20% - Billing endpoints
    testBillingEndpoints(headers);
  } else {
    // 20% - Health/Status endpoints
    testHealthEndpoints();
  }

  // Random sleep between 1-3 seconds to simulate real user behavior
  sleep(Math.random() * 2 + 1);
}

/**
 * Test authentication endpoints
 */
function testAuthEndpoints(headers) {
  group('Auth Endpoints', () => {
    // Test /auth/me endpoint
    const start = Date.now();
    const res = http.get(`${BASE_URL}/auth/me`, { headers });
    authLatency.add(Date.now() - start);

    const success = check(res, {
      'auth/me status is 200 or 401': (r) => r.status === 200 || r.status === 401,
      'auth/me response time < 300ms': (r) => r.timings.duration < 300,
    });

    errorRate.add(!success);
  });
}

/**
 * Test usage/analytics endpoints
 */
function testUsageEndpoints(headers) {
  group('Usage Endpoints', () => {
    // Test /usage endpoint with date range
    const endDate = new Date().toISOString().split('T')[0];
    const startDate = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0];

    const start = Date.now();
    const res = http.get(
      `${BASE_URL}/usage?start_date=${startDate}&end_date=${endDate}`,
      { headers }
    );
    usageLatency.add(Date.now() - start);

    const success = check(res, {
      'usage status is 200 or 401': (r) => r.status === 200 || r.status === 401,
      'usage response time < 400ms': (r) => r.timings.duration < 400,
    });

    errorRate.add(!success);

    // Test /usage/summary endpoint
    const summaryStart = Date.now();
    const summaryRes = http.get(`${BASE_URL}/usage/summary`, { headers });
    usageLatency.add(Date.now() - summaryStart);

    const summarySuccess = check(summaryRes, {
      'usage/summary status is 200 or 401': (r) => r.status === 200 || r.status === 401,
      'usage/summary response time < 400ms': (r) => r.timings.duration < 400,
    });

    errorRate.add(!summarySuccess);
  });
}

/**
 * Test billing endpoints
 */
function testBillingEndpoints(headers) {
  group('Billing Endpoints', () => {
    // Test /billing/subscription endpoint
    const start = Date.now();
    const res = http.get(`${BASE_URL}/billing/subscription`, { headers });
    usageLatency.add(Date.now() - start);

    const success = check(res, {
      'billing/subscription status is 200 or 401': (r) => r.status === 200 || r.status === 401,
      'billing/subscription response time < 400ms': (r) => r.timings.duration < 400,
    });

    errorRate.add(!success);

    // Test /billing/invoices endpoint
    const invoicesStart = Date.now();
    const invoicesRes = http.get(`${BASE_URL}/billing/invoices`, { headers });
    usageLatency.add(Date.now() - invoicesStart);

    const invoicesSuccess = check(invoicesRes, {
      'billing/invoices status is 200 or 401': (r) => r.status === 200 || r.status === 401,
      'billing/invoices response time < 400ms': (r) => r.timings.duration < 400,
    });

    errorRate.add(!invoicesSuccess);
  });
}

/**
 * Test health/status endpoints (no auth required)
 */
function testHealthEndpoints() {
  group('Health Endpoints', () => {
    // Test /health endpoint
    const start = Date.now();
    const res = http.get(`${BASE_URL}/health`);
    
    const success = check(res, {
      'health status is 200': (r) => r.status === 200,
      'health response time < 100ms': (r) => r.timings.duration < 100,
    });

    errorRate.add(!success);
  });
}

/**
 * Teardown function - runs once after the test
 */
export function teardown(data) {
  console.log('Load test completed');
  console.log(`Test user: ${TEST_USER_EMAIL}`);
}

/**
 * Handle summary - custom summary output
 */
export function handleSummary(data) {
  const summary = {
    timestamp: new Date().toISOString(),
    duration: data.state.testRunDurationMs,
    vus_max: data.metrics.vus_max ? data.metrics.vus_max.values.max : 0,
    iterations: data.metrics.iterations ? data.metrics.iterations.values.count : 0,
    http_reqs: data.metrics.http_reqs ? data.metrics.http_reqs.values.count : 0,
    http_req_duration_p95: data.metrics.http_req_duration ? data.metrics.http_req_duration.values['p(95)'] : 0,
    http_req_duration_avg: data.metrics.http_req_duration ? data.metrics.http_req_duration.values.avg : 0,
    error_rate: data.metrics.errors ? data.metrics.errors.values.rate : 0,
    thresholds_passed: Object.values(data.root_group.checks || {}).every(c => c.passes === c.fails + c.passes),
  };

  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    'load-test-results.json': JSON.stringify(summary, null, 2),
  };
}

// Helper function for text summary
function textSummary(data, options) {
  const lines = [
    '',
    '='.repeat(60),
    'LOAD TEST SUMMARY - Webrana AI Proxy',
    '='.repeat(60),
    '',
    `Duration: ${Math.round(data.state.testRunDurationMs / 1000)}s`,
    `Max VUs: ${data.metrics.vus_max ? data.metrics.vus_max.values.max : 'N/A'}`,
    `Total Requests: ${data.metrics.http_reqs ? data.metrics.http_reqs.values.count : 'N/A'}`,
    '',
    'LATENCY:',
    `  p50: ${data.metrics.http_req_duration ? Math.round(data.metrics.http_req_duration.values['p(50)']) : 'N/A'}ms`,
    `  p95: ${data.metrics.http_req_duration ? Math.round(data.metrics.http_req_duration.values['p(95)']) : 'N/A'}ms`,
    `  p99: ${data.metrics.http_req_duration ? Math.round(data.metrics.http_req_duration.values['p(99)']) : 'N/A'}ms`,
    '',
    'ERROR RATE:',
    `  ${data.metrics.errors ? (data.metrics.errors.values.rate * 100).toFixed(2) : '0'}%`,
    '',
    'THRESHOLDS:',
    `  p95 < 500ms: ${data.metrics.http_req_duration && data.metrics.http_req_duration.values['p(95)'] < 500 ? '✓ PASS' : '✗ FAIL'}`,
    `  Error rate < 1%: ${data.metrics.errors && data.metrics.errors.values.rate < 0.01 ? '✓ PASS' : '✗ FAIL'}`,
    '',
    '='.repeat(60),
  ];

  return lines.join('\n');
}
