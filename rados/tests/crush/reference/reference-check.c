/* Audit runner for the MSR, distribution and bad-mappings functional ports.
 * Link unmodified pinned Ceph mapper/hash/builder/crush.c; see verify-reference.py.
 * This reproduces test inputs, not the mapper algorithm.
 */
#include "mapper.h"
#include "builder.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define W 65536u
static struct crush_map map;
static struct crush_bucket_straw2 buckets[12];
static struct crush_bucket *bucket_ptrs[12];
static int items[12][24];
static unsigned bucket_weights[12][24];
static struct crush_rule *rule;
static void *work;

static int add_bucket(int type, int size, unsigned weight)
{
    int index = map.max_buckets++;
    buckets[index] = (struct crush_bucket_straw2){
        .h = {.id = -1-index, .type = type, .alg = 5, .hash = 0,
              .size = size, .weight = size*weight, .items = items[index]},
        .item_weights = bucket_weights[index]};
    bucket_ptrs[index] = &buckets[index].h;
    for (int i = 0; i < size; ++i) bucket_weights[index][i] = weight;
    return index;
}

static void init_work(int count)
{
    map.buckets = bucket_ptrs;
    map.max_rules = 1;
    map.rules = &rule;
    map.working_size = sizeof(struct crush_work)
        + map.max_buckets * (sizeof(struct crush_work_bucket *)
                            + sizeof(struct crush_work_bucket));
    for (int i = 0; i < map.max_buckets; ++i)
        map.working_size += buckets[i].h.size * sizeof(unsigned);
    work = calloc(1, crush_work_size(&map, count));
    assert(work);
    crush_init_workspace(&map, work);
}

#ifndef QUINCY
static void place(unsigned x, int count, const unsigned *weights, int *out)
{
    int n = crush_do_rule(&map, 0, x, out, count, weights, map.max_devices, work, NULL);
    printf("ORACLE devices=%d x=%u count=%d weights=[", map.max_devices, x, count);
    for (int i = 0; i < map.max_devices; ++i) printf("%s%u", i ? ", " : "", weights[i]);
    printf("] out=[");
    for (int i = 0; i < n; ++i) printf("%s%d", i ? ", " : "", out[i]);
    puts("]");
    assert(n == count);
}

static void host_out(unsigned *weights, int osd, int per_host)
{
    int first = osd/per_host*per_host;
    for (int i = first; i < first+per_host; ++i) weights[i] = 0;
}

static void msr(int roots, int hosts, int osds, int choose_hosts, int choose_osds, int count)
{
    map = (struct crush_map){.max_devices = roots*hosts*osds,
        .choose_total_tries = 50, .chooseleaf_descend_once = 1,
        .chooseleaf_vary_r = 1, .chooseleaf_stable = 1,
        .msr_descents = 100, .msr_collision_tries = 100};
    rule = calloc(1, crush_rule_size(roots*4));
    assert(rule);
    rule->type = 5;
    rule->len = roots*4;
    for (int root = 0; root < roots; ++root) {
        int r = add_bucket(2, hosts, osds*W);
        for (int host = 0; host < hosts; ++host) {
            int h = add_bucket(1, osds, W);
            items[r][host] = -1-h;
            for (int i = 0; i < osds; ++i) items[h][i] = (root*hosts+host)*osds+i;
        }
        rule->steps[root*4] = (struct crush_rule_step){1, -1-r, 0};
        rule->steps[root*4+1] = (struct crush_rule_step){16, choose_hosts, 1};
        rule->steps[root*4+2] = (struct crush_rule_step){16, choose_osds, 0};
        rule->steps[root*4+3] = (struct crush_rule_step){4, 0, 0};
    }
    init_work(count);
    for (unsigned x = 0; x < (roots == 2 ? 1000 : 1); ++x) {
        unsigned weights[24];
        int before[24], after[24];
        for (int i = 0; i < map.max_devices; ++i) weights[i] = W;
        place(x, count, weights, before);
        if (roots == 2) {
            weights[before[1]] = weights[before[5]] = 0;
            place(x, count, weights, after);
            for (int i = 0; i < map.max_devices; ++i) weights[i] = W;
            host_out(weights, before[2], osds);
            host_out(weights, before[6], osds);
        } else {
            host_out(weights, before[0], osds);
        }
        place(x, count, weights, after);
        if (roots == 1 && hosts == 4) {
            for (int i = 0; i < map.max_devices; ++i) weights[i] = W;
            weights[before[0]] = 0;
            place(x, count, weights, after);
        }
    }
    free(work);
    free(rule);
}
#endif

static void distribution(int count)
{
    map = (struct crush_map){.max_devices = 5, .choose_local_tries = 2,
        .choose_local_fallback_tries = 5, .choose_total_tries = 19};
    add_bucket(1, 5, W);
    buckets[0].h.weight = 41*W;
    for (int i = 0; i < 5; ++i) {
        items[0][i] = i;
        bucket_weights[0][i] = (i == 4 ? 1 : 10)*W;
    }
    rule = calloc(1, crush_rule_size(3));
    assert(rule);
    rule->type = 1;
    rule->len = 3;
    rule->steps[0] = (struct crush_rule_step){1, -1, 0};
    rule->steps[1] = (struct crush_rule_step){2, 0, 0};
    rule->steps[2] = (struct crush_rule_step){4, 0, 0};
    init_work(count);
    unsigned weights[] = {W, W, W, W, W};
    long counts[5] = {0};
    for (int x = 1; x <= 1000000; ++x) {
        int out[3];
        int n = crush_do_rule(&map, 0, x, out, count, weights, 5, work, NULL);
        for (int i = 0; i < n; ++i) { assert(out[i] >= 0 && out[i] < 5); ++counts[out[i]]; }
    }
    printf("COUNTS replicas=%d [%ld, %ld, %ld, %ld, %ld]\n", count,
           counts[0], counts[1], counts[2], counts[3], counts[4]);
    free(work);
    free(rule);
}

static void bad_mappings(void)
{
    /* Original bad-mappings.crushmap.txt, with CrushCompiler's legacy defaults. */
    struct crush_map *m = crush_create();
    assert(m);
    m->choose_local_tries = 2;
    m->choose_local_fallback_tries = 5;
    m->choose_total_tries = 19;
    m->chooseleaf_descend_once = 0;
    m->chooseleaf_vary_r = 0;
    m->chooseleaf_stable = 0;
    m->straw_calc_version = 0;
    int devices[] = {0, 1, 2, 3, 4};
    int item_weights[] = {W, W, W, W, W};
    struct crush_bucket_straw *root = crush_make_straw_bucket(m, 0, 1, 5,
                                                             devices, item_weights);
    assert(root);
    for (int i = 0; i < 5; ++i) assert(root->straws[i] == W);
    assert(crush_add_bucket(m, -1, &root->h, NULL) == 0);
    for (int id = 0; id < 2; ++id) {
        struct crush_rule *r = crush_make_rule(3, id == 0 ? 1 : 3);
        assert(r);
        crush_rule_set_step(r, 0, CRUSH_RULE_TAKE, -1, 0);
        crush_rule_set_step(r, 1, id == 0 ? CRUSH_RULE_CHOOSE_FIRSTN : CRUSH_RULE_CHOOSE_INDEP, 0, 0);
        crush_rule_set_step(r, 2, CRUSH_RULE_EMIT, 0, 0);
        assert(crush_add_rule(m, r, id) == id);
    }
    crush_finalize(m);
    void *workspace = calloc(1, crush_work_size(m, 10));
    assert(workspace);
    crush_init_workspace(m, workspace);
    unsigned weights[] = {W, W, W, W, W};
    for (int id = 0; id < 2; ++id) {
        int out[10];
        int n = crush_do_rule(m, id, 1, out, 10, weights, 5, workspace, NULL);
        printf("BAD_MAPPINGS rule=%d out=[", id);
        for (int i = 0; i < n; ++i) printf("%s%d", i ? ", " : "", out[i]);
        puts("]");
    }
    free(workspace);
    crush_destroy(m);
}

int main(void)
{
#ifndef QUINCY
    msr(1, 4, 3, 3, 1, 3);
    msr(1, 3, 2, 2, 2, 3);
    msr(1, 5, 4, 4, 4, 14);
    msr(2, 4, 3, 2, 2, 8);
#endif
    distribution(1);
    distribution(3);
    bad_mappings();
}
