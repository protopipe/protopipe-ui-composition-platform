# 06 – Runtime View

## Server-Side Rendering (SSR) Flow

The platform follows an **SSR-first rendering model** (see [ADR-0007](../adr/0007-ssr-first-rendering-model.md)). All pages are composed and rendered on the server before being delivered to the client.

```text
Request
 → Experiment Assignment
 → Template Resolution
 → Slot Composition
 → RFA Execution (parallel)
 → HTML Response
 → Telemetry Emission
```

### Detailed Steps

1. **User Request**: Client requests a page via HTTP.
2. **Experiment Assignment** (centralized, see [ADR-0008](../adr/0008-centralize-experiment-routing-before-rendering.md)):
   - If no consent is given: assign default (control) variant (see [ADR-0009](../adr/0009-use-default-variant-when-no-consent-is-given.md))
   - If consent given: assign variant based on assignment rules
   - Assignment is sticky and based on session or user identifiers
   - Pinned assignments are supported for development and preview use cases
3. **Template Resolution**: Load the Page's Template (immutable layout definition)
4. **Slot Composition**: For each Slot in the Template:
   - Apply experiment-based artifact selection
   - Collect required backend data
   - Execute RFA in parallel
5. **RFA Execution**: Each RFA:
   - Receives explicit input data and context
   - Executes deterministically
   - Returns HTML (and optional metadata)
   - Does NOT perform experiment assignment (already done)
   - Does NOT contain branching logic based on variants
6. **HTML Assembly**: Combine rendered slots into final HTML
7. **Response Delivery**: Return fully rendered HTML to client
8. **Telemetry Emission**:
   - Observability metrics (request counts, latencies)
   - Experiment exposure events (which variant the user saw)
   - Both pipelines are separated (see [ADR-0010](../adr/0010-seperate-observability-from-experiment-analytics.md))

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

