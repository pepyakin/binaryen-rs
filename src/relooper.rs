
use std::rc::Rc;
use std::ptr;
use ffi;
use {InnerModule, Expr};

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

    pub fn add_block(&mut self, expr: Expr) -> RelooperBlockId {
        debug_assert!(Rc::ptr_eq(&self.module_ref, &expr._module_ref));
        let raw = unsafe { ffi::RelooperAddBlock(self.raw, expr.raw) };
        let index = self.blocks.len();
        self.blocks.push(raw);
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
                .map_or(true, |e| { Rc::ptr_eq(&self.module_ref, &e._module_ref) })
        );
        debug_assert!(
            code.as_ref()
                .map_or(true, |e| { Rc::ptr_eq(&self.module_ref, &e._module_ref) })
        );

        let from_block = self.blocks[from.0];
        let to_block = self.blocks[to.0];

        unsafe {
            let condition_ptr = condition.map_or(ptr::null_mut(), |e| e.raw);
            let code_ptr = code.map_or(ptr::null_mut(), |e| e.raw);
            ffi::RelooperAddBranch(from_block as _, to_block as _, condition_ptr, code_ptr)
        }
    }
}
