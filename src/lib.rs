pub mod core;

#[cfg(test)]
mod tests {
    use std::future::poll_fn;
    use sysinfo::get_current_pid;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowThreadProcessId,
        WS_EX_LAYERED,
        WS_EX_TRANSPARENT,
        WS_VISIBLE
    };
    use crate::core::{
        overlay::{
            find_overlays,
            overlay_finder::OverlayFinder,
            overlay_finder_params::OverlayFinderParams,
            debug_print_overlays,
            window_info::WindowInfo
        },
        handle::{
            handle_manager::HandleManager,
            handle_info::HandleInfo,
            SystemHandleType
        },
        process::{
            Process,
        }
    };
    use crate::core::anticheat::Anticheat;
    use dotenvy::dotenv;
    use std::env;

    // This is a completely impractical test
    // Scans for possibly suspicious activity for assault cube.
    #[test]
    fn it_works() -> anyhow::Result<()> {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .format_target(false)
            .format_timestamp_secs()
            .init();

        let process = Process::new("ac_client.exe");

        log::info!("Starting checks...");

        let mut anticheat = Anticheat::new(process.clone());
        // Set overlay finder params
        anticheat.overlay_finder_mut()
            .with_style(WS_VISIBLE.0)
            .with_style_ex(WS_EX_LAYERED.0 | WS_EX_TRANSPARENT.0)
            .with_percent_main_screen(80.0)
            .satisfy_all_criteria(true);

        anticheat.run()?;
        log::info!("--- Anticheat scan results ---\n{}", anticheat);
        log::info!("Completed all checks!");

        Ok(())
    }
}
