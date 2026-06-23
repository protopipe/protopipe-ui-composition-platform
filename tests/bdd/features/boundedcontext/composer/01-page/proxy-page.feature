@WIP
Feature: Proxy Page without markers
  As a platform engineer,
  I want to register a page that proxies an upstream monolith response,
  so that existing monolith pages can be served through the composer before any replacement markers exist.

  Rule: A Proxy Page without active markers streams the upstream response unchanged.

    Example: Proxy a monolith page without marker replacement
      Given an upstream monolith responds to GET /shop/cart with:
        """
        <!doctype html>
        <html>
          <body>
            <h1>Legacy cart</h1>
          </body>
        </html>
        """
      And a registered page config:
        """
        {
          "path": "/shop/cart",
          "page_id": "legacy-cart",
          "type": "rfa",
          "delivery": {
            "type": "upstream-proxy",
            "origin": "http://legacy-monolith"
          },
          "timeout_ms": 3000
        }
        """
      When I request GET /shop/cart
      Then the response status should be 200
      And the response should contain "<h1>Legacy cart</h1>"
      And the upstream response should be streamed without marker replacement
