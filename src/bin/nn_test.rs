use agricola_game::nn::NeuralNetwork;
use rand::Rng;

fn main() {
    let mut nn = NeuralNetwork::new(&[2, 10, 1]);

    let data = vec![
        (vec![0.0, 0.0], vec![0.0]),
        (vec![0.0, 1.0], vec![1.0]),
        (vec![1.0, 0.0], vec![1.0]),
        (vec![1.0, 1.0], vec![0.0]),
    ];

    let mut rng = rand::thread_rng();

    for i in 0..500000 {
        let (inputs, targets) = &data[rng.gen_range(0..data.len())];
        let outputs = nn.feed_forward(inputs);
        nn.backpropagate(inputs, targets, 0.15);
        println!("{:?} -> {:?}", inputs, outputs);
        println!(
            "Iteration {} Loss = {}",
            i + 1,
            nn.l2_loss(&outputs, targets)
        );
    }

    for (inputs, _) in &data {
        let outputs = nn.feed_forward(inputs);
        println!("{:?} -> {:?}", inputs, outputs);
    }

    // Best results:
    /*
        Iteration 500000 Loss = 0.005712643
    [0.0, 0.0] -> [0.042051688]
    [0.0, 1.0] -> [0.92525727]
    [1.0, 0.0] -> [0.92525727]
    [1.0, 1.0] -> [0.069337614]
         */
}
