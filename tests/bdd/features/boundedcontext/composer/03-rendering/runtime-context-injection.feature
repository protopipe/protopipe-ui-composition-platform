Feature: Runtime Context Injection
  As a product manager,
  I want to inject runtime context into RFAs,
  so that I can render dynamic content based on the request or user data.


  Rule: Namespace context is injected into Partials
  
    Scenario: RFA receives always namespace-context and renders it in partial
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
      And a registered RFA "p_landing_v1":
        """
        function(context, partials) { return partials.include('a_namespace-printer_v1', context); }
        """
      And a registered RFA "a_namespace-printer_v1":
        """
        function(context) { return context.namespace; }
        """
      When I request GET /index.html
      Then the response should contain "p_landing_v1.a_namespace-printer_v1"

  Rule: Namespace and index context is injected into Partials, when using include with index
  
    Scenario: RFA receives always namespace-context and renders it in partial
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
      And a registered RFA "p_landing_v1":
        """
        function(context, partials) { return partials.include('a_namespace-printer_v1', context, 1); }
        """
      And a registered RFA "a_namespace-printer_v1":
        """
        function(context) { return context.namespace; }
        """
      When I request GET /index.html
      Then the response should contain "p_landing_v1.1.a_namespace-printer_v1"

  Rule: URL is injected into Partials, when requested by page-config

    Scenario: RFA receives URL (without GET-Parameters) context and renders dynamic content
      Given a registered page config:
        """
        {
          "path": "/index.html",
          "page_id": "landing",
          "type": "rfa",
          "template": "landing",
          "rfa": "p_landing_v1",
          "timeout_ms": 3000,
          "data": {
            "url": {
              "type": "url"
            }
          }
        }
        """
      And a registered RFA "p_landing_v1":
        """
        function(context, partials) { return partials.include('a_url-printer_v1', context); }
        """
      And a registered RFA "a_url-printer_v1":
        """
        function(context) { return context.url; }
        """
      When I request GET /index.html?message=Hello
      Then the response should contain "/index.html"    

  Rule: GET-Parameter context is injected into Partials, when requested by page-config

    Scenario: RFA receives GET-Parameter context and renders dynamic content
      Given a registered page config:
        """
        {
          "path": "/index.html",
          "page_id": "landing",
          "type": "rfa",
          "template": "landing",
          "rfa": "p_landing_v1",
          "timeout_ms": 3000,
          "data": {
            "getMessage": {
              "type": "getParameter",
              "key": "message"
            }
          }
        }
        """
      And a registered RFA "p_landing_v1":
        """
        function(context, partials) { return partials.include('a_get-param-printer_v1', context); }
        """
      And a registered RFA "a_get-param-printer_v1":
        """
        function(context) { return context.getMessage; }
        """
      When I request GET /index.html?message=Hello
      Then the response should contain "Hello"
  
