use eframe::egui;
use poll_promise::Promise;
use redis::aio::MultiplexedConnection;
use std::{collections::HashMap, error::Error, sync::Arc, time::Duration};

use crate::{ConnectionManager, SPTransformStamped, TransformsManager};

async fn get_all_transforms(con: Arc<ConnectionManager>) -> HashMap<String, SPTransformStamped> {
    let mut connection = con.get_connection().await;
    match TransformsManager::get_all_transforms(&mut connection).await {
        Ok(tfs) => tfs,
        Err(e) => {
            log::error!("GUI Failed to get all transforms with: {e}!");
            HashMap::new()
        }
    }
}

pub struct MyApp {
    handle: tokio::runtime::Handle,
    // connection: ConnectionManager,
    connection: Arc<ConnectionManager>,
    get_all_transforms_promise: Option<Promise<HashMap<String, SPTransformStamped>>>,
}

impl MyApp {
    pub async fn new(handle: tokio::runtime::Handle) -> Self {
        let connection = Arc::new(ConnectionManager::new().await);
        Self {
            handle,
            connection,
            get_all_transforms_promise: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Async GUI Demo");
            ui.separator();

            if let Some(promise) = &self.get_all_transforms_promise {
                match promise.poll() {
                    std::task::Poll::Ready(result) => {
                        ui.label("Result is ready!");
                        // Collect all keys into a Vec<&str>, then join them into a single String
                        let keys_string = result
                            .keys()
                            .map(|s| s.as_str()) // Convert from &String to &str
                            .collect::<Vec<&str>>() // Collect into a vector of string slices
                            .join("\n"); // Join them with newlines

                        ui.monospace(keys_string);
                        // ui.monospace(result.keys());

                        if ui.button("Fetch again").clicked() {
                            self.get_all_transforms_promise = None;
                        }
                    }
                    std::task::Poll::Pending => {
                        ui.spinner();
                        ui.label("Loading data...");
                    }
                }
            } else {
                if ui.button("Fetch data").clicked() {
                    let handle = self.handle.clone();

                    let con_clone = self.connection.clone();
                    self.get_all_transforms_promise =
                        Some(Promise::spawn_thread("fetcher", move || {
                            handle.block_on(get_all_transforms(con_clone))
                        }));
                }
            }
        });
    }
}
