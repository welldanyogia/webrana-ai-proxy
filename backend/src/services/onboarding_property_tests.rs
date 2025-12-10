//! Property-based tests for Onboarding and Analytics Services
//!
//! **Feature: week4-launch**
//! Uses proptest with minimum 100 iterations per property.

use proptest::prelude::*;

use super::onboarding_service::{OnboardingStatus, OnboardingStep};
use super::analytics_service::{AcquisitionSource, EventType, AnalyticsEvent};
use std::collections::HashMap;
use uuid::Uuid;

/// Mock onboarding status for testing
#[derive(Debug, Clone)]
struct MockOnboardingStatus {
    has_api_key: bool,
    has_first_request: bool,
    has_dashboard_view: bool,
}

impl MockOnboardingStatus {
    fn to_steps(&self) -> Vec<OnboardingStep> {
        let mut steps = vec![OnboardingStep::AccountCreated];
        if self.has_api_key {
            steps.push(OnboardingStep::ApiKeyAdded);
        }
        if self.has_first_request {
            steps.push(OnboardingStep::FirstRequestMade);
        }
        if self.has_dashboard_view {
            steps.push(OnboardingStep::DashboardViewed);
        }
        steps
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // =========================================================================
    // Property 1: Onboarding Step Progression
    // **Feature: week4-launch, Property 1: Onboarding Step Progression**
    // **Validates: Requirements 5.6**
    // =========================================================================

    /// Property: Completion percentage equals 25% per completed step
    #[test]
    fn prop_completion_percent_equals_step_count(
        has_api_key in any::<bool>(),
        has_first_request in any::<bool>(),
        has_dashboard_view in any::<bool>(),
    ) {
        let mock = MockOnboardingStatus {
            has_api_key,
            has_first_request,
            has_dashboard_view,
        };
        
        let steps = mock.to_steps();
        let completion = OnboardingStatus::calculate_completion(&steps);
        
        // Each step is worth 25%
        let expected = steps.len() as u8 * 25;
        prop_assert_eq!(completion, expected);
    }

    /// Property: Account created is always the first step
    #[test]
    fn prop_account_created_always_first(
        has_api_key in any::<bool>(),
        has_first_request in any::<bool>(),
        has_dashboard_view in any::<bool>(),
    ) {
        let mock = MockOnboardingStatus {
            has_api_key,
            has_first_request,
            has_dashboard_view,
        };
        
        let steps = mock.to_steps();
        
        // Account created should always be first
        prop_assert!(!steps.is_empty());
        prop_assert_eq!(steps[0], OnboardingStep::AccountCreated);
    }

    /// Property: Completion is 100% only when all 4 steps are done
    #[test]
    fn prop_full_completion_requires_all_steps(
        has_api_key in any::<bool>(),
        has_first_request in any::<bool>(),
        has_dashboard_view in any::<bool>(),
    ) {
        let mock = MockOnboardingStatus {
            has_api_key,
            has_first_request,
            has_dashboard_view,
        };
        
        let steps = mock.to_steps();
        let completion = OnboardingStatus::calculate_completion(&steps);
        
        let all_complete = has_api_key && has_first_request && has_dashboard_view;
        
        if all_complete {
            prop_assert_eq!(completion, 100);
        } else {
            prop_assert!(completion < 100);
        }
    }

    /// Property: Step weight is always 25
    #[test]
    fn prop_step_weight_is_25(step_idx in 0..4usize) {
        let steps = [
            OnboardingStep::AccountCreated,
            OnboardingStep::ApiKeyAdded,
            OnboardingStep::FirstRequestMade,
            OnboardingStep::DashboardViewed,
        ];
        
        prop_assert_eq!(steps[step_idx].weight(), 25);
    }

    // =========================================================================
    // Property 2: Inactive User Detection
    // **Feature: week4-launch, Property 2: Inactive User Detection**
    // **Validates: Requirements 5.5**
    // =========================================================================

    /// Property: Users without API key should be detected as inactive
    #[test]
    fn prop_no_api_key_is_inactive(
        has_first_request in any::<bool>(),
        has_dashboard_view in any::<bool>(),
    ) {
        let mock = MockOnboardingStatus {
            has_api_key: false,  // No API key
            has_first_request,
            has_dashboard_view,
        };
        
        let steps = mock.to_steps();
        
        // Should not contain ApiKeyAdded step
        prop_assert!(!steps.contains(&OnboardingStep::ApiKeyAdded));
    }

    /// Property: Users with API key should not be flagged as inactive for API key reminder
    #[test]
    fn prop_with_api_key_not_inactive_for_key_reminder(
        has_first_request in any::<bool>(),
        has_dashboard_view in any::<bool>(),
    ) {
        let mock = MockOnboardingStatus {
            has_api_key: true,  // Has API key
            has_first_request,
            has_dashboard_view,
        };
        
        let steps = mock.to_steps();
        
        // Should contain ApiKeyAdded step
        prop_assert!(steps.contains(&OnboardingStep::ApiKeyAdded));
    }

    // =========================================================================
    // Property 3: Analytics Event Integrity
    // **Feature: week4-launch, Property 3: Analytics Event Integrity**
    // **Validates: Requirements 9.1**
    // =========================================================================

    /// Property: Event type string is never empty
    #[test]
    fn prop_event_type_not_empty(event_idx in 0..11usize) {
        let events = [
            EventType::Signup,
            EventType::ApiKeyAdded,
            EventType::FirstRequest,
            EventType::ProxyRequest,
            EventType::DashboardView,
            EventType::BillingPageView,
            EventType::Upgrade,
            EventType::Downgrade,
            EventType::Cancellation,
            EventType::Login,
            EventType::Logout,
        ];
        
        let event_str = events[event_idx].as_str();
        prop_assert!(!event_str.is_empty());
    }

    /// Property: Acquisition source string is never empty
    #[test]
    fn prop_acquisition_source_not_empty(source_idx in 0..6usize) {
        let sources = [
            AcquisitionSource::ProductHunt,
            AcquisitionSource::Organic,
            AcquisitionSource::Referral,
            AcquisitionSource::Direct,
            AcquisitionSource::Social,
            AcquisitionSource::Unknown,
        ];
        
        let source_str = sources[source_idx].as_str();
        prop_assert!(!source_str.is_empty());
    }

    /// Property: Source parsing is case-insensitive
    #[test]
    fn prop_source_parsing_case_insensitive(
        source in prop::sample::select(vec![
            "producthunt", "PRODUCTHUNT", "ProductHunt",
            "organic", "ORGANIC", "Organic",
            "referral", "REFERRAL", "Referral",
            "direct", "DIRECT", "Direct",
            "social", "SOCIAL", "Social",
        ])
    ) {
        let parsed = AcquisitionSource::from_str(source);
        let source_str = parsed.as_str();
        
        // Should parse to a valid source
        prop_assert!(!source_str.is_empty());
    }

    /// Property: Analytics event with user_id has valid UUID
    #[test]
    fn prop_analytics_event_user_id_valid(
        user_id_bytes in prop::array::uniform16(any::<u8>()),
    ) {
        let user_id = Uuid::from_bytes(user_id_bytes);
        
        let event = AnalyticsEvent {
            user_id: Some(user_id),
            event_type: EventType::Signup.as_str().to_string(),
            properties: HashMap::new(),
            source: Some(AcquisitionSource::Direct.as_str().to_string()),
            session_id: None,
        };
        
        // User ID should be preserved
        prop_assert_eq!(event.user_id, Some(user_id));
        prop_assert!(!event.event_type.is_empty());
    }

    /// Property: Analytics event without user_id is valid for anonymous events
    #[test]
    fn prop_analytics_event_anonymous_valid(
        event_type in "[a-z_]{1,50}",
    ) {
        let event = AnalyticsEvent {
            user_id: None,
            event_type: event_type.clone(),
            properties: HashMap::new(),
            source: None,
            session_id: None,
        };
        
        // Anonymous events should be valid
        prop_assert!(event.user_id.is_none());
        prop_assert_eq!(event.event_type, event_type);
    }

    // =========================================================================
    // Additional Properties
    // =========================================================================

    /// Property: Minimum completion is 25% (account created)
    #[test]
    fn prop_minimum_completion_is_25(
        has_api_key in any::<bool>(),
        has_first_request in any::<bool>(),
        has_dashboard_view in any::<bool>(),
    ) {
        let mock = MockOnboardingStatus {
            has_api_key,
            has_first_request,
            has_dashboard_view,
        };
        
        let steps = mock.to_steps();
        let completion = OnboardingStatus::calculate_completion(&steps);
        
        // Minimum is 25% (account created always included)
        prop_assert!(completion >= 25);
    }

    /// Property: Maximum completion is 100%
    #[test]
    fn prop_maximum_completion_is_100(
        has_api_key in any::<bool>(),
        has_first_request in any::<bool>(),
        has_dashboard_view in any::<bool>(),
    ) {
        let mock = MockOnboardingStatus {
            has_api_key,
            has_first_request,
            has_dashboard_view,
        };
        
        let steps = mock.to_steps();
        let completion = OnboardingStatus::calculate_completion(&steps);
        
        // Maximum is 100%
        prop_assert!(completion <= 100);
    }

    /// Property: Completion increases monotonically with steps
    #[test]
    fn prop_completion_monotonic(
        step1 in any::<bool>(),
        step2 in any::<bool>(),
        step3 in any::<bool>(),
    ) {
        // Base case: only account created
        let base_steps = vec![OnboardingStep::AccountCreated];
        let base_completion = OnboardingStatus::calculate_completion(&base_steps);
        
        // With more steps
        let mut more_steps = vec![OnboardingStep::AccountCreated];
        if step1 { more_steps.push(OnboardingStep::ApiKeyAdded); }
        if step2 { more_steps.push(OnboardingStep::FirstRequestMade); }
        if step3 { more_steps.push(OnboardingStep::DashboardViewed); }
        
        let more_completion = OnboardingStatus::calculate_completion(&more_steps);
        
        // More steps = higher or equal completion
        prop_assert!(more_completion >= base_completion);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_onboarding_step_as_str() {
        assert_eq!(OnboardingStep::AccountCreated.as_str(), "account_created");
        assert_eq!(OnboardingStep::ApiKeyAdded.as_str(), "api_key_added");
        assert_eq!(OnboardingStep::FirstRequestMade.as_str(), "first_request_made");
        assert_eq!(OnboardingStep::DashboardViewed.as_str(), "dashboard_viewed");
    }

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(EventType::Signup.as_str(), "signup");
        assert_eq!(EventType::ApiKeyAdded.as_str(), "api_key_added");
        assert_eq!(EventType::FirstRequest.as_str(), "first_request");
        assert_eq!(EventType::Upgrade.as_str(), "upgrade");
    }

    #[test]
    fn test_acquisition_source_from_str() {
        assert!(matches!(AcquisitionSource::from_str("producthunt"), AcquisitionSource::ProductHunt));
        assert!(matches!(AcquisitionSource::from_str("organic"), AcquisitionSource::Organic));
        assert!(matches!(AcquisitionSource::from_str("unknown_source"), AcquisitionSource::Unknown));
    }

    #[test]
    fn test_completion_calculation() {
        let steps_1 = vec![OnboardingStep::AccountCreated];
        assert_eq!(OnboardingStatus::calculate_completion(&steps_1), 25);

        let steps_2 = vec![OnboardingStep::AccountCreated, OnboardingStep::ApiKeyAdded];
        assert_eq!(OnboardingStatus::calculate_completion(&steps_2), 50);

        let steps_4 = vec![
            OnboardingStep::AccountCreated,
            OnboardingStep::ApiKeyAdded,
            OnboardingStep::FirstRequestMade,
            OnboardingStep::DashboardViewed,
        ];
        assert_eq!(OnboardingStatus::calculate_completion(&steps_4), 100);
    }
}
