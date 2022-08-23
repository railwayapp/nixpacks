use std::{env, process::Command};

use diesel::mysql::MysqlConnection;
use diesel::Connection;

fn get_mysql_connection() {
    MysqlConnection::establish("mysql://mysql:mysql@127.0.0.1:3306")
        .expect("Error connecting to database");
}

fn main() {
    get_mysql_connection();
    println!("Hello from rust")
}
