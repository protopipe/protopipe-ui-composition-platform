@WIP
Feature: Proxy Page marker replacement
  As a product manager,
  I want accepted Proxy Page markers to be replaced by RFAs,
  so that validated monolith regions can be permanently delivered by composer-native artifacts.

  Rule: Accepted marker replacements belong to the Page definition.

    Example: Replace an accepted marker with an RFA
      Given an upstream monolith responds to GET /checkout with:
        """
        <!doctype html>
        <html>
          <body>
            <h1>Checkout</h1>
            <!-- protopipe:marker checkout.summary -->
            <section>Legacy checkout summary</section>
            <!-- /protopipe:marker checkout.summary -->
          </body>
        </html>
        """
      And a registered page config:
        """
        {
          "path": "/checkout",
          "page_id": "checkout",
          "type": "rfa",
          "delivery": {
            "type": "upstream-proxy",
            "origin": "http://legacy-monolith",
            "markers": [
              {
                "id": "checkout.summary",
                "rfa": "checkout-summary-rfa",
                "fallback": "keep-upstream"
              }
            ]
          },
          "timeout_ms": 3000
        }
        """
      And a registered RFA "checkout-summary-rfa":
        """
        function(context) { return "<section>Composer checkout summary</section>"; }
        """
      When I request GET /checkout
      Then the response status should be 200
      And the response should contain "<section>Composer checkout summary</section>"
      And the response should not contain "<section>Legacy checkout summary</section>"

  Rule: A marker without an active replacement passes through unchanged.

    Example: Keep upstream marker content when no replacement is configured
      Given an upstream monolith responds to GET /checkout with:
        """
        <!-- protopipe:marker checkout.summary -->
        <section>Legacy checkout summary</section>
        <!-- /protopipe:marker checkout.summary -->
        """
      And a registered Proxy Page without marker replacements for "/checkout"
      When I request GET /checkout
      Then the response status should be 200
      And the response should contain "<section>Legacy checkout summary</section>"
