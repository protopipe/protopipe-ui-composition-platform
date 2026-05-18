Feature: Experiment Variant Feature 
As a product manager, I want to be able to define experiment variants in the composer page configuration, so that I can run A/B tests on different page versions.
Users should be assigned consistently to experiment variants.

Rule: If user does not allow any tracking, he should be assigned to a default variant that does not require tracking.
Example: User with no tracking allowed
    Given a registered page config:
      """
      {
        "path": "/index.html",
        "page_id": "landing",
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
                  "value": "Hello Mate"
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

Rule: If there is an experiment defined for a page, users who accpeted the experiment cookie, should be assigned to a variant.
Example: User assignment
    Given a registered page config:
      """
      {
        "path": "/index.html",
        "page_id": "landing",
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

Rule: If a user is assigned to an experiment variant, with different static data, he should see the content defined for that variant.

Scenario Outline: Experiment variant <variant_id> assigned content
Example: Experiment variant A assigned content 
    Given a registered page config:
      """
      {
        "path": "/index.html",
        "page_id": "landing",
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
                  "value": "Hello Mate"
                }
              }
            }
          }
        ]
      }
      """
    And I have accepted the experiment cookie "pp_experiment_welcome_message_test" with value "<variant_id>"
    And a registered RFA "cart-rfa":
      """
      function(context) { return "Rendered: Introduction is " + context.introduction; }
      """
    When I request GET /index.html
    Then the response should contain "Rendered: Introduction is <expected_greeting>"

    Examples:
      | variant_id  | expected_greeting     |
      | variant_a   | Hello dear Customer  |
      | variant_b   | Hello Mate           |


  Example: Experiment variant B assigned but no experiment consent given 
     Given a registered page config:
      """
      {
        "path": "/index.html",
        "page_id": "landing",
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
                  "value": "Hello Mate"
                }
              }
            }
          }
        ]
      }
      """
    And I have the experiment cookie "pp_experiment_welcome_message_test" with value "variant_b" without consenting to the experiment cookies
    And a registered RFA "cart-rfa":
      """
      function(context) { return "Rendered: Introduction is " + context.introduction; }
      """
    When I request GET /index.html
    Then the response should not contain a Cookie "pp_experiment_welcome_message_test" with value "variant_a" or "variant_b" 
    And the response should delete the Cookie "pp_experiment_welcome_message_test"
    And the response should contain "Rendered: Introduction is Hello dear Customer"
