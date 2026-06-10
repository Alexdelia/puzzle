#ifndef NH
#define NH 1
#endif

#ifndef NZ
#define NZ 1
#endif

#define PLAYER_MOVE     1000.0
#define ZOMBIE_MOVE     400.0
#define PLAYER_RANGE_SQ 4000000.0

__constant__ double c_player_init[2];
__constant__ double c_human_list_init[NH * 2];
__constant__ double c_zombie_list_init[NZ * 2];

extern "C" __global__ void simulate(
    const int simulation_count,
    const int max_turns,
    const double* __restrict__ solution,
    long long*    __restrict__ scores
#ifdef OUTPUT_STATE
  , double*    __restrict__ out_player
  , double*    __restrict__ out_zombie_list
  , int*       __restrict__ out_zombie_alive_list
  , int*       __restrict__ out_human_alive_list
  , long long* __restrict__ out_score_per_turn
#endif
) {
    const int tid = blockIdx.x * blockDim.x + threadIdx.x;
    if (tid >= simulation_count) return;

    double player_x = c_player_init[0];
    double player_y = c_player_init[1];

    double hx[NH], hy[NH];
    bool   h_alive[NH];
    #pragma unroll
    for (int i = 0; i < NH; i++) {
        hx[i] = c_human_list_init[i * 2 + 0];
        hy[i] = c_human_list_init[i * 2 + 1];
        h_alive[i] = true;
    }
    int alive_h = NH;

    double zx[NZ], zy[NZ];
    bool   z_alive[NZ];
    #pragma unroll
    for (int i = 0; i < NZ; i++) {
        zx[i] = c_zombie_list_init[i * 2 + 0];
        zy[i] = c_zombie_list_init[i * 2 + 1];
        z_alive[i] = true;
    }
    int alive_z = NZ;

    long long score = 0;
    bool game_over = false;

#ifdef OUTPUT_STATE
    {
        const long long player_base = (long long)tid * 2;
        out_player[player_base + 0] = player_x;
        out_player[player_base + 1] = player_y;
        #pragma unroll
        for (int i = 0; i < NZ; i++) {
            const long long zbase = ((long long)tid) * NZ + i;
            out_zombie_list[zbase * 2 + 0] = zx[i];
            out_zombie_list[zbase * 2 + 1] = zy[i];
            out_zombie_alive_list[zbase] = 1;
        }
        #pragma unroll
        for (int i = 0; i < NH; i++) {
            const long long hbase = ((long long)tid) * NH + i;
            out_human_alive_list[hbase] = 1;
        }
        out_score_per_turn[tid] = 0;
    }
#endif

    for (int turn = 0; turn < max_turns; turn++) {
        if (!game_over) {
            double nzx[NZ], nzy[NZ];

            for (int z = 0; z < NZ; z++) {
                if (!z_alive[z]) continue;
                const double pzx = zx[z];
                const double pzy = zy[z];

                double best_d2 = 1.0e300;
                double tx = 0.0, ty = 0.0;

                for (int h = 0; h < NH; h++) {
                    if (!h_alive[h]) continue;
                    const double ddx = hx[h] - pzx;
                    const double ddy = hy[h] - pzy;
                    const double d2  = ddx * ddx + ddy * ddy;
                    if (d2 < best_d2) {
                        best_d2 = d2;
                        tx = hx[h];
                        ty = hy[h];
                    }
                }
                {
                    const double ddx = player_x - pzx;
                    const double ddy = player_y - pzy;
                    const double d2  = ddx * ddx + ddy * ddy;
                    if (d2 < best_d2) {
                        best_d2 = d2;
                        tx = player_x;
                        ty = player_y;
                    }
                }

                const double dist = sqrt(best_d2);
                if (dist <= ZOMBIE_MOVE) {
                    nzx[z] = tx;
                    nzy[z] = ty;
                } else {
                    nzx[z] = trunc(pzx + (tx - pzx) * ZOMBIE_MOVE / dist);
                    nzy[z] = trunc(pzy + (ty - pzy) * ZOMBIE_MOVE / dist);
                }
            }
            for (int z = 0; z < NZ; z++) {
                if (z_alive[z]) {
                    zx[z] = nzx[z];
                    zy[z] = nzy[z];
                }
            }

            const long long action_base = (long long)turn * simulation_count * 2 + (long long)tid * 2;
            const double ax = solution[action_base + 0];
            const double ay = solution[action_base + 1];
            const double adx = ax - player_x;
            const double ady = ay - player_y;
            const double adist = sqrt(adx * adx + ady * ady);
            if (adist <= PLAYER_MOVE) {
                player_x = ax;
                player_y = ay;
            } else {
                player_x = trunc(player_x + adx * PLAYER_MOVE / adist);
                player_y = trunc(player_y + ady * PLAYER_MOVE / adist);
            }

            const long long base_score = (long long)alive_h * (long long)alive_h * 10LL;
            long long fib_prev = 1, fib_cur = 1;
            for (int z = 0; z < NZ; z++) {
                if (!z_alive[z]) continue;
                const double kdx = zx[z] - player_x;
                const double kdy = zy[z] - player_y;
                const double kd2 = kdx * kdx + kdy * kdy;
                if (kd2 <= PLAYER_RANGE_SQ) {
                    z_alive[z] = false;
                    alive_z--;
                    score += base_score * fib_cur;
                    const long long fib_next = fib_prev + fib_cur;
                    fib_prev = fib_cur;
                    fib_cur  = fib_next;
                }
            }

            for (int z = 0; z < NZ; z++) {
                if (!z_alive[z]) continue;
                for (int h = 0; h < NH; h++) {
                    if (!h_alive[h]) continue;
                    if (zx[z] == hx[h] && zy[z] == hy[h]) {
                        h_alive[h] = false;
                        alive_h--;
                        break;
                    }
                }
            }

            if (alive_h == 0) {
                score = 0;
                game_over = true;
            } else if (alive_z == 0) {
                game_over = true;
            }
        }

#ifdef OUTPUT_STATE
        {
            const long long t = (long long)(turn + 1);
            const long long player_base = t * simulation_count * 2 + (long long)tid * 2;
            out_player[player_base + 0] = player_x;
            out_player[player_base + 1] = player_y;
            #pragma unroll
            for (int i = 0; i < NZ; i++) {
                const long long zbase = (t * simulation_count + tid) * NZ + i;
                out_zombie_list[zbase * 2 + 0] = zx[i];
                out_zombie_list[zbase * 2 + 1] = zy[i];
                out_zombie_alive_list[zbase] = z_alive[i] ? 1 : 0;
            }
            #pragma unroll
            for (int i = 0; i < NH; i++) {
                const long long hbase = (t * simulation_count + tid) * NH + i;
                out_human_alive_list[hbase] = h_alive[i] ? 1 : 0;
            }
            out_score_per_turn[t * simulation_count + tid] = score;
        }
#endif
    }

    scores[tid] = score;
}
