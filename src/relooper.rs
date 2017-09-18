
use std::rc::Rc;
use std::ptr;
use ffi;
use {Expr, InnerModule};

/// Relooper transforms arbitrary control-flow graph into 
/// a structured control-flow that used by Binaryen IR.
///
/// It takes control-flow graph in form of the blocks of code,
/// created by [`add_block`] or [`add_block_with_switch`] and connected
/// with [`add_branch`] and [`add_branch_for_switch`] respectively.
///
/// After blocks created and connected [`render`] is used to create
/// structured control-flow in form of `Expr`s.
/// 
/// [`add_block`]: #method.add_block
/// [`add_block_with_switch`]: #method.add_block_with_switch
/// [`add_branch`]: #method.add_branch
/// [`add_branch_for_switch`]: #method.add_branch_for_switch
/// [`render`]: #method.render
///
/// # Examples
/// 
/// ```
/// # use binaryen::*;
/// let module = Module::new();
/// let mut relooper = module.relooper();
/// 
/// // Create two blocks that do nothing.
/// let b1 = relooper.add_block(module.nop());
/// let b2 = relooper.add_block(module.nop());
///
/// // Add unconditional branch from `b1` to `b2`.
/// relooper.add_branch(b1, b2, None, None);
///
/// // Render final expression
/// let result: Expr = relooper.render(b1, 0);
/// result.print();
/// ```
///
/// If you want conditional branch, you can use. 
///
/// ```
/// # use binaryen::*;
/// # let module = Module::new();
/// # let mut relooper = module.relooper();
///
/// let entry = relooper.add_block(module.nop());
/// let if_true = relooper.add_block(module.nop());
/// let if_false = relooper.add_block(module.nop());
///
/// let always_true_condition: Expr = module.const_(Literal::I32(1));
/// 
/// relooper.add_branch(entry, if_true, Some(always_true_condition), None);
/// relooper.add_branch(entry, if_false, None, None);
/// 
/// let result: Expr = relooper.render(entry, 0);
/// result.print();
/// ```
///
pub struct Relooper {
    raw: ffi::RelooperRef,
    blocks: Vec<ffi::RelooperBlockRef>,
    module_ref: Rc<InnerModule>,
}

/// Represents Relooper's block. 
///
/// Can be either:
///
/// * [`PlainBlock`]
/// * [`SwitchBlock`]
///
/// [`PlainBlock`]: struct.PlainBlock.html
/// [`SwitchBlock`]: struct.SwitchBlock.html
pub trait Block {
    fn get_id(&self) -> usize;
}

/// Block that ends with nothing or with simple branching.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PlainBlock(usize);

impl Block for PlainBlock {
    fn get_id(&self) -> usize {
        self.0
    }
}

/// Block that ends with a switch.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SwitchBlock(usize);

impl Block for SwitchBlock {
    fn get_id(&self) -> usize {
        self.0
    }
}

impl Relooper {
    pub(crate) fn new(module_ref: Rc<InnerModule>) -> Relooper {
        Relooper {
            raw: unsafe { ffi::RelooperCreate() },
            blocks: Vec::new(),
            module_ref,
        }
    }

    /// Add a plain block that executes `code` and ends with nothing or with simple branching.
    /// 
    /// Plain blocks can have multiple conditional branches to the other blocks 
    /// and at most one unconditional.
    ///
    /// Be careful with branching in `code`, branches to the outside won't work.
    pub fn add_block(&mut self, code: Expr) -> PlainBlock {
        debug_assert!(self.is_expr_from_same_module(&code));
        let raw = unsafe { ffi::RelooperAddBlock(self.raw, code.into_raw()) };
        PlainBlock(self.add_raw_block(raw))
    }

    /// Add a block that executes `code` and ends with a switch on the `condition`.
    /// 
    /// Be careful with branching in `code`, branches to the outside won't work.
    pub fn add_block_with_switch(&mut self, code: Expr, condition: Expr) -> SwitchBlock {
        debug_assert!(self.is_expr_from_same_module(&code));
        debug_assert!(self.is_expr_from_same_module(&condition));
        let raw = unsafe {
            ffi::RelooperAddBlockWithSwitch(self.raw, code.into_raw(), condition.into_raw())
        };
        SwitchBlock(self.add_raw_block(raw))
    }

    /// Add a branch from `PlainBlock` to any other block.
    ///
    /// You can provide optional `condition` that will be used to decide if the branch should be taken.
    /// 
    /// The branch can have `code` on it, that is executed as the branch happens. 
    /// This is useful for phis if you use SSA.
    pub fn add_branch<B: Block>(
        &self,
        from: PlainBlock,
        to: B,
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

        let from_block = self.get_raw_block(from);
        let to_block = self.get_raw_block(to);

        unsafe {
            let condition_ptr = condition.map_or(ptr::null_mut(), |e| e.into_raw());
            let code_ptr = code.map_or(ptr::null_mut(), |e| e.into_raw());
            ffi::RelooperAddBranch(from_block as _, to_block as _, condition_ptr, code_ptr)
        }
    }

    /// Add a switch-style branch to any other block. 
    /// 
    /// The block's switch table will have these indices going to that target
    pub fn add_branch_for_switch<B: Block>(
        &self,
        from: SwitchBlock,
        to: B,
        indices: &[u32],
        code: Option<Expr>,
    ) {
        debug_assert!(
            code.as_ref()
                .map_or(true, |e| { self.is_expr_from_same_module(e) })
        );
        let from_block = self.get_raw_block(from);
        let to_block = self.get_raw_block(to);

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

    /// Render an structured control-flow graph from provided blocks and branches.
    /// 
    /// # Arguments
    ///
    /// * entry - Entrypoint of this control-flow graph.
    /// * label_helper - Index of i32 local variable that is free to use. This may be needed
    /// to render irreducible control-flow graph.
    /// 
    pub fn render<B: Block>(self, entry: B, label_helper: u32) -> Expr {
        let entry = self.get_raw_block(entry);
        let raw = unsafe {
            ffi::RelooperRenderAndDispose(self.raw, entry, label_helper as _, self.module_ref.raw)
        };
        Expr {
            _module_ref: self.module_ref.clone(),
            raw,
        }
    }

    fn add_raw_block(&mut self, raw_block: ffi::RelooperBlockRef) -> usize {
        let index = self.blocks.len();
        self.blocks.push(raw_block);
        index
    }

    fn get_raw_block<B: Block>(&self, block: B) -> ffi::RelooperBlockRef {
        self.blocks[block.get_id()]
    }

    fn is_expr_from_same_module(&self, expr: &Expr) -> bool {
        Rc::ptr_eq(&self.module_ref, &expr._module_ref)
    }
}

#[cfg(test)]
mod tests {
    use {Module};

    #[test]
    fn test() {
        let module = Module::new();
        let mut relooper = module.relooper();
        let b1 = relooper.add_block(module.nop());
        let b2 = relooper.add_block(module.nop());

        relooper.add_branch(b1, b2, None, None);

        let result = relooper.render(b1, 0);
        result.print();
    }

    // TODO: Create 2 reloopers?
}
