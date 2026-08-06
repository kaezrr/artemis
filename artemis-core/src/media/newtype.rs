use std::ops::Deref;

use sqlx::Decode;
use sqlx::Encode;
use sqlx::Sqlite;
use sqlx::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct UtcDateTime(pub time::UtcDateTime);

impl UtcDateTime {
    #[must_use]
    pub fn now() -> Self {
        Self(time::UtcDateTime::now())
    }

    /// Returns the `UtcDateTime` for the given Unix timestamp.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TimeStampConversionError`](crate::Error::TimeStampConversionError) if `timestamp` is out of range.
    pub fn from_unix_timestamp(timestamp: i64) -> crate::Result<Self> {
        Ok(Self(time::UtcDateTime::from_unix_timestamp(timestamp)?))
    }
}

impl Deref for UtcDateTime {
    type Target = time::UtcDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl sqlx::Type<Sqlite> for UtcDateTime {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <i64 as Type<Sqlite>>::type_info()
    }
}

impl<'r> sqlx::Encode<'r, Sqlite> for UtcDateTime {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i64 as Encode<'r, Sqlite>>::encode_by_ref(&self.unix_timestamp(), buf)
    }
}

impl<'r> sqlx::Decode<'r, Sqlite> for UtcDateTime {
    fn decode(
        value: <Sqlite as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <i64 as Decode<'r, Sqlite>>::decode(value)?;
        Ok(Self(time::UtcDateTime::from_unix_timestamp(s)?))
    }
}
