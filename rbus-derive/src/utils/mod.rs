pub use attr::*;
pub use debug::*;
pub use rbus_metas::*;

mod attr;
mod debug;
mod rbus_metas;

pub fn split_tuples<A, B>(tuples: &[(A, B)]) -> (Vec<&A>, Vec<&B>) {
    let mut first = Vec::with_capacity(tuples.len());
    let mut second = Vec::with_capacity(tuples.len());

    for (a, b) in tuples.iter() {
        first.push(a);
        second.push(b);
    }

    (first, second)
}
