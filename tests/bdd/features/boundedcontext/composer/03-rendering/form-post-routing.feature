Feature: Form POST routing
  As a product manager,
  I want the Composer to route submitted form data to one configured backend service,
  so that successful submissions redirect to a GET result page without resubmitting the form.

  Successful POST submissions follow ADR-0022 and return 303 See Other before
  rendering the result page. This keeps browser reloads from repeating form
  submissions and keeps experiment-driven side effects manageable: experiments
  may replace the configured post service, but RFAs still render only after the
  side effect has been acknowledged and represented as version-fenced GET
  parameters for the result page.

  Rule: A submitted form is processed by one POST service before redirecting to a GET result page.

    Scenario: Contact request submission redirects with a version-fenced write acknowledgement

      Given a backend service "contact"
      And backend service "contact" returns JSON for POST /contact-requests:
        """
        {
          "stream": "contact-requests",
          "subject": "contactRequest",
          "businessKey": "REQ-123",
          "partitionKey": "contact:REQ-123",
          "version": 7,
          "eventId": "evt-contact-123"
        }
        """
      And a registered page config:
        """
        {
          "path": "/contact.html",
          "method": "POST",
          "page_id": "contact-page-result",
          "timeout_ms": 3000,
          "postService": {
            "service": "contact",
            "path": "/contact-requests",
            "content_type": "application/x-www-form-urlencoded",
            "timeout_ms": 500,
            "redirect": {
              "path": "/contact/received.html"
            }
          }
        }
        """
      When I submit POST /contact.html with form data:
        """
        name=Margaret Hamilton&email=margaret.hamilton@example.test&message=Bitte bestaetigen Sie die Landeprozedur.
        """
      Then the response status should be 303
      And the response header "Location" should contain "/contact/received.html"
      And the response header "Location" should contain "stream=contact-requests"
      And the response header "Location" should contain "businessKey=REQ-123"
      And the response header "Location" should contain "partitionKey=contact%3AREQ-123"
      And the response header "Location" should contain "version=7"
      And backend service "contact" should have received 1 request for POST /contact-requests
      And backend service "contact" should have received a POST request for /contact-requests containing form field "email" with value "margaret.hamilton@example.test"

    Scenario: Redirected result page reads the acknowledged write through GET data mappings

      Given a backend service "contact"
      And backend service "contact" returns JSON for POST /contact-requests:
        """
        {
          "stream": "contact-requests",
          "subject": "contactRequest",
          "businessKey": "REQ-123",
          "partitionKey": "contact:REQ-123",
          "version": 7,
          "eventId": "evt-contact-123"
        }
        """
      And backend service "contact" returns JSON for GET /contact-request-results?businessKey=REQ-123&partitionKey=contact%3AREQ-123&stream=contact-requests&version=7:
        """
        {
          "businessKey": "REQ-123",
          "status": "received",
          "message": "Your formular was received. Thank you Ms. Hamilton!"
        }
        """
      And a registered page config:
        """
        {
          "path": "/contact.html",
          "method": "POST",
          "page_id": "contact-page-result",
          "timeout_ms": 3000,
          "postService": {
            "service": "contact",
            "path": "/contact-requests",
            "content_type": "application/x-www-form-urlencoded",
            "timeout_ms": 500,
            "redirect": {
              "path": "/contact/received.html"
            }
          }
        }
        """
      And a registered page config:
        """
        {
          "path": "/contact/received.html",
          "method": "GET",
          "page_id": "contact-received",
          "type": "rfa",
          "template": "contact-received",
          "rfa": "p_contact_received_v1",
          "timeout_ms": 3000,
          "data": {
            "result": {
              "type": "restService",
              "service": "contact",
              "path": "/contact-request-results",
              "method": "GET",
              "timeout_ms": 250,
              "request": {
                "query": {
                  "stream": {
                    "from": "query",
                    "name": "stream"
                  },
                  "businessKey": {
                    "from": "query",
                    "name": "businessKey"
                  },
                  "partitionKey": {
                    "from": "query",
                    "name": "partitionKey"
                  },
                  "version": {
                    "from": "query",
                    "name": "version"
                  }
                }
              },
              "error_default": {
                "message": "Your formular is processed but the process did take longer then expected. Please reload page."
              }
            }
          }
        }
        """
      And a registered RFA "p_contact_received_v1":
        """
        function(context) { return context.result.message + " Reference: " + context.result.businessKey; }
        """
      When I submit POST /contact.html with form data:
        """
        name=Margaret Hamilton&email=margaret.hamilton@example.test&message=Please confirm the landing procedure.
        """
      Then the response status should be 303
      When I follow the response redirect
      Then the response status should be 200
      And the response should contain "Your formular was received. Thank you Ms. Hamilton!"
      And the response should contain "Reference: REQ-123"
      And backend service "contact" should have received 1 request for POST /contact-requests
      And backend service "contact" should have received 1 request for GET /contact-request-results?businessKey=REQ-123&partitionKey=contact%3AREQ-123&stream=contact-requests&version=7
