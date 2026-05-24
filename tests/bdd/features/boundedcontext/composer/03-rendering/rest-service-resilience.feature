@WIP
Feature: REST service resilience
  As an operator,
  I want REST-backed page data to use explicit resilience policies,
  so that unstable backend services do not exhaust Composer rendering capacity.

  Rule: REST service calls are isolated by bounded concurrency per service.

  Rule: REST service calls use a small bounded retry policy with backoff.

  Rule: REST service calls must use bounded timeouts and explicit degraded rendering behavior.

  Rule: REST service calls can switch to an explicit fallback when the primary service fails.

  Rule: A circuit breaker opens when a service exceeds its configured failure threshold.

  Rule: An open circuit uses the configured fallback or error default without calling the primary service.

  Rule: A half-open circuit periodically probes the primary service and closes again after successful probes.

  Rule: Circuit breaker transitions are emitted as operational telemetry.
