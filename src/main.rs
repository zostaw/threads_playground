use rayon::prelude::*;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn fibonacci(num: i128) -> i128 {
    match num {
        0 => 0,
        1 | 2 => 1,
        _ => fibonacci(num - 1) + fibonacci(num - 2),
    }
}

#[allow(dead_code)]
fn fibonacci_atomic_threaded(_num: i128) -> i128 {
    todo!();
}

fn fibonacci_atomic_rayon(num: i128) -> i128 {
    match num {
        0 => 0,
        1 | 2 => 1,
        _ => {
            let prevs = vec![num - 1, num - 2];
            let result = prevs
                .par_iter()
                .map(|x| fibonacci_atomic_rayon(*x))
                .sum::<i128>();
            return result;
        }
    }
}

fn sequential<F>(f: F, gen_num: i128, times: usize)
where
    F: FnOnce(i128) -> i128 + std::marker::Send + 'static + Copy,
{
    let total_start = Instant::now();
    let mut results = Vec::new();
    let mut durations: Vec<Duration> = Vec::new();

    for _ in 0..times {
        let start = Instant::now();
        results.push(f(gen_num));
        durations.push(start.elapsed());
    }
    let total_cpu_time: Duration = durations.iter().map(|&d| d).sum();
    let average_duration = total_cpu_time / durations.len() as u32;
    println!(
        "f({})={}, average time: {:?}",
        gen_num,
        results
            .first()
            .expect("No result of sequential calculated."),
        average_duration
    );
    println!("Total cpu time: {:?}", total_cpu_time);
    println!("Total time: {:?}\n", total_start.elapsed());
}

fn threaded<F>(f: F, gen_num: i128, times: usize)
where
    F: FnOnce(i128) -> i128 + std::marker::Send + 'static + Copy,
{
    let total_start = Instant::now();
    let mut results: Vec<i128> = Vec::new();
    let mut durations: Vec<Duration> = Vec::new();
    let mut handles = Vec::new();

    let (sender, receiver) = mpsc::channel();

    for _ in 0..times {
        let sender_clone = sender.clone();
        handles.push(thread::spawn(move || {
            let start = Instant::now();
            let result = f(gen_num);
            let duration = start.elapsed();
            sender_clone.send((result, duration)).unwrap();
        }));
    }

    for handle in handles {
        let _ = handle.join().unwrap();
    }
    drop(sender);

    for received in receiver {
        match received {
            rslt => {
                results.push(rslt.0);
                durations.push(rslt.1);
            }
        };
    }

    let total_cpu_time: Duration = durations.iter().map(|&d| d).sum();
    let average_duration = total_cpu_time / durations.len() as u32;
    println!(
        "f({})={}, average time: {:?}",
        gen_num,
        results.first().expect("No result of threaded calculated."),
        average_duration
    );
    println!("Total cpu time: {:?}", total_cpu_time);
    println!("Total time: {:?}\n", total_start.elapsed());
}

fn parallel<F>(f: F, gen_num: i128, times: usize)
where
    F: FnOnce(i128) -> i128 + std::marker::Send + std::marker::Sync + Clone,
{
    let total_start = Instant::now();
    let nums = vec![gen_num; times];
    let (results, durations): (Vec<i128>, Vec<Duration>) = nums
        .par_iter()
        .map(|&x| {
            let f_clone = f.clone();
            let start = Instant::now();
            let result = f_clone(x);
            let duration = start.elapsed();
            (result, duration)
        })
        .unzip();

    let total_cpu_time: Duration = durations.iter().map(|&d| d).sum();
    let average_duration = total_cpu_time / durations.len() as u32;
    println!(
        "f({})={}, average time: {:?}",
        gen_num,
        results.first().expect("No result of parallel calculated."),
        average_duration
    );

    println!("Total cpu time: {:?}", total_cpu_time);
    println!("Total time: {:?}\n", total_start.elapsed());
}

fn chatgpt_rayon_threads1() {
    // Create a vector of numbers from 1 to 1 million
    let numbers: Vec<u64> = (1..=1_000_000).collect();

    let toyme = Instant::now();
    // Sequential sum of squares
    let sequential_sum: u64 = numbers.iter().map(|&x| x * x).sum();
    println!(
        "Sequential Sum: {} executed in {:?}",
        sequential_sum,
        toyme.elapsed()
    );

    let toyme = Instant::now();
    // Parallel sum of squares using threads
    let thread_sum: u64 =
        {
            let chunk_size = numbers.len() / 2;
            let numbers_clone = numbers.clone();

            let handle1 =
                thread::spawn(move || {
                    numbers_clone[..chunk_size]
                        .iter()
                        .map(|&x| x * x)
                        .sum::<u64>()
                });

            let numbers_clone = numbers.clone();
            let handle2 =
                thread::spawn(move || {
                    numbers_clone[chunk_size..]
                        .iter()
                        .map(|&x| x * x)
                        .sum::<u64>()
                });

            let sum1 = handle1.join().unwrap();
            let sum2 = handle2.join().unwrap();

            sum1 + sum2
        };

    println!(
        "Thread Sum: {} executed in {:?}",
        thread_sum,
        toyme.elapsed()
    );
    let toyme = Instant::now();

    // Parallel sum of squares using rayon
    let parallel_sum: u64 = numbers.par_iter().map(|&x| x * x).sum();
    println!(
        "Rayon Sum: {} executed in {:?}\n",
        parallel_sum,
        toyme.elapsed()
    );
}

fn chatgpt_rayon_threads2() {
    // Create a large vector of numbers
    let numbers: Vec<u64> = (1..=1_000_000).collect();

    let toyme = Instant::now();
    // Sequential sum of numbers
    let sequential_sum: u64 = numbers.iter().sum();
    println!(
        "Sequential Sum: {} executed in {:?}",
        sequential_sum,
        toyme.elapsed()
    );

    let toyme = Instant::now();
    // Parallel sum of numbers using threads
    let thread_sum: u64 =
        {
            let chunk_size = numbers.len() / 2;
            let numbers_clone = numbers.clone();

            let handle1 = thread::spawn(move || numbers_clone[..chunk_size].iter().sum::<u64>());

            let numbers_clone = numbers.clone();

            let handle2 = thread::spawn(move || numbers_clone[chunk_size..].iter().sum::<u64>());

            let sum1 = handle1.join().unwrap();
            let sum2 = handle2.join().unwrap();

            sum1 + sum2
        };

    println!(
        "Thread Sum: {} executed in {:?}",
        thread_sum,
        toyme.elapsed()
    );

    let toyme = Instant::now();
    // Parallel sum of numbers using rayon
    let parallel_sum: u64 = numbers.par_iter().cloned().sum();
    println!(
        "Rayon Sum: {} executed in {:?}\n",
        parallel_sum,
        toyme.elapsed()
    );
}

fn main() {
    let gen_num = 38;
    let times = 64;

    println!(
        "__________________\nMultithreading benchmarks for fibonacci sequence (generation of {}th number in sequence), {} times in row...\n",
        gen_num,
        times
    );

    println!("Sequential fibonacci...");
    sequential(fibonacci, gen_num, times);

    println!("Threaded fibonacci...");
    threaded(fibonacci, gen_num, times);

    println!("Parallel fibonacci...");
    parallel(fibonacci, gen_num, times);

    println!("Parallel atomic fibonacci...");
    parallel(fibonacci_atomic_rayon, gen_num, times);

    println!("__________________\nDifferent benchmark, comparing atomic calculations between different approaches...\n");
    println!("Sum of squares of 10e6...");
    chatgpt_rayon_threads1();
    println!("Simple sum of 10e6...");
    chatgpt_rayon_threads2();
}
