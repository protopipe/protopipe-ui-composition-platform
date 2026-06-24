# 0022 – Handle Composer POST Requests With 303 Redirects

Date: 2026-05-24

## Status

Accepted

## Context

ADR-0021 restricts frontend submissions to `POST`. The Composer therefore needs
a consistent runtime flow for POST pages.

Returning `200 OK` with rendered HTML directly from a successful POST response
leaves the browser on a POST result. Reloading the page can resubmit the form or
trigger browser warnings. It also blurs the boundary between side-effect
processing and page rendering.

The platform already uses server-side rendering and explicit page
configuration. A POST page can therefore process the user intent and then move
the browser back to a normal GET rendering flow.

## Decision

Successful Composer POST pages respond with `303 See Other`.

The canonical POST flow is:

```text
POST page route
 → Composer resolves the effective POST page configuration
 → Composer calls exactly one configured postService
 → Composer returns 303 See Other
 → Browser follows the Location with GET
 → Composer resolves the GET result page
 → Composer resolves result page data
 → RFA execution
 → HTML response
```

The Composer MUST NOT render the final HTML response directly from the POST
response after a successful submission.

The Composer MUST NOT store submitted form data or submit result state in
process memory in order to render the redirected result page. Any state needed
after the redirect must be owned by backend services or encoded as
non-sensitive routing information allowed by the page configuration.

The POST page configuration MUST declare one `postService`. The `postService`
receives the submitted request body and returns a successful write
acknowledgement according to its contract. After a successful acknowledgement,
the Composer returns a redirect to the configured result route.

The redirect response uses `303 See Other` so that the browser follows the
redirect with `GET`, even though the original request was `POST`. The Composer
MUST NOT use `307` or `308` for successful POST page redirects because those
status codes preserve the original method and could repeat the POST at the
redirect target.

![Composer POST 303 flow](../plantuml/out/composer_post_303_flow.svg)

## Consequences

### Positive

- Browser reload repeats the final GET, not the submitted POST.
- Side-effect processing and result rendering are separated.
- POST pages remain compatible with browser-native forms and progressive
  enhancement.
- The Composer does not become a stateful store for submit results.
- Result rendering follows the same SSR path as other GET pages.

### Negative / Risks

- Result pages cannot rely on Composer-local POST response state.
- Workflows that need action-specific result details need a separate
  consistency contract between the write acknowledgement and the redirected GET
  page.
- A second browser request is required after each successful POST submission.

### Mitigations

- Keep simple confirmations generic when no action-specific result data is
  needed.
- Use backend-owned opaque identifiers and version-fenced reads for workflows
  that need action-specific result rendering.
- Define the version-fenced consistency model separately so the basic 303 flow
  stays simple.
