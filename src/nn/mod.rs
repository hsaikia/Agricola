use rand_distr::Distribution;

struct Neuron {
    input_weights: Vec<f32>,
    output: f32,
    error: f32,
    gradient: f32,
}

struct Layer {
    neurons: Vec<Neuron>,
}

pub struct NeuralNetwork {
    layers: Vec<Layer>,
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

// Assuming output is the output of a sigmoid activation
fn dsigmoid(y: f32) -> f32 {
    y * (1.0 - y)
}

impl NeuralNetwork {
    pub fn new(layer_sizes: &[usize]) -> Self {
        let mut rng = rand::thread_rng();

        let mut layers = Vec::new();
        for i in 1..layer_sizes.len() {
            let mut neurons = Vec::new();
            for _ in 0..layer_sizes[i] {
                let variance = 1.0 / (layer_sizes[i - 1] as f32).sqrt();
                let normal = rand_distr::Normal::new(0.0, variance).unwrap();
                let input_weights = vec![normal.sample(&mut rng); layer_sizes[i - 1]];
                neurons.push(Neuron {
                    input_weights,
                    output: 0.0,
                    error: 0.0,
                    gradient: 0.0,
                });
            }
            layers.push(Layer { neurons });
        }
        NeuralNetwork { layers }
    }

    pub fn feed_forward(&mut self, inputs: &[f32]) -> Vec<f32> {
        let mut outputs = inputs.to_vec();
        for layer in &mut self.layers {
            let mut new_outputs = Vec::new();
            for neuron in &mut layer.neurons {
                let mut sum = 1.0; // Bias
                for (input, weight) in outputs.iter().zip(neuron.input_weights.iter()) {
                    sum += input * weight;
                }
                neuron.output = sigmoid(sum);
                new_outputs.push(neuron.output);
            }
            outputs = new_outputs;
        }
        outputs
    }

    pub fn l2_loss(&mut self, outputs: &[f32], targets: &[f32]) -> f32 {
        let mut loss = 0.0;
        for (output, target) in outputs.iter().zip(targets.iter()) {
            loss += (output - target).powf(2.0);
        }
        loss
    }

    fn calculate_output_layer_error(&mut self, targets: &[f32]) {
        let output_layer = self.layers.last_mut().unwrap();
        for (neuron, target) in output_layer.neurons.iter_mut().zip(targets.iter()) {
            neuron.error = neuron.output - target;
            neuron.gradient = dsigmoid(neuron.output);
        }
    }

    fn calculate_hidden_layer_error(layer1: &mut Layer, layer2: &Layer) {
        for (i, neuron1) in layer1.neurons.iter_mut().enumerate() {
            let mut sum = 0.0;
            for neuron2 in &layer2.neurons {
                sum += neuron2.error * neuron2.input_weights[i];
            }
            neuron1.error = sum;
            neuron1.gradient = dsigmoid(neuron1.output);
        }
    }

    pub fn backpropagate(&mut self, inputs: &[f32], targets: &[f32], learning_rate: f32) {
        self.calculate_output_layer_error(targets);
        for i in (1..self.layers.len() - 1).rev() {
            let (layer1, layer2) = self.layers.split_at_mut(i);
            NeuralNetwork::calculate_hidden_layer_error(&mut layer1[0], &layer2[0]);
        }
        let mut outputs = inputs.to_vec();
        for layer in &mut self.layers {
            for neuron in &mut layer.neurons {
                for (i, input) in outputs.iter().enumerate() {
                    neuron.input_weights[i] -=
                        learning_rate * neuron.error * input * neuron.gradient;
                }
            }
            outputs = layer.neurons.iter().map(|neuron| neuron.output).collect();
        }
    }
}
