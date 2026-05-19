Feature: Experiment assignment
  As a product manager,
  I want users to be assigned to explicit experiment variants,
  so that experiment exposure is reproducible and measurable.

  Rule: If there is an experiment defined for a page, users who accepted the experiment cookie should be assigned to a variant.

    Example: User assignment
      Given a registered page config:
        """
        {
          "path": "/index.html",
          "page_id": "landing",
          "type": "rfa",
          "template": "cart-v1",
          "rfa": "cart-rfa",
          "timeout_ms": 3000,
          "data": {
            "introduction": {
              "type": "static",
              "value": "Hello dear Customer"
            }
          }
        }
        """
      And a registered experiment:
        """
        {
          "experiment_id": "welcome_message_test",
          "variants": [
            {
              "id": "variant_a",
              "weight": 50
            },
            {
              "id": "variant_b",
              "weight": 50,
              "overrides": {
                "data": {
                  "introduction": {
                    "type": "static",
                    "value": "Hello dear Customer"
                  }
                }
              }
            }
          ]
        }
        """
      And a registered RFA "cart-rfa":
        """
        function(context) { return "Rendered: Introduction is " + context.introduction; }
        """
      When I have accepted experiment cookies
      And I request GET /index.html
      Then the response should contain a Cookie "pp_experiment_welcome_message_test" with value "variant_a" or "variant_b"

  Rule: If the use does not consent experiments he will not be assigned to any experiment variant.

    Example: No user assignment without consent
      Given a registered page config:
        """
        {
          "path": "/index.html",
          "page_id": "landing",
          "type": "rfa",
          "template": "cart-v1",
          "rfa": "cart-rfa",
          "timeout_ms": 3000,
          "data": {
            "introduction": {
              "type": "static",
              "value": "Hello dear Customer"
            }
          }
        }
        """
      And a registered experiment:
        """
        {
          "experiment_id": "welcome_message_test",
          "variants": [
            {
              "id": "variant_a",
              "weight": 50
            },
            {
              "id": "variant_b",
              "weight": 50,
              "overrides": {
                "data": {
                  "introduction": {
                    "type": "static",
                    "value": "Hello dear Customer"
                  }
                }
              }
            }
          ]
        }
        """
      And a registered RFA "cart-rfa":
        """
        function(context) { return "Rendered: Introduction is " + context.introduction; }
        """
      When I have not accepted any tracking and experiment cookies
      And I request GET /index.html
      Then the response should not contain a Cookie "pp_experiment_welcome_message_test" with value "variant_a" or "variant_b"
