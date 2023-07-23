use agricola_game::agricola::farm::{Farm, MAX_FENCES};

fn main() {
    let mut farm = Farm::new();
    let fenc_opt = farm.fencing_options(3);
    assert_eq!(fenc_opt.len(), 0);
    let fenc_opt = farm.fencing_options(4);
    //println!("{fenc_opt:?}");
    assert_eq!(fenc_opt.len(), 2);
    let fenc_opt = farm.fencing_options(6);
    assert_eq!(fenc_opt.len(), 6);
    let fenc_opt = farm.fencing_options(8);
    assert_eq!(fenc_opt.len(), 7);
    let fenc_opt = farm.fencing_options(MAX_FENCES);
    assert_eq!(fenc_opt.len(), 10);

    farm.fence_spaces(&vec![9, 19, 21, 31]);
    let fenc_opt = farm.fencing_options(5);
    assert_eq!(fenc_opt.len(), 3);
    let fenc_opt = farm.fencing_options(7);
    assert_eq!(fenc_opt.len(), 6);
    for opt in &fenc_opt {
        println!("{opt:?}\n{}", Farm::format_fence_layout(opt));
    }
}
