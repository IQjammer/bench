use clap::Parser;
use nix::sched::{CpuSet, sched_setaffinity};
use nix::unistd::Pid;
use std::process::{self, Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, default_value_t = 10)]
    runs: u32,
    #[arg(short, default_value_t = 5)]
    warmup: u32,
    #[arg(short, default_value_t = 2)]
    core: usize,

    bin: String,
}

fn set_core_affinity(core: usize) {
    let mut cpuset = CpuSet::new();
    cpuset.set(core).expect("core index out of range");
    sched_setaffinity(Pid::from_raw(0), &cpuset).expect("failed to set CPU affinity");
}

fn test(args: &Args) -> Vec<Duration> {
    if args.runs == 0 {
        eprintln!("runs must be greater than 0");
        process::exit(0);
    }
    let mut result = Vec::with_capacity(args.runs as usize);
    for _ in 0..args.warmup {
        Command::new(&args.bin)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("failed to warmup");
    }
    for _ in 0..args.runs {
        let now = Instant::now();
        let status = Command::new(&args.bin)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("failed to start");
        result.push(now.elapsed());

        if !status.success() {
            eprintln!("Process exited with status: {}", status);
        }
    }
    result
}

fn main() {
    let args = Args::parse();
    set_core_affinity(args.core);

    let mut res = test(&args);
    res.sort_unstable();
    let mid = res.len() /2;

    let result = if res.len().is_multiple_of(2) {
        (res[mid -1] + res[mid])/2
    } else {
        res[mid]
    };

    println!(
        "Result: {:?}",
        result
    );
}
