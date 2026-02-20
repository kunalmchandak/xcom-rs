# uat-required-messages Specification

## Purpose
Defines error message behavior when user-context-required operations are attempted with application-only tokens, ensuring users receive clear guidance on next steps.

## Requirements
### Requirement: User-context-required operation authentication error guidance
`xcom-rs` MUST return `auth_required` and `nextSteps` when user-context-required API calls are rejected with an application-only token, presenting clear guidance for next actions.

#### Scenario: `/2/users/me` is rejected with 403
- **Given** `XCOM_RS_BEARER_TOKEN` is set
- **And** user access token (user context) does not exist
- **And** `GET /2/users/me` fails with `403` and a response containing `application-only`
- **When** a user-context-required command is executed
- **Then** return `error.code=auth_required`
- **And** `error.details.nextSteps` includes `xcom-rs auth login`
- **And** `error.details.nextSteps` includes guidance for `XCOM_RS_BEARER_TOKEN`

