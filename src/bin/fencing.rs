use agricola_game::agricola::farm::{Farm, MAX_FENCES};

fn main() {
    let mut farm = Farm::new();
    let fenc_opt = farm.fencing_options(3);
    assert_eq!(fenc_opt.len(), 0);
    let fenc_opt = farm.fencing_options(4);
    assert_eq!(fenc_opt.len(), 13);
    let fenc_opt = farm.fencing_options(6);
    assert_eq!(fenc_opt.len(), 31);
    let fenc_opt = farm.fencing_options(8);
    assert_eq!(fenc_opt.len(), 73);
    let fenc_opt = farm.fencing_options(MAX_FENCES);
    assert_eq!(fenc_opt.len(), 762);

    farm.fence_spaces(&vec![9, 19, 21, 31]);
    let fenc_opt = farm.fencing_options(5);
    assert_eq!(fenc_opt.len(), 6);
    let fenc_opt = farm.fencing_options(7);
    assert_eq!(fenc_opt.len(), 18);
    for opt in &fenc_opt {
        println!("{opt:?}");
        Farm::display_fence_layout(&opt);
    }
}
