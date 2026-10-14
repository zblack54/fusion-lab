use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, StandardNormal};

/// Independent streams keep one sensor's faults from changing another's observations.
pub(crate) struct Sensors {
    position: ChaCha8Rng,
    velocity: ChaCha8Rng,
}

impl Sensors {
    pub(crate) fn new(seed: u64) -> Self {
        let position = ChaCha8Rng::seed_from_u64(seed);
        let mut velocity = position.clone();
        velocity.set_stream(1);
        Self { position, velocity }
    }

    pub(crate) fn position(&mut self, truth: [f64; 2], sigma: f64) -> [f64; 2] {
        noisy(&mut self.position, truth, sigma)
    }

    pub(crate) fn velocity(&mut self, truth: [f64; 2], sigma: f64) -> [f64; 2] {
        noisy(&mut self.velocity, truth, sigma)
    }
}

fn noisy(rng: &mut ChaCha8Rng, truth: [f64; 2], sigma: f64) -> [f64; 2] {
    truth.map(|v| {
        let noise: f64 = StandardNormal.sample(rng);
        v + sigma * noise
    })
}
