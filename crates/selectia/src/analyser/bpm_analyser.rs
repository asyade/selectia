//! The BPM analyser provide a way to detect the BPM of an audio file based one the onesets provided for the audio file.
//! It is not able to find the onesets events but instead normalize the onesets events to a BPM value with a certain confidence as well as a well placed BPM grid
#![allow(dead_code)]
use crate::prelude::*;
use std::collections::BTreeMap;

use selectia_audio_file::audio_file::AudioBeatOneset;

pub struct BpmAnalyserOptions {
    pub range: (f32, f32),
}

pub struct BpmAnalyser {
    options: BpmAnalyserOptions,
    onesets: Vec<AudioBeatOneset>,
}

#[derive(Debug)]
pub struct BpmAnalyserResult {
    pub average_bpm: Option<BpmPlot>,
    pub alternative_bpm: Vec<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct BpmPlot {
    bpm: f32,
    offset: usize,
}

#[derive(Debug)]
struct OnesetGroup {
    average_bpm: f32,
    total_beat_duration: u64,
    total_beat_count: u64,
    average_beat_duration: f32,
    onesets: Vec<AudioBeatOneset>,
}

impl BpmAnalyser {
    pub fn new(options: BpmAnalyserOptions, onesets: Vec<AudioBeatOneset>) -> Self {
        Self { options, onesets }
    }

    pub fn get_result(&self) -> Result<BpmAnalyserResult> {
        let regions = self.grouped_onesets()?;
        let mut regions = regions
            .into_iter()
            .filter(|region| {
                region.average_bpm >= self.options.range.0
                    && region.average_bpm <= self.options.range.1
            })
            .collect::<Vec<_>>();
        regions.sort_by(|a, b| a.total_beat_count.cmp(&b.total_beat_count));

        let prefered_region = regions.pop();

        let average_bpm = prefered_region.as_ref().map(|region| region.plot());

        let alternative_bpm = regions
            .into_iter()
            .map(|region| region.average_bpm)
            .filter(|bpm| Some(*bpm) != average_bpm.map(|bpm| bpm.bpm))
            .collect::<Vec<_>>();

        Ok(BpmAnalyserResult {
            average_bpm,
            alternative_bpm,
        })
    }

    fn grouped_onesets(&self) -> Result<Vec<OnesetGroup>> {
        const BPM_REGION_SCALING: f32 = 50.0;

        let mut sorted_onesets = self.onesets.clone();
        sorted_onesets.sort_by(|a, b| {
            a.bpm
                .partial_cmp(&b.bpm)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut grouped_onesets = BTreeMap::new();

        for oneset in sorted_onesets {
            let scaled_bpm =
                ((oneset.bpm / BPM_REGION_SCALING).floor() * BPM_REGION_SCALING) as i64;
            grouped_onesets
                .entry(scaled_bpm)
                .or_insert(Vec::new())
                .push(oneset);
        }

        let regions = grouped_onesets
            .into_iter()
            .map(|(_bpm, onesets)| {
                let average_bpm =
                    onesets.iter().map(|oneset| oneset.bpm).sum::<f32>() / onesets.len() as f32;
                let total_beat_duration = onesets
                    .iter()
                    .map(|oneset| oneset.duration as u64)
                    .sum::<u64>();
                let total_beat_count = onesets.len() as u64;
                let average_beat_duration = total_beat_duration as f32 / total_beat_count as f32;
                OnesetGroup {
                    average_bpm,
                    total_beat_duration,
                    total_beat_count,
                    average_beat_duration,
                    onesets,
                }
            })
            .collect::<Vec<_>>();

        Ok(regions)
    }
}

impl OnesetGroup {
    fn plot(&self) -> BpmPlot {
        assert!(self.onesets.len() > 0);

        if self.onesets.len() == 1 {
            return BpmPlot {
                bpm: self.onesets[0].bpm,
                offset: self.onesets[0].offset,
            };
        }

        let mut max_bpm = f32::MIN;
        let mut min_bpm = f32::MAX;
        for oneset in self.onesets.iter() {
            max_bpm = max_bpm.max(oneset.bpm as f32);
            min_bpm = min_bpm.min(oneset.bpm as f32);
        }

        BpmPlot {
            bpm: self.average_bpm,
            offset: self.onesets[0].offset,
        }
    }

    fn missalignement_for_bpm(&self, bpm: f32) -> i64 {
        let base_offset = self.onesets[0].offset;
        let base_duration = self.onesets[0].duration;

        let bpm_diff = self.onesets[0].bpm - bpm;

        let bpm_diff_ratio = bpm_diff / bpm;
        let bpm_diff_duration = bpm_diff_ratio * base_duration as f32;
        let base_duration = f32::ceil(base_duration as f32 - bpm_diff_duration) as usize;

        let mut max_missalignement = 0;

        for oneset in self.onesets.iter() {
            let relative_missalignement =
                (oneset.offset as i64 - base_offset as i64) % base_duration as i64;
            max_missalignement =
                std::cmp::Ord::max(max_missalignement, relative_missalignement).abs();
        }

        max_missalignement
    }
}
