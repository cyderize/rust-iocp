//! Provides an I/O completion port for asynchronous operations.
//!
//! This crate is only available on Windows. See the example in ```examples/main.rs```.
//!
#![cfg(windows)]
#![allow(unstable)]

extern crate libc;

use std::{os, ptr, mem};
use std::result::Result;
use std::error::Error;
use std::sync::Arc;
use std::rt::heap;
use std::slice;

mod ffi;

/// Represents an I/O completion port.
pub struct IoCompletionPort {
	inner: Arc<IocpImp>
}

unsafe impl Send for IoCompletionPort { }
unsafe impl Sync for IoCompletionPort { }

impl Clone for IoCompletionPort {
	fn clone(&self) -> IoCompletionPort {
		IoCompletionPort {
			inner: self.inner.clone()
		}
	}
}

impl IoCompletionPort {
	/// Create a new IoCompletionPort with the specified number of concurrent threads.
	///
	/// If zero threads are specified, the system allows as many concurrently running
	/// threads as there are processors in the system.
	pub fn new(concurrent_threads: usize) -> IocpResult<IoCompletionPort> {
		Ok(IoCompletionPort {
			inner: Arc::new(try!(IocpImp::new(concurrent_threads)))
		})
	}
	/// Assoicates the given file handle with this IoCompletionPort.
	///
	/// The completion key is included in every I/O completion packet for the specified file handle.
	pub fn associate(&self, handle: libc::HANDLE, completion_key: usize) -> IocpResult<()> {
		self.inner.associate(handle, completion_key)
	}
	/// Attempts to dequeue an I/O completion packet from the IoCompletionPort.
	pub fn get_queued(&self, timeout: u32) -> IocpResult<CompletionStatus> {
		self.inner.get_queued(timeout)
	}
	/// Attempts to dequeue multiple I/O completion packets from the IoCompletionPort simultaneously.
	///
	/// Returns the number of CompletionStatus objects dequeued.
	pub fn get_many_queued(&self, buf: &mut [CompletionStatus], timeout: u32) -> IocpResult<usize> {
		self.inner.get_many_queued(buf, timeout)
	}
	/// Posts an I/O completion packet to the IoCompletionPort.
	///
	/// Note that the OVERLAPPED structure in the CompletionStatus does not have to be valid (it can be a null pointer).
	/// Ensure that if you intend to post an OVERLAPPED structure, it is not freed until the CompletionStatus is dequeued.
	pub fn post_queued(&self, packet: CompletionStatus) -> IocpResult<()> {
		self.inner.post_queued(packet)
	}
}

/// Represents an I/O completion status packet
pub struct CompletionStatus {
	/// The number of bytes transferred during the operation
	pub byte_count: usize,
	/// The completion key associated with this packet
	pub completion_key: usize,
	/// A pointer to the overlapped structure which may or may not be valid
	pub overlapped: *mut libc::OVERLAPPED
}

impl CompletionStatus {
	/// Creates a new CompletionStatus
	pub fn new() -> CompletionStatus {
		CompletionStatus {
			byte_count: 0,
			completion_key: 0,
			overlapped: ptr::null_mut()
		}
	}
}

impl Copy for CompletionStatus { }

struct IocpImp {
	inner: libc::HANDLE
}

impl IocpImp {
	pub fn new(concurrent_threads: usize) -> IocpResult<IocpImp> {
		let handle = unsafe { ffi::CreateIoCompletionPort(libc::INVALID_HANDLE_VALUE, ptr::null_mut(), 0, concurrent_threads as libc::DWORD) };
		
		if handle.is_null() {
			return Err(
				IocpError::HostError(os::last_os_error())
			);
		}
		
		Ok(IocpImp {
			inner: handle
		})
	}
	pub fn associate(&self, handle: libc::HANDLE, completion_key: usize) -> IocpResult<()> {
		let handle = unsafe { ffi::CreateIoCompletionPort(handle, self.inner, completion_key as ffi::ULONG_PTR, 0) };
		
		if handle.is_null() {
			return Err(
				IocpError::HostError(os::last_os_error())
			);
		}
		
		Ok(())
	}
	pub fn get_queued(&self, timeout: u32) -> IocpResult<CompletionStatus> {
		let mut length: ffi::DWORD = 0;
		let mut key: ffi::ULONG_PTR = 0;
		let mut overlapped = ptr::null_mut();

		let queued = unsafe { ffi::GetQueuedCompletionStatus(self.inner, &mut length, &mut key, &mut overlapped, timeout) };
		
		if queued == 0 {
			return Err(
				IocpError::GetQueuedError(os::last_os_error(), overlapped)
			);
		}
		
		Ok(CompletionStatus {
			byte_count: length as usize,
			completion_key: key as usize,
			overlapped: overlapped
		})
	}
	pub fn get_many_queued(&self, buf: &mut [CompletionStatus], timeout: u32) -> IocpResult<usize> {
		let allocation = unsafe { heap::allocate(buf.len() * mem::size_of::<ffi::OVERLAPPED_ENTRY>(), mem::align_of::<ffi::OVERLAPPED_ENTRY>()) };
		
		let ptr: *mut ffi::OVERLAPPED_ENTRY = unsafe { mem::transmute(allocation) };
		let mut removed = 0;
		
		let queued = unsafe { ffi::GetQueuedCompletionStatusEx(self.inner, ptr, buf.len() as ffi::DWORD, &mut removed, timeout, 0) };
		
		if queued == 0 {
			return Err(
				IocpError::HostError(os::last_os_error())
			);
		}
		
		let entries = unsafe { slice::from_raw_mut_buf(&ptr, buf.len()) };
		
		for (status, entry) in buf.iter_mut().zip(entries.iter()).take(removed as usize) {
			*status = CompletionStatus {
				byte_count: entry.dwNumberOfBytesTransferred as usize,
				completion_key: entry.lpCompletionKey as usize,
				overlapped: entry.lpOverlapped
			};
		}
		
		Ok(removed as usize)
	}
	pub fn post_queued(&self, packet: CompletionStatus) -> IocpResult<()> {
		let posted = unsafe {
			ffi::PostQueuedCompletionStatus(
				self.inner,
				packet.byte_count as libc::DWORD,
				packet.completion_key as ffi::ULONG_PTR,
				packet.overlapped
			)
		};
		
		if posted == 0 {
			return Err(
				IocpError::HostError(os::last_os_error())
			);
		}
		
		Ok(())
	}
}

impl Drop for IocpImp {
	fn drop(&mut self) {
		unsafe { let _ = libc::CloseHandle(self.inner); }
	}
}

pub type IocpResult<T> = Result<T, IocpError>;

#[allow(raw_pointer_derive)]
#[derive(Show)]
pub enum IocpError {
	GetQueuedError(String, *mut libc::OVERLAPPED),
	HostError(String)
}

unsafe impl Send for IocpError { }

impl Error for IocpError {
    fn description(&self) -> &str {
		match *self {
			IocpError::GetQueuedError(_, _) => "Call to GetQueuedCompletionStatus failed",
			IocpError::HostError(_) => "Call to function failed"
		}
	}
}