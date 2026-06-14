typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef uint8_t Cell;
typedef uint8_t Tile;
typedef uint8_t RobotCount;
typedef uint16_t Score;
typedef uint16_t Turn;
typedef uint32_t SolutionCount;

#define MAP_AREA 190
#define UP 0
#define RIGHT 1
#define DOWN 2
#define LEFT 3
#define NONE 4
#define VOID 5
#define MAX_TURNS (MAP_AREA * 4)
#define VISITED_WORDS 12

struct Robot {
	Cell cell;
	Tile heading;
};

struct Solution {
	Tile arrow[MAP_AREA];
};

struct SimOutput {
	Score score;
	Turn game_length;
};

extern "C" __global__ __launch_bounds__(256, 4) void simulate(
	const Tile* __restrict__ base,
	const Cell* __restrict__ next_table,
	const Robot* __restrict__ robot_list,
	RobotCount robot_count,
	const Solution* __restrict__ solution_list,
	SimOutput* __restrict__ output_list,
	SolutionCount n
) {
	__shared__ Tile s_base[MAP_AREA];
	__shared__ Cell s_next[MAP_AREA * 4];

	for (int k = threadIdx.x; k < MAP_AREA; k += blockDim.x) {
		s_base[k] = base[k];
	}
	for (int k = threadIdx.x; k < MAP_AREA * 4; k += blockDim.x) {
		s_next[k] = next_table[k];
	}
	__syncthreads();

	SolutionCount i = blockIdx.x * blockDim.x + threadIdx.x;
	if (i >= n) {
		return;
	}

	const Tile* arrow = solution_list[i].arrow;

	Score total_score = 0;
	Turn game_length = 0;

	for (RobotCount r = 0; r < robot_count; ++r) {
		uint64_t visited[VISITED_WORDS];
		#pragma unroll
		for (int w = 0; w < VISITED_WORDS; ++w) {
			visited[w] = 0ULL;
		}

		Cell cell = robot_list[r].cell;
		Tile start_arrow = arrow[cell];
		Tile heading = (start_arrow != NONE) ? start_arrow : robot_list[r].heading;

		int state = cell * 4 + heading;
		visited[state >> 6] |= (1ULL << (state & 63));

		Turn turn = 0;
		for (;;) {
			++turn;
			cell = s_next[cell * 4 + heading];

			Tile tile = s_base[cell];
			if (tile == NONE) {
				tile = arrow[cell];
			}
			if (tile == VOID) {
				total_score += turn;
				break;
			}
			if (tile != NONE) {
				heading = tile;
			}

			state = cell * 4 + heading;
			uint64_t bit = 1ULL << (state & 63);
			if (visited[state >> 6] & bit) {
				total_score += turn;
				break;
			}
			visited[state >> 6] |= bit;

			if (turn >= MAX_TURNS) {
				total_score += turn;
				break;
			}
		}

		if (turn > game_length) {
			game_length = turn;
		}
	}

	output_list[i].score = total_score;
	output_list[i].game_length = game_length;
}
