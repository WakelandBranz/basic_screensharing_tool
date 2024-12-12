use webhook::client::{WebhookClient, WebhookResult};
use webhook::models::NonLinkButtonStyle;

use std::any::Any;

use std::fmt;
use crate::core::{
    handle::{
        handle_context::HandleContext,
        handle_manager::HandleManager,
        SystemHandleType,
    },
    overlay::{
        overlay_finder::OverlayFinder,
        window_info::WindowInfo,
    },
    process::{
        Process
    },
    uploading,
};
use anyhow::Result;
use dotenvy_macro::dotenv;
use crate::core::uploading::upload_string_to_tmpfile;

pub struct Anticheat<'a> {
    process: Process,
    handle_manager: HandleManager,
    overlay_finder: OverlayFinder,
    handle_detections: usize,
    overlay_detections: usize,
    past_processes: usize, // TODO! Implement this later!
    pub webhook_url: &'a str
}

impl Anticheat {
    pub fn new(process: Process) -> Self {
        Self {
            process,
            handle_manager: HandleManager::new().expect("Failed to initialize handle manager!"),
            overlay_finder: OverlayFinder::new(),
            handle_detections: 0,
            overlay_detections: 0,
            past_processes: 0,
            webhook_url: "No webhook url parsed!",
        }
    }

    /// Updates fields within struct after proper filtering has been completed
    pub fn run(&mut self) -> anyhow::Result<()> {
        // Run handle scanning
        log::debug!("Filtering possibly malicious handles for process: {} (PID: {})", self.process.name, self.process.pid);
        // Filters handles and assigns amount of detections
        self.handle_detections = self.handle_manager
            .filter_by_handle_type(SystemHandleType::Process)
            .filter_suspicious_handles()
            .filter_anticheat_handles()
            .filter_handles_to_target(self.process.pid)?
            .collect_handle_info()?
            .get_handles()
            .len();

        log::debug!("Done ({} handles)! Handles for process {}...", self.handle_manager.handles.len(), self.process.name);

        // Run overlay scanning
        let overlays = &mut self.overlay_finder.find();
        self.overlay_detections = overlays.len();

        log::debug!("Found {} suspicious overlays (not all suspicious overlays are malicious!)", self.overlay_detections);
        Ok(())
    }

    pub fn has_detections(&self) -> bool {
        self.handle_detections > 0 || self.overlay_detections > 0
    }

    /// Update
    pub fn parse_webhook_url(&mut self) {
        let url = dotenv!("WEBHOOK_URL");
        self.webhook_url = url;
    }

    pub async fn send_webhook(&self) -> WebhookResult<bool> {
        let client = WebhookClient::new(self.webhook_url);

        let (description, color) = if self.has_detections() {
            ("Found suspicious activity", "red")
        }
        else {
            ("Did not find suspicious activity.", "green")
        };

        let all_scan_results_url = upload_string_to_tmpfile(
            format!("{}", self),
            &format!("temp_scan_results_{}.txt", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs())
        ).await.expect("Failed to upload file!");

        log::debug!("Sending webhook...");

        client.send(|message| message
            .username("Anticheat Bot")
            .embed(|embed| embed
                .title("Scan results")
                .description(description)
                .color(color) // Gotta figure this out eventually
                .footer("Made by wakeland", None)
                .field("All scan results", all_scan_results_url.as_str(), false)
            )).await
    }

    // MUTABLE GETTERS FOR BUILDING ----------------------------------------------------------------
    /// Builder for handle manager
    pub fn handle_manager_mut(&mut self) -> &mut HandleManager { &mut self.handle_manager }

    /// Builder for overlay finder
    pub fn overlay_finder_mut(&mut self) -> &mut OverlayFinder { &mut self.overlay_finder }

    // GETTERS -------------------------------------------------------------------------------------
    pub fn process(&self) -> &Process { &self.process }
    pub fn handle_manager(&self) -> &HandleManager { &self.handle_manager }
    pub fn handles(&self) -> &Vec<HandleContext> { &self.handle_manager.handles }
    pub fn handle_detections(&self) -> usize { self.handle_detections }
    pub fn overlay_finder(&self) -> &OverlayFinder { &self.overlay_finder }
    pub fn overlays(&self) -> &Vec<WindowInfo> { &self.overlay_finder.overlays }
    pub fn overlay_detections(&self) -> usize { self.overlay_detections }
}

impl fmt::Display for Anticheat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.has_detections() {
            return writeln!(f, "No suspicious handles or overlays detected.");
        }

        writeln!(f, "{} suspicious handles found.", self.handle_detections)?;
        if self.handle_detections > 0 {
            for handle in &self.handle_manager.handles.clone() {
                writeln!(f, "{}", handle)?;
            }
        }

        writeln!(f, "{} suspicious overlays found.", self.overlay_detections)?;
        if self.overlay_detections > 0 {
            for overlay in self.overlay_finder.overlays.clone() {
                writeln!(f, "{}", overlay)?;
            }
        }

        Ok(())
    }
}