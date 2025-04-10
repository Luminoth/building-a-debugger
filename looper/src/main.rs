fn main() -> anyhow::Result<()> {
    println!("My PID is {}", std::process::id());

    let duration = std::time::Duration::from_secs(5);
    loop {
        println!("Sleeping ...");
        std::thread::sleep(duration);
    }
}
