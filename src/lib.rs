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
pub struct Parser {
    inner: *mut gdbmi_parser,
    _callback_data: ParserCallback,
}

/// gdbmi parser callback
struct ParserCallback {
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
    pub fn push(&self, data: &str) -> Result {
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
