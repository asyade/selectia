use dasp::sample::Sample;

pub struct DcBlocker<S: Sample> {
    alpha: S,    // Feedback coefficient (e.g., 0.995 for most cases)
    prev_input: S, // Previous input sample (x[n-1])
    prev_output: S, // Previous output sample (y[n-1])
}

impl<S> DcBlocker<S>
where
    S: Sample + dasp::sample::FloatSample, // For float arithmetic
{
    /// Create a new DC Blocker with a given feedback coefficient
    pub fn new(alpha: S) -> Self {
        Self {
            alpha,
            prev_input: S::EQUILIBRIUM, // Zero-initialized state
            prev_output: S::EQUILIBRIUM,
        }
    }

    /// Process a single sample
    pub fn process(&mut self, input: S) -> S {
        let output = input - self.prev_input + self.alpha * self.prev_output;
        self.prev_input = input;
        self.prev_output = output;
        output
    }

    /// Process a buffer of samples
    pub fn process_buffer(&mut self, input: &[S]) -> Vec<S> {
        input.iter().map(|&sample| self.process(sample)).collect()
    }
}

fn main() {
    // Example: Process a signal with a DC blocker
    let mut dc_blocker = DcBlocker::new(0.995); // Initialize with alpha = 0.995

    // Example input signal with DC offset
    let input_signal: Vec<f32> = vec![1.1, 1.2, 1.3, 1.2, 1.1, 1.0, 0.9, 0.8];

    // Process the signal
    let output_signal = dc_blocker.process_buffer(&input_signal);

    // Print the result
    println!("Output signal: {:?}", output_signal);
}
