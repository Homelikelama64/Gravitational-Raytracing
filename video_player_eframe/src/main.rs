use eframe::{
    egui::{self, vec2, Pos2, Rect},
    egui_wgpu, wgpu,
};
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

        let window_size = ctx.input(|i: &egui::InputState| i.screen_rect());

        let mut scale = (window_size.height() as f32 - 100.0) / self.texture.size().height as f32;
        if self.texture.size().width as f32 * scale + 40.0 > window_size.width() as f32 {
            scale = (window_size.width() as f32 - 40.0) / self.texture.size().width as f32
        }

        let pos = vec2(
            (window_size.width() - self.texture.size().width as f32 * scale + 10.0) / 2.0,
            (window_size.height() - self.texture.size().height as f32 * scale - 80.0) / 2.0,
        ) - vec2(5.0, 5.0);

        egui::Window::new("Settings")
            .default_pos(Pos2 {
                x: 30.0,
                y: 900.0,
            })
            .show(ctx, |ui| {
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
            .anchor(egui::Align2::LEFT_TOP, pos)
            .title_bar(false)
            .fixed_size(vec2(
                self.texture.size().width as f32 * scale,
                self.texture.size().height as f32 * scale,
            ))
            .show(ctx, |ui| {
                let egui_wgpu::RenderState { queue, .. } = frame.wgpu_render_state().unwrap();

                if slider_changed {
                    self.update_texture(queue);
                }
                ui.painter().image(
                    self.texture_id,
                    Rect::from_points(&[
                        Pos2::new(pos.x, pos.y),
                        Pos2::new(
                            self.texture.size().width as f32 * scale + pos.x,
                            self.texture.size().height as f32 * scale + pos.y,
                        ),
                    ]),
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
