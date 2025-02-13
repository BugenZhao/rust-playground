use tokio_postgres::NoTls;

const SQL: &'static str = r#"
drop table if exists mv0 cascade;
create table mv0 (v int);
create MATERIALIZED view mv1 as select * from mv0;
create MATERIALIZED view mv2 as select * from mv1;
create MATERIALIZED view mv3 as select * from mv2;
create MATERIALIZED view mv4 as select * from mv3;
create MATERIALIZED view mv5 as select * from mv4;
create MATERIALIZED view mv6 as select * from mv5;
create MATERIALIZED view mv7 as select * from mv6;
create MATERIALIZED view mv8 as select * from mv7;
create MATERIALIZED view mv9 as select * from mv8;
create MATERIALIZED view mv10 as select * from mv9;
create MATERIALIZED view mv11 as select * from mv10;
create MATERIALIZED view mv12 as select * from mv11;

"#;

#[tokio::main]
async fn main() {
    let mut config = tokio_postgres::config::Config::new();
    config
        .host("localhost")
        .port(4566)
        .user("root")
        .dbname("dev");

    let mut clients = Vec::new();

    for _ in 0..4 {
        let (client, conn) = config.connect(NoTls).await.unwrap();
        tokio::spawn(async move {
            conn.await.unwrap();
        });
        clients.push(client);
    }

    let clients = clients.iter().cycle();

    for (sql, client) in SQL.lines().zip(clients) {
        println!("RUN: {}", sql);
        let result = client.simple_query(sql).await.unwrap();
        println!("RESULT: {:?}", result);
    }
}
