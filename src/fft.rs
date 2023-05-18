use std::f32::consts::PI;

use rustfft::num_complex::Complex;

pub fn get_window(_name: &'static str, size: usize) -> Vec<f32> {
    let mut buffer = vec![0.; size];

    for (i, buf) in buffer.iter_mut().enumerate().take(size) {
        let a0 = 0.54;
        *buf = a0 - (1. - a0) * (2. * PI * i as f32 / size as f32).cos()
    }
    buffer
}

pub fn stft(
    signal: &[f32],
    window: &'static str,
    window_size: usize,
    hop_size: usize,
) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
    use ordered_float::OrderedFloat;
    dbg!(
        signal.iter().map(|x| OrderedFloat(*x)).max(),
        signal.iter().map(|x| OrderedFloat(*x)).min()
    );

    let window = get_window(window, window_size);
    let _half_window_round = (window_size + 1) / 2;
    let _half_window_floor = window_size / 2;
    let _length = signal.len();
    let mut magnitudes = Vec::new();
    let mut phases = Vec::new();

    signal
        .windows(window_size)
        .step_by(hop_size)
        .for_each(|chunk| {
            let (m, p) = dft(chunk, &window);

            magnitudes.push(m);
            phases.push(p);
        });

    (magnitudes, phases)
}

fn dft(signal: &[f32], window: &[f32]) -> (Vec<f32>, Vec<f32>) {
    //dbg!(window.len());
    let _sum: f32 = window.iter().sum();
    let windowed = signal
        .iter()
        .zip(window.iter())
        //.map(|(x, y)| x * y / sum)
        .map(|(x, y)| x * y)
        .map(|x| Complex { re: x, im: 0.0 })
        .collect::<Vec<_>>();
    //dbg!(&windowed.len());
    /*use ordered_float::OrderedFloat;
    dbg!(
        windowed.iter().map(|x| OrderedFloat(x.re)).max(),
        windowed.iter().map(|x| OrderedFloat(x.re)).min()
    );*/

    let size = window.len();
    let positive_spectrum = (size / 2) + 1;
    let half_window_round = (size + 1) / 2;
    let half_window_floor = size / 2;

    let mut fft_buffer = vec![Complex { re: 0., im: 0. }; size];
    let (left, right) = fft_buffer.split_at_mut(half_window_round);
    left.copy_from_slice(&windowed[half_window_floor..]);
    right.copy_from_slice(&windowed[..half_window_floor]);
    //dbg!(&fft_buffer.len());

    let mut planner = rustfft::FftPlanner::new();
    let fft = planner.plan_fft_forward(size);
    fft.process(&mut fft_buffer);

    let positive_side = fft_buffer[..positive_spectrum]
        .iter()
        .map(|x| 20. * x.re.abs().log10())
        //.map(|x| x.re)
        //.map(|x| x.re.powi(2) / size as f32)
        .collect::<Vec<_>>();

    (positive_side.to_vec(), vec![])
}
