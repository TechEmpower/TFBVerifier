use crate::database::DatabaseInterface;
use crate::verification::Messages;
use mongodb::bson::Bson;
use mongodb::bson::Document;
use mongodb::sync::Client;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug)]
pub struct Mongodb {}
impl Mongodb {
    fn get_client(&self) -> mongodb::error::Result<Client> {
        Client::with_uri_str("mongodb://tfb-database")
    }
}
impl DatabaseInterface for Mongodb {
    fn wait_for_database_to_be_available(&self) {
        let mut messages = Messages::default();
        let max = 60;
        let mut slept = 0;
        while slept < max {
            if self.get_client().is_ok() {
                return;
            }

            sleep(Duration::from_secs(1));
            slept += 1;
        }
        messages.error(
            format!(
                "Database connection could not be established after {} seconds.",
                max
            ),
            "Database unavailable",
        );
    }

    fn get_all_from_world_table(&self) -> HashMap<i32, i32> {
        let mut to_ret: HashMap<i32, i32> = HashMap::default();
        if let Ok(client) = self.get_client() {
            let database = client.database("hello_world");

            if let Ok(cursor) = database.collection("world").find(None, None) {
                for item in cursor {
                    if let Ok(world) = item {
                        if let Some(id) = world.get("id").and_then(Bson::as_i32) {
                            if let Some(random_number) =
                                world.get("randomNumber").and_then(Bson::as_i32)
                            {
                                to_ret.insert(id, random_number);
                            }
                        }
                    }
                }
            }
        }
        to_ret
    }

    fn insert_one_thousand_fortunes(&self) {
        if let Ok(client) = self.get_client() {
            let database = client.database("hello_world");
            for i in 0..1_000 {
                let mut document = Document::new();
                document.insert("id", i + 13);
                document.insert("message", "フレームワークのベンチマーク");
                database
                    .collection("fortune")
                    .insert_one(document, None)
                    .unwrap();
            }
        }
    }

    fn get_count_of_all_queries_for_table(&self, _table_name: &str) -> u32 {
        if let Ok(client) = self.get_client() {
            let database = client.database("hello_world");
            let mut command = Document::new();
            command.insert("serverStatus", 1);
            if let Ok(bson_doc) = database.run_command(command, None) {
                if let Ok(opcounters) = bson_doc.get_document("opcounters") {
                    let mut sum = 0;
                    if let Ok(update) = opcounters.get_i64("update") {
                        sum += update as u32;
                    }
                    if let Ok(query) = opcounters.get_i64("query") {
                        sum += query as u32;
                    }
                    return sum;
                }
            }
        }

        0
    }

    fn get_count_of_rows_selected_for_table(
        &self,
        _table_name: &str,
        expected_rows_per_query: u32,
    ) -> u32 {
        if let Ok(client) = self.get_client() {
            let database = client.database("hello_world");
            let mut command = Document::new();
            command.insert("serverStatus", 1);
            if let Ok(bson_doc) = database.run_command(command, None) {
                if let Ok(op_counters) = bson_doc.get_document("opcounters") {
                    let mut sum = 0;
                    if let Ok(query) = op_counters.get_i64("query") {
                        sum += query as u32;
                    }
                    return sum * expected_rows_per_query;
                }
            }
        }

        0
    }

    fn get_count_of_rows_updated_for_table(
        &self,
        _table_name: &str,
        expected_rows_per_query: u32,
    ) -> u32 {
        if let Ok(client) = self.get_client() {
            let database = client.database("hello_world");
            let mut command = Document::new();
            command.insert("serverStatus", 1);
            if let Ok(bson_doc) = database.run_command(command, None) {
                if let Ok(op_counters) = bson_doc.get_document("opcounters") {
                    let mut sum = 0;
                    if let Ok(update) = op_counters.get_i64("update") {
                        sum += update as u32;
                    }
                    return sum * expected_rows_per_query;
                }
            }
        }

        0
    }
}
