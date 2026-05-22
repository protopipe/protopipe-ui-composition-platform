Feature: Experiment RFA overrides
  As a product manager,
  I want an experiment variant to replace a selected RFA,
  so that I can test artifact changes without adding variant logic to RFAs.

  Rule: If a user is assigned to an experiment variant with a targeted RFA replacement, he should see the RFA defined for that variant.

    Scenario Outline: Experiment testing new button globally
      Given a registered page config:
        """
        {
          "path": "/index.html",
          "page_id": "landing",
          "type": "rfa",
          "template": "landing",
          "rfa": "p_landing_v1",
          "timeout_ms": 3000
        }
        """
      And a registered experiment:
        """
        {
          "experiment_id": "cart_rfa_test",
          "variants": [
            {
              "id": "variant_a",
              "weight": 50
            },
            {
              "id": "variant_b",
              "weight": 50,
              "overrides": {
                "rfa": {
                  "old": "a_primary-button_v1",
                  "new": "a_primary-button_v2"
                }
              }
            }
          ]
        }
        """
      And I have accepted the experiment cookie "pp_experiment_cart_rfa_test" with value "<variant_id>"
      And a registered RFA "p_landing_v1":
        """
        function(context, partials) { return "Do you want to try our product?" + partials.include("a_primary-button_v1"); }
        """
      And a registered RFA "a_primary-button_v1":
        """
        function(context) { return "&lt;button&gt;Ok&lt;/button&gt;"; }
        """
      And a registered RFA "a_primary-button_v2":
        """
        function(context) { return "&lt;button&gt;Let's GO!&lt;/button&gt;"; }
        """
      When I request GET /index.html
      Then the response should contain "Do you want to try our product?&lt;button&gt;<expected_button_text>&lt;/button&gt;"

      Examples:
        | variant_id | expected_button_text |
        | variant_a  | Ok                   |
        | variant_b  | Let's GO!            |

  Rule: Targeted RFA replacements do not apply when the current RFA does not match the expected old RFA.
