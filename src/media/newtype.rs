use std::ops::Deref;

use sqlx::Decode;
use sqlx::Encode;
use sqlx::Sqlite;
use sqlx::Type;

#[derive(sqlx::Type, Debug)]
#[sqlx(transparent)]
pub struct Tag(pub String);

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Duration(pub time::Duration);

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct UtcDateTime(pub time::UtcDateTime);

impl UtcDateTime {
    pub fn now() -> Self {
        Self(time::UtcDateTime::now())
    }
}

impl Deref for Duration {
    type Target = time::Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for UtcDateTime {
    type Target = time::UtcDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl sqlx::Type<Sqlite> for Duration {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <i64 as Type<Sqlite>>::type_info()
    }
}

impl<'r> sqlx::Encode<'r, Sqlite> for Duration {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i64 as Encode<'r, Sqlite>>::encode_by_ref(&self.whole_seconds(), buf)
    }
}

impl<'r> sqlx::Decode<'r, Sqlite> for Duration {
    fn decode(
        value: <Sqlite as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <i64 as Decode<'r, Sqlite>>::decode(value)?;
        Ok(Self(time::Duration::seconds(s)))
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
