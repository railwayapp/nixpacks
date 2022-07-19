use diesel::{mysql::MysqlConnection, Connection};

fn main() {
    MysqlConnection::establish("mysql://mysql:mysql@127.0.0.1:3306")
        .expect("Error connecting to database");
    println!("Hello from rust")
}
