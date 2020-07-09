use crate::database::DatabaseVerifier;

pub struct Mongodb {}
impl DatabaseVerifier for Mongodb {
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
