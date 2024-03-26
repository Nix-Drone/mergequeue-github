#[cfg(test)]
mod tests {
    use confique::Config;
    use gen::config::Conf;
    use rand::Rng;
    use std::thread;

    #[test]
    fn check() {
        let config = Conf::builder()
            .env()
            .file("demo.toml")
            .file(".config/demo.toml")
            .load()
            .unwrap_or_else(|err| {
                eprintln!("test cannot run: {}", err);
                std::process::exit(1);
            });

        println!("sleeping for {} seconds", config.sleep_duration().as_secs());
        thread::sleep(config.sleep_duration());

        let mut rng = rand::thread_rng();
        let random_float = rng.gen_range(0.0..1.0);

        println!("Random float: {}", random_float);
        println!("Flake rate: {}", config.flake_rate);

        assert!(random_float > config.flake_rate);
    }
}
