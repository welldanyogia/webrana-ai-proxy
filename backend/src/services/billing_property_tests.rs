//! Property-Based Tests for Week 3: Billing & Analytics
//!
//! **Feature: week3-billing-analytics**
//!
//! These tests verify correctness properties for:
//! - Usage aggregation (Property 1)
//! - Payment amount calculation with PPN (Property 2)
//! - Subscription lifecycle integrity (Property 3)
//! - Proration calculation (Property 4)
//! - Rate limiting enforcement (Property 5)
//! - Webhook signature verification (Property 6)
//! - Invoice number uniqueness (Property 7)
//! - CSV export completeness (Property 8)

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use chrono::{DateTime, Duration, Utc};
    use sha2::{Digest, Sha512};

    // ============================================================
    // Property Test 1: Usage Aggregation Correctness
    // **Feature: week3-billing-analytics, Property 1: Usage Aggregation Correctness**
    // **Validates: Requirements 1.2, 1.3, 1.4**
    // ============================================================

    /// Simulated proxy request for testing aggregation
    #[derive(Debug, Clone)]
    struct MockProxyRequest {
        prompt_tokens: i64,
        completion_tokens: i64,
        total_tokens: i64,
        estimated_cost_idr: i64,
        latency_ms: i64,
        status_code: i32,
    }

    /// Aggregate usage stats from a list of requests (pure function)
    fn aggregate_usage(requests: &[MockProxyRequest]) -> (i64, i64, i64, i64, i64, f64) {
        let successful: Vec<_> = requests.iter().filter(|r| r.status_code < 400).collect();
        
        let total_requests = successful.len() as i64;
        let total_input_tokens: i64 = successful.iter().map(|r| r.prompt_tokens).sum();
        let total_output_tokens: i64 = successful.iter().map(|r| r.completion_tokens).sum();
        let total_tokens: i64 = successful.iter().map(|r| r.total_tokens).sum();
        let total_cost_idr: i64 = successful.iter().map(|r| r.estimated_cost_idr).sum();
        let avg_latency_ms = if successful.is_empty() {
            0.0
        } else {
            successful.iter().map(|r| r.latency_ms as f64).sum::<f64>() / successful.len() as f64
        };

        (total_requests, total_input_tokens, total_output_tokens, total_tokens, total_cost_idr, avg_latency_ms)
    }

    fn mock_request_strategy() -> impl Strategy<Value = MockProxyRequest> {
        (
            1i64..10000i64,      // prompt_tokens
            1i64..10000i64,      // completion_tokens
            200i32..600i32,      // status_code
            1i64..1000i64,       // latency_ms
            1i64..100000i64,     // estimated_cost_idr
        ).prop_map(|(prompt, completion, status, latency, cost)| {
            MockProxyRequest {
                prompt_tokens: prompt,
                completion_tokens: completion,
                total_tokens: prompt + completion,
                estimated_cost_idr: cost,
                latency_ms: latency,
                status_code: status,
            }
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Aggregated totals equal sum of individual records
        /// Requirements: 1.2, 1.3 - Usage aggregation correctness
        #[test]
        fn prop_usage_aggregation_equals_sum(
            requests in prop::collection::vec(mock_request_strategy(), 0..50)
        ) {
            let (total_requests, total_input, total_output, total_tokens, total_cost, _avg_latency) = 
                aggregate_usage(&requests);

            // Manual calculation for verification
            let successful: Vec<_> = requests.iter().filter(|r| r.status_code < 400).collect();
            let expected_requests = successful.len() as i64;
            let expected_input: i64 = successful.iter().map(|r| r.prompt_tokens).sum();
            let expected_output: i64 = successful.iter().map(|r| r.completion_tokens).sum();
            let expected_tokens: i64 = successful.iter().map(|r| r.total_tokens).sum();
            let expected_cost: i64 = successful.iter().map(|r| r.estimated_cost_idr).sum();

            prop_assert_eq!(total_requests, expected_requests, "Request count mismatch");
            prop_assert_eq!(total_input, expected_input, "Input tokens mismatch");
            prop_assert_eq!(total_output, expected_output, "Output tokens mismatch");
            prop_assert_eq!(total_tokens, expected_tokens, "Total tokens mismatch");
            prop_assert_eq!(total_cost, expected_cost, "Cost mismatch");
        }

        /// Property: Total tokens equals input + output tokens
        /// Requirements: 1.2 - Token counting consistency
        #[test]
        fn prop_total_tokens_equals_input_plus_output(
            requests in prop::collection::vec(mock_request_strategy(), 1..20)
        ) {
            let (_, total_input, total_output, total_tokens, _, _) = aggregate_usage(&requests);
            
            prop_assert_eq!(
                total_tokens,
                total_input + total_output,
                "Total tokens should equal input + output"
            );
        }

        /// Property: Failed requests (status >= 400) are excluded from aggregation
        /// Requirements: 1.2 - Only successful requests counted
        #[test]
        fn prop_failed_requests_excluded(
            requests in prop::collection::vec(mock_request_strategy(), 1..30)
        ) {
            let (total_requests, _, _, _, _, _) = aggregate_usage(&requests);
            let successful_count = requests.iter().filter(|r| r.status_code < 400).count() as i64;
            
            prop_assert_eq!(
                total_requests,
                successful_count,
                "Only successful requests should be counted"
            );
        }
    }


    // ============================================================
    // Property Test 2: Payment Amount Calculation
    // **Feature: week3-billing-analytics, Property 2: Payment Amount Calculation**
    // **Validates: Requirements 2.1, 4.2**
    // ============================================================

    const PPN_RATE: f64 = 0.11;

    /// Calculate total amount with PPN (11% VAT)
    fn calculate_total_with_ppn(base_price: i64) -> (i64, i64, i64) {
        let ppn = (base_price as f64 * PPN_RATE).round() as i64;
        let total = base_price + ppn;
        (base_price, ppn, total)
    }

    /// Plan tier pricing
    fn plan_price(tier: &str) -> i64 {
        match tier {
            "free" => 0,
            "starter" => 49_000,
            "pro" => 99_000,
            "team" => 299_000,
            _ => 0,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Total equals base price + PPN (11%)
        /// Requirements: 2.1, 4.2 - Payment amount calculation
        #[test]
        fn prop_total_equals_base_plus_ppn(base_price in 1000i64..1000000i64) {
            let (subtotal, ppn, total) = calculate_total_with_ppn(base_price);
            
            prop_assert_eq!(subtotal, base_price, "Subtotal should equal base price");
            prop_assert_eq!(total, subtotal + ppn, "Total should equal subtotal + PPN");
        }

        /// Property: PPN is exactly 11% of base price (rounded)
        /// Requirements: 2.1 - 11% PPN calculation
        #[test]
        fn prop_ppn_is_eleven_percent(base_price in 1000i64..1000000i64) {
            let (_, ppn, _) = calculate_total_with_ppn(base_price);
            let expected_ppn = (base_price as f64 * 0.11).round() as i64;
            
            prop_assert_eq!(ppn, expected_ppn, "PPN should be 11% of base price");
        }

        /// Property: All plan tiers have correct pricing with PPN
        /// Requirements: 2.1 - Plan tier pricing
        #[test]
        fn prop_plan_tier_pricing(tier in prop_oneof![
            Just("starter"),
            Just("pro"),
            Just("team"),
        ]) {
            let base = plan_price(&tier);
            let (subtotal, ppn, total) = calculate_total_with_ppn(base);
            
            // Verify expected totals
            let expected_totals = [
                ("starter", 49_000, 5_390, 54_390),
                ("pro", 99_000, 10_890, 109_890),
                ("team", 299_000, 32_890, 331_890),
            ];
            
            for (t, exp_sub, exp_ppn, exp_total) in expected_totals {
                if tier == t {
                    prop_assert_eq!(subtotal, exp_sub, "Subtotal mismatch for {}", tier);
                    prop_assert_eq!(ppn, exp_ppn, "PPN mismatch for {}", tier);
                    prop_assert_eq!(total, exp_total, "Total mismatch for {}", tier);
                }
            }
        }
    }

    // ============================================================
    // Property Test 3: Subscription Lifecycle Integrity
    // **Feature: week3-billing-analytics, Property 3: Subscription Lifecycle Integrity**
    // **Validates: Requirements 3.1, 3.3**
    // ============================================================

    /// Simulated subscription for testing
    #[derive(Debug, Clone)]
    struct MockSubscription {
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        status: String,
    }

    impl MockSubscription {
        fn new_active(start: DateTime<Utc>) -> Self {
            Self {
                start_date: start,
                end_date: start + Duration::days(30),
                status: "active".to_string(),
            }
        }

        fn is_expired(&self, now: DateTime<Utc>) -> bool {
            now > self.end_date
        }

        fn days_remaining(&self, now: DateTime<Utc>) -> i64 {
            if now > self.end_date {
                0
            } else {
                (self.end_date - now).num_days()
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Subscription end_date is exactly 30 days after start_date
        /// Requirements: 3.1 - 30-day subscription period
        #[test]
        fn prop_subscription_period_is_30_days(
            days_ago in 0i64..365i64
        ) {
            let start = Utc::now() - Duration::days(days_ago);
            let sub = MockSubscription::new_active(start);
            
            let duration = sub.end_date - sub.start_date;
            prop_assert_eq!(
                duration.num_days(),
                30,
                "Subscription period should be exactly 30 days"
            );
        }

        /// Property: Expired subscriptions have end_date in the past
        /// Requirements: 3.3 - Subscription expiration
        #[test]
        fn prop_expired_subscription_end_date_in_past(
            days_ago in 31i64..365i64
        ) {
            let start = Utc::now() - Duration::days(days_ago);
            let sub = MockSubscription::new_active(start);
            let now = Utc::now();
            
            prop_assert!(
                sub.is_expired(now),
                "Subscription started {} days ago should be expired",
                days_ago
            );
            prop_assert_eq!(
                sub.days_remaining(now),
                0,
                "Expired subscription should have 0 days remaining"
            );
        }

        /// Property: Active subscriptions have end_date in the future
        /// Requirements: 3.1 - Active subscription validation
        #[test]
        fn prop_active_subscription_end_date_in_future(
            days_ago in 0i64..29i64
        ) {
            let start = Utc::now() - Duration::days(days_ago);
            let sub = MockSubscription::new_active(start);
            let now = Utc::now();
            
            prop_assert!(
                !sub.is_expired(now),
                "Subscription started {} days ago should not be expired",
                days_ago
            );
            prop_assert!(
                sub.days_remaining(now) > 0,
                "Active subscription should have days remaining"
            );
        }
    }


    // ============================================================
    // Property Test 4: Proration Calculation
    // **Feature: week3-billing-analytics, Property 4: Proration Calculation**
    // **Validates: Requirements 3.4**
    // ============================================================

    /// Calculate prorated amount for mid-cycle upgrade
    fn calculate_proration(old_price: i64, new_price: i64, remaining_days: i64) -> i64 {
        if new_price <= old_price || remaining_days <= 0 {
            return 0;
        }
        let price_diff = new_price - old_price;
        ((price_diff as f64 * remaining_days as f64) / 30.0).round() as i64
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Proration equals (new_price - old_price) * (remaining_days / 30)
        /// Requirements: 3.4 - Proration calculation
        #[test]
        fn prop_proration_formula_correct(
            old_price in 0i64..100000i64,
            new_price in 0i64..500000i64,
            remaining_days in 1i64..30i64
        ) {
            let prorated = calculate_proration(old_price, new_price, remaining_days);
            
            if new_price > old_price {
                let expected = ((new_price - old_price) as f64 * remaining_days as f64 / 30.0).round() as i64;
                prop_assert_eq!(
                    prorated,
                    expected,
                    "Proration should follow formula: (new - old) * remaining / 30"
                );
            } else {
                prop_assert_eq!(
                    prorated,
                    0,
                    "Downgrade or same plan should have 0 proration"
                );
            }
        }

        /// Property: Proration is 0 when downgrading
        /// Requirements: 3.4 - No proration for downgrades
        #[test]
        fn prop_no_proration_for_downgrade(
            old_price in 50000i64..500000i64,
            remaining_days in 1i64..30i64
        ) {
            let new_price = old_price / 2; // Downgrade
            let prorated = calculate_proration(old_price, new_price, remaining_days);
            
            prop_assert_eq!(prorated, 0, "Downgrade should have 0 proration");
        }

        /// Property: Full month upgrade equals full price difference
        /// Requirements: 3.4 - Full month proration
        #[test]
        fn prop_full_month_proration(
            old_price in 0i64..100000i64,
            new_price in 100001i64..500000i64
        ) {
            let prorated = calculate_proration(old_price, new_price, 30);
            let expected = new_price - old_price;
            
            prop_assert_eq!(
                prorated,
                expected,
                "30 days remaining should equal full price difference"
            );
        }
    }

    // ============================================================
    // Property Test 5: Rate Limiting Enforcement
    // **Feature: week3-billing-analytics, Property 5: Rate Limiting Enforcement**
    // **Validates: Requirements 5.1, 5.4**
    // ============================================================

    /// Plan tier request limits
    fn plan_request_limit(tier: &str) -> i64 {
        match tier {
            "free" => 1_000,
            "starter" => 10_000,
            "pro" => 50_000,
            "team" => 200_000,
            _ => 1_000,
        }
    }

    /// Check if request should be allowed
    fn check_rate_limit(current_usage: i64, limit: i64) -> bool {
        current_usage < limit
    }

    /// Check if at warning threshold (80%)
    fn is_at_warning_threshold(used: i64, limit: i64) -> bool {
        let percentage = (used as f64 / limit as f64) * 100.0;
        percentage >= 80.0 && percentage < 100.0
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Requests at or above limit are rejected
        /// Requirements: 5.1, 5.4 - Rate limit enforcement
        #[test]
        fn prop_requests_at_limit_rejected(
            tier in prop_oneof![Just("free"), Just("starter"), Just("pro"), Just("team")],
            over_limit in 0i64..1000i64
        ) {
            let limit = plan_request_limit(&tier);
            let usage = limit + over_limit;
            
            prop_assert!(
                !check_rate_limit(usage, limit),
                "Requests at or above limit should be rejected"
            );
        }

        /// Property: Requests below limit are allowed
        /// Requirements: 5.1 - Allow requests under limit
        #[test]
        fn prop_requests_below_limit_allowed(
            tier in prop_oneof![Just("free"), Just("starter"), Just("pro"), Just("team")],
            usage_percent in 0u8..99u8
        ) {
            let limit = plan_request_limit(&tier);
            let usage = (limit as f64 * usage_percent as f64 / 100.0) as i64;
            
            prop_assert!(
                check_rate_limit(usage, limit),
                "Requests below limit should be allowed"
            );
        }

        /// Property: Warning threshold triggers at 80%
        /// Requirements: 5.3 - 80% quota warning
        #[test]
        fn prop_warning_at_80_percent(
            tier in prop_oneof![Just("free"), Just("starter"), Just("pro"), Just("team")],
            usage_percent in 80u8..100u8
        ) {
            let limit = plan_request_limit(&tier);
            let usage = (limit as f64 * usage_percent as f64 / 100.0) as i64;
            
            if usage_percent < 100 {
                prop_assert!(
                    is_at_warning_threshold(usage, limit),
                    "80-99% usage should trigger warning"
                );
            }
        }

        /// Property: No warning below 80%
        /// Requirements: 5.3 - No warning under threshold
        #[test]
        fn prop_no_warning_below_80_percent(
            tier in prop_oneof![Just("free"), Just("starter"), Just("pro"), Just("team")],
            usage_percent in 0u8..79u8
        ) {
            let limit = plan_request_limit(&tier);
            let usage = (limit as f64 * usage_percent as f64 / 100.0) as i64;
            
            prop_assert!(
                !is_at_warning_threshold(usage, limit),
                "Below 80% usage should not trigger warning"
            );
        }
    }


    // ============================================================
    // Property Test 6: Webhook Signature Verification
    // **Feature: week3-billing-analytics, Property 6: Webhook Signature Verification**
    // **Validates: Requirements 2.4**
    // ============================================================

    /// Compute Midtrans webhook signature
    fn compute_signature(order_id: &str, status_code: &str, gross_amount: &str, server_key: &str) -> String {
        let signature_input = format!("{}{}{}{}", order_id, status_code, gross_amount, server_key);
        let mut hasher = Sha512::new();
        hasher.update(signature_input.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify webhook signature
    fn verify_signature(
        order_id: &str,
        status_code: &str,
        gross_amount: &str,
        provided_signature: &str,
        server_key: &str,
    ) -> bool {
        let computed = compute_signature(order_id, status_code, gross_amount, server_key);
        computed == provided_signature
    }

    fn order_id_strategy() -> impl Strategy<Value = String> {
        "[A-Z]{3}-[0-9]{14}-[a-f0-9]{8}".prop_map(|s| s)
    }

    fn status_code_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("200".to_string()),
            Just("201".to_string()),
            Just("202".to_string()),
        ]
    }

    fn gross_amount_strategy() -> impl Strategy<Value = String> {
        (10000i64..1000000i64).prop_map(|n| format!("{}.00", n))
    }

    fn server_key_strategy() -> impl Strategy<Value = String> {
        "[A-Za-z0-9]{20,40}".prop_map(|s| s)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Valid signature passes verification
        /// Requirements: 2.4 - Signature verification
        #[test]
        fn prop_valid_signature_passes(
            order_id in order_id_strategy(),
            status_code in status_code_strategy(),
            gross_amount in gross_amount_strategy(),
            server_key in server_key_strategy()
        ) {
            let signature = compute_signature(&order_id, &status_code, &gross_amount, &server_key);
            
            prop_assert!(
                verify_signature(&order_id, &status_code, &gross_amount, &signature, &server_key),
                "Valid signature should pass verification"
            );
        }

        /// Property: Invalid signature fails verification
        /// Requirements: 2.4, 2.5 - Reject invalid signatures
        #[test]
        fn prop_invalid_signature_fails(
            order_id in order_id_strategy(),
            status_code in status_code_strategy(),
            gross_amount in gross_amount_strategy(),
            server_key in server_key_strategy(),
            wrong_key in server_key_strategy()
        ) {
            // Compute signature with correct key
            let signature = compute_signature(&order_id, &status_code, &gross_amount, &server_key);
            
            // Verify with wrong key (if keys are different)
            if server_key != wrong_key {
                prop_assert!(
                    !verify_signature(&order_id, &status_code, &gross_amount, &signature, &wrong_key),
                    "Signature with wrong key should fail"
                );
            }
        }

        /// Property: Tampered data fails verification
        /// Requirements: 2.4 - Detect tampering
        #[test]
        fn prop_tampered_data_fails(
            order_id in order_id_strategy(),
            status_code in status_code_strategy(),
            gross_amount in gross_amount_strategy(),
            server_key in server_key_strategy(),
            tampered_amount in gross_amount_strategy()
        ) {
            let signature = compute_signature(&order_id, &status_code, &gross_amount, &server_key);
            
            // Verify with tampered amount (if different)
            if gross_amount != tampered_amount {
                prop_assert!(
                    !verify_signature(&order_id, &status_code, &tampered_amount, &signature, &server_key),
                    "Tampered amount should fail verification"
                );
            }
        }
    }

    // ============================================================
    // Property Test 7: Invoice Number Uniqueness
    // **Feature: week3-billing-analytics, Property 7: Invoice Number Uniqueness**
    // **Validates: Requirements 4.2**
    // ============================================================

    /// Generate invoice number in format WEB-YYYY-MM-XXX
    fn generate_invoice_number(timestamp: DateTime<Utc>, sequence: u32) -> String {
        format!(
            "WEB-{}-{:03}",
            timestamp.format("%Y-%m"),
            sequence % 1000
        )
    }

    /// Validate invoice number format
    fn validate_invoice_format(invoice_number: &str) -> bool {
        // Format: WEB-YYYY-MM-XXX
        let parts: Vec<&str> = invoice_number.split('-').collect();
        if parts.len() != 4 {
            return false;
        }
        if parts[0] != "WEB" {
            return false;
        }
        // Year should be 4 digits
        if parts[1].len() != 4 || !parts[1].chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        // Month should be 2 digits (01-12)
        if parts[2].len() != 2 || !parts[2].chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        let month: u32 = parts[2].parse().unwrap_or(0);
        if !(1..=12).contains(&month) {
            return false;
        }
        // Sequence should be 3 digits
        if parts[3].len() != 3 || !parts[3].chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        true
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Generated invoice numbers follow WEB-YYYY-MM-XXX format
        /// Requirements: 4.2 - Invoice number format
        #[test]
        fn prop_invoice_number_format(
            days_offset in 0i64..365i64,
            sequence in 0u32..1000u32
        ) {
            let timestamp = Utc::now() - Duration::days(days_offset);
            let invoice_number = generate_invoice_number(timestamp, sequence);
            
            prop_assert!(
                validate_invoice_format(&invoice_number),
                "Invoice number should follow WEB-YYYY-MM-XXX format: {}",
                invoice_number
            );
        }

        /// Property: Different sequences produce different invoice numbers
        /// Requirements: 4.2 - Invoice uniqueness
        #[test]
        fn prop_different_sequences_unique(
            days_offset in 0i64..365i64,
            seq1 in 0u32..999u32,
            seq2 in 0u32..999u32
        ) {
            let timestamp = Utc::now() - Duration::days(days_offset);
            let invoice1 = generate_invoice_number(timestamp, seq1);
            let invoice2 = generate_invoice_number(timestamp, seq2);
            
            if seq1 != seq2 {
                prop_assert_ne!(
                    invoice1,
                    invoice2,
                    "Different sequences should produce different invoice numbers"
                );
            }
        }

        /// Property: Different months produce different invoice numbers
        /// Requirements: 4.2 - Monthly uniqueness
        #[test]
        fn prop_different_months_unique(
            month1 in 1u32..12u32,
            month2 in 1u32..12u32,
            sequence in 0u32..1000u32
        ) {
            let year = 2024;
            let ts1 = chrono::NaiveDate::from_ymd_opt(year, month1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            let ts2 = chrono::NaiveDate::from_ymd_opt(year, month2, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            
            let invoice1 = generate_invoice_number(ts1, sequence);
            let invoice2 = generate_invoice_number(ts2, sequence);
            
            if month1 != month2 {
                prop_assert_ne!(
                    invoice1,
                    invoice2,
                    "Different months should produce different invoice numbers"
                );
            }
        }
    }


    // ============================================================
    // Property Test 8: CSV Export Completeness
    // **Feature: week3-billing-analytics, Property 8: CSV Export Completeness**
    // **Validates: Requirements 1.5**
    // ============================================================

    /// Required CSV columns
    const CSV_REQUIRED_COLUMNS: [&str; 7] = [
        "timestamp",
        "provider",
        "model",
        "input_tokens",
        "output_tokens",
        "cost_idr",
        "latency_ms",
    ];

    /// Simulated usage record for CSV export
    #[derive(Debug, Clone)]
    struct CsvUsageRecord {
        timestamp: String,
        provider: String,
        model: String,
        input_tokens: i32,
        output_tokens: i32,
        cost_idr: i64,
        latency_ms: i32,
    }

    /// Generate CSV from records
    fn generate_csv(records: &[CsvUsageRecord]) -> String {
        let mut csv = String::from("timestamp,provider,model,input_tokens,output_tokens,cost_idr,latency_ms\n");
        
        for record in records {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                record.timestamp,
                record.provider,
                record.model,
                record.input_tokens,
                record.output_tokens,
                record.cost_idr,
                record.latency_ms
            ));
        }
        
        csv
    }

    /// Validate CSV has all required columns
    fn validate_csv_columns(csv: &str) -> bool {
        let first_line = csv.lines().next().unwrap_or("");
        let columns: Vec<&str> = first_line.split(',').collect();
        
        CSV_REQUIRED_COLUMNS.iter().all(|col| columns.contains(col))
    }

    /// Count rows in CSV (excluding header)
    fn count_csv_rows(csv: &str) -> usize {
        csv.lines().count().saturating_sub(1)
    }

    fn provider_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("openai".to_string()),
            Just("anthropic".to_string()),
            Just("google".to_string()),
            Just("qwen".to_string()),
        ]
    }

    fn model_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("gpt-4".to_string()),
            Just("claude-3-opus".to_string()),
            Just("gemini-pro".to_string()),
            Just("qwen-turbo".to_string()),
        ]
    }

    fn csv_record_strategy() -> impl Strategy<Value = CsvUsageRecord> {
        (
            "[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}",
            provider_strategy(),
            model_strategy(),
            1i32..10000i32,
            1i32..10000i32,
            1i64..100000i64,
            10i32..5000i32,
        ).prop_map(|(ts, provider, model, input, output, cost, latency)| {
            CsvUsageRecord {
                timestamp: ts,
                provider,
                model,
                input_tokens: input,
                output_tokens: output,
                cost_idr: cost,
                latency_ms: latency,
            }
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: CSV export contains all required columns
        /// Requirements: 1.5 - CSV column completeness
        #[test]
        fn prop_csv_has_required_columns(
            records in prop::collection::vec(csv_record_strategy(), 0..20)
        ) {
            let csv = generate_csv(&records);
            
            prop_assert!(
                validate_csv_columns(&csv),
                "CSV must contain all required columns: {:?}",
                CSV_REQUIRED_COLUMNS
            );
        }

        /// Property: CSV row count matches record count
        /// Requirements: 1.5 - All records exported
        #[test]
        fn prop_csv_row_count_matches(
            records in prop::collection::vec(csv_record_strategy(), 0..50)
        ) {
            let csv = generate_csv(&records);
            let row_count = count_csv_rows(&csv);
            
            prop_assert_eq!(
                row_count,
                records.len(),
                "CSV row count should match record count"
            );
        }

        /// Property: Each record's data appears in CSV
        /// Requirements: 1.5 - Data integrity
        #[test]
        fn prop_csv_contains_record_data(
            records in prop::collection::vec(csv_record_strategy(), 1..10)
        ) {
            let csv = generate_csv(&records);
            
            for record in &records {
                prop_assert!(
                    csv.contains(&record.provider),
                    "CSV should contain provider: {}",
                    record.provider
                );
                prop_assert!(
                    csv.contains(&record.model),
                    "CSV should contain model: {}",
                    record.model
                );
                prop_assert!(
                    csv.contains(&record.input_tokens.to_string()),
                    "CSV should contain input_tokens: {}",
                    record.input_tokens
                );
            }
        }

        /// Property: Empty records produce header-only CSV
        /// Requirements: 1.5 - Handle empty data
        #[test]
        fn prop_empty_records_header_only(_seed in 0u32..100u32) {
            let csv = generate_csv(&[]);
            
            prop_assert!(
                validate_csv_columns(&csv),
                "Empty CSV should still have header"
            );
            prop_assert_eq!(
                count_csv_rows(&csv),
                0,
                "Empty CSV should have 0 data rows"
            );
        }
    }
}
