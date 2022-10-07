use tabled::Table;

mod timers;
pub use timers::*;

fn main() {
    let languages = vec![
        PostgresTimer::new()
            .add_test_name("integration")
            .postgres_exec_time("1s")
            .rpc_exec_time("1s")
            .build(),
        PostgresTimer::new()
            .add_test_name("rpcpool")
            .postgres_exec_time("1s")
            .rpc_exec_time("1s")
            .build(),
        PostgresTimer::new()
            .add_test_name("mainnet-beta")
            .postgres_exec_time("1s")
            .rpc_exec_time("1s")
            .build(),
    ];

    let table = Table::new(languages)
        .with(tabled::Style::psql())
        .to_string();

    println!("{}", &table);

    let bar = PostgresTimer::new()
        .add_test_name("mainnet-beta")
        .postgres_exec_time("2s")
        .rpc_exec_time("1s")
        .build();

    let elapsed = bar.with_timer(foo);

    println!("took {} ms.", elapsed.as_millis());
}

fn foo() {
    std::thread::sleep(std::time::Duration::from_secs(2));
}
