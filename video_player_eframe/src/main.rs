use eframe::{egui, egui_wgpu, wgpu};
use simple_video::{read_video_from_file, Video};

fn main() {
    eframe::run_native(
        "Video Player",
        eframe::NativeOptions {
            vsync: false,
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: egui_wgpu::WgpuConfiguration {
                present_mode: wgpu::PresentMode::AutoNoVsync,
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
    .unwrap();
}

struct App {
    video: Video,
    frame_index: u32,
    texture: wgpu::Texture,
    texture_id: egui::TextureId,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let egui_wgpu::RenderState {
            device,
            queue,
            renderer,
            ..
        } = cc.wgpu_render_state.as_ref().unwrap();

        let video = read_video_from_file(std::env::args().nth(1).unwrap()).unwrap();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: wgpu::Extent3d {
                width: video.width(),
                height: video.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_id = renderer.write().register_native_texture(
            device,
            &texture_view,
            wgpu::FilterMode::Nearest,
        );

        let mut this = Self {
            video,
            frame_index: 0,
            texture,
            texture_id,
        };
        this.update_texture(queue);
        this
    }

    fn update_texture(&mut self, queue: &wgpu::Queue) {
        if self.frame_index < self.video.frame_count() {
            let texture_size = self.texture.size();
            let pixels = self.video[self.frame_index as _]
                .iter()
                .flat_map(|color| [color.r, color.g, color.b, 255])
                .collect::<Vec<_>>();
            assert_eq!(
                pixels.len(),
                texture_size.width as usize * texture_size.height as usize * 4
            );
            queue.write_texture(
                self.texture.as_image_copy(),
                &pixels,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(texture_size.width * 4),
                    rows_per_image: None,
                },
                texture_size,
            );
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut slider_changed = false;

        egui::Window::new("Settings").show(ctx, |ui| {
            ui.label(format!("Frame Count: {}", self.video.frame_count()));
            ui.horizontal(|ui| {
                ui.label("Frame Index: ");
                slider_changed = ui
                    .add(egui::Slider::new(
                        &mut self.frame_index,
                        0..=self.video.frame_count().saturating_sub(1),
                    ))
                    .changed();
            });
        });

        egui::Window::new("Video")
            .frame(egui::Frame::none())
            .auto_sized()
            .show(ctx, |ui| {
                let egui_wgpu::RenderState { queue, .. } = frame.wgpu_render_state().unwrap();

                let texture_size = self.texture.size();
                let (rect, _response) = ui.allocate_exact_size(
                    egui::vec2(texture_size.width as _, texture_size.height as _),
                    egui::Sense::drag(),
                );

                if slider_changed {
                    self.update_texture(queue);
                }

                ui.painter().image(
                    self.texture_id,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 1.0), egui::pos2(1.0, 0.0)),
                    egui::Color32::WHITE,
                );
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                _ = ui;
            });
    }
}
