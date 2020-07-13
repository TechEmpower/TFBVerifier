use crate::database::DatabaseInterface;
use std::collections::HashMap;

pub struct Mysql {}
impl DatabaseInterface for Mysql {
    fn get_all_from_world_table(&self) -> HashMap<i32, i32> {
        HashMap::default()
    }

    fn insert_one_thousand_fortunes(&self) {}

    fn get_count_of_all_queries_for_table(&self, _table_name: &str) -> i64 {
        0
    }

    fn get_count_of_rows_selected_for_table(&self, _table_name: &str) -> i64 {
        0
    }

    fn get_count_of_rows_updated_for_table(&self, _table_name: &str) -> i64 {
        0
    }
}
