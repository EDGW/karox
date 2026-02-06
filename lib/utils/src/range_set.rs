use alloc::collections::linked_list::{Iter, LinkedList};
use core::{cmp::max, fmt::Debug, ops::Range};

#[derive(Debug)]
pub struct SortedRangeSet {
    inner: LinkedList<Range<usize>>,
}

impl SortedRangeSet {
    pub const fn new() -> SortedRangeSet {
        SortedRangeSet {
            inner: LinkedList::new(),
        }
    }

    pub fn iter(&self) -> Iter<'_, Range<usize>> {
        self.inner.iter()
    }

    fn combine(&mut self) {
        let mut cur = self.inner.cursor_front_mut();
        while let Some(mut current) = cur.current().cloned() {
            if current.len() == 0 {
                cur.remove_current();
                continue;
            }
            while let Some(next) = cur.peek_next() {
                if current.end >= next.start {
                    current.end = max(current.end, next.end);
                    cur.move_next();
                    cur.remove_current();
                    cur.move_prev();
                } else {
                    break;
                }
            }
            *cur.current().unwrap() = current;
            cur.move_next();
        }
    }

    pub fn add(&mut self, range: Range<usize>) {
        if self.inner.is_empty() {
            self.inner.push_back(range);
            return;
        }
        let mut cur = self.inner.cursor_front_mut();
        while let Some(current) = cur.current().cloned() {
            if let Some(next) = cur.peek_next() {
                if current.start <= range.start && next.start >= range.start {
                    cur.insert_after(range);
                    break;
                }
            } else {
                cur.insert_after(range);
                break;
            }
            cur.move_next();
        }
        self.combine();
    }

    pub fn sub(&mut self, range: Range<usize>) {
        let mut changed = false;
        let mut cur = self.inner.cursor_front_mut();
        while let Some(mut current) = cur.current().cloned() {
            if current.start > range.end || current.end < range.start {
                cur.move_next();
                continue;
            }
            // [      ]
            //   [  ]
            if current.start <= range.start && current.end >= range.end {
                cur.insert_after(range.end..current.end);
                current.end = range.start;
                *cur.current().unwrap() = current;
                changed = true;
                cur.move_next();
                cur.move_next();
                continue;
            }
            // [      ]
            //    [      ]
            else if current.start <= range.start {
                current.end = range.start;
                *cur.current().unwrap() = current;
                changed = true;
            }
            //     [      ]
            //  [           ]
            else if current.start >= range.start && current.end <= range.end {
                current.end = current.start;
                *cur.current().unwrap() = current;
                changed = true;
            }
            //     [      ]
            //  [       ]
            else {
                current.start = range.end;
                *cur.current().unwrap() = current;
                changed = true;
            }
            cur.move_next();
        }
        if changed {
            self.combine();
        }
    }
}
