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

#[macro_use]
#[path = "utils/mod.rs"]
pub mod utils;

use sqlparser::ast::*;
use sqlparser::dialect::{GenericDialect, SnowflakeDialect};
use sqlparser::parser::ParserError;
use sqlparser::test_utils::*;
use utils::*;

fn table_alias(alias: &str) -> TableAlias {
    TableAlias {
        name: Ident {
            value: alias.to_owned(),
            quote_style: None,
        },
        columns: Vec::new(),
    }
}

#[test]
fn test_snowflake_create_table() {
    let sql = "CREATE TABLE _my_$table (am00unt number)";
    match snowflake_and_generic().verified_stmt(sql) {
        Statement::CreateTable { name, .. } => {
            assert_eq!("_my_$table", name.to_string());
        }
        _ => unreachable!(),
    }
}

fn get_from_section_from_select_query(query: &str) -> Vec<TableWithJoins> {
    let statement = snowflake().parse_sql_statements(query).unwrap()[0].clone();

    let query = match statement {
        Statement::Query(query) => query,
        _ => panic!("Not a query"),
    };

    let select = match query.body {
        SetExpr::Select(select) => select,
        _ => panic!("not a select query"),
    };

    select.from.clone()
}

#[test]
fn test_sf_derives_single_table_in_parenthesis() {
    let from = get_from_section_from_select_query("SELECT * FROM (((SELECT 1) AS t))");

    assert_eq!(
        from[0].relation,
        TableFactor::Derived {
            lateral: false,
            subquery: Box::new(snowflake().verified_query("SELECT 1")),
            alias: Some(TableAlias {
                name: "t".into(),
                columns: vec![],
            })
        }
    );
}

#[test]
fn test_single_table_in_parenthesis() {
    //Parenthesized table names are non-standard, but supported in Snowflake SQL
    let from = get_from_section_from_select_query("SELECT * FROM (a NATURAL JOIN (b))");

    assert_eq!(from[0].relation, nest!(table("a", None), table("b", None)));

    let from = get_from_section_from_select_query("SELECT * FROM (a NATURAL JOIN ((b)))");
    assert_eq!(from[0].relation, nest!(table("a", None), table("b", None)));
}

#[test]
fn test_single_table_in_parenthesis_with_alias() {
    let sql = "SELECT * FROM (a NATURAL JOIN (b) c )";
    let table_with_joins = get_from_section_from_select_query(sql)[0].clone();
    assert_eq!(
        table_with_joins.relation,
        nest!(table("a", None), table("b", Some(table_alias("c"))))
    );

    let sql = "SELECT * FROM (a NATURAL JOIN ((b)) c )";
    let table_with_joins = get_from_section_from_select_query(sql)[0].clone();
    assert_eq!(
        table_with_joins.relation,
        nest!(table("a", None), table("b", Some(table_alias("c"))))
    );

    let sql = "SELECT * FROM (a NATURAL JOIN ( (b) c ) )";
    let table_with_joins = get_from_section_from_select_query(sql)[0].clone();
    assert_eq!(
        table_with_joins.relation,
        nest!(table("a", None), table("b", Some(table_alias("c"))))
    );

    let sql = "SELECT * FROM (a NATURAL JOIN ( (b) as c ) )";
    let table_with_joins = get_from_section_from_select_query(sql)[0].clone();
    assert_eq!(
        table_with_joins.relation,
        nest!(table("a", None), table("b", Some(table_alias("c"))))
    );

    let sql = "SELECT * FROM (a alias1 NATURAL JOIN ( (b) c ) )";
    let table_with_joins = get_from_section_from_select_query(sql)[0].clone();
    assert_eq!(
        table_with_joins.relation,
        nest!(
            table("a", Some(table_alias("alias1"))),
            table("b", Some(table_alias("c")))
        )
    );

    let sql = "SELECT * FROM (a as alias1 NATURAL JOIN ( (b) as c ) )";
    let table_with_joins = get_from_section_from_select_query(sql)[0].clone();
    assert_eq!(
        table_with_joins.relation,
        nest!(
            table("a", Some(table_alias("alias1"))),
            table("b", Some(table_alias("c")))
        )
    );

    let res = snowflake().parse_sql_statements("SELECT * FROM (a NATURAL JOIN b) c");
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: c".to_string()),
        res.unwrap_err()
    );

    let res = snowflake().parse_sql_statements("SELECT * FROM (a b) c");
    assert_eq!(
        ParserError::ParserError("duplicate alias b".to_string()),
        res.unwrap_err()
    );
}

fn snowflake() -> TestedDialects {
    TestedDialects {
        // we don't have a separate SQLite dialect, so test only the generic dialect for now
        dialects: vec![Box::new(SnowflakeDialect {})],
    }
}


fn snowflake_and_generic() -> TestedDialects {
    TestedDialects {
        // we don't have a separate SQLite dialect, so test only the generic dialect for now
        dialects: vec![Box::new(SnowflakeDialect {}), Box::new(GenericDialect {})],
    }
}
