Feature: Experiment analytics separation
  As a data owner,
  I want experiment analytics to be separated from operational telemetry,
  so that consent and retention rules can be enforced clearly.

  Rule: Experiment analytics is only collected when explicit consent is available.

  Rule: Experiment analytics and observability are routed through separate pipelines.

  Rule: User-related analytics data follows explicit retention, deletion, and anonymization rules.
