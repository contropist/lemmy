use crate::{newtypes::LocalUserId, sensitive::SensitiveString};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::login_token;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Stores data related to a specific user login session.
#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = login_token))]
#[cfg_attr(feature = "full", diesel(primary_key(token)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct LoginToken {
  /// Jwt token for this login
  #[serde(skip)]
  pub token: SensitiveString,
  pub user_id: LocalUserId,
  /// Time of login
  pub published_at: DateTime<Utc>,
  /// IP address where login was made from, allows invalidating logins by IP address.
  /// Could be stored in truncated format, or store derived information for better privacy.
  pub ip: Option<String>,
  pub user_agent: Option<String>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = login_token))]
pub struct LoginTokenCreateForm {
  pub token: SensitiveString,
  pub user_id: LocalUserId,
  pub ip: Option<String>,
  pub user_agent: Option<String>,
}
