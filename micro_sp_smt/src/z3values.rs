//! Z3 values for SP

use std::ffi::{CString};
use z3_sys::*;
use std::f64;
use super::*;

pub struct BoolZ3<'ctx> {
    pub ctx: &'ctx ContextZ3,
    pub r: Z3_ast
}

pub struct IntZ3<'ctx, 'isrt> {
    pub ctx: &'ctx ContextZ3,
    pub isrt: &'isrt IntSortZ3<'ctx>,
    pub r: Z3_ast
}

pub struct RealZ3<'ctx, 'rsrt> {
    pub ctx: &'ctx ContextZ3,
    pub rsrt: &'rsrt RealSortZ3<'ctx>,
    pub r: Z3_ast
}

pub struct StringZ3<'ctx, 'a> {
    pub ctx: &'ctx ContextZ3,
    pub val: &'a str,
    pub r: Z3_ast
}

impl <'ctx> BoolZ3<'ctx> {
    /// Create an AST node representing `true` or 'false'.
    /// 
    /// NOTE: See macro! `bool_z3!`
    pub fn new(ctx: &'ctx ContextZ3, val: bool) -> Z3_ast {
        let z3 = if val == true { unsafe {
                Z3_mk_true(ctx.r)
            }} else { unsafe { 
                Z3_mk_false(ctx.r)
            }
        };
        BoolZ3 {ctx, r: z3}.r
    }
}

impl <'ctx, 'isrt> IntZ3<'ctx, 'isrt> {
    /// Create an int constant from a rust i32.
    ///
    /// - `ctx`: logical context.
    /// - `isrt`: int sort.
    /// - `val`: int to be realized.
    ///
    /// NOTE: See macro! `int_z3!`
    pub fn new(ctx: &'ctx ContextZ3, isrt: &'isrt IntSortZ3<'ctx>, val: i32) -> Z3_ast {
        let z3 = unsafe {
            Z3_mk_int(ctx.r, val, isrt.r)
        };
        IntZ3 {ctx, isrt, r: z3}.r
    }
}

impl <'ctx, 'rsrt> RealZ3<'ctx, 'rsrt> {
    /// Create a real constant from a rust f64.
    ///
    /// - `ctx`: logical context.
    /// - `rsrt`: real sort.
    /// - `val`: float to be realized.
    /// 
    /// NOTE: See macro! `real_z3!`
    pub fn new(ctx: &'ctx ContextZ3, rsrt: &'rsrt RealSortZ3<'ctx>, val: f64) -> Z3_ast {
        let num_string = val.to_string();
        let cstring = CString::new(num_string).unwrap();
        let z3 = unsafe {
            Z3_mk_numeral(ctx.r, cstring.as_ptr(), rsrt.r)
        };
        RealZ3 {ctx, rsrt, r: z3}.r
    }
}

impl <'ctx, 'a> StringZ3<'ctx, 'a> {
    /// Create a string constant from a rust string.
    ///
    /// - `ctx`: logical context.
    /// - `ssrt`: string sort.
    /// - `val`: string to be realized.
    /// 
    /// NOTE: See macro! `string_z3!`
    pub fn new(ctx: &'ctx ContextZ3, val: &'a str) -> Z3_ast {
        let string_val = CString::new(val.to_string()).unwrap();
        let z3 = unsafe {
            Z3_mk_string(ctx.r, string_val.as_ptr())
        };
        StringZ3 {ctx, r: z3, val}.r
    }
}

/// create a boolean constant
/// 
/// Macro rule for:
/// ```text
/// z3values::BoolZ3::new(&ctx, a)
/// ```
/// Using a specific context:
/// ```text
/// bool_z3!(&ctx, a)
/// ```
#[macro_export]
macro_rules! bool_z3 {
    ($ctx:expr, $a:expr) => {
        BoolZ3::new($ctx, $a)
    }
}

/// create an integer constant
/// 
/// Macro rule for:
/// ```text
/// z3values::IntZ3::new(&ctx, a)
/// ```
/// Using a specific context:
/// ```text
/// int_z3!(&ctx, a)
/// ```
#[macro_export]
macro_rules! int_z3 {
    ($ctx:expr, $a:expr) => {
        IntZ3::new($ctx, &IntSortZ3::new($ctx), $a)
    }
}

/// create a real constant
/// 
/// Macro rule for:
/// ```text
/// z3values::RealZ3::new(&ctx, a)
/// ```
/// Using a specific context:
/// ```text
/// real_z3!(&ctx, a)
/// ```
#[macro_export]
macro_rules! real_z3 {
    ($ctx:expr, $a:expr) => {
        RealZ3::new($ctx, &RealSortZ3::new($ctx), $a)
    }
}

/// create a string constant
/// 
/// Macro rule for:
/// ```text
/// z3values::StringZ3::new(&ctx, a)
/// ```
/// Using a specific context:
/// ```text
/// string_z3!(&ctx, a)
/// ```
#[macro_export]
macro_rules! string_z3 {
    ($ctx:expr, $a:expr) => {
        StringZ3::new($ctx, $a)
    }
}

#[test]
fn test_new_bool(){
    let conf = ConfigZ3::new();
    let ctx = ContextZ3::new(&conf);

    let t = BoolZ3::new(&ctx, true);
    let f = BoolZ3::new(&ctx, false);

    assert_eq!("true", ast_to_string_z3!(&ctx, t));
    assert_eq!("false", ast_to_string_z3!(&ctx, f));
}

#[test]
fn test_new_int(){
    let conf = ConfigZ3::new();
    let ctx = ContextZ3::new(&conf);
    let intsort = IntSortZ3::new(&ctx);

    let int1 = IntZ3::new(&ctx, &intsort, 7);
    let int2 = IntZ3::new(&ctx, &intsort, -1012390);

    assert_eq!("7", ast_to_string_z3!(&ctx, int1));
    assert_eq!("(- 1012390)", ast_to_string_z3!(&ctx, int2));
}

#[test]
fn test_new_string(){
    let conf = ConfigZ3::new();
    let ctx = ContextZ3::new(&conf);

    let str1 = StringZ3::new(&ctx, "hello!@#$ @# $%@#$ ");

    assert_eq!("\"hello!@#$ @# $%@#$ \"", ast_to_string_z3!(&ctx, str1));
}

#[test]
fn test_bool_macro_1(){
    let cfg = cfg_z3!();
    let ctx = ctx_z3!(&cfg);
    let bool1 = bool_z3!(&ctx, true);
    assert_eq!("true", ast_to_string_z3!(&ctx, bool1));
    assert_eq!("Bool", sort_to_string_z3!(&ctx, get_sort_z3!(&ctx, bool1)));
}

#[test]
fn test_int_macro_1(){
    let cfg = cfg_z3!();
    let ctx = ctx_z3!(&cfg);
    let int1 = int_z3!(&ctx, 76);
    assert_eq!("76", ast_to_string_z3!(&ctx, int1));
    assert_eq!("Int", sort_to_string_z3!(&ctx, get_sort_z3!(&ctx, int1)));
}

#[test]
fn test_real_macro_1(){
    let cfg = cfg_z3!();
    let ctx = ctx_z3!(&cfg);
    let real1 = real_z3!(&ctx, 76.456);
    assert_eq!("(/ 9557.0 125.0)", ast_to_string_z3!(&ctx, real1));
    assert_eq!("Real", sort_to_string_z3!(&ctx, get_sort_z3!(&ctx, real1)));
}

#[test]
fn test_string_macro_1(){
    let cfg = cfg_z3!();
    let ctx = ctx_z3!(&cfg);
    let string1 = string_z3!(&ctx, "asdf_ASDF_!@#$");
    assert_eq!("\"asdf_ASDF_!@#$\"", ast_to_string_z3!(&ctx, string1));
    assert_eq!("String", sort_to_string_z3!(&ctx, get_sort_z3!(&ctx, string1)));
}