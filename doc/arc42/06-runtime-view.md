# 06 – Runtime View

## Server-Side Rendering (SSR) Flow

The platform follows an **SSR-first rendering model** (see [ADR-0007](../adr/0007-ssr-first-rendering-model.md)). All pages are composed and rendered on the server before being delivered to the client.

```text
Request
 → Page Route Resolution
 → Experiment Assignment and Page Overrides
 → Page Data Resolution, including restService calls
 → RFA Execution
 → HTML Response
 → Telemetry Emission
```

### Detailed Steps

1. **User Request**: Client requests a page via HTTP `GET`.
2. **Page Route Resolution**: The Composer resolves the matching Page
   configuration for the request path and method.
3. **Experiment Assignment** (centralized, see [ADR-0008](../adr/0008-centralize-experiment-routing-before-rendering.md)):
   - If no consent is given: assign default (control) variant (see [ADR-0009](../adr/0009-use-default-variant-when-no-consent-is-given.md))
   - If consent given: assign variant based on assignment rules
   - Assignment is sticky and based on session or user identifiers
   - Pinned assignments are supported for development and preview use cases
4. **Page Override Application**: Apply experiment-based Page data and artifact
   overrides before any backend data is loaded.
5. **Page Data Resolution**: Resolve declared data values for the effective Page
   configuration. `restService` values are loaded server-side by the Composer
   through named Services, with explicit timeouts and degraded rendering
   defaults where configured (see [ADR-0020](../adr/0020-resolve-page-rest-service-data-server-side.md)).
   Independent `restService` data values may be resolved concurrently.
6. **RFA Execution**: Each RFA:
   - Receives explicit input data and context
   - Executes deterministically
   - Returns HTML (and optional metadata)
   - Does NOT perform experiment assignment (already done)
   - Does NOT perform backend service calls
   - Does NOT contain branching logic based on variants
7. **HTML Assembly**: Combine rendered output into final HTML
8. **Response Delivery**: Return fully rendered HTML to client
9. **Telemetry Emission**:
   - Observability metrics (request counts, latencies)
   - Experiment exposure events (which variant the user saw)
   - Both pipelines are separated (see [ADR-0010](../adr/0010-seperate-observability-from-experiment-analytics.md))

## Form POST Flow

Frontend submissions to the Composer use `POST` only (see
[ADR-0021](../adr/0021-use-post-only-for-frontend-submissions.md)). Successful
POST routes process one configured `postService` and then return `303 See
Other`; the Composer does not render the final HTML directly from the POST
response (see [ADR-0022](../adr/0022-handle-composer-post-requests-with-303-redirects.md)).

```text
POST page route
 → Resolve effective POST route
 → Call exactly one postService
 → 303 See Other
 → Browser follows Location with GET
 → Resolve GET result page
 → Resolve result page data
 → RFA Execution
 → HTML Response
```

When a POST result needs action-specific state, the post service returns
non-sensitive acknowledgement metadata such as stream, business key,
partition key, and version. The Composer may include allowed acknowledgement
fields in the redirect URL so the redirected GET page can use them as a
version fence (see [ADR-0023](../adr/0023-use-version-fenced-post-results-for-eventual-consistency.md)).
The Composer must not store submitted form data or result payloads in process
memory to bridge POST and redirected GET requests.

## Experiment Assignment

Experiment routing is centralized and executes **before rendering begins** (see [ADR-0008](../adr/0008-centralize-experiment-routing-before-rendering.md)):

- Assignment happens once per request
- Result determines all subsequent artifact selection
- RFAs MUST NOT perform experiment assignment
- Templates and Slots define structure only (no branching logic)
- Clear separation between decision logic and rendering logic

## Client-Side Rendering

Client-side logic is **optional and non-authoritative**:

- Progressive enhancement only
- Client receives fully rendered HTML as canonical output
- Interactive UI features are handled via interactive artifacts (see [ADR-0013](../adr/0013-distinguish-render-and-interactive-ui-artifacts.md))
- Event-based interaction model (see [ADR-0011](../adr/0011-use-event-based-interaction-model-for-ui-artifacts.md))
<<<<<<< HEAD

## Proxy Page Rendering Flow

Proxy Pages support monolith decomposition through streamed upstream responses
and marker replacement (see
[ADR-0020](../adr/0020-support-proxy-page-markers-as-stable-and-experimental-composition-points.md)).

```text
Request
 → Page Resolution
 → Experiment Assignment
 → Effective Proxy Page Configuration
 → Upstream Request
 → Data Loading and RFA Rendering for Active Markers (parallel)
 → Streaming Marker Detection
 → Marker Replacement or Fallback
 → Streamed HTML Response
 → Telemetry Emission
```

The Composer forwards upstream bytes as early as possible. For active marker
replacements, it starts data loading and RFA rendering in parallel with upstream
streaming. If the upstream response reaches a configured marker before the
replacement is ready, the Composer may wait at that marker until the RFA output,
timeout, or fallback decision is available.
=======
>>>>>>> ab022e2 (cleanup)
