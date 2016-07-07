// Copyright (c) 2015 gdbwire crate developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

extern crate gdbwire_sys;
use gdbwire_sys::*;

use std::ffi::{CStr, CString};
use std::os::raw;

pub trait FromRawLinkedList<T> {
    type Output;
    fn one_from_raw(ptr: *mut T) -> Self::Output;
    fn from_raw(ptr: *mut T) -> Vec<Self::Output>;
}

/// gdbwire result
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GdbwireResult {
    Ok,
    Assert,
    Logic,
}
impl From<gdbwire_result> for GdbwireResult {
    fn from(result: gdbwire_result) -> GdbwireResult {
        match result {
            gdbwire_result::GDBWIRE_OK => GdbwireResult::Ok,
            gdbwire_result::GDBWIRE_ASSERT => GdbwireResult::Assert,
            gdbwire_result::GDBWIRE_LOGIC => GdbwireResult::Logic,
        }
    }
}

/// gdbmi parser
pub struct Parser {
    inner: *mut gdbmi_parser,
    _callback_data: ParserCallback,
}

/// gdbmi parser callback
struct ParserCallback {
    inner: gdbmi_parser_callbacks,
}

/// gdbmi output
#[derive(Clone,Debug)]
pub struct Output {
    pub line: String,
    pub variant: OutputVariant,
}
impl FromRawLinkedList<gdbmi_output> for Output {
    type Output = Output;
    fn one_from_raw(out: *mut gdbmi_output) -> Output {
        unsafe {
            Output {
                line: CStr::from_ptr((*out).line).to_string_lossy().into_owned(),
                variant: OutputVariant::from(*out),
            }
        }
    }
    fn from_raw(out: *mut gdbmi_output) -> Vec<Output> {
        let mut ret = vec![];
        let mut tmp = out;
        loop {
            ret.push(Output::one_from_raw(tmp));
            let next = unsafe { (*tmp).next };
            if !next.is_null() {
                tmp = next;
            } else {
                break;
            }
        }
        ret
    }
}

#[derive(Clone,Debug,PartialEq)]
pub enum OutputVariant {
    Oob(OobRecord),
    Result(ResultRecord),
    Error(ErrorRecord),
    Prompt,
}
impl From<gdbmi_output> for OutputVariant {
    fn from(mut output: gdbmi_output) -> OutputVariant {
        unsafe {
            match output.kind {
                gdbmi_output_kind::GDBMI_OUTPUT_OOB => {
                    let rec = output.variant.oob_record();
                    OutputVariant::Oob(OobRecord::from(*rec))
                }
                gdbmi_output_kind::GDBMI_OUTPUT_RESULT => {
                    let rec = output.variant.result_record();
                    OutputVariant::Result(ResultRecord::from(*rec))
                }
                gdbmi_output_kind::GDBMI_OUTPUT_PARSE_ERROR => {
                    let rec = output.variant.error();
                    OutputVariant::Error(ErrorRecord::from(rec))
                }
                gdbmi_output_kind::GDBMI_OUTPUT_PROMPT => OutputVariant::Prompt,
            }
        }
    }
}

#[derive(Clone,Debug,PartialEq)]
pub enum OobRecord {
    Async(AsyncRecord),
    Stream(StreamRecord),
}
impl From<*mut gdbmi_oob_record> for OobRecord {
    fn from(rec: *mut gdbmi_oob_record) -> OobRecord {
        unsafe {
            match (*rec).kind {
                gdbmi_oob_record_kind::GDBMI_ASYNC => {
                    let rec = (*rec).variant.async_record();
                    OobRecord::Async(AsyncRecord::from(*rec))
                }
                gdbmi_oob_record_kind::GDBMI_STREAM => {
                    let rec = (*rec).variant.stream_record();
                    OobRecord::Stream(StreamRecord::from(*rec))
                }
            }
        }
    }
}

#[derive(Clone,Debug,PartialEq)]
pub struct ResultRecord {
    pub token: String,
    pub class: ResultClass,
    pub results: Vec<Result>,
}
impl From<*mut gdbmi_result_record> for ResultRecord {
    fn from(rec: *mut gdbmi_result_record) -> ResultRecord {
        unsafe {
            let token = CStr::from_ptr((*rec).token).to_string_lossy().into_owned();
            ResultRecord {
                token: token,
                class: ResultClass::from((*rec).result_class),
                results: Result::from_raw((*rec).result),
            }
        }
    }
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum ResultClass {
    Done,
    Running,
    Connected,
    Error,
    Exit,
    Unsupported,
}
impl From<gdbmi_result_class> for ResultClass {
    fn from(cls: gdbmi_result_class) -> ResultClass {
        match cls {
            gdbmi_result_class::GDBMI_DONE => ResultClass::Done,
            gdbmi_result_class::GDBMI_RUNNING => ResultClass::Running,
            gdbmi_result_class::GDBMI_CONNECTED => ResultClass::Connected,
            gdbmi_result_class::GDBMI_ERROR => ResultClass::Error,
            gdbmi_result_class::GDBMI_EXIT => ResultClass::Exit,
            gdbmi_result_class::GDBMI_UNSUPPORTED => ResultClass::Unsupported,
        }
    }
}

#[derive(Clone,Debug,PartialEq)]
pub struct ErrorRecord {
    pub token: String,
    pub position: Position,
}
impl From<*mut gdbmi_error_variant> for ErrorRecord {
    fn from(rec: *mut gdbmi_error_variant) -> ErrorRecord {
        unsafe {
            let token = CStr::from_ptr((*rec).token).to_string_lossy().into_owned();
            ErrorRecord {
                token: token,
                position: Position::from((*rec).pos),
            }
        }
    }
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct Position {
    start_column: i32,
    end_column: i32,
}
impl From<gdbmi_position> for Position {
    fn from(pos: gdbmi_position) -> Position {
        Position {
            start_column: pos.start_column,
            end_column: pos.end_column,
        }
    }
}

#[derive(Clone,Debug,PartialEq)]
pub struct AsyncRecord {
    // token field is omitted (see good documentation from gdbwire)
    pub kind: AsyncRecordKind,
    pub class: AsyncClass,
    pub results: Vec<Result>,
}
impl From<*mut gdbmi_async_record> for AsyncRecord {
    fn from(rec: *mut gdbmi_async_record) -> AsyncRecord {
        unsafe {
            AsyncRecord {
                kind: AsyncRecordKind::from((*rec).kind),
                class: AsyncClass::from((*rec).async_class),
                results: Result::from_raw((*rec).result),
            }
        }
    }
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum AsyncRecordKind {
    Status,
    Exec,
    Notify,
}
impl From<gdbmi_async_record_kind> for AsyncRecordKind {
    fn from(kind: gdbmi_async_record_kind) -> AsyncRecordKind {
        match kind {
            gdbmi_async_record_kind::GDBMI_STATUS => AsyncRecordKind::Status,
            gdbmi_async_record_kind::GDBMI_EXEC => AsyncRecordKind::Exec,
            gdbmi_async_record_kind::GDBMI_NOTIFY => AsyncRecordKind::Notify,
        }
    }
}

#[derive(Clone,Debug,PartialEq)]
pub enum StreamRecord {
    Console(String),
    Target(String),
    Log(String),
}
impl From<*mut gdbmi_stream_record> for StreamRecord {
    fn from(rec: *mut gdbmi_stream_record) -> StreamRecord {
        unsafe {
            let mut p = (*rec).cstring;
            while (*p) != 0i8 {
                print!("{} ", *p);
                p = p.offset(1);
            }
            println!("");
            let c = CStr::from_ptr((*rec).cstring);
            println!("{:?}", c.to_bytes());
            let payload = c.to_string_lossy().into_owned();
            println!("{:?}", payload);
            match (*rec).kind {
                gdbmi_stream_record_kind::GDBMI_CONSOLE => StreamRecord::Console(payload),
                gdbmi_stream_record_kind::GDBMI_TARGET => StreamRecord::Target(payload),
                gdbmi_stream_record_kind::GDBMI_LOG => StreamRecord::Log(payload),
            }
        }
    }
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum AsyncClass {
    Download,
    Stopped,
    Running,
    ThreadGroupAdded,
    ThreadGroupRemoved,
    ThreadGroupStarted,
    ThreadGroupExited,
    ThreadCreated,
    ThreadExited,
    ThreadSelected,
    LibraryLoaded,
    LibraryUnloaded,
    TraceFrameChanged,
    TsvCreated,
    TsvModified,
    TsvDeleted,
    BreakpointCreated,
    BreakpointModified,
    BreakpointDeleted,
    RecordStarted,
    RecordStopped,
    CmdParamChanged,
    MemoryChanged,
    Unsupported,
}
impl From<gdbmi_async_class> for AsyncClass {
    fn from(cls: gdbmi_async_class) -> AsyncClass {
        match cls {
            gdbmi_async_class::GDBMI_ASYNC_DOWNLOAD => AsyncClass::Download,
            gdbmi_async_class::GDBMI_ASYNC_STOPPED => AsyncClass::Stopped,
            gdbmi_async_class::GDBMI_ASYNC_RUNNING => AsyncClass::Running,
            gdbmi_async_class::GDBMI_ASYNC_THREAD_GROUP_ADDED => AsyncClass::ThreadGroupAdded,
            gdbmi_async_class::GDBMI_ASYNC_THREAD_GROUP_REMOVED => AsyncClass::ThreadGroupRemoved,
            gdbmi_async_class::GDBMI_ASYNC_THREAD_GROUP_STARTED => AsyncClass::ThreadGroupStarted,
            gdbmi_async_class::GDBMI_ASYNC_THREAD_GROUP_EXITED => AsyncClass::ThreadGroupExited,
            gdbmi_async_class::GDBMI_ASYNC_THREAD_CREATED => AsyncClass::ThreadCreated,
            gdbmi_async_class::GDBMI_ASYNC_THREAD_EXITED => AsyncClass::ThreadExited,
            gdbmi_async_class::GDBMI_ASYNC_THREAD_SELECTED => AsyncClass::ThreadSelected,
            gdbmi_async_class::GDBMI_ASYNC_LIBRARY_LOADED => AsyncClass::LibraryLoaded,
            gdbmi_async_class::GDBMI_ASYNC_LIBRARY_UNLOADED => AsyncClass::LibraryUnloaded,
            gdbmi_async_class::GDBMI_ASYNC_TRACEFRAME_CHANGED => AsyncClass::TraceFrameChanged,
            gdbmi_async_class::GDBMI_ASYNC_TSV_CREATED => AsyncClass::TsvCreated,
            gdbmi_async_class::GDBMI_ASYNC_TSV_MODIFIED => AsyncClass::TsvModified,
            gdbmi_async_class::GDBMI_ASYNC_TSV_DELETED => AsyncClass::TsvDeleted,
            gdbmi_async_class::GDBMI_ASYNC_BREAKPOINT_CREATED => AsyncClass::BreakpointCreated,
            gdbmi_async_class::GDBMI_ASYNC_BREAKPOINT_MODIFIED => AsyncClass::BreakpointModified,
            gdbmi_async_class::GDBMI_ASYNC_BREAKPOINT_DELETED => AsyncClass::BreakpointDeleted,
            gdbmi_async_class::GDBMI_ASYNC_RECORD_STARTED => AsyncClass::RecordStarted,
            gdbmi_async_class::GDBMI_ASYNC_RECORD_STOPPED => AsyncClass::RecordStopped,
            gdbmi_async_class::GDBMI_ASYNC_CMD_PARAM_CHANGED => AsyncClass::CmdParamChanged,
            gdbmi_async_class::GDBMI_ASYNC_MEMORY_CHANGED => AsyncClass::MemoryChanged,
            gdbmi_async_class::GDBMI_ASYNC_UNSUPPORTED => AsyncClass::Unsupported,
        }
    }
}

#[derive(Clone,Debug,PartialEq)]
pub struct Result {
    pub key: Option<String>,
    pub value: Option<ResultType>,
}
#[derive(Clone,Debug,PartialEq)]
pub enum ResultType {
    String(String),
    Result(Box<Result>),
    List(Vec<Result>),
}
impl FromRawLinkedList<gdbmi_result> for Result {
    type Output = Result;
    fn one_from_raw(res: *mut gdbmi_result) -> Result {
        unsafe {
            let key = if (*res).variable.is_null() {
                None
            } else {
                Some(CStr::from_ptr((*res).variable).to_string_lossy().into_owned())
            };
            match (*res).kind {
                gdbmi_result_kind::GDBMI_CSTRING => {
                    let cstring = (*res).variant.cstring();
                    let cstring = CStr::from_ptr(*cstring).to_string_lossy().into_owned();
                    Result {
                        key: key,
                        value: Some(ResultType::String(cstring)),
                    }
                }
                gdbmi_result_kind::GDBMI_TUPLE => {
                    let result = (*res).variant.result();
                    Result {
                        key: key,
                        value: Some(ResultType::Result(Box::new(Result::one_from_raw(*result)))),
                    }
                }
                gdbmi_result_kind::GDBMI_LIST => {
                    let result = (*res).variant.result();
                    Result {
                        key: key,
                        value: Some(ResultType::List(Result::from_raw(*result))),
                    }
                }
            }
        }
    }
    fn from_raw(res: *mut gdbmi_result) -> Vec<Result> {
        let mut ret = vec![];
        let mut tmp = res;
        loop {
            ret.push(Result::one_from_raw(tmp));
            let next = unsafe { (*tmp).next };
            if !next.is_null() {
                tmp = next;
            } else {
                break;
            }
        }
        ret
    }
}

unsafe extern "C" fn callback_wrapper<F>(context: *mut raw::c_void, output: *mut gdbmi_output)
    where F: Fn(Vec<Output>) + 'static
{
    let cb_ptr = context as *const F;
    (*cb_ptr)(Output::from_raw(output));
    gdbwire_sys::gdbmi_output_free(output);
}

impl ParserCallback {
    fn new<F>(callback: F) -> ParserCallback
        where F: Fn(Vec<Output>) + 'static
    {
        let cb_ptr = &callback as *const F as *const raw::c_void;
        ParserCallback {
            inner: gdbmi_parser_callbacks {
                context: cb_ptr as *mut raw::c_void,
                gdbmi_output_callback: Some(callback_wrapper::<F>),
            },
        }
    }
}

impl Parser {
    pub fn new<F>(callback: F) -> Parser
        where F: Fn(Vec<Output>) + 'static
    {
        let parser_callback = ParserCallback::new(callback);
        let inner = unsafe { gdbmi_parser_create(parser_callback.inner) };
        Parser {
            _callback_data: parser_callback,
            inner: inner,
        }
    }
    pub fn push(&self, data: &str) -> GdbwireResult {
        let cstr = CString::new(data).unwrap(); // TODO: Remove unwrap
        unsafe { gdbmi_parser_push_data(self.inner, cstr.as_ptr(), data.len()) }.into()
    }
}
impl Drop for Parser {
    fn drop(&mut self) {
        unsafe {
            gdbmi_parser_destroy(self.inner);
        }
    }
}
