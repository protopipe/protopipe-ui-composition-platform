Feature: Page artifact type
  As a product manager,
  I want a page config to declare whether it renders an RFA or an IFA,
  so that interaction-capable pages can be configured explicitly.

  Rule: RFA pages render deterministic server-side output only and must not define interaction config.

  Rule: IFA pages render deterministic initial server-side output and must define interaction config.

  Rule: The page artifact type is part of the page config used by the composer.
