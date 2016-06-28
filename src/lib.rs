extern crate gdbwire_sys;
use gdbwire_sys::*;

use std::default::Default;
use std::ffi::{CStr, CString};
use std::os::raw;
use std::ptr;
use std::fmt::Debug;

/// gdbwire result
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Result {
    Ok,
    Assert,
    Logic,
}
impl From<gdbwire_result> for Result {
    fn from(result: gdbwire_result) -> Result {
        match result {
            gdbwire_result::GDBWIRE_OK => Result::Ok,
            gdbwire_result::GDBWIRE_ASSERT => Result::Assert,
            gdbwire_result::GDBWIRE_LOGIC => Result::Logic,
        }
    }
}

/// gdbmi parser
pub struct Parser<T> {
    inner: *mut gdbmi_parser,
    callback_data: ParserCallback<T>,
}

/// gdbmi parser callback
pub struct ParserCallback<T> {
    data: *mut (Option<T>, *const raw::c_void),
    inner: gdbmi_parser_callbacks,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OutputKind {
    Oob,
    Result,
    Prompt,
    ParseError,
}
impl From<gdbmi_output_kind> for OutputKind {
    fn from(kind: gdbmi_output_kind) -> OutputKind {
        match kind {
            gdbmi_output_kind::GDBMI_OUTPUT_OOB => OutputKind::Oob,
            gdbmi_output_kind::GDBMI_OUTPUT_RESULT => OutputKind::Result,
            gdbmi_output_kind::GDBMI_OUTPUT_PROMPT => OutputKind::Prompt,
            gdbmi_output_kind::GDBMI_OUTPUT_PARSE_ERROR => OutputKind::ParseError,
        }
    }
}

/// gdbmi output
#[derive(Clone,Debug)]
pub struct Output {
    pub kind: OutputKind,
    pub line: String,
}
impl Output {
    fn one_from_raw(out: *const gdbmi_output) -> Output {
        Output {
            kind: unsafe { (*out).kind }.into(),
            line: unsafe { CStr::from_ptr((*out).line) }.to_string_lossy().into_owned(),
        }
    }
    fn from_raw(out: *const gdbmi_output) -> Vec<Output> {
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

unsafe extern "C" fn callback_wrapper<T, F>(context: *mut raw::c_void, output: *mut gdbmi_output)
    where F: Fn(Option<T>, Vec<Output>)
{
    let (ctx, cb_ptr) = ptr::read(context as *mut (Option<T>, *const F));
    (*cb_ptr)(ctx, Output::from_raw(output));
    gdbwire_sys::gdbmi_output_free(output);
}

impl<T: Debug> ParserCallback<T> {
    pub fn new<F>(context: Option<T>, callback: F) -> ParserCallback<T>
        where F: Fn(Option<T>, Vec<Output>)
    {
        let mut ret = ParserCallback {
            data: unsafe { std::mem::uninitialized() },
            inner: gdbmi_parser_callbacks { ..Default::default() },
        };
        let cb_ptr = &callback as *const F as *const raw::c_void;
        ret.data = Box::into_raw(Box::new((context, cb_ptr)));
        ret.inner = gdbmi_parser_callbacks {
            context: ret.data as *mut raw::c_void,
            gdbmi_output_callback: Some(callback_wrapper::<T, F>),
        };
        ret
    }
}

impl<T> Parser<T> {
    pub fn new(callback: ParserCallback<T>) -> Parser<T> {
        let inner = unsafe { gdbmi_parser_create(callback.inner) };
        Parser {
            callback_data: callback,
            inner: inner,
        }
    }
    pub fn push(&self, data: &str) -> Result {
        let cstr = CString::new(data).unwrap(); // TODO: Remove unwrap
        unsafe { gdbmi_parser_push_data(self.inner, cstr.as_ptr(), data.len()) }.into()
    }
}
impl<T> Drop for Parser<T> {
    fn drop(&mut self) {
        unsafe {
            gdbmi_parser_destroy(self.inner);
        }
    }
}
