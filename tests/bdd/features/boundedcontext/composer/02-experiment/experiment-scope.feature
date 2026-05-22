Feature: Experiement Scope
In order to limit the impact of experiments, so its easier to maintain and reason about them
As a product manager
I want experiment overrides to only apply to a given experiment scope, so that I can be sure that changes only affect the intended RFAs and data. 

  Rule: Replacement only effects Experiment-Scope
    Scenario: Two RFAs with same included RFA, only one is replaced, because of namespace 
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
      And a registered page config:
        """
        {
          "path": "/other.html",
          "page_id": "other",
          "type": "rfa",
          "template": "other",
          "rfa": "p_other_v1",
          "timeout_ms": 3000
        }
        """
      And a registered experiment:
        """
        {
          "experiment_id": "cart_rfa_test",
          "scope": {
            "namespace": "p_landing_v1.*"
          },
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
      And I have accepted the experiment cookie "pp_experiment_cart_rfa_test" with value "variant_b"
      And a registered RFA "p_landing_v1":
        """
        function(context, partials) { return "Do you want to try our product?" + partials.include("a_primary-button_v1"); }
        """
      And a registered RFA "p_other_v1":
        """
        function(context, partials) { return "Welcome to the other page!" + partials.include("a_primary-button_v1"); }
        """
      And a registered RFA "a_primary-button_v1":
        """
        function(context) { return "<button>Ok</button>"; }
        """
      And a registered RFA "a_primary-button_v2":
        """
        function(context) { return "<button>Let's GO!</button>"; }
        """
      When I request GET /index.html
      Then the response should contain "Do you want to try our product?<button>Let's GO!</button>"
      When I request GET /other.html
      Then the response should contain "Welcome to the other page!<button>Ok</button>"

    Scenario: Two Pages with same RFA, only one is replaced, because of exact path match 
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
      Given a registered page config:
        """
        {
          "path": "/other.html",
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
          "scope": {
            "path": "/index.html"
          },
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
      And I have accepted the experiment cookie "pp_experiment_cart_rfa_test" with value "variant_b"
      And a registered RFA "p_landing_v1":
        """
        function(context, partials) { return "Do you want to try our product?" + partials.include("a_primary-button_v1"); }
        """
      And a registered RFA "a_primary-button_v1":
        """
        function(context) { return "<button>Ok</button>"; }
        """
      And a registered RFA "a_primary-button_v2":
        """
        function(context) { return "<button>Let's GO!</button>"; }
        """
      When I request GET /index.html
      Then the response should contain "Do you want to try our product?<button>Let's GO!</button>"
      When I request GET /other.html
      Then the response should contain "Do you want to try our product?<button>Ok</button>"

      Scenario: Two Pages with same RFA, only one is replaced, because of path prefix match 
       Given a registered page config:
        """
        {
          "path": "/stable/index.html",
          "page_id": "landing",
          "type": "rfa",
          "template": "landing",
          "rfa": "p_landing_v1",
          "timeout_ms": 3000
        }
        """
      Given a registered page config:
        """
        {
          "path": "/experiment/index.html",
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
          "scope": {
            "path": "/experiment/*"
          },
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
      And I have accepted the experiment cookie "pp_experiment_cart_rfa_test" with value "variant_b"
      And a registered RFA "p_landing_v1":
        """
        function(context, partials) { return "Do you want to try our product?" + partials.include("a_primary-button_v1"); }
        """
      And a registered RFA "a_primary-button_v1":
        """
        function(context) { return "<button>Ok</button>"; }
        """
      And a registered RFA "a_primary-button_v2":
        """
        function(context) { return "<button>Let's GO!</button>"; }
        """
      When I request GET /experiment/index.html
      Then the response should contain "Do you want to try our product?<button>Let's GO!</button>"
      When I request GET /stable/index.html
       Then the response should contain "Do you want to try our product?<button>Ok</button>"


      @WIP
      Scenario: Path infix wildcard does apply experiment override 
       Given a registered page config:
        """
        {
          "path": "/shop/some/folders/index.html",
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
          "scope": {
            "path": "/shop/*/index.html"
          },
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
      And I have accepted the experiment cookie "pp_experiment_cart_rfa_test" with value "variant_b"
      And a registered RFA "p_landing_v1":
        """
        function(context, partials) { return "Do you want to try our product?" + partials.include("a_primary-button_v1"); }
        """
      And a registered RFA "a_primary-button_v1":
        """
        function(context) { return "<button>Ok</button>"; }
        """
      And a registered RFA "a_primary-button_v2":
        """
        function(context) { return "<button>Let's GO!</button>"; }
        """
      When I request GET /shop/some/folders/index.html
      Then the response should contain "Do you want to try our product?<button>Let's GO!</button>"