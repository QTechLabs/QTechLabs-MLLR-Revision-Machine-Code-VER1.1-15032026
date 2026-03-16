fn logistic(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

fn compute_signal(norm_ret: f64, norm_synth: f64) -> f64 {

    let w0 = -2.0;
    let w1 = 2.0;
    let w2 = 1.0;

    let z = w0 + w1 * norm_ret + w2 * norm_synth;

    logistic(z)
}

fn main() {

    let norm_ret = 0.6;
    let norm_synth = 0.4;

    let probability = compute_signal(norm_ret, norm_synth);

    println!("Signal probability: {}", probability);
}
