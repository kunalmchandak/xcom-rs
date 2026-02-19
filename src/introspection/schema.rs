//! JSON Schema generation for all commands.
//!
//! [`CommandSchema::for_command`] returns the input/output JSON schema for a
//! given command name. The set of recognized command names must stay in sync
//! with [`super::registry::CommandsList`].

use serde::{Deserialize, Serialize};

/// JSON Schema descriptor for a command's input and output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSchema {
    pub command: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
    #[serde(rename = "outputSchema")]
    pub output_schema: serde_json::Value,
}

impl CommandSchema {
    /// Wrap a data schema inside the standard envelope schema.
    fn wrap_in_envelope_schema(data_schema: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["ok", "type", "schemaVersion"],
            "properties": {
                "ok": { "type": "boolean" },
                "type": { "type": "string" },
                "schemaVersion": { "type": "integer", "const": 1 },
                "data": data_schema,
                "error": {
                    "type": "object",
                    "required": ["code", "message", "isRetryable"],
                    "properties": {
                        "code": { "type": "string" },
                        "message": { "type": "string" },
                        "isRetryable": { "type": "boolean" },
                        "details": { "type": "object" }
                    }
                },
                "meta": {
                    "type": "object",
                    "properties": {
                        "traceId": { "type": "string" }
                    }
                }
            }
        })
    }

    /// Common output schema shared by timeline commands.
    fn timeline_output_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["tweets"],
            "properties": {
                "tweets": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["id"],
                        "properties": {
                            "id": { "type": "string" },
                            "text": { "type": "string" },
                            "author_id": { "type": "string" },
                            "created_at": { "type": "string", "format": "date-time" }
                        }
                    }
                },
                "meta": {
                    "type": "object",
                    "properties": {
                        "pagination": {
                            "type": "object",
                            "properties": {
                                "next_token": { "type": "string" },
                                "previous_token": { "type": "string" }
                            }
                        }
                    }
                }
            }
        })
    }

    /// Build the [`CommandSchema`] for the given command name.
    pub fn for_command(command: &str) -> Self {
        match command {
            "commands" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["commands"],
                    "properties": {
                        "commands": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["name", "description", "arguments", "risk", "hasCost"],
                                "properties": {
                                    "name": { "type": "string" },
                                    "description": { "type": "string" },
                                    "arguments": {
                                        "type": "array",
                                        "items": {
                                            "type": "object",
                                            "required": ["name", "description", "required", "type"],
                                            "properties": {
                                                "name": { "type": "string" },
                                                "description": { "type": "string" },
                                                "required": { "type": "boolean" },
                                                "type": { "type": "string" },
                                                "default": { "type": "string" }
                                            }
                                        }
                                    },
                                    "risk": {
                                        "type": "string",
                                        "enum": ["safe", "low", "medium", "high"]
                                    },
                                    "hasCost": { "type": "boolean" }
                                }
                            }
                        }
                    }
                })),
            },
            "schema" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["command"],
                    "properties": {
                        "command": { "type": "string" }
                    }
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["command", "inputSchema", "outputSchema"],
                    "properties": {
                        "command": { "type": "string" },
                        "inputSchema": { "type": "object" },
                        "outputSchema": { "type": "object" }
                    }
                })),
            },
            "help" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["command"],
                    "properties": {
                        "command": { "type": "string" }
                    }
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["command", "description", "usage", "exitCodes", "errorVocabulary", "examples"],
                    "properties": {
                        "command": { "type": "string" },
                        "description": { "type": "string" },
                        "usage": { "type": "string" },
                        "exitCodes": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["code", "description"],
                                "properties": {
                                    "code": { "type": "integer" },
                                    "description": { "type": "string" }
                                }
                            }
                        },
                        "errorVocabulary": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["code", "description", "isRetryable"],
                                "properties": {
                                    "code": { "type": "string" },
                                    "description": { "type": "string" },
                                    "isRetryable": { "type": "boolean" }
                                }
                            }
                        },
                        "examples": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["description", "command"],
                                "properties": {
                                    "description": { "type": "string" },
                                    "command": { "type": "string" }
                                }
                            }
                        }
                    }
                })),
            },
            "demo-interactive" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["message", "confirmed"],
                    "properties": {
                        "message": { "type": "string" },
                        "confirmed": { "type": "boolean" }
                    }
                })),
            },
            "install-skills" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "skill": { "type": "string" },
                        "agent": { "type": "string", "enum": ["claude", "opencode"] },
                        "global": { "type": "boolean", "default": false },
                        "yes": { "type": "boolean", "default": false }
                    },
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["installed_skills"],
                    "properties": {
                        "installed_skills": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["name", "success", "canonical_path", "target_paths"],
                                "properties": {
                                    "name": { "type": "string" },
                                    "success": { "type": "boolean" },
                                    "canonical_path": { "type": "string" },
                                    "target_paths": {
                                        "type": "array",
                                        "items": { "type": "string" }
                                    },
                                    "error": { "type": "string" },
                                    "used_symlink": { "type": "boolean" }
                                }
                            }
                        }
                    }
                })),
            },
            "search recent" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "query": { "type": "string" },
                        "limit": { "type": "integer", "minimum": 1, "maximum": 100 },
                        "cursor": { "type": "string" }
                    },
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["tweets"],
                    "properties": {
                        "tweets": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["id"],
                                "properties": {
                                    "id": { "type": "string" },
                                    "text": { "type": "string" },
                                    "author_id": { "type": "string" },
                                    "created_at": { "type": "string" }
                                }
                            }
                        },
                        "meta": {
                            "type": "object",
                            "properties": {
                                "pagination": {
                                    "type": "object",
                                    "required": ["result_count"],
                                    "properties": {
                                        "next_token": { "type": "string" },
                                        "prev_token": { "type": "string" },
                                        "result_count": { "type": "integer" }
                                    }
                                }
                            }
                        }
                    }
                })),
            },
            "search users" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "query": { "type": "string" },
                        "limit": { "type": "integer", "minimum": 1, "maximum": 100 },
                        "cursor": { "type": "string" }
                    },
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["users"],
                    "properties": {
                        "users": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["id"],
                                "properties": {
                                    "id": { "type": "string" },
                                    "name": { "type": "string" },
                                    "username": { "type": "string" },
                                    "description": { "type": "string" }
                                }
                            }
                        },
                        "meta": {
                            "type": "object",
                            "properties": {
                                "pagination": {
                                    "type": "object",
                                    "required": ["result_count"],
                                    "properties": {
                                        "next_token": { "type": "string" },
                                        "prev_token": { "type": "string" },
                                        "result_count": { "type": "integer" }
                                    }
                                }
                            }
                        }
                    }
                })),
            },
            "tweets reply" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["tweet_id", "text"],
                    "properties": {
                        "tweet_id": { "type": "string" },
                        "text": { "type": "string" },
                        "client_request_id": { "type": "string" },
                        "if_exists": {
                            "type": "string",
                            "enum": ["return", "error"],
                            "default": "return"
                        }
                    },
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["tweet", "meta"],
                    "properties": {
                        "tweet": {
                            "type": "object",
                            "required": ["id"],
                            "properties": {
                                "id": { "type": "string" },
                                "text": { "type": "string" },
                                "author_id": { "type": "string" },
                                "created_at": { "type": "string" },
                                "conversation_id": { "type": "string" },
                                "referenced_tweets": { "type": "array" }
                            }
                        },
                        "meta": {
                            "type": "object",
                            "required": ["clientRequestId"],
                            "properties": {
                                "clientRequestId": { "type": "string" },
                                "fromCache": { "type": "boolean" }
                            }
                        }
                    }
                })),
            },
            "tweets thread" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["texts"],
                    "properties": {
                        "texts": {
                            "type": "array",
                            "items": { "type": "string" },
                            "minItems": 1
                        },
                        "client_request_id_prefix": { "type": "string" },
                        "if_exists": {
                            "type": "string",
                            "enum": ["return", "error"],
                            "default": "return"
                        }
                    },
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["tweets", "meta"],
                    "properties": {
                        "tweets": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["id"],
                                "properties": {
                                    "id": { "type": "string" },
                                    "text": { "type": "string" }
                                }
                            }
                        },
                        "meta": {
                            "type": "object",
                            "required": ["count", "createdTweetIds"],
                            "properties": {
                                "count": { "type": "integer" },
                                "failedIndex": { "type": "integer" },
                                "createdTweetIds": {
                                    "type": "array",
                                    "items": { "type": "string" }
                                },
                                "fromCache": { "type": "boolean" }
                            }
                        }
                    }
                })),
            },
            "tweets show" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["tweet_id"],
                    "properties": {
                        "tweet_id": { "type": "string" }
                    },
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["tweet"],
                    "properties": {
                        "tweet": {
                            "type": "object",
                            "required": ["id"],
                            "properties": {
                                "id": { "type": "string" },
                                "text": { "type": "string" },
                                "author_id": { "type": "string" },
                                "created_at": { "type": "string" },
                                "conversation_id": { "type": "string" },
                                "referenced_tweets": { "type": "array" }
                            }
                        }
                    }
                })),
            },
            "tweets conversation" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["tweet_id"],
                    "properties": {
                        "tweet_id": { "type": "string" }
                    },
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["conversation_id", "posts", "edges"],
                    "properties": {
                        "conversation_id": {
                            "type": "string",
                            "description": "The conversation_id that identifies this conversation thread"
                        },
                        "posts": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["id"],
                                "properties": {
                                    "id": { "type": "string" },
                                    "text": { "type": "string" },
                                    "conversation_id": { "type": "string" },
                                    "referenced_tweets": { "type": "array" }
                                }
                            }
                        },
                        "edges": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["parent_id", "child_id"],
                                "properties": {
                                    "parent_id": { "type": "string" },
                                    "child_id": { "type": "string" }
                                }
                            }
                        }
                    }
                })),
            },
            "timeline.home" | "timeline.mentions" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 100,
                            "default": 10
                        },
                        "cursor": { "type": "string" }
                    },
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(Self::timeline_output_schema()),
            },
            "timeline.user" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["handle"],
                    "properties": {
                        "handle": { "type": "string" },
                        "limit": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 100,
                            "default": 10
                        },
                        "cursor": { "type": "string" }
                    },
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(Self::timeline_output_schema()),
            },
            "media.upload" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["path"],
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the media file to upload"
                        }
                    },
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["media_id"],
                    "properties": {
                        "media_id": {
                            "type": "string",
                            "description": "The media ID returned by the X API"
                        }
                    }
                })),
            },
            "tweets like" | "tweets unlike" | "tweets retweet" | "tweets unretweet" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["tweet_id"],
                    "properties": {
                        "tweet_id": { "type": "string", "description": "Tweet ID" }
                    },
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["tweet_id", "success"],
                    "properties": {
                        "tweet_id": { "type": "string" },
                        "success": { "type": "boolean" }
                    }
                })),
            },
            "bookmarks add" | "bookmarks remove" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["tweet_id"],
                    "properties": {
                        "tweet_id": { "type": "string", "description": "Tweet ID" }
                    },
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["tweet_id", "success"],
                    "properties": {
                        "tweet_id": { "type": "string" },
                        "success": { "type": "boolean" }
                    }
                })),
            },
            "bookmarks list" => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer", "default": 10 },
                        "cursor": { "type": "string" }
                    },
                    "additionalProperties": false
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "required": ["tweets"],
                    "properties": {
                        "tweets": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["id"],
                                "properties": {
                                    "id": { "type": "string" },
                                    "text": { "type": "string" },
                                    "author_id": { "type": "string" },
                                    "created_at": { "type": "string" }
                                }
                            }
                        },
                        "meta": {
                            "type": "object",
                            "properties": {
                                "pagination": {
                                    "type": "object",
                                    "properties": {
                                        "next_token": { "type": "string" }
                                    }
                                }
                            }
                        }
                    }
                })),
            },
            _ => Self {
                command: command.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
                output_schema: Self::wrap_in_envelope_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            },
        }
    }
}
