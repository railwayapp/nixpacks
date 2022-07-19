use diesel::mysql::MysqlConnection;
use diesel::pg::PgConnection;
use diesel::Connection;

fn main() {
    MysqlConnection::establish("mysql://mysql:mysql@127.0.0.1:3306")
        .expect("Error connecting to database");
    PgConnection::establish("postgres://mysql:mysql@127.0.0.1:3306")
        .expect("Error connecting to database");
    println!("Hello from rust")
}
