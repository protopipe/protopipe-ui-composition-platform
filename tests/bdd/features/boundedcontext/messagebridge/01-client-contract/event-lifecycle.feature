Feature: Message Bridge interaction event lifecycle
  As a UI artifact owner,
  I want interaction events to move through explicit lifecycle states,
  so that optimistic UI behavior can be reconciled predictably.

  Rule: Interaction events move from observed to buffered, delivered, processed, and resolved.

  Rule: Delivered interaction events are not authoritative until they are processed and resolved.

  Rule: Technical failures are represented separately from business events.
