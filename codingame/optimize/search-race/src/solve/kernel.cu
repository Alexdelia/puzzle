typedef signed char int8_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define MAX_STEP 600
#define CROSSING_LIST_SIZE 3
#define CHECKPOINT_RADIUS 600.0
#define MAX_DISTANCE 18357.6
#define FRICTION 0.85
#define EPSILON 0.00001
#define PI 3.141592653589793
#define TAU 6.283185307179586
#define INF_F64 __longlong_as_double(0x7ff0000000000000ULL)
#define NAN_F64 __longlong_as_double(0x7ff8000000000000ULL)

struct Step {
	int8_t tilt;
	uint8_t thrust;
};

struct Coord {
	double x;
	double y;
};

struct Car {
	double x;
	double y;
	double sx;
	double sy;
	double angle;
};

struct FrozenPrefix {
	uint64_t resume_from_step;
	Car car;
	uint64_t checkpoint_index;
	uint64_t reentry_step_count;
};

struct Solution {
	Step steps[MAX_STEP];
	uint16_t len;
};

struct SimOutput {
	uint32_t finished;
	float score;
	uint32_t step_count;
	uint32_t reached_checkpoint_count;
	double turn_to_finish;
	FrozenPrefix frozen;
};

struct Slot {
	FrozenPrefix value;
	int present;
};

__device__ inline double truncate_(double x) {
	double rounded = round(x);
	if (fabs(rounded - x) < EPSILON) return rounded;
	return trunc(x);
}

__device__ inline double dist_sq(double ax, double ay, double bx, double by) {
	double dx = ax - bx;
	double dy = ay - by;
	return dx * dx + dy * dy;
}

__device__ inline double dist_(double ax, double ay, double bx, double by) {
	return sqrt(dist_sq(ax, ay, bx, by));
}

__device__ Coord process_step(Car* car, Step step) {
	double new_angle_deg = car->angle * (180.0 / PI) + (double)step.tilt;
	car->angle = new_angle_deg * (PI / 180.0);

	double sin_v, cos_v;
	sincos(car->angle, &sin_v, &cos_v);

	car->sx += cos_v * (double)step.thrust;
	car->sy += sin_v * (double)step.thrust;

	car->x += car->sx;
	car->y += car->sy;

	Coord moved_to = {car->x, car->y};

	car->x = truncate_(car->x);
	car->y = truncate_(car->y);
	car->sx = truncate_(car->sx * FRICTION);
	car->sy = truncate_(car->sy * FRICTION);

	double degrees = round(car->angle * (180.0 / PI));
	car->angle = degrees * (PI / 180.0);
	while (car->angle > TAU) car->angle -= TAU;
	while (car->angle < 0.0) car->angle += TAU;

	return moved_to;
}

__device__ bool intersect_(Coord checkpoint, Coord from, Coord to) {
	if (dist_(from.x, from.y, checkpoint.x, checkpoint.y) <= CHECKPOINT_RADIUS) return true;

	double vx = to.x - from.x;
	double vy = to.y - from.y;
	double fx = from.x - checkpoint.x;
	double fy = from.y - checkpoint.y;

	double a = vx * vx + vy * vy;
	if (a <= 0.0) return false;

	double b = 2.0 * (fx * vx + fy * vy);
	double c = fx * fx + fy * fy - CHECKPOINT_RADIUS * CHECKPOINT_RADIUS;
	double discr = b * b - 4.0 * a * c;
	if (discr < 0.0) return false;

	double t = (-b - sqrt(discr)) / (2.0 * a);
	return t > 0.0 && t <= 1.0;
}

__device__ double checkpoint_entry_fraction(Coord from, Coord to, Coord checkpoint) {
	double dx = to.x - from.x;
	double dy = to.y - from.y;
	double fx = from.x - checkpoint.x;
	double fy = from.y - checkpoint.y;

	double a = dx * dx + dy * dy;
	if (a == 0.0) return 0.0;

	double b = 2.0 * (fx * dx + fy * dy);
	double c = fx * fx + fy * fy - CHECKPOINT_RADIUS * CHECKPOINT_RADIUS;
	double discr = b * b - 4.0 * a * c;
	if (discr < 0.0) return 1.0;

	double t = (-b - sqrt(discr)) / (2.0 * a);
	if (t < 0.0) return 0.0;
	if (t > 1.0) return 1.0;
	return t;
}

__device__ float get_score(int checkpoint_count, int reached_count,
                           double closest, int step_count, bool finished, double ttf) {
	if (reached_count == checkpoint_count) {
		double ttf_val = finished ? ttf : (double)step_count;
		return (float)ttf_val - (float)MAX_STEP;
	}
	int remaining = checkpoint_count - reached_count - 1;
	return (float)remaining * (float)MAX_DISTANCE + (float)closest;
}

__device__ SimOutput simulate_one(
	const Solution* solution,
	const FrozenPrefix* frozen_in,
	const Coord* checkpoints,
	int checkpoint_count,
	Car car_init,
	int step_to_checkpoint_limit
) {
	bool resuming = frozen_in->resume_from_step > 0;
	Car car = resuming ? frozen_in->car : car_init;
	uint64_t checkpoint_index = resuming ? frozen_in->checkpoint_index : 0;

	uint64_t reached_at_step = frozen_in->resume_from_step == 0 ? 0 : frozen_in->resume_from_step - 1;
	uint64_t window_start = reached_at_step;
	int window_len = step_to_checkpoint_limit;

	double closest_to_checkpoint = INF_F64;

	Slot crossing_list[CROSSING_LIST_SIZE];
	for (int k = 0; k < CROSSING_LIST_SIZE; ++k) crossing_list[k].present = 0;

	uint64_t solution_len = (uint64_t)solution->len;
	uint64_t start = frozen_in->resume_from_step;

	for (uint64_t step_index = start; step_index < solution_len; ++step_index) {
		Step step = solution->steps[step_index];
		Coord from = {car.x, car.y};
		Coord moved_to = process_step(&car, step);

		if (window_start + (uint64_t)window_len < step_index) break;

		Coord current_checkpoint = checkpoints[checkpoint_index];

		double d = dist_(car.x, car.y, current_checkpoint.x, current_checkpoint.y);
		if (d < closest_to_checkpoint) closest_to_checkpoint = d;

		if (intersect_(current_checkpoint, from, moved_to)) {
			uint64_t crossed_segment_step_count = step_index - reached_at_step;

			for (int k = CROSSING_LIST_SIZE - 1; k > 0; --k) {
				crossing_list[k] = crossing_list[k - 1];
			}
			if (crossing_list[1].present) {
				crossing_list[1].value.reentry_step_count = crossed_segment_step_count;
			}
			crossing_list[0].value.resume_from_step = step_index + 1;
			crossing_list[0].value.car = car;
			crossing_list[0].value.checkpoint_index = checkpoint_index + 1;
			crossing_list[0].value.reentry_step_count = 0;
			crossing_list[0].present = 1;

			reached_at_step = step_index;
			checkpoint_index += 1;
			closest_to_checkpoint = INF_F64;

			window_start = step_index;
			window_len = step_to_checkpoint_limit;

			if ((int)checkpoint_index == checkpoint_count) {
				uint64_t step_count = step_index + 1;
				double entry_t = checkpoint_entry_fraction(from, moved_to, current_checkpoint);
				double ttf = (double)step_index + entry_t;

				SimOutput out;
				out.finished = 1;
				out.step_count = (uint32_t)step_count;
				out.reached_checkpoint_count = (uint32_t)checkpoint_index;
				out.turn_to_finish = ttf;
				out.score = get_score(checkpoint_count, (int)checkpoint_index, closest_to_checkpoint, (int)step_count, true, ttf);
				out.frozen = crossing_list[CROSSING_LIST_SIZE - 1].present
					? crossing_list[CROSSING_LIST_SIZE - 1].value
					: *frozen_in;
				return out;
			}
		}
	}

	uint64_t step_count = solution_len;
	if (isinf(closest_to_checkpoint)) {
		Coord current_checkpoint = checkpoints[checkpoint_index];
		closest_to_checkpoint = dist_(car.x, car.y, current_checkpoint.x, current_checkpoint.y);
	}

	SimOutput out;
	out.finished = 0;
	out.step_count = (uint32_t)step_count;
	out.reached_checkpoint_count = (uint32_t)checkpoint_index;
	out.turn_to_finish = NAN_F64;
	out.frozen = crossing_list[CROSSING_LIST_SIZE - 1].present
		? crossing_list[CROSSING_LIST_SIZE - 1].value
		: *frozen_in;
	out.score = get_score(checkpoint_count, (int)checkpoint_index, closest_to_checkpoint, (int)step_count, false, 0.0);
	return out;
}

extern "C" __global__ void simulate(
	const Solution* __restrict__ solutions,
	const FrozenPrefix* __restrict__ frozens,
	const Coord* __restrict__ checkpoints,
	int checkpoint_count,
	Car car_init,
	int step_to_checkpoint_limit,
	SimOutput* __restrict__ outputs,
	int n
) {
	int i = blockIdx.x * blockDim.x + threadIdx.x;
	if (i >= n) return;
	outputs[i] = simulate_one(
		&solutions[i],
		&frozens[i],
		checkpoints,
		checkpoint_count,
		car_init,
		step_to_checkpoint_limit
	);
}
