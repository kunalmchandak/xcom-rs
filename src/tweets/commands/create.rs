//! Tweet creation and idempotency handling.

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::tweets::http_client::XApiClient;
use crate::tweets::ledger::IdempotencyLedger;
use crate::tweets::models::{Tweet, TweetMeta};

use super::types::{CreateArgs, CreateResult, IdempotencyConflictError, IfExistsPolicy};

/// Create a tweet with idempotency support.
pub fn create(
    ledger: &IdempotencyLedger,
    http_client: &XApiClient,
    args: CreateArgs,
) -> Result<CreateResult> {
    // Generate client_request_id if not provided
    let client_request_id = args
        .client_request_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Compute request hash for storing (but not for lookup key)
    let request_hash = IdempotencyLedger::compute_request_hash(&args.text);

    // Check ledger for existing operation by client_request_id only
    if let Some(entry) = ledger
        .lookup(&client_request_id)
        .context("Failed to lookup operation in ledger")?
    {
        // Found existing operation with this client_request_id
        match args.if_exists {
            IfExistsPolicy::Return => {
                // Return cached result (even if parameters differ)
                let mut tweet = Tweet::new(entry.tweet_id.clone());
                tweet.text = Some(args.text.clone());

                let meta = TweetMeta {
                    client_request_id: client_request_id.clone(),
                    from_cache: Some(true),
                };

                return Ok(CreateResult { tweet, meta });
            }
            IfExistsPolicy::Error => {
                // Return error for duplicate client_request_id
                return Err(IdempotencyConflictError {
                    client_request_id: client_request_id.clone(),
                }
                .into());
            }
        }
    }

    // Call X API to create tweet
    let tweet = http_client
        .create_tweet(&args.text, None)
        .context("Failed to create tweet via X API")?;

    // Record successful operation in ledger
    ledger
        .record(&client_request_id, &request_hash, &tweet.id, "success")
        .context("Failed to record operation in ledger")?;

    let meta = TweetMeta {
        client_request_id,
        from_cache: None,
    };

    Ok(CreateResult { tweet, meta })
}
