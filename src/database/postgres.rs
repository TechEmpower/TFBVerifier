use crate::database::DatabaseInterface;
use postgres::{Client, NoTls};
use std::collections::HashMap;

#[derive(Debug)]
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

    fn run_counting_query(&self, query: &str, output_column_name: &str) -> u32 {
        if let Some(mut client) = self.get_client() {
            if let Ok(rows) = client.query(&*query, &[]) {
                if let Some(row) = rows.get(0) {
                    let sum: i64 = row.get(output_column_name);
                    return sum as u32;
                }
            }
        }

        0
    }
}
impl DatabaseInterface for Postgres {
    fn get_all_from_world_table(&self) -> HashMap<i32, i32> {
        let mut to_ret = HashMap::new();
        if let Some(mut client) = self.get_client() {
            if let Ok(rows) = client.query("SELECT * FROM world", &[]) {
                for row in rows {
                    to_ret.insert(row.get("id"), row.get("randomnumber"));
                }
            }
        }

        to_ret
    }

    fn insert_one_thousand_fortunes(&self) {
        if let Some(mut client) = self.get_client() {
            let mut update = String::new();
            for i in 0..1_000 {
                update.push_str(&format!(
                    "INSERT INTO fortune(id,message) VALUES ({},'フレームワークのベンチマーク');",
                    i + 13
                ));
            }
            client.batch_execute(update.as_str()).unwrap();
        }
    }

    fn get_count_of_all_queries_for_table(&self, table_name: &str) -> u32 {
        let query = format!(
            "SELECT SUM(calls::INTEGER) FROM pg_stat_statements WHERE query ~* '[[:<:]]{}[[:>:]]'",
            table_name
        );

        self.run_counting_query(&query, "sum")
    }

    fn get_count_of_rows_selected_for_table(
        &self,
        table_name: &str,
        _expected_rows_per_query: u32,
    ) -> u32 {
        let query = format!("SELECT SUM(rows::INTEGER) FROM pg_stat_statements WHERE query ~* '[[:<:]]{}[[:>:]]' AND query ~* 'select'", table_name);

        self.run_counting_query(&query, "sum")
    }

    fn get_count_of_rows_updated_for_table(
        &self,
        table_name: &str,
        _expected_rows_per_query: u32,
    ) -> u32 {
        let query = format!("SELECT SUM(rows::INTEGER) FROM pg_stat_statements WHERE query ~* '[[:<:]]{}[[:>:]]' AND query ~* 'update'", table_name);

        self.run_counting_query(&query, "sum")
    }
}
