use std::iter::Peekable;
use std::cmp::Ordering;

// ============================================
// merge code by shepmaster
// https://stackoverflow.com/a/32020190/4261231
// ============================================

pub struct MergeAscending<L, R>
    where L: Iterator<Item = R::Item>,
          R: Iterator,
{
    left: Peekable<L>,
    right: Peekable<R>,
}

impl<L, R> MergeAscending<L, R>
    where L: Iterator<Item = R::Item>,
          R: Iterator,
{
    pub fn new(left: L, right: R) -> Self {
        MergeAscending {
            left: left.peekable(),
            right: right.peekable(),
        }
    }
}

impl<L, R> Iterator for MergeAscending<L, R>
    where L: Iterator<Item = R::Item>,
          R: Iterator,
          L::Item: Ord,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<L::Item> {
        let which = match (self.left.peek(), self.right.peek()) {
            (Some(l), Some(r)) => Some(l.cmp(r)),
            (Some(_), None)    => Some(Ordering::Less),
            (None, Some(_))    => Some(Ordering::Greater),
            (None, None)       => None,
        };

        match which {
            Some(Ordering::Less)    => self.left.next(),
            Some(Ordering::Equal)   => self.left.next(),
            Some(Ordering::Greater) => self.right.next(),
            None                    => None,
        }
    }
}
// ============================================
// end shepmaster code
// ============================================

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}