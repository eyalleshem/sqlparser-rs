// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod ansi;
mod generic;
pub mod keywords;
mod mssql;
mod mysql;
mod postgresql;
mod snowflake;
mod sqlite;

use std::fmt::Debug;

pub use self::ansi::AnsiDialect;
pub use self::generic::GenericDialect;
pub use self::mssql::MsSqlDialect;
pub use self::mysql::MySqlDialect;
pub use self::postgresql::PostgreSqlDialect;
pub use self::snowflake::SnowflakeDialect;
pub use self::sqlite::SQLiteDialect;

pub trait Dialect: Debug {
    /// Determine if a character starts a quoted identifier. The default
    /// implementation, accepting "double quoted" ids is both ANSI-compliant
    /// and appropriate for most dialects (with the notable exception of
    /// MySQL, MS SQL, and sqlite). You can accept one of characters listed
    /// in `Word::matching_end_quote` here
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '"'
    }
    /// Determine if a character is a valid start character for an unquoted identifier
    fn is_identifier_start(&self, ch: char) -> bool;
    /// Determine if a character is a valid unquoted identifier character
    fn is_identifier_part(&self, ch: char) -> bool;

    /// The name of the dialect
    fn dialect_name(&self) -> &'static str;

    /// Enable the parser to implement dialect specific functionality.
    /// The input for this function a list dialect names.
    /// Function will return true if the current dialect is a subset of the input.
    ///
    /// parser usage exmple:
    /// `if self.dialect.is_dialect(vec!["mssql"]) {
    ///    // some special mssql behaviour  
    /// } else {
    ///   // defualt bahviour
    /// }`
    fn is_dialect(&self, dialects: Vec<&str>) -> bool {
        dialects.contains(&self.dialect_name())
    }
}

#[cfg(test)]
mod tests {
    use super::generic::GenericDialect;
    use super::*;

    #[test]
    fn test_is_diaclect() {
        let generic_dailect = GenericDialect {};

        assert_eq!(generic_dailect.is_dialect(vec!["generic"]), true);
        assert_eq!(generic_dailect.is_dialect(vec!["generic", "mssql"]), true);
        assert_eq!(generic_dailect.is_dialect(vec!["mssql"]), false);
        assert_eq!(generic_dailect.is_dialect(vec!["mssql", "mysql"]), false);
    }
}
