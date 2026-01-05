//! Circuit Breaker pattern for preventing cascading failures
//!
//! Tracks failure counts and temporarily blocks requests to failing endpoints
//! to allow them time to recover.

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Circuit breaker state
#[derive(Clone, Debug, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, testing if endpoint has recovered
    HalfOpen,
}

/// Circuit breaker for a single endpoint
#[derive(Clone, Debug)]
struct Circuit {
    state: CircuitState,
    failure_count: u32,
    last_failure_time: Option<Instant>,
    last_state_change: Instant,
}

impl Circuit {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure_time: None,
            last_state_change: Instant::now(),
        }
    }
}

/// Circuit breaker configuration
#[derive(Clone, Debug)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Time window for counting failures (seconds)
    pub failure_window_secs: u64,
    /// How long to keep circuit open before trying again (seconds)
    pub open_timeout_secs: u64,
    /// How long to wait in half-open state before closing (seconds)
    pub half_open_timeout_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_window_secs: 60,
            open_timeout_secs: 30,
            half_open_timeout_secs: 10,
        }
    }
}

/// Global circuit breaker manager
pub struct CircuitBreaker {
    circuits: RwLock<HashMap<String, Circuit>>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default config
    pub fn new() -> Self {
        Self::with_config(CircuitBreakerConfig::default())
    }

    /// Create a new circuit breaker with custom config
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            circuits: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Check if a request to the given endpoint should be allowed
    pub fn should_allow(&self, endpoint: &str) -> bool {
        let mut circuits = self.circuits.write().unwrap();
        let circuit = circuits.entry(endpoint.to_string()).or_insert_with(Circuit::new);

        let now = Instant::now();

        // Update circuit state based on time
        match circuit.state {
            CircuitState::Open => {
                // Check if it's time to try again (half-open)
                if now.duration_since(circuit.last_state_change) >= Duration::from_secs(self.config.open_timeout_secs) {
                    circuit.state = CircuitState::HalfOpen;
                    circuit.last_state_change = now;
                    tracing::info!("Circuit breaker for {} transitioning to HALF-OPEN", endpoint);
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow request in half-open state
                true
            }
            CircuitState::Closed => {
                // Check if failures are outside the window
                if let Some(last_failure) = circuit.last_failure_time {
                    if now.duration_since(last_failure) >= Duration::from_secs(self.config.failure_window_secs) {
                        // Reset failure count
                        circuit.failure_count = 0;
                    }
                }
                true
            }
        }
    }

    /// Record a successful request
    pub fn record_success(&self, endpoint: &str) {
        let mut circuits = self.circuits.write().unwrap();
        let circuit = circuits.entry(endpoint.to_string()).or_insert_with(Circuit::new);

        let now = Instant::now();

        match circuit.state {
            CircuitState::HalfOpen => {
                // Success in half-open state -> close circuit
                circuit.state = CircuitState::Closed;
                circuit.failure_count = 0;
                circuit.last_state_change = now;
                tracing::info!("Circuit breaker for {} transitioning to CLOSED", endpoint);
            }
            CircuitState::Closed => {
                // Reset failure count on success
                circuit.failure_count = 0;
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
                tracing::warn!("Received success for {} while circuit is OPEN", endpoint);
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&self, endpoint: &str) {
        let mut circuits = self.circuits.write().unwrap();
        let circuit = circuits.entry(endpoint.to_string()).or_insert_with(Circuit::new);

        let now = Instant::now();
        circuit.failure_count += 1;
        circuit.last_failure_time = Some(now);

        match circuit.state {
            CircuitState::HalfOpen => {
                // Failure in half-open state -> reopen circuit
                circuit.state = CircuitState::Open;
                circuit.last_state_change = now;
                tracing::warn!("Circuit breaker for {} transitioning to OPEN (half-open failure)", endpoint);
            }
            CircuitState::Closed => {
                // Check if we've exceeded threshold
                if circuit.failure_count >= self.config.failure_threshold {
                    circuit.state = CircuitState::Open;
                    circuit.last_state_change = now;
                    tracing::warn!(
                        "Circuit breaker for {} transitioning to OPEN ({} failures)",
                        endpoint,
                        circuit.failure_count
                    );
                }
            }
            CircuitState::Open => {
                // Already open, just track the failure
                tracing::debug!("Circuit breaker for {} recording failure while OPEN", endpoint);
            }
        }
    }

    /// Get the current state of a circuit
    pub fn get_state(&self, endpoint: &str) -> CircuitState {
        let circuits = self.circuits.read().unwrap();
        circuits
            .get(endpoint)
            .map(|c| c.state.clone())
            .unwrap_or(CircuitState::Closed)
    }

    /// Reset a circuit to closed state
    pub fn reset(&self, endpoint: &str) {
        let mut circuits = self.circuits.write().unwrap();
        if let Some(circuit) = circuits.get_mut(endpoint) {
            circuit.state = CircuitState::Closed;
            circuit.failure_count = 0;
            circuit.last_state_change = Instant::now();
            tracing::info!("Circuit breaker for {} manually reset to CLOSED", endpoint);
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_opens_after_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            failure_window_secs: 60,
            open_timeout_secs: 30,
            half_open_timeout_secs: 10,
        };
        let breaker = CircuitBreaker::with_config(config);
        let endpoint = "test-endpoint";

        // Should allow requests initially
        assert!(breaker.should_allow(endpoint));

        // Record failures
        breaker.record_failure(endpoint);
        assert!(breaker.should_allow(endpoint));
        assert_eq!(breaker.get_state(endpoint), CircuitState::Closed);

        breaker.record_failure(endpoint);
        assert!(breaker.should_allow(endpoint));
        assert_eq!(breaker.get_state(endpoint), CircuitState::Closed);

        breaker.record_failure(endpoint);
        // Circuit should now be open
        assert_eq!(breaker.get_state(endpoint), CircuitState::Open);
        assert!(!breaker.should_allow(endpoint));
    }

    #[test]
    fn test_circuit_recovers() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            failure_window_secs: 60,
            open_timeout_secs: 1, // Short timeout for testing
            half_open_timeout_secs: 10,
        };
        let breaker = CircuitBreaker::with_config(config);
        let endpoint = "test-endpoint";

        // Open the circuit
        breaker.record_failure(endpoint);
        breaker.record_failure(endpoint);
        assert_eq!(breaker.get_state(endpoint), CircuitState::Open);

        // Wait for open timeout
        std::thread::sleep(Duration::from_secs(2));

        // Should transition to half-open
        assert!(breaker.should_allow(endpoint));
        assert_eq!(breaker.get_state(endpoint), CircuitState::HalfOpen);

        // Success in half-open should close circuit
        breaker.record_success(endpoint);
        assert_eq!(breaker.get_state(endpoint), CircuitState::Closed);
    }
}
