#![allow(clippy::zero_prefixed_literal)]

use std::collections::HashMap;
use std::io::Read;

type Depth = u8;

type Board = u32;
type BoardFinalHash = u32;
type BoardIndex = u8;
type DiceValue = u8;

type Sum = u32;
type PathCount = u32;

const SYMMETRY_COUNT: u8 = 8;
type SymmetryPathCount = [PathCount; SYMMETRY_COUNT as usize];

const DICE_MAX: DiceValue = 6;

const SUM_MOD: Sum = 1 << 30;
const DEPTH_MAX: Depth = 40;

const QUEUE_CAPACITY: usize = 1 << 12;

fn main() {
	let (depth, starting_board) = parse();

	println!("{sum}", sum = solve(depth, starting_board));
}

macro_rules! parse_input {
	($x:expr, $t:ident) => {
		$x.trim().parse::<$t>().unwrap()
	};
}

fn parse() -> (Depth, Board) {
	let mut input_line = String::new();
	std::io::stdin().read_line(&mut input_line).unwrap();
	let depth = parse_input!(input_line, u8);

	let starting_board = board_parse(board_read());

	(depth, starting_board)
}

const C_BR: DiceValue = 0;
const C_B_: DiceValue = 3;
const C_BL: DiceValue = 6;
const C_R_: DiceValue = 9;
const C_M_: DiceValue = 12;
const C_L_: DiceValue = 15;
const C_TR: DiceValue = 18;
const C_T_: DiceValue = 21;
const C_TL: DiceValue = 24;

type SymmetryIndex = usize;
const S_I__: SymmetryIndex = 0;
const S_V__: SymmetryIndex = 1;
const S_H__: SymmetryIndex = 2;
const S_DZ_: SymmetryIndex = 3;
const S_DN_: SymmetryIndex = 4;
const S_90_: SymmetryIndex = 5;
const S_180: SymmetryIndex = 6;
const S_270: SymmetryIndex = 7;

static TRANSFORMERS: [fn(Board) -> Board; SYMMETRY_COUNT as usize] = [
	// 0
	#[inline(always)]
	|board| board,
	// vertical flip
	#[inline(always)]
	|board| {
		// ((board & 0b111_000_000_000_000_000_000_000_000) >> 6)
		// | (board & 0b000_111_000_000_000_000_000_000_000)
		// | ((board & 0b000_000_111_000_000_000_000_000_000) << 6)
		// | ((board & 0b000_000_000_111_000_000_000_000_000) >> 6)
		// | (board & 0b000_000_000_000_111_000_000_000_000)
		// | ((board & 0b000_000_000_000_000_111_000_000_000) << 6)
		// | ((board & 0b000_000_000_000_000_000_111_000_000) >> 6)
		// | (board & 0b000_000_000_000_000_000_000_111_000)
		// | ((board & 0b000_000_000_000_000_000_000_000_111) << 6)
		((board & 0b111_000_000_111_000_000_111_000_000) >> 6)
			| (board & 0b000_111_000_000_111_000_000_111_000)
			| ((board & 0b000_000_111_000_000_111_000_000_111) << 6)
	},
	// horizontal flip
	#[inline(always)]
	|board| {
		// ((board & 0b111_000_000_000_000_000_000_000_000) >> 18)
		// | ((board & 0b000_111_000_000_000_000_000_000_000) >> 18)
		// | ((board & 0b000_000_111_000_000_000_000_000_000) >> 18)
		// | (board & 0b000_000_000_111_000_000_000_000_000)
		// | (board & 0b000_000_000_000_111_000_000_000_000)
		// | (board & 0b000_000_000_000_000_111_000_000_000)
		// | ((board & 0b000_000_000_000_000_000_111_000_000) << 18)
		// | ((board & 0b000_000_000_000_000_000_000_111_000) << 18)
		// | ((board & 0b000_000_000_000_000_000_000_000_111) << 18)
		((board & 0b111_111_111_000_000_000_000_000_000) >> 18)
			| (board & 0b000_000_000_111_111_111_000_000_000)
			| ((board & 0b000_000_000_000_000_000_111_111_111) << 18)
	},
	// diagonal /
	#[inline(always)]
	|board| {
		((board & 0b111_000_000_000_000_000_000_000_000) >> 24)
		// | ((board & 0b000_111_000_000_000_000_000_000_000) >> 12)
		// | (board & 0b000_000_111_000_000_000_000_000_000)
		// | ((board & 0b000_000_000_111_000_000_000_000_000) >> 12)
		// | (board & 0b000_000_000_000_111_000_000_000_000)
		// | ((board & 0b000_000_000_000_000_111_000_000_000) << 12)
		// | (board & 0b000_000_000_000_000_000_111_000_000)
		// | ((board & 0b000_000_000_000_000_000_000_111_000) << 12)
			| ((board & 0b000_000_000_000_000_000_000_000_111) << 24)
			| ((board & 0b000_111_000_111_000_000_000_000_000) >> 12)
			| (board & 0b000_000_111_000_111_000_111_000_000)
			| ((board & 0b000_000_000_000_000_111_000_111_000) << 12)
	},
	// diagonal \
	#[inline(always)]
	|board| {
		// (board & 0b111_000_000_000_000_000_000_000_000)
		// | ((board & 0b000_111_000_000_000_000_000_000_000) >> 6)
		((board & 0b000_000_111_000_000_000_000_000_000) >> 12)
		// | ((board & 0b000_000_000_111_000_000_000_000_000) << 6)
		// | (board & 0b000_000_000_000_111_000_000_000_000)
		// | ((board & 0b000_000_000_000_000_111_000_000_000) >> 6)
			| ((board & 0b000_000_000_000_000_000_111_000_000) << 12)
		// | ((board & 0b000_000_000_000_000_000_000_111_000) << 6)
		// | (board & 0b000_000_000_000_000_000_000_000_111)
			| (board & 0b111_000_000_000_111_000_000_000_111)
			| ((board & 0b000_111_000_000_000_111_000_000_000) >> 6)
			| ((board & 0b000_000_000_111_000_000_000_111_000) << 6)
	},
	// 90 clockwise
	#[inline(always)]
	|board| {
		// ((board & 0b111_000_000_000_000_000_000_000_000) >> 6)
		((board & 0b000_111_000_000_000_000_000_000_000) >> 12)
			| ((board & 0b000_000_111_000_000_000_000_000_000) >> 18)
			// | ((board & 0b000_000_000_111_000_000_000_000_000) << 6)
			| (board & 0b000_000_000_000_111_000_000_000_000)
			// | ((board & 0b000_000_000_000_000_111_000_000_000) >> 6)
			| ((board & 0b000_000_000_000_000_000_111_000_000) << 18)
			| ((board & 0b000_000_000_000_000_000_000_111_000) << 12)
			// | ((board & 0b000_000_000_000_000_000_000_000_111) << 6)
			| ((board & 0b111_000_000_000_000_111_000_000_000) >> 6)
			| ((board & 0b000_000_000_111_000_000_000_000_111) << 6)
	},
	// 180
	#[inline(always)]
	|board| {
		((board & 0b111_000_000_000_000_000_000_000_000) >> 24)
			| ((board & 0b000_111_000_000_000_000_000_000_000) >> 18)
			| ((board & 0b000_000_111_000_000_000_000_000_000) >> 12)
			| ((board & 0b000_000_000_111_000_000_000_000_000) >> 6)
			| (board & 0b000_000_000_000_111_000_000_000_000)
			| ((board & 0b000_000_000_000_000_111_000_000_000) << 6)
			| ((board & 0b000_000_000_000_000_000_111_000_000) << 12)
			| ((board & 0b000_000_000_000_000_000_000_111_000) << 18)
			| ((board & 0b000_000_000_000_000_000_000_000_111) << 24)
	},
	// 270 clockwise or 90 counter-clockwise
	#[inline(always)]
	|board| {
		((board & 0b111_000_000_000_000_000_000_000_000) >> 18)
			// | ((board & 0b000_111_000_000_000_000_000_000_000) >> 6)
			// | ((board & 0b000_000_111_000_000_000_000_000_000) << 6)
			| ((board & 0b000_000_000_111_000_000_000_000_000) >> 12)
			| (board & 0b000_000_000_000_111_000_000_000_000)
			| ((board & 0b000_000_000_000_000_111_000_000_000) << 12)
			// | ((board & 0b000_000_000_000_000_000_111_000_000) >> 6)
			// | ((board & 0b000_000_000_000_000_000_000_111_000) << 6)
			| ((board & 0b000_000_000_000_000_000_000_000_111) << 18)
			| ((board & 0b000_111_000_000_000_000_111_000_000) >> 6)
			| ((board & 0b000_000_111_000_000_000_000_111_000) << 6)
	},
];

static REVERSE_TRANSFORMERS: [fn(Board) -> Board; SYMMETRY_COUNT as usize] = [
	// I
	TRANSFORMERS[S_I__],
	// V
	TRANSFORMERS[S_V__],
	// H
	TRANSFORMERS[S_H__],
	// DZ
	TRANSFORMERS[S_DZ_],
	// DN
	TRANSFORMERS[S_DN_],
	// 90
	TRANSFORMERS[S_270],
	// 180
	TRANSFORMERS[S_180],
	// 270
	TRANSFORMERS[S_90_],
];

// SYMMETRY_INDEX_TRANSFORMER[transformation][state]
static SYMMETRY_INDEX_TRANSFORMER: [[SymmetryIndex; SYMMETRY_COUNT as usize];
	SYMMETRY_COUNT as usize] = [
	// I
	[S_I__, S_V__, S_H__, S_DZ_, S_DN_, S_90_, S_180, S_270],
	// V
	[S_V__, S_I__, S_180, S_270, S_90_, S_DN_, S_H__, S_DZ_],
	// H
	[S_H__, S_180, S_I__, S_90_, S_270, S_DZ_, S_V__, S_DN_],
	// DZ
	[S_DZ_, S_90_, S_270, S_I__, S_180, S_V__, S_DN_, S_H__],
	// DN
	[S_DN_, S_270, S_90_, S_180, S_I__, S_H__, S_DZ_, S_V__],
	// 90
	[S_90_, S_DZ_, S_DN_, S_H__, S_V__, S_180, S_270, S_I__],
	// 180
	[S_180, S_H__, S_V__, S_DN_, S_DZ_, S_270, S_I__, S_90_],
	// 270
	[S_270, S_DN_, S_DZ_, S_V__, S_H__, S_I__, S_90_, S_180],
];

#[inline]
fn empty_cell_mask(index: BoardIndex) -> Board {
	!(0b111 << index)
}

// (src & !(0b111 << index)) | (value << index)
#[inline]
fn set(src: Board, mask: Board, index: BoardIndex, value: Board) -> Board {
	(src & mask) | (value << index)
}

type HashDiceTable = [Board; 7];

const fn generate_table(multiplier: Board) -> HashDiceTable {
	let mut table = [0; 7];
	let mut i = 0;
	while i < 7 {
		table[i] = i as u32 * multiplier;
		i += 1;
	}
	table
}

const HASH_TABLE_TL: HashDiceTable = generate_table(100_000_000);
const HASH_TABLE_T_: HashDiceTable = generate_table(010_000_000);
const HASH_TABLE_TR: HashDiceTable = generate_table(001_000_000);
const HASH_TABLE_L_: HashDiceTable = generate_table(000_100_000);
const HASH_TABLE_M_: HashDiceTable = generate_table(000_010_000);
const HASH_TABLE_R_: HashDiceTable = generate_table(000_001_000);
const HASH_TABLE_BL: HashDiceTable = generate_table(000_000_100);
const HASH_TABLE_B_: HashDiceTable = generate_table(000_000_010);
const HASH_TABLE_BR: HashDiceTable = generate_table(000_000_001);

// const BOARD_MAX: Board = 0b110_110_110_110_110_110_110_110_110;

fn board_read() -> [u8; 18] {
	let mut buffer = [0u8; 18];
	std::io::stdin().read_exact(&mut buffer).unwrap();
	buffer
}

fn board_parse(input: [u8; 18]) -> Board {
	let ascii_zero = b'0';

	(((input[0] - ascii_zero) as Board) << C_TL)
		| (((input[2] - ascii_zero) as Board) << C_T_)
		| (((input[4] - ascii_zero) as Board) << C_TR)
		| (((input[6] - ascii_zero) as Board) << C_L_)
		| (((input[8] - ascii_zero) as Board) << C_M_)
		| (((input[10] - ascii_zero) as Board) << C_R_)
		| (((input[12] - ascii_zero) as Board) << C_BL)
		| (((input[14] - ascii_zero) as Board) << C_B_)
		| ((input[16] - ascii_zero) as Board)
}

#[inline]
fn get(board: Board, index: BoardIndex) -> DiceValue {
	(board >> index) as DiceValue & 0b111
}

#[inline]
fn hash(board: Board) -> BoardFinalHash {
	HASH_TABLE_TL[get(board, C_TL) as usize]
		+ HASH_TABLE_T_[get(board, C_T_) as usize]
		+ HASH_TABLE_TR[get(board, C_TR) as usize]
		+ HASH_TABLE_L_[get(board, C_L_) as usize]
		+ HASH_TABLE_M_[get(board, C_M_) as usize]
		+ HASH_TABLE_R_[get(board, C_R_) as usize]
		+ HASH_TABLE_BL[get(board, C_BL) as usize]
		+ HASH_TABLE_B_[get(board, C_B_) as usize]
		+ HASH_TABLE_BR[get(board, C_BR) as usize]
}

macro_rules! canonical_min {
	($board:ident, $min:ident, $symmetry_index:ident) => {
		let transformed = TRANSFORMERS[$symmetry_index]($board);
		if transformed < $min.0 {
			$min = (transformed, $symmetry_index);
		}
	};
}

fn canonical(board: Board) -> (Board, SymmetryIndex) {
	let mut min: (Board, SymmetryIndex) = (board, 0); // start with the identity
	// canonical_min!(board, min, S_I__);

	canonical_min!(board, min, S_V__);
	canonical_min!(board, min, S_H__);
	canonical_min!(board, min, S_DZ_);
	canonical_min!(board, min, S_DN_);
	canonical_min!(board, min, S_90_);
	canonical_min!(board, min, S_180);
	canonical_min!(board, min, S_270);

	min
}

macro_rules! queue_insert {
	($queue:ident, $board:expr, $symmetry_path_count:ident) => {
		let canonical = canonical($board);
		let mut transformed_count = [0; SYMMETRY_COUNT as usize];
		transformed_count[SYMMETRY_INDEX_TRANSFORMER[canonical.1][0]] = $symmetry_path_count[0];
		transformed_count[SYMMETRY_INDEX_TRANSFORMER[canonical.1][1]] = $symmetry_path_count[1];
		transformed_count[SYMMETRY_INDEX_TRANSFORMER[canonical.1][2]] = $symmetry_path_count[2];
		transformed_count[SYMMETRY_INDEX_TRANSFORMER[canonical.1][3]] = $symmetry_path_count[3];
		transformed_count[SYMMETRY_INDEX_TRANSFORMER[canonical.1][4]] = $symmetry_path_count[4];
		transformed_count[SYMMETRY_INDEX_TRANSFORMER[canonical.1][5]] = $symmetry_path_count[5];
		transformed_count[SYMMETRY_INDEX_TRANSFORMER[canonical.1][6]] = $symmetry_path_count[6];
		transformed_count[SYMMETRY_INDEX_TRANSFORMER[canonical.1][7]] = $symmetry_path_count[7];

		$queue
			.entry(canonical.0)
			.and_modify(|count| {
				count[0] = count[0].wrapping_add(transformed_count[0]);
				count[1] = count[1].wrapping_add(transformed_count[1]);
				count[2] = count[2].wrapping_add(transformed_count[2]);
				count[3] = count[3].wrapping_add(transformed_count[3]);
				count[4] = count[4].wrapping_add(transformed_count[4]);
				count[5] = count[5].wrapping_add(transformed_count[5]);
				count[6] = count[6].wrapping_add(transformed_count[6]);
				count[7] = count[7].wrapping_add(transformed_count[7]);
			})
			.or_insert(transformed_count);
	};
}

macro_rules! play_single_move {
	($board:ident, $index:ident, $symmetry_path_count:ident, $queue:ident, $moved:ident, $neighbors_buf:ident, $($neighbors:literal),+) => {
		let n = 0 $(
			+ $neighbors_buf[$neighbors].1
		)+;
		if n <= DICE_MAX {
			$moved = true;
			queue_insert!($queue, set(
				$board,
				empty_cell_mask($index)
				$(
					& $neighbors_buf[$neighbors].2
				)+,
				$index,
				n as Board
			), $symmetry_path_count);
		}
	};
}

macro_rules! play_move {
	($board:ident, $index:ident, $symmetry_path_count:ident, $queue:ident, $moved:ident, $neighbors_buf:ident, $neighbors_len:ident, $($neighbors:ident),+) => {
		if get($board, $index) == 0 {
			$moved = true;

			$neighbors_len = 0;
			$(
				let neighbor = get($board, $neighbors);
				if neighbor != 0 && neighbor != DICE_MAX {
					$neighbors_buf[$neighbors_len as usize] = ($neighbors, neighbor, empty_cell_mask($neighbors));
					$neighbors_len += 1;
				}
			)+

			if $neighbors_len <= 1 {
				queue_insert!($queue, set($board, empty_cell_mask($index), $index, 1), $symmetry_path_count);
			} else {
				match $neighbors_len {
					2 => {
						let n = $neighbors_buf[0].1 + $neighbors_buf[1].1;
						if n <= DICE_MAX {
							queue_insert!($queue, set(
								$board,
								empty_cell_mask($index)
								& $neighbors_buf[0].2
								& $neighbors_buf[1].2,
								$index,
								n as Board
							), $symmetry_path_count);
						} else {
							queue_insert!($queue, set($board, empty_cell_mask($index), $index, 1), $symmetry_path_count);
						}
					},
					3 => {
						let mut moved_here = false;

						// 2 of 3
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 1);
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 2);
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 1, 2);
						// 3 of 3
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 1, 2);

						if !moved_here {
							queue_insert!($queue, set($board, empty_cell_mask($index), $index, 1), $symmetry_path_count);
						}
					},
					_ => {
						let mut moved_here = false;

						// 2 of 4
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 1);
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 2);
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 3);
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 1, 2);
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 1, 3);
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 2, 3);
						// 3 of 4
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 1, 2);
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 1, 3);
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 2, 3);
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 1, 2, 3);
						// 4 of 4
						play_single_move!($board, $index, $symmetry_path_count, $queue, moved_here, $neighbors_buf, 0, 1, 2, 3);

						if !moved_here {
							queue_insert!($queue, set($board, empty_cell_mask($index), $index, 1), $symmetry_path_count);
						}
					},
				}
			}
		}
	};
}

// we can let it overflow because u32 is 4 times 2^30
macro_rules! sum {
	($sum:ident, $board:ident, $symmetry_path_count:ident) => {
		$sum = $sum
			.wrapping_add(
				// hash(REVERSE_TRANSFORMERS[S_I__]($board)).wrapping_mul($symmetry_path_count[S_I__]),
				hash($board).wrapping_mul($symmetry_path_count[S_I__]),
			)
			.wrapping_add(
				hash(REVERSE_TRANSFORMERS[S_V__]($board)).wrapping_mul($symmetry_path_count[S_V__]),
			)
			.wrapping_add(
				hash(REVERSE_TRANSFORMERS[S_H__]($board)).wrapping_mul($symmetry_path_count[S_H__]),
			)
			.wrapping_add(
				hash(REVERSE_TRANSFORMERS[S_DZ_]($board)).wrapping_mul($symmetry_path_count[S_DZ_]),
			)
			.wrapping_add(
				hash(REVERSE_TRANSFORMERS[S_DN_]($board)).wrapping_mul($symmetry_path_count[S_DN_]),
			)
			.wrapping_add(
				hash(REVERSE_TRANSFORMERS[S_90_]($board)).wrapping_mul($symmetry_path_count[S_90_]),
			)
			.wrapping_add(
				hash(REVERSE_TRANSFORMERS[S_180]($board)).wrapping_mul($symmetry_path_count[S_180]),
			)
			.wrapping_add(
				hash(REVERSE_TRANSFORMERS[S_270]($board)).wrapping_mul($symmetry_path_count[S_270]),
			);
	};
}

const EMPTY_SOLUTION: [Sum; DEPTH_MAX as usize + 1] = [
	0, 111111111, 704035952, 840352818, 600875666, 50441886, 680243700, 597686656, 584450980,
	55305380, 193520836, 521847116, 1054388152, 518795448, 366207036, 678967952, 476916052,
	1009258340, 592651828, 1063467872, 400415524, 233248832, 230461008, 245411624, 899694236,
	384163740, 888060600, 347933640, 340717612, 73295296, 851289228, 221286388, 375032784,
	723342020, 92414440, 745533092, 331519112, 993643868, 72093236, 422667876, 503115192,
];

type Queue = HashMap<
	// canonical board
	Board,
	// current symmetry state as index (S_I__, S_V__, S_H__, S_DZ_, S_DN_, S_90_, S_180, S_270)
	SymmetryPathCount,
>;

fn solve(depth: Depth, starting_board: Board) -> Sum {
	if starting_board == 0 {
		return EMPTY_SOLUTION[depth as usize];
	}

	let mut sum: Sum = 0;
	let mut queue: Queue = HashMap::with_capacity(QUEUE_CAPACITY);
	let mut current_queue: Queue = HashMap::with_capacity(QUEUE_CAPACITY);
	let mut ngb_buf: [(BoardIndex, DiceValue, Board); 4] = [(0, 0, 0); 4];
	let mut ngb_len: u8;
	let mut d = 0;

	// no need to compute first canonical
	// there will be no duplicates possible with only 1 board on depth 0
	queue.insert(starting_board, [1, 0, 0, 0, 0, 0, 0, 0]);

	while d < depth && !queue.is_empty() {
		std::mem::swap(&mut queue, &mut current_queue);

		for (board, spc) in current_queue.drain() {
			let mut moved = false;

			play_move!(board, C_BR, spc, queue, moved, ngb_buf, ngb_len, C_B_, C_R_);
			play_move!(
				board, C_B_, spc, queue, moved, ngb_buf, ngb_len, C_BL, C_M_, C_BR
			);
			play_move!(board, C_BL, spc, queue, moved, ngb_buf, ngb_len, C_L_, C_B_);
			play_move!(
				board, C_R_, spc, queue, moved, ngb_buf, ngb_len, C_M_, C_TR, C_BR
			);
			play_move!(
				board, C_M_, spc, queue, moved, ngb_buf, ngb_len, C_L_, C_T_, C_R_, C_B_
			);
			play_move!(
				board, C_L_, spc, queue, moved, ngb_buf, ngb_len, C_TL, C_M_, C_BL
			);
			play_move!(board, C_TR, spc, queue, moved, ngb_buf, ngb_len, C_T_, C_R_);
			play_move!(
				board, C_T_, spc, queue, moved, ngb_buf, ngb_len, C_TL, C_TR, C_M_
			);
			play_move!(board, C_TL, spc, queue, moved, ngb_buf, ngb_len, C_T_, C_L_);

			if !moved {
				sum!(sum, board, spc);
			}
		}

		d += 1;
	}

	for (board, spc) in queue {
		sum!(sum, board, spc);
	}

	sum % SUM_MOD
}

#[cfg(test)]
mod tests {
	use super::*;

	fn board_from_hash(hash: BoardFinalHash) -> Board {
		(((hash / 100_000_000) % 10) << C_TL)
			| (((hash / 010_000_000) % 10) << C_T_)
			| (((hash / 001_000_000) % 10) << C_TR)
			| (((hash / 000_100_000) % 10) << C_L_)
			| (((hash / 000_010_000) % 10) << C_M_)
			| (((hash / 000_001_000) % 10) << C_R_)
			| (((hash / 000_000_100) % 10) << C_BL)
			| (((hash / 000_000_010) % 10) << C_B_)
			| (hash % 10)
	}

	#[test]
	fn test_parse() {
		let input = [
			b'1', b' ', b'2', b' ', b'3', b'\n', //
			b'4', b' ', b'5', b' ', b'6', b'\n', //
			b'0', b' ', b'2', b' ', b'4', b'\n',
		];
		let board = board_parse(input);
		assert_eq!(board, 0b_001_010_011_100_101_110_000_010_100);
		assert_eq!(get(board, C_TL), 1);
		assert_eq!(get(board, C_T_), 2);
		assert_eq!(get(board, C_TR), 3);
		assert_eq!(get(board, C_L_), 4);
		assert_eq!(get(board, C_M_), 5);
		assert_eq!(get(board, C_R_), 6);
		assert_eq!(get(board, C_BL), 0);
		assert_eq!(get(board, C_B_), 2);
		assert_eq!(get(board, C_BR), 4);
		assert_eq!(hash(board), 123_456_024);
	}

	#[test]
	fn test_set() {
		let src = 0b_001_010_011_100_101_110_000_010_100;
		assert_eq!(
			set(src, empty_cell_mask(C_TL), C_TL, 0),
			0b_000_010_011_100_101_110_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_T_), C_T_, 0),
			0b_001_000_011_100_101_110_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_TR), C_TR, 0),
			0b_001_010_000_100_101_110_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_L_), C_L_, 0),
			0b_001_010_011_000_101_110_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_M_), C_M_, 0),
			0b_001_010_011_100_000_110_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_R_), C_R_, 0),
			0b_001_010_011_100_101_000_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_BL), C_BL, 0),
			0b_001_010_011_100_101_110_000_010_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_B_), C_B_, 0),
			0b_001_010_011_100_101_110_000_000_100
		);
		assert_eq!(
			set(src, empty_cell_mask(C_BR), C_BR, 0),
			0b_001_010_011_100_101_110_000_010_000
		);
	}

	#[test]
	fn test_get() {
		let board = 0b_001_010_011_100_101_110_000_010_100;
		assert_eq!(get(board, C_TL), 1);
		assert_eq!(get(board, C_T_), 2);
		assert_eq!(get(board, C_TR), 3);
		assert_eq!(get(board, C_L_), 4);
		assert_eq!(get(board, C_M_), 5);
		assert_eq!(get(board, C_R_), 6);
		assert_eq!(get(board, C_BL), 0);
		assert_eq!(get(board, C_B_), 2);
		assert_eq!(get(board, C_BR), 4);
	}

	#[test]
	fn test_hash() {
		let board = 0b_001_010_011_100_101_110_000_010_100;
		assert_eq!(hash(board), 123_456_024);

		let board = board_from_hash(123_456_024);
		assert_eq!(board, 0b_001_010_011_100_101_110_000_010_100);
		assert_eq!(hash(board), 123_456_024);
	}

	fn test_transform_from_state(state: Board, expected_result: [Board; SYMMETRY_COUNT as usize]) {
		for i in [S_I__, S_V__, S_H__, S_DZ_, S_DN_, S_90_, S_180, S_270] {
			assert_eq!(hash(TRANSFORMERS[i](state)), hash(expected_result[i]),);
		}
	}

	fn test_symmetry_index_transform(
		state: SymmetryIndex,
		expected_result: [SymmetryIndex; SYMMETRY_COUNT as usize],
	) {
		for transformation in [S_I__, S_V__, S_H__, S_DZ_, S_DN_, S_90_, S_180, S_270] {
			assert_eq!(
				SYMMETRY_INDEX_TRANSFORMER[transformation][state],
				expected_result[transformation],
			);
		}
	}

	#[test]
	fn test_transform() {
		let i = board_from_hash(123_456_024);
		let v = board_from_hash(321_654_420);
		let h = board_from_hash(024_456_123);
		// Z
		let dz = board_from_hash(463_252_041);
		// N
		let dn = board_from_hash(140_252_364);
		let r90 = board_from_hash(041_252_463);
		let r180 = board_from_hash(420_654_321);
		let r270 = board_from_hash(364_252_140);

		test_transform_from_state(i, [i, v, h, dz, dn, r90, r180, r270]);
		test_transform_from_state(v, [v, i, r180, r90, r270, dz, h, dn]);
		test_transform_from_state(h, [h, r180, i, r270, r90, dn, v, dz]);
		test_transform_from_state(dz, [dz, r270, r90, i, r180, h, dn, v]);
		test_transform_from_state(dn, [dn, r90, r270, r180, i, v, dz, h]);
		test_transform_from_state(r90, [r90, dn, dz, v, h, r180, r270, i]);
		test_transform_from_state(r180, [r180, h, v, dn, dz, r270, i, r90]);
		test_transform_from_state(r270, [r270, dz, dn, h, v, i, r90, r180]);

		test_symmetry_index_transform(
			S_I__,
			[S_I__, S_V__, S_H__, S_DZ_, S_DN_, S_90_, S_180, S_270],
		);
		test_symmetry_index_transform(
			S_V__,
			[S_V__, S_I__, S_180, S_90_, S_270, S_DZ_, S_H__, S_DN_],
		);
		test_symmetry_index_transform(
			S_H__,
			[S_H__, S_180, S_I__, S_270, S_90_, S_DN_, S_V__, S_DZ_],
		);
		test_symmetry_index_transform(
			S_DZ_,
			[S_DZ_, S_270, S_90_, S_I__, S_180, S_H__, S_DN_, S_V__],
		);
		test_symmetry_index_transform(
			S_DN_,
			[S_DN_, S_90_, S_270, S_180, S_I__, S_V__, S_DZ_, S_H__],
		);
		test_symmetry_index_transform(
			S_90_,
			[S_90_, S_DN_, S_DZ_, S_V__, S_H__, S_180, S_270, S_I__],
		);
		test_symmetry_index_transform(
			S_180,
			[S_180, S_H__, S_V__, S_DN_, S_DZ_, S_270, S_I__, S_90_],
		);
		test_symmetry_index_transform(
			S_270,
			[S_270, S_DZ_, S_DN_, S_H__, S_V__, S_I__, S_90_, S_180],
		);

		let src = 616_101_616;
		let i = board_from_hash(src);
		assert_eq!(hash(TRANSFORMERS[S_I__](i)), src);
		assert_eq!(hash(TRANSFORMERS[S_V__](i)), src);
		assert_eq!(hash(TRANSFORMERS[S_H__](i)), src);
		assert_eq!(hash(TRANSFORMERS[S_DZ_](i)), src);
		assert_eq!(hash(TRANSFORMERS[S_DN_](i)), src);
		assert_eq!(hash(TRANSFORMERS[S_90_](i)), src);
		assert_eq!(hash(TRANSFORMERS[S_180](i)), src);
		assert_eq!(hash(TRANSFORMERS[S_270](i)), src);
	}

	#[test]
	fn test_canonical() {
		let board = board_from_hash(616_101_616);
		let c = canonical(board);
		assert_eq!(c.1, 0);
		assert_eq!(hash(c.0), 616_101_616);
	}

	#[test]
	fn test_play_single_move() {
		let board = board_from_hash(616101616);
		let neighbors_buf = Vec::<(BoardIndex, DiceValue, Board)>::from([
			(C_L_, 1, empty_cell_mask(C_L_)),
			(C_T_, 1, empty_cell_mask(C_T_)),
		]);
		let mut queue: Queue = HashMap::new();
		let mut moved = false;
		let mut spc: SymmetryPathCount = [0; SYMMETRY_COUNT as usize];
		spc[0] = 1;

		let b = board;
		play_single_move!(b, C_M_, spc, queue, moved, neighbors_buf, 0, 1);

		assert_eq!(queue.len(), 1);
		assert!(moved);
		let first = queue.iter().next().unwrap();
		assert_eq!(hash(*first.0), 606021616);
		assert_eq!(*first.1, spc);
	}

	#[test]
	fn test_solve() {
		assert_eq!(solve(20, board_from_hash(60222161)), 322444322);
		assert_eq!(solve(20, board_from_hash(506450064)), 951223336);
		assert_eq!(solve(1, board_from_hash(555005555)), 36379286);
		assert_eq!(solve(1, board_from_hash(616101616)), 264239762);
		assert_eq!(solve(24, board_from_hash(300362102)), 661168294);
		assert_eq!(solve(36, board_from_hash(604202400)), 350917228);
		assert_eq!(solve(32, board_from_hash(54105)), 999653138);
		assert_eq!(solve(40, board_from_hash(4024134)), 521112022);
		assert_eq!(solve(40, board_from_hash(54030030)), 667094338);
		assert_eq!(solve(20, board_from_hash(51000401)), 738691369);
		assert_eq!(solve(20, board_from_hash(100352100)), 808014757);
	}

	#[test]
	fn test_solve_empty() {
		let board = board_from_hash(0);
		assert_eq!(solve(0, board), 0);
		assert_eq!(solve(1, board), 111111111);
		assert_eq!(solve(2, board), 704035952);
		assert_eq!(solve(3, board), 840352818);
		assert_eq!(solve(4, board), 600875666);
		assert_eq!(solve(5, board), 50441886);
		assert_eq!(solve(6, board), 680243700);
		assert_eq!(solve(7, board), 597686656);
		assert_eq!(solve(8, board), 584450980);
		assert_eq!(solve(9, board), 55305380);
		assert_eq!(solve(10, board), 193520836);
		assert_eq!(solve(11, board), 521847116);
		assert_eq!(solve(12, board), 1054388152);
		assert_eq!(solve(13, board), 518795448);
		assert_eq!(solve(14, board), 366207036);
		assert_eq!(solve(15, board), 678967952);
		assert_eq!(solve(16, board), 476916052);
		assert_eq!(solve(17, board), 1009258340);
		assert_eq!(solve(18, board), 592651828);
		assert_eq!(solve(19, board), 1063467872);
		assert_eq!(solve(20, board), 400415524);
		assert_eq!(solve(21, board), 233248832);
		assert_eq!(solve(22, board), 230461008);
		assert_eq!(solve(23, board), 245411624);
		assert_eq!(solve(24, board), 899694236);
		assert_eq!(solve(25, board), 384163740);
		assert_eq!(solve(26, board), 888060600);
		assert_eq!(solve(27, board), 347933640);
		assert_eq!(solve(28, board), 340717612);
		assert_eq!(solve(29, board), 73295296);
		assert_eq!(solve(30, board), 851289228);
		assert_eq!(solve(31, board), 221286388);
		assert_eq!(solve(32, board), 375032784);
		assert_eq!(solve(33, board), 723342020);
		assert_eq!(solve(34, board), 92414440);
		assert_eq!(solve(35, board), 745533092);
		assert_eq!(solve(36, board), 331519112);
		assert_eq!(solve(37, board), 993643868);
		assert_eq!(solve(38, board), 72093236);
		assert_eq!(solve(39, board), 422667876);
		assert_eq!(solve(DEPTH_MAX, board), 503115192);
	}
}
