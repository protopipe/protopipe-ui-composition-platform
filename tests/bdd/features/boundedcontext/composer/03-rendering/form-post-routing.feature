@WIP
Feature: Form POST routing
  As a product manager,
  I want the Composer to route submitted form data to one configured backend service,
  so that RFAs can render a confirmation page from the service result and supporting page data.

  Rule: A submitted form is processed by one POST service while supporting data is still resolved with GET.

    Scenario: Contact request submission renders a PageResult confirmation

      Given a backend service "contact"
      And a backend service "content"
      And backend service "contact" returns JSON for POST /contact-requests:
        """
        {
          "status": "received",
          "reference": "REQ-123"
        }
        """
      And backend service "content" returns JSON for GET /contact-page:
        """
        {
          "headline": "Kontaktanfrage"
        }
        """
      And a registered page config:
        """
        {
          "path": "/contact.html",
          "method": "POST",
          "page_id": "contact-page-result",
          "type": "rfa",
          "template": "PageResult",
          "rfa": "contact_page_result_v1",
          "timeout_ms": 3000,
          "submit": {
            "target": "submission",
            "service": "contact",
            "path": "/contact-requests",
            "method": "POST",
            "content_type": "application/x-www-form-urlencoded",
            "timeout_ms": 500,
            "error_default": {
              "status": "failed",
              "reference": null
            }
          },
          "data": {
            "page": {
              "type": "restService",
              "service": "content",
              "path": "/contact-page",
              "method": "GET",
              "timeout_ms": 250,
              "error_default": {
                "headline": "Kontakt"
              }
            }
          }
        }
        """
      And a registered RFA "contact_page_result_v1":
        """
        function(context) { return context.page.headline + ": Ihr Formular wurde erhalten. Referenz " + context.submission.reference; }
        """
      When I submit POST /contact.html with form data:
        """
        name=Margaret Hamilton&email=margaret.hamilton@example.test&message=Bitte bestaetigen Sie die Landeprozedur.
        """
      Then the response status should be 200
      And the response should contain "Kontaktanfrage: Ihr Formular wurde erhalten. Referenz REQ-123"
      And backend service "contact" should have received 1 request for POST /contact-requests
      And backend service "contact" should have received a POST request for /contact-requests containing form field "email" with value "margaret.hamilton@example.test"
      And backend service "content" should have received 1 request for GET /contact-page
      And backend service "content" should have received 0 requests for POST /contact-page
