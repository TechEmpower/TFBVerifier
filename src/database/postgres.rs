use crate::database::DatabaseVerifier;
use postgres::{Client, NoTls};
use std::collections::HashMap;

pub struct Postgres {}
impl Postgres {
    fn get_client(&self) -> Option<Client> {
        if let Ok(client) = Client::connect(
            "postgresql://benchmarkdbuser:benchmarkdbpass@tfb-database/hello_world",
            NoTls,
        ) {
            Some(client)
        } else {
            None
        }
    }

    fn run_counting_query(&self, query: &str, output_column_name: &str) -> i64 {
        if let Some(mut client) = self.get_client() {
            if let Ok(rows) = client.query(&*query, &[]) {
                if let Some(row) = rows.get(0) {
                    let sum: i64 = row.get(output_column_name);
                    return sum;
                }
            }
        }
        // todo - what do we do on any failure?
        0
    }
}
impl DatabaseVerifier for Postgres {
    fn get_all_from_world_table(&self) -> Option<HashMap<i32, i32>> {
        let mut client = self.get_client().unwrap();

        if let Ok(rows) = client.query("SELECT * FROM world", &[]) {
            let mut to_ret = HashMap::new();
            for row in rows {
                to_ret.insert(row.get("id"), row.get("randomnumber"));
            }
            Some(to_ret)
        } else {
            None
        }
    }

    fn get_count_of_all_queries_for_table(&self, table_name: &str) -> i64 {
        let query = format!(
            "SELECT SUM(calls::INTEGER) FROM pg_stat_statements WHERE query ~* '[[:<:]]{}[[:>:]]'",
            table_name
        );

        self.run_counting_query(&query, "sum")
    }

    fn get_count_of_rows_selected_for_table(&self, table_name: &str) -> i64 {
        let query = format!("SELECT SUM(rows::INTEGER) FROM pg_stat_statements WHERE query ~* '[[:<:]]{}[[:>:]]' AND query ~* 'select'", table_name);

        self.run_counting_query(&query, "sum")
    }

    fn get_count_of_rows_updated_for_table(&self, table_name: &str) -> i64 {
        let query = format!("SELECT SUM(rows::INTEGER) FROM pg_stat_statements WHERE query ~* '[[:<:]]{}[[:>:]]' AND query ~* 'update'", table_name);

        self.run_counting_query(&query, "sum")
    }
}
