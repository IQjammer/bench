use clap::Parser;
use nix::sched::{CpuSet, sched_setaffinity};
use nix::unistd::Pid;
use std::process::{self, Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, default_value_t = 10)]
    rc: u32,
    #[arg(short, default_value_t = 5)]
    wp: u32,
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
    if args.rc == 0 {
        eprintln!("rc must be greater than 0");
        process::exit(0);
    }
    let mut result = Vec::with_capacity(args.rc as usize);
    for _ in 0..args.wp {
        Command::new(&args.bin)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("failed to warmup");
    }
    for _ in 0..args.rc {
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

    let res = test(&args);

    println!(
        "Result: {:?}",
        res.iter().sum::<Duration>() / res.len() as u32
    );
}
