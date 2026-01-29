use crate::mm::paging::PageTable;

#[derive(Debug)]
pub struct MemSpace {
    pub asid: usize,
    pub page_table: PageTable,
}

impl MemSpace {
    pub fn new(asid: usize, ptable: PageTable) -> MemSpace {
        MemSpace {
            asid,
            page_table: ptable,
        }
    }
}
