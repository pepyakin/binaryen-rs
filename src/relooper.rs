
use std::rc::Rc;
use std::ptr;
use ffi;
use {Expr, InnerModule};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RelooperBlockId(usize);

pub struct Relooper {
    raw: ffi::RelooperRef,
    blocks: Vec<ffi::RelooperBlockRef>,
    module_ref: Rc<InnerModule>,
}

impl Relooper {
    pub(crate) fn new(module_ref: Rc<InnerModule>) -> Relooper {
        Relooper {
            raw: unsafe { ffi::RelooperCreate() },
            blocks: Vec::new(),
            module_ref,
        }
    }

    pub fn add_block(&mut self, code: Expr) -> RelooperBlockId {
        debug_assert!(self.is_expr_from_same_module(&code));
        let raw = unsafe { ffi::RelooperAddBlock(self.raw, code.into_raw()) };
        self.register_block(raw)
    }

    pub fn add_block_with_switch(&mut self, code: Expr, condition: Expr) -> RelooperBlockId {
        debug_assert!(self.is_expr_from_same_module(&code));
        debug_assert!(self.is_expr_from_same_module(&condition));
        let raw = unsafe {
            ffi::RelooperAddBlockWithSwitch(self.raw, code.into_raw(), condition.into_raw())
        };
        self.register_block(raw)
    }

    fn register_block(&mut self, raw_block: ffi::RelooperBlockRef) -> RelooperBlockId {
        let index = self.blocks.len();
        self.blocks.push(raw_block);
        RelooperBlockId(index)
    }

    pub fn render(self, entry_id: RelooperBlockId, label_helper: u32) -> Expr {
        let entry = self.blocks[entry_id.0];
        let raw = unsafe {
            ffi::RelooperRenderAndDispose(self.raw, entry, label_helper as _, self.module_ref.raw)
        };
        Expr {
            _module_ref: self.module_ref.clone(),
            raw,
        }
    }

    pub fn add_branch(
        &self,
        from: RelooperBlockId,
        to: RelooperBlockId,
        condition: Option<Expr>,
        code: Option<Expr>,
    ) {
        debug_assert!(
            condition
                .as_ref()
                .map_or(true, |e| { self.is_expr_from_same_module(e) })
        );
        debug_assert!(
            code.as_ref()
                .map_or(true, |e| { self.is_expr_from_same_module(e) })
        );

        let from_block = self.blocks[from.0];
        let to_block = self.blocks[to.0];

        unsafe {
            let condition_ptr = condition.map_or(ptr::null_mut(), |e| e.into_raw());
            let code_ptr = code.map_or(ptr::null_mut(), |e| e.into_raw());
            ffi::RelooperAddBranch(from_block as _, to_block as _, condition_ptr, code_ptr)
        }
    }

    pub fn add_branch_for_switch(
        &self,
        from: RelooperBlockId,
        to: RelooperBlockId,
        indices: &[u32],
        code: Option<Expr>,
    ) {
        debug_assert!(
            code.as_ref()
                .map_or(true, |e| { self.is_expr_from_same_module(e) })
        );
        let from_block = self.blocks[from.0];
        let to_block = self.blocks[to.0];

        unsafe {
            let code_ptr = code.map_or(ptr::null_mut(), |e| e.into_raw());
            ffi::RelooperAddBranchForSwitch(
                from_block,
                to_block,
                indices.as_ptr() as *mut _,
                indices.len() as _,
                code_ptr,
            );
        }
    }

    fn is_expr_from_same_module(&self, expr: &Expr) -> bool {
        Rc::ptr_eq(&self.module_ref, &expr._module_ref)
    }
}
