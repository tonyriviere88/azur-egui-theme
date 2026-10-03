//! Which graphics backend an Azur window presents on, and why that is not the
//! application's choice to make.
//!
//! A design system that specifies one-pixel strokes and a seam 6 ΔL\* from the
//! surface it divides has made a claim about pixels, and the claim only holds if
//! the frame reaches the screen the way it was drawn. On Windows that depends on
//! the graphics backend, which is why it is specified here rather than left to
//! each application.
//!
//! # What was measured
//!
//! On an NVIDIA GPU, the Desktop Window Manager's *composed* copy of a window is
//! a vertically resampled version of what the application rendered — on wgpu's
//! OpenGL backend and on its D3D12 backend alike. On Vulkan it is byte-identical.
//!
//! Measured by comparing a frame read back off the GPU against a capture of the
//! same window on screen:
//!
//! | backend | pixels differing | 1px hairline, amplitude 29.99 | edges altered |
//! | --- | --- | --- | --- |
//! | OpenGL | 4.85% | **22.99**, leaking into the row below | 68.5% |
//! | D3D12 | 4.84% | — | — |
//! | **Vulkan** | **0.37%** | **29.99**, intact | **2.8%** |
//!
//! The offset drifts from nothing at the top of the frame to a full row at the
//! bottom, and the residual falls to zero wherever it lands on a whole row: a
//! one-row size mismatch, resampled to fit. It costs a hairline a quarter of its
//! contrast, moves lines by up to a row, and erases the faintest ones outright —
//! which is to say it costs exactly the things this token set is built from. Text
//! goes soft with it.
//!
//! # Why it looks intermittent, and why no screenshot shows it
//!
//! A window that is presenting frames continuously reaches the display on a path
//! that skips composition, and on that path it is correct. Stop presenting — which
//! an on-demand painter does the moment the user stops moving — and the compositor
//! takes it back, resample and all. So the fault appears seconds after the window
//! goes quiet and clears seconds after it is touched.
//!
//! Taking a screen capture *forces* composition, so a capture always lands on the
//! degraded path: a screenshot taken while the screen looks sharp comes out soft,
//! and a series of them all look equally soft, which reads as nothing being wrong.
//! Do not conclude a window is fine from a screen capture. Compare against a frame
//! read back from the GPU.
//!
//! # What it costs
//!
//! Vulkan starts heavier than OpenGL — around 200 MB of private bytes on the
//! window this was measured on, which came out as 181 MB against 398 and 216
//! against 413 in two runs, the spread being what the window had been doing rather
//! than the backend. That is the price of the frame arriving intact, and it is the
//! right way round: a window that renders the design system and then has it
//! resampled away has not rendered it.
//!
//! # The adapter, and the allocator
//!
//! Two more choices come with the backend, both measured on the same window:
//!
//! | | private bytes | working set | threads | frame intact |
//! | --- | --- | --- | --- | --- |
//! | throughput allocator, discrete GPU | 396 MB | 189 MB | 43 | yes |
//! | **memory allocator, discrete GPU** | **216 MB** | **189 MB** | 43 | **yes** |
//! | memory allocator, integrated GPU | 115 MB | 137 MB | 28 | yes |
//!
//! **The allocator is asked to favour memory over throughput.** wgpu's default
//! sub-allocates in large blocks to keep allocation off the hot path; an interface
//! that uploads a few hundred kilobytes of vertices a second gains nothing from
//! that and pays for the blocks. It is worth 180 MB and costs nothing measurable —
//! the plainest choice on this page.
//!
//! **The adapter is the high-performance one**, which is not the cheapest. The
//! integrated GPU is another 100 MB lighter and, on the machine this was measured
//! on, draws an identical frame. It is not the default for two reasons: every
//! correctness measurement here was taken on the discrete GPU, and rendering on the
//! integrated one while the displays hang off the discrete one means each frame
//! crosses adapters to be presented — a path that was lossless here but is a
//! property of how a particular machine is wired, not something a library can
//! promise. `AZUR_ADAPTER=low` takes the saving on a machine where you have checked.
//!
//! # Overriding
//!
//! `AZUR_ADAPTER=low` or `=high` picks the adapter for a session without a rebuild.
//! `AZUR_BACKEND=vulkan` still names the backend, though on Windows it now only
//! names the one already in use.
//!
//! **`=gl` and `=d3d12` are both gone, and deliberately.** `Cargo.toml` compiles
//! exactly one backend per platform, so those two names have nothing left to reach:
//! OpenGL went first, as the one row of the table above that both leaks *and* loses
//! the hairlines, and D3D12 went after it — between them 938,852 bytes of every
//! executable, held for a machine nobody has reported. A name wgpu cannot serve is
//! worse than no name at all: `Backends::DX12` with no D3D12 backend compiled finds
//! no adapter, and the window fails to open with nothing to say why. So both now
//! fall through to the default like any other unknown name, and both say so on the
//! console.
//!
//! What that costs, stated plainly: **a Windows machine with no working Vulkan
//! driver no longer has a fallback.** The window will not open. That is a
//! deliberate trade and not an oversight — the measurement above is why there was
//! only ever one backend worth presenting on, and carrying the other three to hedge
//! against a driver nobody has met was 3.5% of an executable.
//!
//! Neither is read from wgpu's own `WGPU_BACKEND` and `WGPU_POWER_PREF`, which the
//! defaults here deliberately override: what a window presents on should be a
//! property of the program, not of the shell it happened to be started from.

use egui_wgpu::{wgpu, WgpuConfiguration, WgpuSetup};

/// The backends an Azur window may present on, most preferred first — in practice
/// exactly one, so the choice cannot silently fall through to another.
///
/// Reads `AZUR_BACKEND`; see the [module documentation](self) for the measurement
/// behind the default.
pub fn backends() -> wgpu::Backends {
    let chosen = std::env::var("AZUR_BACKEND").ok();
    from_name(chosen.as_deref().filter(|name| !name.is_empty()))
}

/// Which adapter an Azur window renders on.
///
/// Reads `AZUR_ADAPTER`; see the [module documentation](self) for why the default is
/// the dear one.
pub fn adapter() -> wgpu::PowerPreference {
    let chosen = std::env::var("AZUR_ADAPTER").ok();
    preference_from_name(chosen.as_deref().filter(|name| !name.is_empty()))
}

/// Apply the whole of the above to a configuration the caller is otherwise building
/// itself: the backend, the adapter and the allocator.
///
/// For an application with its own opinion about something else in the
/// configuration — the surface status handler, say — set that, then hand the
/// configuration here.
///
/// ```no_run
/// # fn demo() -> egui_wgpu::WgpuConfiguration {
/// let mut configuration = egui_wgpu::WgpuConfiguration::default();
/// azur_egui_theme::render::configure(&mut configuration);
/// configuration
/// # }
/// ```
///
/// Does nothing to a configuration built from an *existing* device
/// ([`WgpuSetup::Existing`]), where all three were decided by whoever created it.
pub fn configure(configuration: &mut WgpuConfiguration) {
    if let WgpuSetup::CreateNew(setup) = &mut configuration.wgpu_setup {
        setup.instance_descriptor.backends = backends();
        setup.power_preference = adapter();
        setup.device_descriptor =
            std::sync::Arc::new(|adapter: &wgpu::Adapter| device(adapter.get_info().backend));
    }
}

/// A configuration carrying the design system's presentation decisions and nothing
/// else of its own.
///
/// The whole of what most applications need:
///
/// ```no_run
/// # fn demo() -> eframe::NativeOptions {
/// eframe::NativeOptions {
///     renderer: eframe::Renderer::Wgpu,
///     wgpu_options: azur_egui_theme::render::wgpu_options(),
///     ..Default::default()
/// }
/// # }
/// ```
pub fn wgpu_options() -> WgpuConfiguration {
    let mut configuration = WgpuConfiguration::default();
    configure(&mut configuration);
    configuration
}

/// The device to ask the adapter for.
///
/// Takes the backend rather than the adapter so it can be tested: a
/// [`wgpu::Adapter`] cannot be built without a live driver.
///
/// The limits match `egui-wgpu`'s own defaults, including the 8192 texture a 4K
/// surface needs for its depth buffer. Only the memory hint differs.
fn device(backend: wgpu::Backend) -> wgpu::DeviceDescriptor<'static> {
    let base = if backend == wgpu::Backend::Gl {
        wgpu::Limits::downlevel_webgl2_defaults()
    } else {
        wgpu::Limits::default()
    };
    wgpu::DeviceDescriptor {
        label: Some("azur"),
        required_limits: wgpu::Limits {
            max_texture_dimension_2d: 8192,
            ..base
        },
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        ..Default::default()
    }
}

/// The adapter preference for a name, or the default for `None`.
fn preference_from_name(chosen: Option<&str>) -> wgpu::PowerPreference {
    match chosen {
        Some("low") | Some("integrated") => wgpu::PowerPreference::LowPower,
        Some("high") | Some("discrete") | None => wgpu::PowerPreference::HighPerformance,
        Some(other) => {
            eprintln!("AZUR_ADAPTER={other}: not one of low, high — using high");
            wgpu::PowerPreference::HighPerformance
        }
    }
}

/// The backend for a name, or [`preferred`] for `None`.
///
/// Separated from reading the environment so it can be tested: an environment
/// variable is process-wide, and tests share a process.
fn from_name(chosen: Option<&str>) -> wgpu::Backends {
    match chosen {
        None => preferred(),
        Some("vulkan") | Some("vk") => wgpu::Backends::VULKAN,
        Some("metal") => wgpu::Backends::METAL,
        // `gl`, `opengl`, `d3d12` and `dx12` all land here on purpose, and it is not an oversight:
        // neither backend is compiled in — see the module documentation — so naming one could only
        // produce a window that fails to open. Falling through says so and starts anyway.
        Some(other) => {
            eprintln!("AZUR_BACKEND={other}: not one of vulkan, metal — using the default");
            preferred()
        }
    }
}

/// What to present on when nobody has said otherwise.
///
/// Vulkan on Windows, for the reason the [module documentation](self) measures.
/// Elsewhere wgpu's own primary set: the fault is a Desktop Window Manager one,
/// and pinning Vulkan on a platform where Metal is the native path would trade a
/// measured problem for an invented one.
fn preferred() -> wgpu::Backends {
    if cfg!(windows) {
        wgpu::Backends::VULKAN
    } else {
        wgpu::Backends::PRIMARY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_presents_on_vulkan_and_nothing_else() {
        // Exactly one bit: a set of two would let wgpu pick the other one on a
        // machine where the order came out differently, and the fault would be
        // back with nothing in the configuration to show why.
        if cfg!(windows) {
            assert_eq!(preferred(), wgpu::Backends::VULKAN);
            assert_eq!(preferred().iter().count(), 1);
        } else {
            assert_eq!(preferred(), wgpu::Backends::PRIMARY);
        }
    }

    #[test]
    fn the_override_names_reach_their_backends() {
        assert_eq!(from_name(Some("vulkan")), wgpu::Backends::VULKAN);
        assert_eq!(from_name(Some("vk")), wgpu::Backends::VULKAN);
        assert_eq!(from_name(Some("metal")), wgpu::Backends::METAL);
    }

    #[test]
    fn an_unreadable_name_falls_back_rather_than_asking_for_nothing() {
        // `Backends::empty()` would leave wgpu with no adapter to find and the
        // window would fail to open, which is a worse answer than the default.
        assert_eq!(from_name(Some("direct3d9")), preferred());
        assert_eq!(from_name(Some("")), preferred());
        assert!(!from_name(Some("nonsense")).is_empty());
    }

    #[test]
    fn gl_and_d3d12_fall_back_rather_than_naming_a_backend_that_is_not_compiled_in() {
        // `Cargo.toml` enables neither `wgpu/gles` nor `wgpu/dx12`, so `Backends::GL` or
        // `Backends::DX12` here would ask wgpu for a backend it does not have: no adapter, and a
        // window that never opens. The names still parse — they just cannot be honoured — so they
        // take the same path as any other name wgpu cannot serve.
        for name in ["gl", "opengl", "d3d12", "dx12"] {
            assert_eq!(from_name(Some(name)), preferred(), "{name}");
        }
        assert!(!from_name(Some("gl")).contains(wgpu::Backends::GL));
        assert!(!from_name(Some("d3d12")).contains(wgpu::Backends::DX12));
    }

    #[test]
    fn configure_sets_all_three_decisions() {
        let mut configuration = WgpuConfiguration::default();
        configure(&mut configuration);
        let WgpuSetup::CreateNew(setup) = &configuration.wgpu_setup else {
            panic!("the default configuration should be creating its own device");
        };
        assert_eq!(setup.instance_descriptor.backends, backends());
        assert_eq!(setup.power_preference, adapter());
        assert_eq!(
            setup.power_preference,
            wgpu::PowerPreference::HighPerformance,
            "the default adapter is the discrete one; see the module docs for why"
        );
    }

    #[test]
    fn the_device_asks_for_memory_over_throughput() {
        // The 180 MB this module is mostly about. wgpu's default is `Performance`,
        // which sub-allocates in large blocks an interface has no use for.
        for backend in [wgpu::Backend::Vulkan, wgpu::Backend::Dx12, wgpu::Backend::Gl] {
            let it = device(backend);
            assert!(
                matches!(it.memory_hints, wgpu::MemoryHints::MemoryUsage),
                "{backend:?} asked for {:?}",
                it.memory_hints
            );
            assert_eq!(
                it.required_limits.max_texture_dimension_2d, 8192,
                "{backend:?}: a 4K surface needs a depth texture this big"
            );
        }
    }

    #[test]
    fn the_adapter_override_names_reach_their_preferences() {
        assert_eq!(
            preference_from_name(Some("low")),
            wgpu::PowerPreference::LowPower
        );
        assert_eq!(
            preference_from_name(Some("integrated")),
            wgpu::PowerPreference::LowPower
        );
        assert_eq!(
            preference_from_name(Some("high")),
            wgpu::PowerPreference::HighPerformance
        );
        // Anything unreadable, and nothing at all, land on the safe one rather than
        // on the cheap one: the cheap one's correctness is machine-dependent.
        assert_eq!(
            preference_from_name(Some("cheapest")),
            wgpu::PowerPreference::HighPerformance
        );
        assert_eq!(
            preference_from_name(None),
            wgpu::PowerPreference::HighPerformance
        );
    }
}
