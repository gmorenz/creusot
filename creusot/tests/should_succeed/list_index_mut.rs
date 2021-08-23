// SHOULD_SUCCEED: parse-print
#![feature(register_tool, rustc_attrs)]
#![register_tool(creusot)]
#![feature(proc_macro_hygiene, stmt_expr_attributes)]

extern crate creusot_contracts;

use creusot_contracts::*;

enum Option<T> {
    None,
    Some(T),
}

use Option::*;

pub struct List(u32,Option<Box<List>>);

unsafe impl Resolve for List {
    predicate! { fn resolve(self) -> bool {
        true
    } }    
}

logic!{
#[ensures(result >= 0)]
fn len(l: List) -> Int {{
    let List(_, ls) = l;
    1 + match ls {
        Some(ls) => len(*ls),
        None => 0
    }
}}
}

logic!{
#[variant(len(l))]
fn get(l : List, ix : Int) -> Option<u32> {{
    let List(i, ls) = l;
    match (ix > 0) {
        false => Some(i),
        true => match ls {
            Some(ls) => get(*ls, ix - 1),
            None => None
        }
    }
}}
}

#[requires(Int::from(param_ix) < len(*param_l))]
#[ensures(equal(Some(* result), get(*param_l, Int::from(param_ix))))]
#[ensures(equal(Some(^ result), get(^param_l, Int::from(param_ix))))]
#[ensures(equal(len(^param_l), len(* param_l)))]
#[ensures(forall<i:Int> 0 <= i && i < len(* param_l) && i != (Int::from(param_ix)) -> equal(get(*param_l, i), get(^param_l, i)))]
pub fn index_mut(param_l: &mut List, param_ix: usize) -> &mut u32 {
    let mut l = param_l;
    let mut ix = param_ix;
    #[invariant(valid_ix, 0usize <= ix && (Int::from(ix)) < len (* l))]
    #[invariant(get_target_now, equal(get(*l, Int::from(ix)), get(*param_l, Int::from(param_ix))))]
    #[invariant(get_target_fin, equal(get(^l, Int::from(ix)), get(^param_l, Int::from(param_ix))))]
    #[invariant(len, (len(^l) == len(*l) -> len(^param_l) == len(*param_l)))]
    #[invariant(untouched,
        (forall<i:Int> 0 <= i && i < len ( * l) && i != Int::from(ix) -> equal(get( ^l, i), get( * l, i))) ->
        (forall<i:Int> 0 <= i && i < len ( * param_l) && i != (Int::from(param_ix)) -> equal(get ( ^param_l, i), get ( * param_l, i)))
    )]
    while ix > 0 {
        match l.1 {
            Some(ref mut n) => {
                l = n;
            }
            None => std::process::abort(),
        }
        ix -= 1;
    }

    &mut l.0
}

// Ensure that this performs a set on the list
#[requires(Int::from(ix) < len(*l))]
#[ensures(equal(Some(val), get(^l, Int::from(ix))))]
#[ensures(equal(len(^l), len(* l)))]
#[ensures(forall<i:Int> 0 <= i && i < len(* l) && i != (Int::from(ix)) -> equal(get(*l, i), get(^l, i)))]
pub fn write(l: &mut List, ix: usize, val: u32) {
    *index_mut(l, ix) = val;
}

fn main() {
    let mut l = List(1, Some(Box::new(List(10, None))));
    write(&mut l, 0, 2);

    // assert!(get 0 l == 2 && get 1 l == 10);
}
