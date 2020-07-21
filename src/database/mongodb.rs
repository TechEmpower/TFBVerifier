use crate::database::DatabaseInterface;
use mongodb::bson::Bson;
use mongodb::bson::Document;
use mongodb::sync::Client;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Mongodb {}
impl Mongodb {
    fn get_client(&self) -> Client {
        Client::with_uri_str("mongodb://tfb-database").unwrap()
    }
}
impl DatabaseInterface for Mongodb {
    fn get_all_from_world_table(&self) -> HashMap<i32, i32> {
        let mut to_ret: HashMap<i32, i32> = HashMap::default();
        let database = self.get_client().database("hello_world");

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
        to_ret
    }

    fn insert_one_thousand_fortunes(&self) {
        let client = self.get_client();

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

    fn get_count_of_all_queries_for_table(&self, _table_name: &str) -> i64 {
        let client = self.get_client();

        let database = client.database("hello_world");
        let mut command = Document::new();
        command.insert("serverStatus", 1);
        if let Ok(bson_doc) = database.run_command(command, None) {
            if let Ok(opcounters) = bson_doc.get_document("opcounters") {
                let mut sum = 0;
                if let Ok(update) = opcounters.get_i64("update") {
                    sum += update;
                }
                if let Ok(query) = opcounters.get_i64("query") {
                    sum += query;
                }
                return sum;
            }
        }

        0
    }

    fn get_count_of_rows_selected_for_table(&self, table_name: &str) -> i64 {
        let client = self.get_client();
        // todo - this tightly couples this database type with a verifier; fix
        let rows_per_query = if table_name == "fortune" { 12 } else { 1 };

        let database = client.database("hello_world");
        let mut command = Document::new();
        command.insert("serverStatus", 1);
        if let Ok(bson_doc) = database.run_command(command, None) {
            if let Ok(opcounters) = bson_doc.get_document("opcounters") {
                let mut sum = 0;
                if let Ok(query) = opcounters.get_i64("query") {
                    sum += query;
                }
                return sum * rows_per_query;
            }
        }

        0
    }

    fn get_count_of_rows_updated_for_table(&self, table_name: &str) -> i64 {
        let client = self.get_client();
        // todo - this tightly couples this database type with a verifier; fix
        let rows_per_query = if table_name == "fortune" { 12 } else { 1 };

        let database = client.database("hello_world");
        let mut command = Document::new();
        command.insert("serverStatus", 1);
        if let Ok(bson_doc) = database.run_command(command, None) {
            if let Ok(opcounters) = bson_doc.get_document("opcounters") {
                let mut sum = 0;
                if let Ok(update) = opcounters.get_i64("update") {
                    sum += update;
                }
                return sum * rows_per_query;
            }
        }

        0
    }
}
