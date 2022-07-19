use diesel::{mysql::MysqlConnection, Connection};

fn main() -> &str {
    MysqlConnection::establish("mysql://root:root_password@127.0.0.1:3306")
        .expect("Error connecting to database");
    "Hello from rust"
}
