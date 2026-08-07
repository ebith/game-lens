use criterion::{Criterion, criterion_group, criterion_main};
use image::{ImageBuffer, Rgba};
use windows::Win32::{Graphics::Gdi::MonitorFromWindow, UI::WindowsAndMessaging::GetForegroundWindow};
use xcap::Monitor;

fn capture_with_gdi() {
    let monitors = Monitor::all().unwrap();
    let _image = monitors[0].capture_image().unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Capture with GDI", |b| b.iter(|| capture_with_gdi()));

    let item = wgc::new_item_from_monitor(unsafe {
        MonitorFromWindow(GetForegroundWindow(), windows::Win32::Graphics::Gdi::MONITOR_DEFAULTTONEAREST)
    }).unwrap();
    let mut wgc = wgc::Wgc::new(item, Default::default()).unwrap();

    c.bench_function("Capture with WGC", |b| b.iter(|| {
        if let Some(frame_result) = wgc.next() {
            let frame = frame_result.unwrap();
            let frame_size = frame.size().unwrap();
            let buffer = frame.read_pixels(None).unwrap();
            let _image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(frame_size.width, frame_size.height, buffer).unwrap();
        }
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
