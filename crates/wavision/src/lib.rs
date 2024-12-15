//! This crate provide simple interface to generate image representation of raw audio data (PCM)
use std::collections::BTreeMap;
use std::fs::File;
use std::os::windows::ffi::EncodeWide;
use std::path::Path;

use image::ImageBuffer;
use imageproc::drawing::{
    draw_cross_mut, draw_filled_circle_mut, draw_filled_rect_mut, draw_hollow_circle_mut,
    draw_hollow_rect_mut, draw_line_segment_mut,
};
use imageproc::rect::Rect;

use prelude::*;

pub mod error;
pub mod prelude;

pub trait Layer {
    fn renderer(&self, generator: &GraphGenerator<'_>, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>);
    fn width(&self) -> usize;
}

pub struct GraphGenerator<'a> {
    height: usize,
    width: usize,
    max_width: usize,
    samples_rate: usize,
    layers: Vec<GraphLayer<'a>>,
}

pub enum GraphLayer<'a> {
    Samples(SamplesLayer<'a>),
    Cues(CuesLayer),
}

pub struct SamplesLayer<'a> {
    style: SamplesLayerStyle,
    samples: &'a [f32],
    channels: usize,
}

pub struct CuesLayer {
    style: CuesLayerStyle,
    width: usize,
    cues: BTreeMap<usize, Cue>,
}

pub struct Cue {
    pub start: usize,
    pub end: usize,
}

pub struct SamplesLayerStyle {
    pub color: Rgb<u8>,
}

pub struct CuesLayerStyle {}

impl<'a> GraphGenerator<'a> {
    pub fn new(samples_rate: usize, height: usize, width: usize) -> Self {
        Self {
            height,
            width,
            max_width: 0,
            samples_rate,
            layers: vec![],
        }
    }

    pub fn layer(&mut self, layer: impl Into<GraphLayer<'a>>) -> &mut Self {
        let layer = layer.into();
        self.max_width = self.max_width.max(layer.width());
        self.layers.push(layer);
        self
    }

    pub fn generate(&self) -> WavisionResult<WavImage> {
        info!(
            width = self.max_width,
            height = self.height,
            "Begin image renderer"
        );
        let mut image = RgbImage::new(self.width as u32, self.height as u32);

        for layer in self.layers.iter() {
            layer.renderer(self, &mut image);
        }


        Ok(WavImage {
            sample_rate: self.samples_rate,
            buffer: image,
        })
    }
}

pub struct WavImage {
    sample_rate: usize,
    buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
}

impl<'a> SamplesLayer<'a> {
    pub fn new(style: SamplesLayerStyle, samples: &'a [f32], channels: usize) -> Self {
        Self {
            style,
            samples,
            channels,
        }
    }
}

impl<'a> Layer for SamplesLayer<'a> {
    fn width(&self) -> usize {
        self.samples.len() as usize / self.channels
    }

    fn renderer(&self, generator: &GraphGenerator<'_>, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let y_offset_base = generator.height as f32 / 2.0;
        let mut prev_coords = (0.0, y_offset_base);

        let elem_per_pixel = ((self.samples.len() as f32 / self.channels as f32) / generator.width as f32).ceil() as usize;
        let mut offset = 0;
        loop {
            let idx = offset * elem_per_pixel * self.channels as usize;
            if idx >= self.samples.len() {
                break;
            }
            let sample = self.samples[idx];
            let normalized = sample * generator.height as f32 / 2.0;
            let coords = (offset as f32, y_offset_base - normalized);
            draw_line_segment_mut(image, prev_coords, coords, self.style.color);
            prev_coords = coords;
            offset += 1;
        }
        info!("Rendering samples layer");
    }
}

impl Layer for CuesLayer {
    fn width(&self) -> usize {
        self.width as usize
    }

    fn renderer(&self, generator: &GraphGenerator<'_>, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        info!("Rendering cues layer");

        let pixel_per_sample = (generator.max_width as f32 / generator.width as f32).ceil() as usize;
        let mut offset = 0;
        for (_, cue) in self.cues.iter() {
            let x_offset = cue.start as f32 / pixel_per_sample as f32;
            let x_end = x_offset + 10.0;
            draw_filled_rect_mut(image, Rect::at(x_offset as i32, 1).of_size((x_end - x_offset) as u32, generator.height as u32 - 2), Rgb([255, 0, 0]));
        }
    }
}

macro_rules! impl_graph_layer_fn {
    ($layer:expr, $expr:ident) => {{
        match $layer {
            GraphLayer::Samples(layer) => layer.$expr(),
            GraphLayer::Cues(layer) => layer.$expr(),
        }
    }};
}

impl<'a> Layer for GraphLayer<'a> {
    fn width(&self) -> usize {
        impl_graph_layer_fn!(self, width)
    }

    fn renderer(&self, generator: &GraphGenerator<'_>, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        match self {
            GraphLayer::Samples(layer) => layer.renderer(generator, image),
            GraphLayer::Cues(layer) => layer.renderer(generator, image),
        }
    }
}

impl<'a> Into<GraphLayer<'a>> for SamplesLayer<'a> {
    fn into(self) -> GraphLayer<'a> {
        GraphLayer::Samples(self)
    }
}

impl<'a> Into<GraphLayer<'a>> for CuesLayer {
    fn into(self) -> GraphLayer<'a> {
        GraphLayer::Cues(self)
    }
}

impl WavImage {
    pub fn save(&self, path: impl AsRef<Path>) -> WavisionResult<()> {
        File::create(path.as_ref())?;
        self.buffer.save(path.as_ref()).map_err(|e| WavisionError::Export(e))?;
        Ok(())
    }
}

impl CuesLayer {
    pub fn new(style: CuesLayerStyle, width: usize, cues: BTreeMap<usize, Cue>) -> Self {
        Self {
            style,
            width,
            cues,
        }
    }
}
