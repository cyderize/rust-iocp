extern crate iocp;
extern crate libc;

use iocp::{IoCompletionPort, CompletionStatus};
use std::sync::TaskPool;
use std::os::num_cpus;
use std::io::timer::sleep;
use std::time::duration::Duration;
use std::rand::random;

fn main() {
	let iocp = IoCompletionPort::new(0).unwrap();
	
	let threads = num_cpus() * 2;
	let taskpool = TaskPool::new(threads);
	
	for i in range(0, threads) {
		let iocp_clone = iocp.clone();
		taskpool.execute(move || {
			loop {
				let status = iocp_clone.get_queued(libc::INFINITE).unwrap();
				println!("Dequeued: {} from {}", status.completion_key, i);
				sleep(Duration::milliseconds(1100));
			}
		});
	}
	
	loop {
		let status = CompletionStatus {
			byte_count: 100,
			completion_key: random(),
			overlapped: None
		};
		iocp.post_queued(status).unwrap();
		println!("Queued: {}", status.completion_key);
		sleep(Duration::seconds(1));
	}
}
