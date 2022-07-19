use std::env;

// use diesel::mysql::MysqlConnection;
use diesel::pg::PgConnection;
use diesel::Connection;

fn get_postgress_connection() -> PgConnection {
    let pg_user = env::var("PGUSER").unwrap();
    let pg_password = env::var("PGPASSWORD").unwrap();
    let pg_host = env::var("PGHOST").unwrap();
    let pg_port = env::var("PGPORT").unwrap();
    let pg_database = env::var("PGDATABASE").unwrap();

    let connection_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        pg_user, pg_password, pg_host, pg_port, pg_database
    );

    PgConnection::establish(&connection_url).expect("Error connecting to the postgress database")
}

// fn get_mysql_connection() {
//     MysqlConnection::establish("mysql://mysql:mysql@127.0.0.1:3306")
//         .expect("Error connecting to database")
// }

fn main() {
    // get_mysql_connection();
    // let o = Command::new("find / -name libpq.so")
    //     .output()
    //     .expect("the command to run");
    println!("{:?}", o.stdout);
    get_postgress_connection();

    println!("Hello from rust")
}
