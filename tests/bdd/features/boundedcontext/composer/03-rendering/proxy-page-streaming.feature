Feature: Proxy Page chunk-based streaming
  As a platform engineer,
  I want Proxy Pages to flush upstream body chunks before the complete upstream response has arrived,
  so that browsers can start parsing monolith HTML while the rest of the page is still streaming.

  Rule: Proxy Pages without active marker replacement pass upstream body chunks through as they arrive.

    Example: Flush the first upstream body chunk before the full upstream body is complete
      Given an upstream monolith streams GET /slow-proxy as 4 chunks over 2000 ms with:
        """
        FIRST-CHUNK-PREFIX
        second chunk content
        third chunk content
        final chunk content
        """
      And a registered page config:
        """
        {
          "path": "/slow-proxy",
          "page_id": "slow-proxy",
          "type": "rfa",
          "delivery": {
            "type": "upstream-proxy",
            "origin": "http://legacy-monolith"
          },
          "timeout_ms": 3000
        }
        """
      When I stream GET /slow-proxy until the first response body chunk
      Then the first streamed response body chunk should arrive before 1500 ms
      And the first streamed response body chunk should contain "FIRST-CHUNK-PREFIX"
