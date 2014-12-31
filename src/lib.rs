//! Provides an I/O completion port for asynchronous operations.
//!
//! This crate is only available on Windows.
#![cfg(target_os = "windows")]

extern crate libc;

use std::{os, ptr, mem};
use std::result::Result;
use std::sync::Arc;

mod ffi;

/// Represents an I/O completion port.
pub struct IoCompletionPort {
	inner: libc::HANDLE
}

unsafe impl Sync for IoCompletionPort { }
unsafe impl Send for IoCompletionPort { }

impl IoCompletionPort {
	/// Create a new IoCompletionPort with the specified number of concurrent threads.
	///
	/// If zero threads are specified, the system allows as many concurrently running
	/// threads as there are processors in the system.
	///
	/// Returns an Arc containing the IoCompletionPort, allowing the port to be shared
	/// between threads.
	pub fn new(concurrent_threads: uint) -> Result<Arc<IoCompletionPort>, String> {
		let handle = unsafe { ffi::CreateIoCompletionPort(libc::INVALID_HANDLE_VALUE, ptr::null_mut(), 0, concurrent_threads as libc::DWORD) };
		
		if handle == ptr::null_mut() {
			return Err(os::last_os_error());
		}
		
		Ok(Arc::new(IoCompletionPort {
			inner: handle
		}))
	}
	/// Assoicates the given file handle with this IoCompletionPort.
	///
	/// The completion key is included in every I/O completion packet for the specified file handle.
	pub fn associate(&self, handle: libc::HANDLE, completion_key: uint) -> Result<(), String> {
		let handle = unsafe { ffi::CreateIoCompletionPort(handle, self.inner, completion_key as ffi::ULONG_PTR, 0) };
		
		if handle.is_null() {
			return Err(os::last_os_error());
		}
		
		Ok(())
	}
	/// Attempts to dequeue an I/O completion packet from the IoCompletionPort.
	pub fn get_queued(&self, timeout: u32) -> Result<CompletionStatus, String> {
		let mut length: ffi::DWORD = 0;
		let mut key: ffi::ULONG_PTR = 0;
		let mut overlapped = unsafe { mem::zeroed() };
		
		let queued = unsafe { ffi::GetQueuedCompletionStatus(self.inner, &mut length, &mut key, &mut overlapped, timeout) };
		
		if queued == 0 {
			return Err(os::last_os_error());
		}
		
		let overlapped_option = 
			if overlapped == ptr::null_mut() {
				None
			}
			else {
				Some(unsafe { *overlapped })
			};
		
		Ok(CompletionStatus {
			byte_count: length as uint,
			completion_key: key as uint,
			overlapped: overlapped_option
		})
	}
	/// Posts an I/O completion packet to the IoCompletionPort.
	pub fn post_queued(&self, packet: CompletionStatus) -> Result<(), String> {
		let posted = unsafe {
			ffi::PostQueuedCompletionStatus(
				self.inner,
				packet.byte_count as libc::DWORD,
				packet.completion_key as ffi::ULONG_PTR,
				match packet.overlapped {
					Some(mut overlapped) => &mut overlapped,
					None => ptr::null_mut(),
				}
			)
		};
		
		if posted == 0 {
			return Err(os::last_os_error());
		}
		
		Ok(())
	}
}

impl Drop for IoCompletionPort {
	fn drop(&mut self) {
		unsafe { let _ = libc::CloseHandle(self.inner); }
	}
}

/// Represents an I/O completion status packet
#[deriving(Copy)]
pub struct CompletionStatus {
	/// The number of bytes transferred during the operation
	pub byte_count: uint,
	/// The completion key associated with this packet
	pub completion_key: uint,
	/// The overlapped structure
	pub overlapped: Option<libc::OVERLAPPED>
}