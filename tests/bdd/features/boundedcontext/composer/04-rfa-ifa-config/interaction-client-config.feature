Feature: RFA and IFA client config
  As a product manager,
  I want client-relevant RFA and IFA configuration to be rendered into the initial page response,
  so that artifacts receive the runtime context they need without owning platform decisions.

  Rule: RFA pages receive deterministic render context but no message client configuration.

  Rule: IFA pages receive message client configuration from the effective Page and Experiment configuration.

  Rule: IFA client configuration includes page, artifact, and experiment context from the Composer assignment.
