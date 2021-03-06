use std::time::Instant;

use rand::Rng;

use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};

use evm_state::{
    types::{AccountState, Slot, H160 as Address},
    EvmState,
};

mod utils;

fn squashed_state_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("squashed_state");
    group.throughput(Throughput::Elements(1));
    const N_ACCOUNTS: usize = 1024;

    for (n_forks, squash_target) in vec![
        (0, None),
        (1, None),
        (10, None),
        (20, None),
        (40, None),
        (100, None),
        (100, Some(100)),
        (120, Some(100)),
        (140, Some(100)),
        (200, Some(100)),
        (240, Some(200)),
        (400, Some(200)),
        (800, Some(400)),
    ] {
        let slot = Slot::default();
        let mut state = EvmState::default();

        let accounts: Vec<(Address, AccountState)> =
            utils::unique_random_accounts().take(N_ACCOUNTS).collect();

        for (address, account) in accounts.iter().cloned() {
            state.set_account(address, account);
        }

        for slot in (slot + 1)..=n_forks {
            state.freeze();
            if squash_target == Some(slot) {
                state.squash();
            }
            state = state.try_fork(slot).expect("Unable to fork EVM state");
        }

        group.bench_with_input(
            BenchmarkId::new(
                "get_account",
                format!(" {} forks, squashed on {:?}", n_forks, squash_target),
            ),
            &n_forks,
            move |b, _| {
                let accounts = &accounts;
                let state = &state;

                b.iter_custom(move |iters| {
                    let start = Instant::now();

                    for idx in 0..iters {
                        let (address, account) = &accounts[idx as usize % accounts.len()];
                        black_box({
                            let acc = state.get_account(*address);
                            assert_eq!(acc.as_ref(), Some(account));
                            acc
                        });
                    }

                    start.elapsed()
                });
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("squash_time");
    group.sample_size(10);

    const ACCOUNTS_PER_SLOT: usize = 1;

    for squash_targets in vec![
        (None, 0),
        (None, 1),
        (None, 10),
        (None, 50),
        (None, 100),
        (Some(100), 101),
        (Some(100), 110),
        (Some(100), 150),
        (Some(100), 200),
        (None, 1000),
        (Some(1000), 2000),
        (None, 10000),
        (Some(5000), 10000),
        (Some(9000), 10000),
        (Some(100_000), 100_100),
    ] {
        group.bench_with_input(
            BenchmarkId::new(
                "squash",
                format!(
                    " first on {:?}, then {}",
                    squash_targets.0, squash_targets.1
                ),
            ),
            &squash_targets,
            move |b, (squash_target_1, squash_target_2)| {
                b.iter_with_large_setup(
                    || {
                        let slot = Slot::default();
                        let mut state = EvmState::default();

                        // repeat 1/3 of accounts from previous slots
                        let mut addresses = utils::AddrMixer::new(3);

                        for new_slot in (slot + 1)..=*squash_target_2 {
                            addresses.advance();
                            for _ in 0..ACCOUNTS_PER_SLOT {
                                let (address, account) =
                                    (addresses.some_addr(), utils::some_account());
                                state.set_account(address, account);
                            }

                            state.freeze();
                            if squash_target_1.as_ref() == Some(&new_slot) {
                                state.squash();
                            }
                            state = state.try_fork(new_slot).expect("Unable to fork EVM state");
                        }
                        state
                    },
                    move |mut state| {
                        state.freeze();
                        state.squash();
                    },
                );
            },
        );
    }

    group.finish();
}

fn large_squash_time_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_squash_time_bench");
    group.sample_size(10);

    for &(first_squash, target_squash, accounts_per_100k) in &[
        (100_000, 100_100, 10_000),
        (1_000_000, 1_000_100, 100),
        (1_000_000, 1_000_100, 1000),
        (1_000_000, 1_000_100, 10_000),
        (1_000_000, 1_000_100, 100_000),
    ] {
        assert!(first_squash <= target_squash);
        assert!(accounts_per_100k <= 100_000);

        group.bench_with_input(
            BenchmarkId::new(
                "large squash",
                format!(
                    " {} accounts per 100k slots, first on {}, then {}",
                    accounts_per_100k, first_squash, target_squash
                ),
            ),
            &(first_squash, target_squash),
            move |b, &(first_squash, target_squash)| {
                b.iter_batched_ref(
                    || {
                        let slot = Slot::default();
                        let mut state = EvmState::default();

                        // repeat 1/3 of accounts from previous slots
                        let mut addresses = utils::AddrMixer::new(3);
                        let mut rng = rand::thread_rng();

                        for new_slot in (slot + 1)..target_squash {
                            addresses.advance();
                            if rng.gen_ratio(accounts_per_100k, 100_000) {
                                let (address, account) =
                                    (addresses.some_addr(), utils::some_account());
                                state.set_account(address, account);
                            }

                            state.freeze();
                            if new_slot == first_squash {
                                state.squash();
                            }
                            state = state.try_fork(new_slot).expect("Unable to fork EVM state");
                        }
                        state
                    },
                    |state| {
                        state.freeze();
                        state.squash();
                    },
                    BatchSize::NumIterations(1),
                )
            },
        );
    }

    group.finish();
}

criterion_group!(
    squashed_state,
    squashed_state_bench,
    large_squash_time_bench
);
criterion_main!(squashed_state);
