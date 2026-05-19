Feature: Message Bridge login response events
  As an interactive frontend artifact,
  I want login responses to arrive as correlated response events,
  so that the frontend can update its authentication context without the Composer owning authentication.

  Rule: Response events resolve interaction events by correlation id.

    Example: Login response returns a short-lived bearer token
      Given a registered IFA response channel for "login-ifa"
      When the frontend emits login interaction event "auth.login.requested" with correlation id "corr-login-1"
      And the auth service produces response event "auth.login.succeeded" for correlation id "corr-login-1"
      Then the frontend should receive response event "auth.login.succeeded"
      And the response event should contain a short-lived bearer token
      And the response event should correlate to "corr-login-1"

  Rule: Authentication response events are scoped to the originating client context.

  Rule: Authentication tokens are response-event payloads, not business-event payloads.
