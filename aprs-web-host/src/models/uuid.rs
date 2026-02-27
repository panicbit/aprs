use std::ops::Deref;
use std::str::FromStr;

use color_eyre::eyre;
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::expression::AsExpression;
use diesel::serialize::{IsNull, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use uuid::Uuid;

#[derive(Copy, Clone, Debug, AsExpression)]
#[diesel(sql_type = Text)]
pub struct UUID(Uuid);

impl UUID {
    pub fn new_random() -> Self {
        Self(Uuid::new_v4())
    }
}

impl FromStr for UUID {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = s.parse::<Uuid>()?;

        Ok(Self(uuid))
    }
}

impl Deref for UUID {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromSql<Text, Sqlite> for UUID {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let uuid = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        let uuid = Uuid::parse_str(&uuid)?;

        Ok(UUID(uuid))
    }
}

impl ToSql<Text, Sqlite> for UUID
where
    str: ToSql<Text, Sqlite>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        let uuid = self.0.as_hyphenated().to_string();
        out.set_value(uuid);

        Ok(IsNull::No)
    }
}
