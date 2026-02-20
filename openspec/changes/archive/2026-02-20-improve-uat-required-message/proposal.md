# Proposal: Improve Error Messages for Missing UAT

## Why
When `XCOM_RS_BEARER_TOKEN` is set but the OAuth2 user access token (user context) is missing, user-context-required operations like `/2/users/me` fail with 403. Current errors like `Authorization failed` or `InternalError` are unclear about the root cause and next steps.

## What Changes
- Detect UAT-missing scenarios using 403 responses containing `application-only` / `user context` keywords.
- Return `error.code=auth_required` with structured `nextSteps` pointing to `xcom-rs auth login` or manual UAT configuration.
- Ensure consistent structured errors in non-interactive mode.

### In Scope
- Improve error messages for user-context-required commands relying on `/2/users/me`.
- Parse 403 response body to detect UAT absence and return `auth_required`.
- Add `nextSteps` to error responses.

### Out of Scope
- Implementing the OAuth2 login flow itself (`auth login` command).
- Changing actual API behavior or permission models.

## Expected Behavior
- When a user-context-required command is executed with an app-only token, return `error.code=auth_required` and `nextSteps`.
- `nextSteps` includes guidance to run `xcom-rs auth login` or set up UAT manually.

## References
- Align with existing `auth_required` error and `nextSteps` output format.
- Use 403 response body containing `application-only` / `user context` as detection criteria.
