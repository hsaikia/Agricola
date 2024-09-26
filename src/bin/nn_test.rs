use agricola_game::nn::NeuralNetwork;
use rand::Rng;

fn main() {
    let mut nn = NeuralNetwork::new(&[2, 9, 1]);

    let data = vec![
        (vec![0.0, 0.0], vec![0.0]),
        (vec![0.0, 1.0], vec![1.0]),
        (vec![1.0, 0.0], vec![1.0]),
        (vec![1.0, 1.0], vec![0.0]),
    ];

    let mut rng = rand::thread_rng();

    for i in 0..=5000000 {
        let (inputs, targets) = &data[rng.gen_range(0..data.len())];
        let outputs = nn.feed_forward(inputs);
        nn.backpropagate(inputs, targets, 0.15);
        if i % 100000 == 0 {
            println!("Iteration {} Loss = {}", i, nn.l2_loss(&outputs, targets));
        }
    }

    for (inputs, _) in &data {
        let outputs = nn.feed_forward(inputs);
        println!("{:?} -> {:?}", inputs, outputs);
    }

    /*
    Best results:
    Iteration 5000000 Loss = 0.00090728153
    [0.0, 0.0] -> [0.030100394]
    [0.0, 1.0] -> [0.96897024]
    [1.0, 0.0] -> [0.96897024]
    [1.0, 1.0] -> [0.033484533]
             */
}
