use log::Level;

pub fn initialize_env_logger() {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "info");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            use chrono::Local;
            // use env_logger::fmt::style::{AnsiColor, Style};
            use std::io::Write;

            // let subtle = Style::new().fg_color(Some(AnsiColor::BrightBlack.into()));
            let level_style = buf.default_level_style(record.level());

            // Check environment variable to see if time should be included
            let show_time =
                std::env::var("LOG_SHOW_TIME").unwrap_or_else(|_| "false".into()) == "true";

            if show_time {
                if record.level() == Level::Info || record.level() == Level::Warn {
                    writeln!(
                        buf,
                        "[{level_style}{:<4}{level_style:#}] [{}] [{}] {}",
                        record.level(),
                        record.target(),
                        Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
                        record.args()
                    )
                } else {
                    writeln!(
                        buf,
                        "[{level_style}{:<5}{level_style:#}][{}] [{}] {}",
                        record.level(),
                        record.target(),
                        Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
                        record.args()
                    )
                }
            } else {
                if record.level() == Level::Info || record.level() == Level::Warn {
                    writeln!(
                        buf,
                        "[{level_style}{:<4}{level_style:#}] [{}] {}",
                        record.level(),
                        record.target(),
                        record.args()
                    )
                } else {
                    writeln!(
                        buf,
                        "[{level_style}{:<5}{level_style:#}][{}] {}",
                        record.level(),
                        record.target(),
                        record.args()
                    )
                }
            }
        })
        .init();
}
