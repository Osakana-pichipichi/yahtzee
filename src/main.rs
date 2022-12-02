use enum_iterator::*;

mod hand;
mod scoring;
mod score_table;
use hand::*;
use scoring::*;

fn main() {
	let mut h: Hand = Hand::new_with_random_n_dice(Hand::DICE_NUM);
	println!("Hand: {:?}", h.get_dice());

	println!(" --------- 1st ---------- ");
	for b in all::<Boxes>() {
		let score = scoring::scoring(&b, h.get_dice());
		println!("{}: {}", box_name(&b), score);
	}

	println!("\n --------- 2nd ---------- ");
	let r: &[u32] = h.get_dice();
	let r: &[u32; 3] = &[r[0], r[1], r[3]];
	h.remove_dice(r);
	h.add_dice(&Hand::new_with_random_n_dice(r.len()));
	println!("Hand: {:?}", h.get_dice());
	for b in all::<Boxes>() {
		let score = scoring::scoring(&b, h.get_dice());
		println!("{}: {}", box_name(&b), score);
	}

	println!("\n --------- 3nd ---------- ");
	let r: &[u32] = h.get_dice();
	let r: &[u32; 2] = &[r[2], r[1]];
	h.remove_dice(r);
	h.add_dice(&Hand::new_with_random_n_dice(r.len()));
	println!("Hand: {:?}", h.get_dice());
	for b in all::<Boxes>() {
		let score = scoring::scoring(&b, h.get_dice());
		println!("{}: {}", box_name(&b), score);
	}
}