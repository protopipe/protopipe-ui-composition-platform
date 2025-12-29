# 06 – Runtime View

## Page Rendering Flow

1. A user requests a Page.
2. The Composer resolves experiment assignments.
3. The Template is loaded.
4. Slots are resolved via RFAs and backend calls (in parallel).
5. HTML is composed and returned.
6. Telemetry and exposure events are emitted.

## Experiment Assignment

Assignments are sticky and based on session or user identifiers.
Pinned assignments are supported for development and preview use cases.

