#![allow(non_camel_case_types)]
#![allow(raw_pointer_derive)]
extern crate libc;
use libc::{c_int,c_char,c_uint,c_void,uint64_t,size_t,sockaddr,sockaddr_in6,int64_t,ssize_t,addrinfo,c_long,sockaddr_in};
#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
mod variable_types {
    use libc::c_uchar;
    use winapi::{ULONG,SOCKET,HANDLE};
    // undocumented
    pub type uv_uid_t = c_uchar;
    pub type uv_gid_t = c_uchar;

    // misc.rst
    #[repr(C)]
    #[derive(Debug,Clone,Copy)]
    pub struct uv_buf_t {
        pub len: ULONG,
        pub base: *mut c_char,
    }

    pub type uv_os_sock_t = SOCKET;
    pub type uv_os_fd_t = HANDLE;
}

#[cfg(not(windows))]
mod variable_types {
    use libc::{c_int, uid_t, gid_t, c_char, size_t};
    // undocumented
    pub type uv_uid_t = uid_t;
    pub type uv_gid_t = gid_t;

    // misc.rst
    #[repr(C)]
    #[derive(Debug,Clone,Copy)]
    pub struct uv_buf_t {
        pub base: *mut c_char,
        pub len: size_t,
    }

    pub type uv_os_sock_t = c_int;
    pub type uv_os_fd_t = c_int;
}

pub use variable_types::*;

// undocumented

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct uv_timeval_t {
    pub tv_sec: c_long,
    pub tv_usec: c_long,
}

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum uv_handle_type {
    UV_UNKNOWN_HANDLE = 0,
    UV_ASYNC = 1,
    UV_CHECK = 2,
    UV_FS_EVENT = 3,
    UV_FS_POLL = 4,
    UV_HANDLE = 5,
    UV_IDLE = 6,
    UV_NAMED_PIPE = 7,
    UV_POLL = 8,
    UV_PREPARE = 9,
    UV_PROCESS = 10,
    UV_STREAM = 11,
    UV_TCP = 12,
    UV_TIMER = 13,
    UV_TTY = 14,
    UV_UDP = 15,
    UV_SIGNAL = 16,
    UV_FILE = 17,
    UV_HANDLE_TYPE_MAX = 18,
}
pub use uv_handle_type::*;

// errors.rst

// TODO UV_E* (scrape uv_errno_t from the preprocessor output?)
extern {
    pub fn uv_strerror(err: c_int) -> *const c_char;
    pub fn uv_err_name(err: c_int) -> *const c_char;
}

// version.rst

// TODO expose version macros?
extern {
    pub fn uv_version() -> c_uint;
    pub fn uv_version_string() -> *const c_char;
}

// loop.rst

#[repr(C)]
pub struct uv_loop_t {
    pub data: *mut c_void,
    _private_fields: [u8; 0],
}

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum uv_run_mode {
    UV_RUN_DEFAULT = 0,
    UV_RUN_ONCE = 1,
    UV_RUN_NOWAIT = 2,
}
pub use uv_run_mode::*;

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum uv_loop_option {
    UV_LOOP_BLOCK_SIGNAL = 0,
    dummy = 99, // https://github.com/rust-lang/rust/issues/10292
}
pub use uv_loop_option::UV_LOOP_BLOCK_SIGNAL;

pub type uv_walk_cb = extern "C" fn (*mut uv_handle_t, *mut c_void);

extern {
    pub fn uv_loop_init(loop_: *mut uv_loop_t) -> c_int;
    pub fn uv_loop_configure(loop_: *mut uv_loop_t, option: uv_loop_option, ...) -> c_int;
    pub fn uv_loop_close(loop_: *mut uv_loop_t) -> c_int;
    pub fn uv_default_loop() -> *mut uv_loop_t;
    pub fn uv_run(loop_: *mut uv_loop_t, mode: uv_run_mode) -> c_int;
    pub fn uv_loop_alive(loop_: *mut uv_loop_t) -> c_int;
    pub fn uv_stop(loop_: *mut uv_loop_t);
    pub fn uv_loop_size() -> size_t;
    pub fn uv_backend_fd(loop_: *const uv_loop_t) -> c_int;
    pub fn uv_backend_timeout(loop_: *const uv_loop_t) -> c_int;
    pub fn uv_now(loop_: *const uv_loop_t) -> uint64_t;
    pub fn uv_update_time(loop_: *mut uv_loop_t);
    pub fn uv_walk(loop_: *mut uv_loop_t, walk_cb: uv_walk_cb, arg: *mut c_void);
}

// handle.rst

#[repr(C)]
pub struct uv_handle_t {
    pub data: *mut c_void,
    pub loop_: *mut uv_loop_t, // readonly
    pub type_: uv_handle_type, // readonly
    _private_fields: [u8; 0],
}

pub type uv_alloc_cb = extern "C" fn(*mut uv_handle_t, size_t, *mut uv_buf_t);
pub type uv_close_cb = extern "C" fn(*mut uv_handle_t);

extern {
    pub fn uv_is_active(handle: *const uv_handle_t) -> c_int;
    pub fn uv_is_closing(handle: *const uv_handle_t) -> c_int;
    pub fn uv_close(handle: *mut uv_handle_t, close_cb: uv_close_cb);
    pub fn uv_ref(handle: *mut uv_handle_t);
    pub fn uv_unref(handle: *mut uv_handle_t);
    pub fn uv_has_ref(handle: *const uv_handle_t) -> c_int;
    pub fn uv_handle_size(type_: uv_handle_type) -> size_t;
    pub fn uv_send_buffer_size(handle: *mut uv_handle_t, value: *mut c_int) -> c_int;
    pub fn uv_recv_buffer_size(handle: *mut uv_handle_t, value: *mut c_int) -> c_int;
    pub fn uv_fileno(handle: *const uv_handle_t, fd: *mut uv_os_fd_t) -> c_int;
}

// request.rst

#[repr(C)]
pub struct uv_req_t {
    pub data: *mut c_void,
    pub type_: uv_req_type, // readonly
    _private_fields: [u8; 0],
}

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum uv_req_type {
    UV_UNKNOWN_REQ = 0,
    UV_REQ = 1,
    UV_CONNECT = 2,
    UV_WRITE = 3,
    UV_SHUTDOWN = 4,
    UV_UDP_SEND = 5,
    UV_FS = 6,
    UV_WORK = 7,
    UV_GETADDRINFO = 8,
    UV_GETNAMEINFO = 9,
    UV_REQ_TYPE_PRIVATE = 10,
    UV_REQ_TYPE_MAX = 11,
}
pub use uv_req_type::*;

extern {
    pub fn uv_cancel(req: *mut uv_req_t) -> c_int;
    pub fn uv_req_size(type_: uv_req_type) -> size_t;
}

// timer.rst

pub type uv_timer_t = uv_handle_t;
pub type uv_timer_cb = extern "C" fn(*mut uv_timer_t);

extern {
    pub fn uv_timer_init(loop_: *mut uv_loop_t, handle: *mut uv_timer_t) -> c_int;
    pub fn uv_timer_start(handle: *mut uv_timer_t, cb: uv_timer_cb,
        timeout: uint64_t, repeat: uint64_t) -> c_int;
    pub fn uv_timer_stop(handle: *mut uv_timer_t) -> c_int;
    pub fn uv_timer_again(handle: *mut uv_timer_t) -> c_int;
    pub fn uv_timer_set_repeat(handle: *mut uv_timer_t, repeat: uint64_t);
    pub fn uv_timer_get_repeat(handle: *const uv_timer_t) -> uint64_t;
}

// prepare.rst

pub type uv_prepare_t = uv_handle_t;
pub type uv_prepare_cb = extern "C" fn(*mut uv_prepare_t);

extern {
    pub fn uv_prepare_init(loop_: *mut uv_loop_t, prepare: *mut uv_prepare_t) -> c_int;
    pub fn uv_prepare_start(prepare: *mut uv_prepare_t, cb: uv_prepare_cb) -> c_int;
    pub fn uv_prepare_stop(prepare: *mut uv_prepare_t) -> c_int;
}

// check.rst

pub type uv_check_t = uv_handle_t;
pub type uv_check_cb = extern "C" fn(*mut uv_check_t);

extern {
    pub fn uv_check_init(loop_: *mut uv_loop_t, check: *mut uv_check_t) -> c_int;
    pub fn uv_check_start(check: *mut uv_check_t, cb: uv_check_cb) -> c_int;
    pub fn uv_check_stop(check: *mut uv_check_t) -> c_int;
}

// idle.rst

pub type uv_idle_t = uv_handle_t;
pub type uv_idle_cb = extern "C" fn(*mut uv_idle_t);

extern {
    pub fn uv_idle_init(loop_: *mut uv_loop_t, idle: *mut uv_idle_t) -> c_int;
    pub fn uv_idle_start(idle: *mut uv_idle_t, cb: uv_idle_cb) -> c_int;
    pub fn uv_idle_stop(idle: *mut uv_idle_t) -> c_int;
}

// async.rst

pub type uv_async_t = uv_handle_t;
pub type uv_async_cb = extern "C" fn(*mut uv_async_t);

extern {
    pub fn uv_async_init(loop_: *mut uv_loop_t, async: *mut uv_async_t, async_cb: uv_async_cb) -> c_int;
    pub fn uv_async_send(async: *mut uv_async_t) -> c_int;
}

// poll.rst

pub type uv_poll_t = uv_handle_t;
pub type uv_poll_cb = extern "C" fn(*mut uv_poll_t, c_int, c_int);

#[repr(C)]
pub enum uv_poll_event {
    UV_READABLE = 1,
    UV_WRITABLE = 2,
}
pub use uv_poll_event::*;

extern {
    pub fn uv_poll_init(loop_: *mut uv_loop_t, handle: *mut uv_poll_t, fd: c_int) -> c_int;
    pub fn uv_poll_init_socket(loop_: *mut uv_loop_t, handle: *mut uv_poll_t, socket: uv_os_sock_t) -> c_int;
    pub fn uv_poll_start(handle: *mut uv_poll_t, events: c_int, cb: uv_poll_cb) -> c_int;
    pub fn uv_poll_stop(poll: *mut uv_poll_t) -> c_int;
}

// signal.rst

pub type uv_signal_t = uv_handle_t; // TODO expose signum
pub type uv_signal_cb = extern "C" fn (*mut uv_signal_t, c_int);

extern {
    pub fn uv_signal_init(loop_: *mut uv_loop_t, signal: *mut uv_signal_t) -> c_int;
    pub fn uv_signal_start(signal: *mut uv_signal_t, cb: uv_signal_cb, signum: c_int) -> c_int;
    pub fn uv_signal_stop(signal: *mut uv_signal_t) -> c_int;
}

// process.rst

pub type uv_process_t = uv_handle_t; // TODO expose pid
pub type uv_exit_cb = extern "C" fn (*mut uv_process_t, int64_t, c_int);

#[repr(C)]
#[derive(Debug,Clone,Copy)]
pub struct uv_process_options_t {
    pub exit_cb: uv_exit_cb,
    pub file: *const c_char,
    pub args: *mut *mut c_char,
    pub env: *mut *mut c_char,
    pub cwd: *const c_char,
    pub flags: c_uint,
    pub stdio_count: c_int,
    pub stdio: *mut uv_stdio_container_t,
    pub uid: uv_uid_t,
    pub gid: uv_gid_t,
}

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum uv_process_flags {
    UV_PROCESS_SETUID = (1 << 0),
    UV_PROCESS_SETGID = (1 << 1),
    UV_PROCESS_WINDOWS_VERBATIM_ARGUMENTS = (1 << 2),
    UV_PROCESS_DETACHED = (1 << 3),
    UV_PROCESS_WINDOWS_HIDE = (1 << 4),
}

#[repr(C)]
#[derive(Clone,Copy)]
pub struct uv_stdio_container_t {
    pub flags: uv_stdio_flags,
    pub stream: *mut uv_stream_t, // UNION
}

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum uv_stdio_flags {
    UV_IGNORE = 0x00,
    UV_CREATE_PIPE = 0x01,
    UV_INHERIT_FD = 0x02,
    UV_INHERIT_STREAM = 0x04,
    UV_READABLE_PIPE = 0x10,
    UV_WRITABLE_PIPE = 0x20,
}

extern {
    pub fn uv_disable_stdio_inheritance();
    pub fn uv_spawn(loop_: *mut uv_loop_t, handle: *mut uv_process_t, options: *const uv_process_options_t) -> c_int;
    pub fn uv_process_kill(handle: *mut uv_process_t, signum: c_int) -> c_int;
    pub fn uv_kill(pid: c_int, signum: c_int) -> c_int;
}

// stream.rst

pub type uv_stream_t = uv_handle_t; // TODO write_queue_size
pub type uv_connect_t = uv_req_t; // TODO handle
pub type uv_shutdown_t = uv_req_t; // TODO handle
pub type uv_write_t = uv_req_t; // TODO handle, send_handle
pub type uv_read_cb = extern "C" fn(*mut uv_stream_t, ssize_t, *const uv_buf_t);
pub type uv_write_cb = extern "C" fn(*mut uv_write_t, c_int);
pub type uv_connect_cb = extern "C" fn(*mut uv_connect_t, c_int);
pub type uv_shutdown_cb = extern "C" fn(*mut uv_shutdown_t, c_int);
pub type uv_connection_cb = extern "C" fn(*mut uv_stream_t, c_int);

extern {
    pub fn uv_shutdown(req: *mut uv_shutdown_t, handle: *mut uv_stream_t, cb: uv_shutdown_cb) -> c_int;
    pub fn uv_listen(stream: *mut uv_stream_t, backlog: c_int, cb: uv_connection_cb) -> c_int;
    pub fn uv_accept(server: *mut uv_stream_t, client: *mut uv_stream_t) -> c_int;
    pub fn uv_read_start(stream: *mut uv_stream_t, alloc_cb: uv_alloc_cb, read_cb: uv_read_cb) -> c_int;
    pub fn uv_read_stop(stream: *mut uv_stream_t) -> c_int;
    pub fn uv_write(req: *mut uv_write_t, handle: *mut uv_stream_t, bufs: *const uv_buf_t, nbufs: c_uint, cb: uv_write_cb) -> c_int;
    pub fn uv_write2(req: *mut uv_write_t, handle: *mut uv_stream_t, bufs: *const uv_buf_t, nbufs: c_uint, send_handle: *mut uv_stream_t, cb: uv_write_cb) -> c_int;
    pub fn uv_try_write(handle: *mut uv_stream_t, bufs: *const uv_buf_t, nbufs: c_uint) -> c_int;
    pub fn uv_is_readable(handle: *const uv_stream_t) -> c_int;
    pub fn uv_is_writable(handle: *const uv_stream_t) -> c_int;
    pub fn uv_stream_set_blocking(handle: *mut uv_stream_t, blocking: c_int) -> c_int;
}

// tcp.rst

pub type uv_tcp_t = uv_handle_t; // TODO stream fields

extern {
    pub fn uv_tcp_init(loop_: *mut uv_loop_t, handle: *mut uv_tcp_t) -> c_int;
    pub fn uv_tcp_init_ex(loop_: *mut uv_loop_t, handle: *mut uv_tcp_t, flags: c_uint) -> c_int;
    pub fn uv_tcp_open(handle: *mut uv_tcp_t, sock: uv_os_sock_t) -> c_int;
    pub fn uv_tcp_nodelay(handle: *mut uv_tcp_t, enable: c_int) -> c_int;
    pub fn uv_tcp_keepalive(handle: *mut uv_tcp_t, enable: c_int, delay: c_uint) -> c_int;
    pub fn uv_tcp_simultaneous_accepts(handle: *mut uv_tcp_t, enable: c_int) -> c_int;
    pub fn uv_tcp_bind(handle: *mut uv_tcp_t, addr: *const sockaddr, flags: c_uint) -> c_int;
    pub fn uv_tcp_getsockname(handle: *const uv_tcp_t, name: *mut sockaddr, namelen: *mut c_int) -> c_int;
    pub fn uv_tcp_getpeername(handle: *const uv_tcp_t, name: *mut sockaddr, namelen: *mut c_int) -> c_int;
    pub fn uv_tcp_connect(req: *mut uv_connect_t, handle: *mut uv_tcp_t, addr: *const sockaddr, cb: uv_connect_cb) -> c_int;
}

// pipe.rst

pub type uv_pipe_t = uv_handle_t; // TODO stream fields

extern {
    pub fn uv_pipe_init(loop_: *mut uv_loop_t, handle: *mut uv_pipe_t, ipc: c_int) -> c_int;
    pub fn uv_pipe_open(handle: *mut uv_pipe_t, file: uv_file) -> c_int;
    pub fn uv_pipe_bind(handle: *mut uv_pipe_t, name: *const c_char) -> c_int;
    pub fn uv_pipe_connect(req: *mut uv_connect_t, handle: *mut uv_pipe_t, name: *const c_char, cb: uv_connect_cb);
    pub fn uv_pipe_getsockname(handle: *const uv_pipe_t, buffer: *mut c_char, size: *mut size_t) -> c_int;
    pub fn uv_pipe_getpeername(handle: *const uv_pipe_t, buffer: *mut c_char, size: *mut size_t) -> c_int;
    pub fn uv_pipe_pending_instances(handle: *mut uv_pipe_t, count: c_int);
    pub fn uv_pipe_pending_count(handle: *mut uv_pipe_t) -> c_int;
    pub fn uv_pipe_pending_type(handle:  *mut uv_pipe_t) -> uv_handle_type;
}

// tty.rst

pub type uv_tty_t = uv_handle_t; // TODO stream fields
#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum uv_tty_mode_t {
    UV_TTY_MODE_NORMAL = 0,
    UV_TTY_MODE_RAW = 1,
    UV_TTY_MODE_IO = 2,
}
pub use uv_tty_mode_t::*;

extern {
    pub fn uv_tty_init(loop_: *mut uv_loop_t, handle: *mut uv_tty_t, fd: uv_file, readable: c_int) -> c_int;
    pub fn uv_tty_set_mode(handle: *mut uv_tty_t, mode: uv_tty_mode_t) -> c_int;
    pub fn uv_tty_reset_mode() -> c_int;
    pub fn uv_tty_get_winsize(handle: *mut uv_tty_t, width: *mut c_int, height: *mut c_int) -> c_int;
}

// udp.rst

pub type uv_udp_t = uv_handle_t; // TODO send_queue_size, send_queue_count
pub type uv_udp_send_t = uv_req_t; // TODO handle
#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum uv_udp_flags {
    UV_UDP_IPV6ONLY = 1,
    UV_UDP_PARTIAL = 2,
    UV_UDP_REUSEADDR = 4,
}
pub use uv_udp_flags::*;
pub type uv_udp_send_cb = extern "C" fn(*mut uv_udp_send_t, c_int);
pub type uv_udp_recv_cb = extern "C" fn(*mut uv_udp_t, ssize_t, *const uv_buf_t, *const sockaddr, c_uint);
#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum uv_membership {
    UV_LEAVE_GROUP = 0,
    UV_JOIN_GROUP = 1,
}
pub use uv_membership::*;

extern {
    pub fn uv_udp_init(loop_: *mut uv_loop_t, handle: *mut uv_udp_t) -> c_int;
    pub fn uv_udp_init_ex(loop_: *mut uv_loop_t, handle: *mut uv_udp_t, flags: c_uint) -> c_int;
    pub fn uv_udp_open(handle: *mut uv_udp_t, sock: uv_os_sock_t) -> c_int;
    pub fn uv_udp_bind(handle: *mut uv_udp_t, addr: *const sockaddr, flags: c_uint) -> c_int;
    pub fn uv_udp_getsockname(handle: *const uv_udp_t, name: *mut sockaddr, namelen: *mut c_int) -> c_int;
    pub fn uv_udp_set_membership(handle: *mut uv_udp_t, multicast_addr: *const c_char, interface_addr: *const c_char, membership: uv_membership) -> c_int;
    pub fn uv_udp_set_multicast_loop(handle: *mut uv_udp_t, on: c_int) -> c_int;
    pub fn uv_udp_set_multicast_ttl(handle: *mut uv_udp_t, ttl: c_int) -> c_int;
    pub fn uv_udp_set_multicast_interface(handle: *mut uv_udp_t, interface_addr: *const c_char) -> c_int;
    pub fn uv_udp_set_broadcast(handle: *mut uv_udp_t, on: c_int) -> c_int;
    pub fn uv_udp_set_ttl(handle: *mut uv_udp_t, ttl: c_int) -> c_int;
    pub fn uv_udp_send(req: *mut uv_udp_send_t, handle: *mut uv_udp_t, bufs: *const uv_buf_t, nbufs: c_uint, addr: *const sockaddr, send_cb: uv_udp_send_cb) -> c_int;
    pub fn uv_udp_try_send(handle: *mut uv_udp_t, bufs: *const uv_buf_t, nbufs: c_uint, addr: *const sockaddr) -> c_int;
    pub fn uv_udp_recv_start(handle: *mut uv_udp_t, alloc_cb: uv_alloc_cb, recv_cb: uv_udp_recv_cb) -> c_int;
    pub fn uv_udp_recv_stop(handle: *mut uv_udp_t) -> c_int;
}

// fs_event.rst

pub type uv_fs_event_t = uv_handle_t;
pub type uv_fs_event_cb = extern "C" fn(*mut uv_fs_event_t, *const c_char, c_int, c_int);
#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum uv_fs_event {
    UV_RENAME = 1,
    UV_CHANGE = 2,
}
pub use uv_fs_event::*;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
#[repr(C)]
pub enum uv_fs_event_flags {
    UV_FS_EVENT_WATCH_ENTRY = 1,
    UV_FS_EVENT_STAT = 2,
    UV_FS_EVENT_RECURSIVE = 4,
}
pub use uv_fs_event_flags::*;

extern {
    pub fn uv_fs_event_init(loop_: *mut uv_loop_t, handle: *mut uv_fs_event_t) -> c_int;
    pub fn uv_fs_event_start(handle: *mut uv_fs_event_t, cb: uv_fs_event_cb, path: *const c_char, flags: c_uint) -> c_int;
    pub fn uv_fs_event_stop(handle: *mut uv_fs_event_t) -> c_int;
    pub fn uv_fs_event_getpath(handle: *mut uv_fs_event_t, buffer: *mut c_char, size: *mut size_t) -> c_int;
}

// fs_poll.rst

pub type uv_fs_poll_t = uv_handle_t;
pub type uv_fs_poll_cb = extern "C" fn(*mut uv_fs_poll_t, c_int, *const uv_stat_t, *const uv_stat_t);

extern {
    pub fn uv_fs_poll_init(loop_: *mut uv_loop_t, handle: *mut uv_fs_poll_t) -> c_int;
    pub fn uv_fs_poll_start(handle: *mut uv_fs_poll_t, poll_cb: uv_fs_poll_cb, path: *const c_char, interval: c_uint) -> c_int;
    pub fn uv_fs_poll_stop(handle: *mut uv_fs_poll_t) -> c_int;
    pub fn uv_fs_poll_getpath(handle: *mut uv_fs_poll_t, buffer: *mut c_char, size: *mut size_t) -> c_int;
}

// fs.rst

pub type uv_fs_t = uv_req_t; // TODO loop, fs_type, path, result, statbuf, ptr

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct uv_timespec_t {
    pub tv_sec: c_long,
    pub tv_nsec: c_long,
}

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct uv_stat_t {
    pub st_dev: uint64_t,
    pub st_mode: uint64_t,
    pub st_nlink: uint64_t,
    pub st_uid: uint64_t,
    pub st_gid: uint64_t,
    pub st_rdev: uint64_t,
    pub st_ino: uint64_t,
    pub st_size: uint64_t,
    pub st_blksize: uint64_t,
    pub st_blocks: uint64_t,
    pub st_flags: uint64_t,
    pub st_gen: uint64_t,
    pub st_atim: uv_timespec_t,
    pub st_mtim: uv_timespec_t,
    pub st_ctim: uv_timespec_t,
    pub st_birthtim: uv_timespec_t,
}

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum uv_fs_type {
    UV_FS_UNKNOWN = -1,
    UV_FS_CUSTOM = 0,
    UV_FS_OPEN = 1,
    UV_FS_CLOSE = 2,
    UV_FS_READ = 3,
    UV_FS_WRITE = 4,
    UV_FS_SENDFILE = 5,
    UV_FS_STAT = 6,
    UV_FS_LSTAT = 7,
    UV_FS_FSTAT = 8,
    UV_FS_FTRUNCATE = 9,
    UV_FS_UTIME = 10,
    UV_FS_FUTIME = 11,
    UV_FS_ACCESS = 12,
    UV_FS_CHMOD = 13,
    UV_FS_FCHMOD = 14,
    UV_FS_FSYNC = 15,
    UV_FS_FDATASYNC = 16,
    UV_FS_UNLINK = 17,
    UV_FS_RMDIR = 18,
    UV_FS_MKDIR = 19,
    UV_FS_MKDTEMP = 20,
    UV_FS_RENAME = 21,
    UV_FS_SCANDIR = 22,
    UV_FS_LINK = 23,
    UV_FS_SYMLINK = 24,
    UV_FS_READLINK = 25,
    UV_FS_CHOWN = 26,
    UV_FS_FCHOWN = 27,
}
pub use uv_fs_type::*;

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum uv_dirent_type_t {
    UV_DIRENT_UNKNOWN = 0,
    UV_DIRENT_FILE = 1,
    UV_DIRENT_DIR = 2,
    UV_DIRENT_LINK = 3,
    UV_DIRENT_FIFO = 4,
    UV_DIRENT_SOCKET = 5,
    UV_DIRENT_CHAR = 6,
    UV_DIRENT_BLOCK = 7,
}
pub use uv_dirent_type_t::*;

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct uv_dirent_t {
    pub name: *const c_char,
    pub type_: uv_dirent_type_t,
}

pub const UV_FS_SYMLINK_DIR : c_int = 0x0001;
pub const UV_FS_SYMLINK_JUNCTION : c_int = 0x0002;

pub type uv_fs_cb = Option<extern "C" fn(*mut uv_fs_t)>;

extern {
    pub fn uv_fs_req_cleanup(req: *mut uv_fs_t);
    pub fn uv_fs_close(loop_: *mut uv_loop_t, req: *mut uv_fs_t, file: uv_file, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_open(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, flags: c_int, mode: c_int, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_read(loop_: *mut uv_loop_t, req: *mut uv_fs_t, file: uv_file, bufs: *const uv_buf_t, nbufs: c_uint, offset: int64_t, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_unlink(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_write(loop_: *mut uv_loop_t, req: *mut uv_fs_t, file: uv_file, bufs: *const uv_buf_t, nbufs: c_uint, offset: int64_t, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_mkdir(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, mode: c_int, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_mkdtemp(loop_: *mut uv_loop_t, req: *mut uv_fs_t, tpl: *const c_char, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_rmdir(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_scandir(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, flags: c_int, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_scandir_next(req: *mut uv_fs_t, ent: *mut uv_dirent_t) -> c_int;
    pub fn uv_fs_stat(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_fstat(loop_: *mut uv_loop_t, req: *mut uv_fs_t, file: uv_file, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_lstat(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_rename(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, new_path: *const c_char, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_fsync(loop_: *mut uv_loop_t, req: *mut uv_fs_t, file: uv_file, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_fdatasync(loop_: *mut uv_loop_t, req: *mut uv_fs_t, file: uv_file, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_ftruncate(loop_: *mut uv_loop_t, req: *mut uv_fs_t, file: uv_file, offset: int64_t, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_sendfile(loop_: *mut uv_loop_t, req: *mut uv_fs_t, out_fd: uv_file, in_fd: uv_file, in_offset: int64_t, length: size_t, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_access(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, mode: c_int, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_chmod(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, mode: c_int, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_fchmod(loop_: *mut uv_loop_t, req: *mut uv_fs_t, file: uv_file, mode: c_int, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_utime(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, atime: f64, mtime: f64, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_futime(loop_: *mut uv_loop_t, req: *mut uv_fs_t, file: uv_file, atime: f64, mtime: f64, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_link(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, new_path: *const c_char, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_symlink(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, new_path: *const c_char, flags: c_int, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_readlink(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_chown(loop_: *mut uv_loop_t, req: *mut uv_fs_t, path: *const c_char, uid: uv_uid_t, gid: uv_gid_t, cb: uv_fs_cb) -> c_int;
    pub fn uv_fs_fchown(loop_: *mut uv_loop_t, req: *mut uv_fs_t, file: uv_file, uid: uv_uid_t, gid: uv_gid_t, cb: uv_fs_cb) -> c_int;
}

// threadpool.rst

pub type uv_work_t = uv_req_t; // TODO loop
pub type uv_work_cb = extern "C" fn(*mut uv_work_t);
pub type uv_after_work_cb = extern "C" fn(*mut uv_work_t, c_int);

extern {
    pub fn uv_queue_work(loop_: *mut uv_loop_t, req: *mut uv_work_t, work_cb: uv_work_cb, after_work_cb: uv_after_work_cb) -> c_int;
}

// dns.rst

pub type uv_getaddrinfo_t = uv_req_t; // TODO loop, addrinfo
pub type uv_getaddrinfo_cb = Option<extern "C" fn(*mut uv_getaddrinfo_t, c_int, *mut addrinfo)>;
pub type uv_getnameinfo_t = uv_req_t; // TODO loop, host, service
pub type uv_getnameinfo_cb = Option<extern "C" fn(*mut uv_getnameinfo_t, c_int, *const c_char, *const c_char)>;

extern {
    pub fn uv_getaddrinfo(loop_: *mut uv_loop_t, req: *mut uv_getaddrinfo_t, getaddrinfo_cb: uv_getaddrinfo_cb, node: *const c_char, service: *const c_char, hints: *const addrinfo) -> c_int;
    pub fn uv_freeaddrint(ai: *mut addrinfo);
    pub fn uv_getnameinfo(loop_: *mut uv_loop_t, req: *mut uv_getnameinfo_t, getnameinfo_cb: uv_getnameinfo_cb, addr: *const sockaddr, flags: c_int) -> c_int;
}

// dll.rst

#[repr(C)]
pub struct uv_lib_t {
    _handle: *mut c_void, // HMODULE is a typedef for void*, so this works on both platforms
    _errmst: *mut c_char,
}

extern {
    pub fn uv_dlopen(filename: *const c_char, lib: *mut uv_lib_t) -> c_int;
    pub fn uv_dlclose(lib: *mut uv_lib_t);
    pub fn uv_dlsym(lib: *mut uv_lib_t, name: *const c_char, ptr: *mut *mut c_void) -> c_int;
    pub fn uv_dlerror(lib: *const uv_lib_t) -> *const c_char;
}

// threading.rst

// Not bound because the types differ widely between platforms and it's thoroughly redundant with native Rust threading

// misc.rst

pub type uv_malloc_func = extern "C" fn(size_t) -> *mut c_void;
pub type uv_realloc_func = extern "C" fn(*mut c_void, size_t) -> *mut c_void;
pub type uv_calloc_func = extern "C" fn(size_t, size_t) -> *mut c_void;
pub type uv_free_func = extern "C" fn(*mut c_void);

pub type uv_file = c_int;

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct uv_rusage_t {
    pub ru_utime: uv_timeval_t,
    pub ru_stime: uv_timeval_t,
    pub ru_maxrss: uint64_t,
    pub ru_ixrss: uint64_t,
    pub ru_idrss: uint64_t,
    pub ru_isrss: uint64_t,
    pub ru_minflt: uint64_t,
    pub ru_majflt: uint64_t,
    pub ru_nswap: uint64_t,
    pub ru_inblock: uint64_t,
    pub ru_oublock: uint64_t,
    pub ru_msgsnd: uint64_t,
    pub ru_msgrcv: uint64_t,
    pub ru_nsignals: uint64_t,
    pub ru_nvcsw: uint64_t,
    pub ru_nivcsw: uint64_t,
}

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct uv_cpu_times_t {
    pub user: uint64_t,
    pub nice: uint64_t,
    pub sys: uint64_t,
    pub idle: uint64_t,
    pub irq: uint64_t,
}

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct uv_cpu_info_t {
    pub model: *mut c_char,
    pub speed: c_int,
    pub cpu_times: uv_cpu_times_t,
}

#[repr(C)]
#[derive(Clone,Copy)]
pub struct uv_interface_address_t {
    pub name: *mut c_char,
    pub phys_addr: [c_char; 6],
    pub is_internal: c_int,
    pub u_address: sockaddr_in6, // UNION
    pub u_netmask: sockaddr_in6, // UNION
}

extern {
    pub fn uv_guess_handle(file: uv_file) -> uv_handle_type;
    pub fn uv_replace_allocator(malloc_func: uv_malloc_func, realloc_func: uv_realloc_func, calloc_func: uv_calloc_func, free_func: uv_free_func) -> c_int;
    pub fn uv_buf_init(base: *mut c_char, len: c_uint) -> uv_buf_t;
    pub fn uv_setup_args(argc: c_int, argv: *mut *mut c_char) -> *mut *mut c_char;
    pub fn uv_get_process_title(buffer: *mut c_char, size: size_t) -> c_int;
    pub fn uv_set_process_title(title: *const c_char) -> c_int;
    pub fn uv_resident_set_memory(rss: *mut size_t) -> c_int;
    pub fn uv_uptime(uptime: *mut f64) -> c_int;
    pub fn uv_getrusage(rusage: *mut uv_rusage_t) -> c_int;
    pub fn uv_cpu_info(cpu_infos: *mut *mut uv_cpu_info_t, count: *mut c_int) -> c_int;
    pub fn uv_free_cpu_info(cpu_infos: *mut uv_cpu_info_t, count: c_int);
    pub fn uv_interface_addresses(addresses: *mut *mut uv_interface_address_t, count: *mut c_int) -> c_int;
    pub fn uv_free_interface_addresses(addresses: *mut uv_interface_address_t, count: c_int);
    pub fn uv_loadavg(avg: *mut [f64; 3]);
    pub fn uv_ip4_addr(ip: *const c_char, port: c_int, addr: *mut sockaddr_in) -> c_int;
    pub fn uv_ip6_addr(ip: *const c_char, port: c_int, addr: *mut sockaddr_in6) -> c_int;
    pub fn uv_ip4_name(src: *const sockaddr_in, dst: *mut c_char, size: size_t) -> c_int;
    pub fn uv_ip6_name(src: *const sockaddr_in6, dst: *mut c_char, size: size_t) -> c_int;
    pub fn uv_inet_ntop(af: c_int, src: *const c_void, dst: *mut c_char, size: size_t) -> c_int;
    pub fn uv_inet_pton(af: c_int, src: *const c_char, dst: *mut c_void) -> c_int;
    pub fn uv_exepath(buffer: *mut c_char, size: *mut size_t) -> c_int;
    pub fn uv_cwd(buffer: *mut c_char, size: *mut size_t) -> c_int;
    pub fn uv_chdir(dir: *const c_char) -> c_int;
    pub fn uv_os_homedir(buffer: *mut c_char, size: *mut size_t) -> c_int;
    pub fn uv_get_free_memory() -> uint64_t;
    pub fn uv_get_total_memory() -> uint64_t;
    pub fn uv_hrtime() -> uint64_t;
}

#[test]
fn smoke() {
    unsafe {
        assert!(uv_version() >= 0x10705);
    }
}
