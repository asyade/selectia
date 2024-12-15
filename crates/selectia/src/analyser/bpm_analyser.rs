//! The BPM analyser provide a way to detect the BPM of an audio file based one the onesets provided for the audio file.
//! It is not able to find the onesets events but instead normalize the onesets events to a BPM value with a certain confidence as well as a well placed BPM grid
#![allow(dead_code)]
use crate::prelude::*;
use std::collections::BTreeMap;

use ndarray::Array1;
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
    pub average_bpm: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct BpmPlot {
    bpm: f32,
    offset: usize,
}


impl BpmAnalyser {
    pub fn new(options: BpmAnalyserOptions, onesets: Vec<AudioBeatOneset>) -> Self {
        Self { options, onesets }
    }

    pub fn get_result(&self) -> Result<BpmAnalyserResult> {
        let diffs: Array1<f64> = Array1::from(self.onesets.windows(2).map(|w| w[1].seconds - w[0].seconds).collect::<Vec<f64>>());
        let bpms: Array1<f64> = diffs.mapv(|period| 60.0 / period);

        // dbg!(&bpms);
        let bpms_variations = bpms.windows(2).into_iter().map(|w| w[1] - w[0]).collect::<Vec<f64>>();
        // dbg!(&bpms_variations);

        
        let median_unfiltered = median(bpms.to_vec());
        let nbr_oneset_before = bpms.len();
        let filtered_bpm = bpms.into_iter().filter((|x| {
            (median_unfiltered - x).abs() < 15.0
        })).collect::<Vec<_>>();


        
        let median_unfiltered = median(filtered_bpm.clone());
        let filtered_bpm = filtered_bpm.into_iter().filter((|x| {
            (median_unfiltered - x).abs() < 4.0
        })).collect::<Vec<_>>();


        


        let nbr_filtered = nbr_oneset_before - filtered_bpm.len();
        let average = average(&filtered_bpm);
        
        info!(nbr_filtered_beat=nbr_filtered, average_bpm=average,  median_bpm_unfiltered=median_unfiltered, "filtered bpm");



        Ok(BpmAnalyserResult {
            average_bpm: average,
        })
    }

}


fn median(arr: impl Into<Vec<f64>>) -> f64 {
    let mut sorted = arr.into();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let len = sorted.len();
    if len % 2 == 0 {
        // Even number of elements: average the two middle ones
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
    } else {
        // Odd number of elements: return the middle one
        sorted[len / 2]
    }
}

fn average(arr: &[f64]) -> f64 {
    arr.iter().sum::<f64>() / arr.len() as f64
}
