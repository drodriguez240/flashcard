use std::{fmt::Display, num::ParseIntError, ops::Deref, path::Path, str::FromStr};

use sqlite::*;

pub struct Database(Option<Sqlite>);

impl Database {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn open(&mut self, path: impl AsRef<Path>) -> SqliteResult<()> {
        let sqlite = Sqlite::open(path)?;
        migrate(&sqlite);
        self.0 = Some(sqlite);

        Ok(())
    }
}

impl Deref for Database {
    type Target = Sqlite;

    fn deref(&self) -> &Self::Target {
        &self.0.as_ref().unwrap()
    }
}

fn migrate(sqlite: &Sqlite) {
    const CURRENT_VERSION: SqliteId = 1;

    match sqlite.version() {
        CURRENT_VERSION => {}
        0 => {
            sqlite.execute_all(include_str!("schema.sql")).unwrap();
            sqlite.set_version(CURRENT_VERSION);
        }
        _ => todo!(),
    }
}

macro_rules! make_id_struct {
    ($pub:vis $t:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $pub struct $t(SqliteId);

        impl $t {
            pub const ZERO: Self = Self(0);

            pub const fn from_raw(id: SqliteId) -> Self {
                Self(id)
            }

            pub const fn raw(self) -> SqliteId {
                self.0
            }
        }

        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromSql for $t {
            fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                let id = SqliteId::column_result(value)?;
                Ok(Self(id))
            }
        }

        impl ToSql for $t {
            fn to_sql(&self) -> SqliteResult<ToSqlOutput<'_>> {
                self.0.to_sql()
            }
        }

        impl FromStr for $t {
            type Err = ParseIntError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let id = s.parse::<SqliteId>()?;
                Ok(Self(id))
            }
        }
    };
}

make_id_struct!(pub CardId);
make_id_struct!(pub ReviewId);
make_id_struct!(pub TagId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Seahash(u64);

impl Seahash {
    pub const fn from_raw(hash: u64) -> Self {
        Self(hash)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl Display for Seahash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&[u8]> for Seahash {
    fn from(buf: &[u8]) -> Self {
        Self(seahash::hash(buf))
    }
}

impl FromStr for Seahash {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hash = s.parse::<u64>()?;
        Ok(Self(hash))
    }
}

impl FromSql for Seahash {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let bytes: [u8; 8] = value.as_blob()?.try_into().unwrap();
        Ok(Seahash(u64::from_be_bytes(bytes)))
    }
}

impl ToSql for Seahash {
    fn to_sql(&self) -> SqliteResult<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Blob(
            self.0.to_be_bytes().to_vec(),
        )))
    }
}
