pub extern crate binaryen_sys;

pub use binaryen_sys as ffi;

use std::rc::Rc;
use std::os::raw::{c_char, c_int};
use std::ptr;

mod name;
use name::Name;

struct InnerModule {
    raw: ffi::BinaryenModuleRef,
}

impl Drop for InnerModule {
    fn drop(&mut self) {
        unsafe { ffi::BinaryenModuleDispose(self.raw) }
    }
}

pub struct Module {
    inner: Rc<InnerModule>,
}

impl Module {
    pub fn new() -> Module {
        let raw = unsafe { ffi::BinaryenModuleCreate() };
        Module::from_raw(raw)
    }

    pub fn from_raw(raw: ffi::BinaryenModuleRef) -> Module {
        Module {
            inner: Rc::new(InnerModule { raw }),
        }
    }

    pub fn trace(&mut self) {
        unsafe {
            ffi::BinaryenSetAPITracing(1);
        }
    }

    pub fn auto_drop(&mut self) {
        unsafe {
            ffi::BinaryenModuleAutoDrop(self.inner.raw);
        }
    }

    pub fn optimize(&mut self) {
        unsafe { ffi::BinaryenModuleOptimize(self.inner.raw) }
    }

    pub fn is_valid(&self) -> bool {
        unsafe { ffi::BinaryenModuleValidate(self.inner.raw) == 1 }
    }

    pub fn print(&self) {
        unsafe { ffi::BinaryenModulePrint(self.inner.raw) }
    }

    pub fn set_start(&mut self, fn_ref: &FnRef) {
        unsafe {
            ffi::BinaryenSetStart(self.inner.raw, fn_ref.inner);
        }
    }

    pub fn write(&mut self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::with_capacity(8192);
        unsafe {
            let written =
                ffi::BinaryenModuleWrite(self.inner.raw, buf.as_mut_ptr() as *mut c_char, 8192);
            if written == buf.capacity() {
                // TODO:
                panic!("unimplemented");
            }
            buf.set_len(written);
        };
        buf.shrink_to_fit();
        buf
    }

    pub fn set_memory(
        &mut self,
        initial: u32,
        maximal: u32,
        name: Option<Name>,
        segments: &[Segment],
    ) {
        let name_ptr = name.map_or(ptr::null(), |n| n.as_ptr());
        let mut segment_datas: Vec<_> = segments.iter().map(|s| s.data.as_ptr()).collect();
        let mut segment_sizes: Vec<_> = segments.iter().map(|s| s.data.len() as u32).collect();
        let segments_count = segments.len();

        unsafe {
            let mut segment_offsets: Vec<_> =
                segments.iter().map(|s| s.offset_expr.to_raw()).collect();

            ffi::BinaryenSetMemory(
                self.inner.raw,
                initial,
                maximal,
                name_ptr,
                segment_datas.as_mut_ptr() as *mut *const c_char,
                segment_offsets.as_mut_ptr(),
                segment_sizes.as_mut_ptr(),
                segments_count as _,
            )
        }
    }

    pub fn add_fn_type(&self, name: Option<Name>, param_tys: &[ValueTy], result_ty: Ty) -> FnType {
        let name_ptr = name.map_or(ptr::null(), |n| n.as_ptr());
        let raw = unsafe {
            let mut param_tys_raw = param_tys
                .iter()
                .cloned()
                .map(|ty| ty.into())
                .collect::<Vec<_>>();
            ffi::BinaryenAddFunctionType(
                self.inner.raw,
                name_ptr,
                result_ty.into(),
                param_tys_raw.as_mut_ptr(),
                param_tys_raw.len() as _,
            )
        };
        FnType { raw }
    }

    pub fn add_fn(&self, name: Name, fn_ty: &FnType, var_tys: &[ValueTy], body: Expr) -> FnRef {
        let name_ptr = name.as_ptr();
        let inner = unsafe {
            let mut var_tys_raw = var_tys
                .iter()
                .cloned()
                .map(|ty| ty.into())
                .collect::<Vec<_>>();
            ffi::BinaryenAddFunction(
                self.inner.raw,
                name_ptr,
                fn_ty.raw,
                var_tys_raw.as_mut_ptr(),
                var_tys_raw.len() as _,
                body.to_raw(),
            )
        };
        FnRef { inner }
    }

    pub fn add_global(&self, name: Name, ty: ValueTy, mutable: bool, init: Expr) {
        let name_ptr = name.as_ptr();
        unsafe {
            ffi::BinaryenAddGlobal(
                self.inner.raw,
                name_ptr,
                ty.into(),
                mutable as c_int,
                init.to_raw(),
            );
        }
    }

    pub fn add_import(
        &mut self,
        internal_name: Name,
        external_module_name: Name,
        external_base_name: Name,
        fn_ty: &FnType,
    ) {
        let internal_name_ptr = internal_name.as_ptr();
        let external_module_name_ptr = external_module_name.as_ptr();
        let external_base_name_ptr = external_base_name.as_ptr();
        unsafe {
            ffi::BinaryenAddImport(
                self.inner.raw,
                internal_name_ptr,
                external_module_name_ptr,
                external_base_name_ptr,
                fn_ty.raw,
            );
        }
    }

    // TODO: undefined ty?
    // https://github.com/WebAssembly/binaryen/blob/master/src/binaryen-c.h#L272
    pub fn block(&mut self, name: Option<Name>, children: &[Expr], ty: Ty) -> Expr {
        let name_ptr = name.map_or(ptr::null(), |n| n.as_ptr());

        let raw_expr = unsafe {
            let mut children_raw: Vec<_> = children.iter().map(|ty| ty.to_raw()).collect();
            ffi::BinaryenBlock(
                self.inner.raw,
                name_ptr,
                children_raw.as_mut_ptr(),
                children_raw.len() as _,
                ty.into(),
            )
        };
        Expr::from_raw(self, raw_expr)
    }

    pub fn const_(&mut self, literal: Literal) -> Expr {
        let raw_expr = unsafe { ffi::BinaryenConst(self.inner.raw, literal.into()) };
        Expr::from_raw(self, raw_expr)
    }

    pub fn load(
        &mut self,
        bytes: u32,
        signed: bool,
        offset: u32,
        align: u32,
        ty: ValueTy,
        ptr: Expr,
    ) -> Expr {
        let raw_expr = unsafe {
            ffi::BinaryenLoad(
                self.inner.raw,
                bytes,
                signed as i8,
                offset,
                align,
                ty.into(),
                ptr.to_raw(),
            )
        };
        Expr::from_raw(self, raw_expr)
    }

    pub fn store(
        &mut self,
        bytes: u32,
        offset: u32,
        align: u32,
        ptr: Expr,
        value: Expr,
        ty: ValueTy,
    ) -> Expr {
        let raw_expr = unsafe {
            ffi::BinaryenStore(
                self.inner.raw,
                bytes,
                offset,
                align,
                ptr.to_raw(),
                value.to_raw(),
                ty.into(),
            )
        };
        Expr::from_raw(self, raw_expr)
    }

    pub fn get_global(&mut self, name: Name, ty: ValueTy) -> Expr {
        let global_name_ptr = name.as_ptr();
        let raw_expr =
            unsafe { ffi::BinaryenGetGlobal(self.inner.raw, global_name_ptr, ty.into()) };
        Expr::from_raw(self, raw_expr)
    }

    pub fn set_global(&mut self, name: Name, value: Expr) -> Expr {
        let global_name_ptr = name.as_ptr();
        let raw_expr =
            unsafe { ffi::BinaryenSetGlobal(self.inner.raw, global_name_ptr, value.to_raw()) };
        Expr::from_raw(self, raw_expr)
    }

    pub fn get_local(&mut self, index: u32, ty: ValueTy) -> Expr {
        let raw_expr = unsafe {
            ffi::BinaryenGetLocal(self.inner.raw, index as ffi::BinaryenIndex, ty.into())
        };
        Expr::from_raw(self, raw_expr)
    }

    pub fn set_local(&mut self, index: u32, value: Expr) -> Expr {
        let raw_expr = unsafe {
            ffi::BinaryenSetLocal(self.inner.raw, index as ffi::BinaryenIndex, value.to_raw())
        };
        Expr::from_raw(self, raw_expr)
    }

    pub fn tee_local(&mut self, index: u32, value: Expr) -> Expr {
        let raw_expr = unsafe {
            ffi::BinaryenTeeLocal(self.inner.raw, index as ffi::BinaryenIndex, value.to_raw())
        };
        Expr::from_raw(self, raw_expr)
    }

    pub fn ret(&mut self, value: Option<Expr>) -> Expr {
        let raw_expr = unsafe {
            let raw_value = value.map_or(ptr::null_mut(), |v| v.to_raw());
            ffi::BinaryenReturn(self.inner.raw, raw_value)
        };
        Expr::from_raw(self, raw_expr)
    }

    pub fn call(&mut self, name: Name, operands: &[Expr]) -> Expr {
        let name_ptr = name.as_ptr();
        let raw_expr = unsafe {
            let mut operands_raw: Vec<_> = operands.iter().map(|ty| ty.to_raw()).collect();
            ffi::BinaryenCall(
                self.inner.raw,
                name_ptr,
                operands_raw.as_mut_ptr(),
                operands_raw.len() as _,
                ffi::BinaryenNone(),
            )
        };
        Expr::from_raw(self, raw_expr)
    }

    pub fn call_import(&mut self, name: Name, operands: &[Expr], ty: Ty) -> Expr {
        let name_ptr = name.as_ptr();
        let raw_expr = unsafe {
            let mut operands_raw: Vec<_> = operands.iter().map(|ty| ty.to_raw()).collect();
            ffi::BinaryenCallImport(
                self.inner.raw,
                name_ptr,
                operands_raw.as_mut_ptr(),
                operands_raw.len() as _,
                ty.into(),
            )
        };
        Expr::from_raw(self, raw_expr)
    }

    pub fn binary(&mut self, op: BinaryOp, lhs: Expr, rhs: Expr) -> Expr {
        let raw_expr =
            unsafe { ffi::BinaryenBinary(self.inner.raw, op.into(), lhs.to_raw(), rhs.to_raw()) };
        Expr::from_raw(self, raw_expr)
    }
}

impl Default for Module {
    fn default() -> Module {
        Module::new()
    }
}

pub struct Segment<'a> {
    data: &'a [u8],
    offset_expr: Expr,
}

impl<'a> Segment<'a> {
    pub fn new(data: &[u8], offset_expr: Expr) -> Segment {
        Segment { data, offset_expr }
    }
}

pub enum BinaryOp {
    ClzI32,
    CtzI32,
    PopcntI32,
    NegF32,
    AbsF32,
    CeilF32,
    FloorF32,
    TruncF32,
    NearestF32,
    SqrtF32,
    EqZI32,
    ClzI64,
    CtzI64,
    PopcntI64,
    NegF64,
    AbsF64,
    CeilF64,
    FloorF64,
    TruncF64,
    NearestF64,
    SqrtF64,
    EqZI64,
    ExtendSI32,
    ExtendUI32,
    WrapI64,
    TruncSF32ToI32,
    TruncSF32ToI64,
    TruncUF32ToI32,
    TruncUF32ToI64,
    TruncSF64ToI32,
    TruncSF64ToI64,
    TruncUF64ToI32,
    TruncUF64ToI64,
    ReinterpretF32,
    ReinterpretF64,
    ConvertSI32ToF32,
    ConvertSI32ToF64,
    ConvertUI32ToF32,
    ConvertUI32ToF64,
    ConvertSI64ToF32,
    ConvertSI64ToF64,
    ConvertUI64ToF32,
    ConvertUI64ToF64,
    PromoteF32,
    DemoteF64,
    ReinterpretI32,
    ReinterpretI64,
    AddI32,
    SubI32,
    MulI32,
    DivSI32,
    DivUI32,
    RemSI32,
    RemUI32,
    AndI32,
    OrI32,
    XorI32,
    ShlI32,
    ShrUI32,
    ShrSI32,
    RotLI32,
    RotRI32,
    EqI32,
    NeI32,
    LtSI32,
    LtUI32,
    LeSI32,
    LeUI32,
    GtSI32,
    GtUI32,
    GeSI32,
    GeUI32,
    AddI64,
    SubI64,
    MulI64,
    DivSI64,
    DivUI64,
    RemSI64,
    RemUI64,
    AndI64,
    OrI64,
    XorI64,
    ShlI64,
    ShrUI64,
    ShrSI64,
    RotLI64,
    RotRI64,
    EqI64,
    NeI64,
    LtSI64,
    LtUI64,
    LeSI64,
    LeUI64,
    GtSI64,
    GtUI64,
    GeSI64,
    GeUI64,
    AddF32,
    SubF32,
    MulF32,
    DivF32,
    CopySignF32,
    MinF32,
    MaxF32,
    EqF32,
    NeF32,
    LtF32,
    LeF32,
    GtF32,
    GeF32,
    AddF64,
    SubF64,
    MulF64,
    DivF64,
    CopySignF64,
    MinF64,
    MaxF64,
    EqF64,
    NeF64,
    LtF64,
    LeF64,
    GtF64,
    GeF64,
}

impl From<BinaryOp> for ffi::BinaryenOp {
    fn from(binop: BinaryOp) -> ffi::BinaryenOp {
        use BinaryOp::*;
        unsafe {
            match binop {
                ClzI32 => ffi::BinaryenClzInt32(),
                CtzI32 => ffi::BinaryenCtzInt32(),
                PopcntI32 => ffi::BinaryenPopcntInt32(),
                NegF32 => ffi::BinaryenNegFloat32(),
                AbsF32 => ffi::BinaryenAbsFloat32(),
                CeilF32 => ffi::BinaryenCeilFloat32(),
                FloorF32 => ffi::BinaryenFloorFloat32(),
                TruncF32 => ffi::BinaryenTruncFloat32(),
                NearestF32 => ffi::BinaryenNearestFloat32(),
                SqrtF32 => ffi::BinaryenSqrtFloat32(),
                EqZI32 => ffi::BinaryenEqZInt32(),
                ClzI64 => ffi::BinaryenClzInt64(),
                CtzI64 => ffi::BinaryenCtzInt64(),
                PopcntI64 => ffi::BinaryenPopcntInt64(),
                NegF64 => ffi::BinaryenNegFloat64(),
                AbsF64 => ffi::BinaryenAbsFloat64(),
                CeilF64 => ffi::BinaryenCeilFloat64(),
                FloorF64 => ffi::BinaryenFloorFloat64(),
                TruncF64 => ffi::BinaryenTruncFloat64(),
                NearestF64 => ffi::BinaryenNearestFloat64(),
                SqrtF64 => ffi::BinaryenSqrtFloat64(),
                EqZI64 => ffi::BinaryenEqZInt64(),
                ExtendSI32 => ffi::BinaryenExtendSInt32(),
                ExtendUI32 => ffi::BinaryenExtendUInt32(),
                WrapI64 => ffi::BinaryenWrapInt64(),
                TruncSF32ToI32 => ffi::BinaryenTruncSFloat32ToInt32(),
                TruncSF32ToI64 => ffi::BinaryenTruncSFloat32ToInt64(),
                TruncUF32ToI32 => ffi::BinaryenTruncUFloat32ToInt32(),
                TruncUF32ToI64 => ffi::BinaryenTruncUFloat32ToInt64(),
                TruncSF64ToI32 => ffi::BinaryenTruncSFloat64ToInt32(),
                TruncSF64ToI64 => ffi::BinaryenTruncSFloat64ToInt64(),
                TruncUF64ToI32 => ffi::BinaryenTruncUFloat64ToInt32(),
                TruncUF64ToI64 => ffi::BinaryenTruncUFloat64ToInt64(),
                ReinterpretF32 => ffi::BinaryenReinterpretFloat32(),
                ReinterpretF64 => ffi::BinaryenReinterpretFloat64(),
                ConvertSI32ToF32 => ffi::BinaryenConvertSInt32ToFloat32(),
                ConvertSI32ToF64 => ffi::BinaryenConvertSInt32ToFloat64(),
                ConvertUI32ToF32 => ffi::BinaryenConvertUInt32ToFloat32(),
                ConvertUI32ToF64 => ffi::BinaryenConvertUInt32ToFloat64(),
                ConvertSI64ToF32 => ffi::BinaryenConvertSInt64ToFloat32(),
                ConvertSI64ToF64 => ffi::BinaryenConvertSInt64ToFloat64(),
                ConvertUI64ToF32 => ffi::BinaryenConvertUInt64ToFloat32(),
                ConvertUI64ToF64 => ffi::BinaryenConvertUInt64ToFloat64(),
                PromoteF32 => ffi::BinaryenPromoteFloat32(),
                DemoteF64 => ffi::BinaryenDemoteFloat64(),
                ReinterpretI32 => ffi::BinaryenReinterpretInt32(),
                ReinterpretI64 => ffi::BinaryenReinterpretInt64(),
                AddI32 => ffi::BinaryenAddInt32(),
                SubI32 => ffi::BinaryenSubInt32(),
                MulI32 => ffi::BinaryenMulInt32(),
                DivSI32 => ffi::BinaryenDivSInt32(),
                DivUI32 => ffi::BinaryenDivUInt32(),
                RemSI32 => ffi::BinaryenRemSInt32(),
                RemUI32 => ffi::BinaryenRemUInt32(),
                AndI32 => ffi::BinaryenAndInt32(),
                OrI32 => ffi::BinaryenOrInt32(),
                XorI32 => ffi::BinaryenXorInt32(),
                ShlI32 => ffi::BinaryenShlInt32(),
                ShrUI32 => ffi::BinaryenShrUInt32(),
                ShrSI32 => ffi::BinaryenShrSInt32(),
                RotLI32 => ffi::BinaryenRotLInt32(),
                RotRI32 => ffi::BinaryenRotRInt32(),
                EqI32 => ffi::BinaryenEqInt32(),
                NeI32 => ffi::BinaryenNeInt32(),
                LtSI32 => ffi::BinaryenLtSInt32(),
                LtUI32 => ffi::BinaryenLtUInt32(),
                LeSI32 => ffi::BinaryenLeSInt32(),
                LeUI32 => ffi::BinaryenLeUInt32(),
                GtSI32 => ffi::BinaryenGtSInt32(),
                GtUI32 => ffi::BinaryenGtUInt32(),
                GeSI32 => ffi::BinaryenGeSInt32(),
                GeUI32 => ffi::BinaryenGeUInt32(),
                AddI64 => ffi::BinaryenAddInt64(),
                SubI64 => ffi::BinaryenSubInt64(),
                MulI64 => ffi::BinaryenMulInt64(),
                DivSI64 => ffi::BinaryenDivSInt64(),
                DivUI64 => ffi::BinaryenDivUInt64(),
                RemSI64 => ffi::BinaryenRemSInt64(),
                RemUI64 => ffi::BinaryenRemUInt64(),
                AndI64 => ffi::BinaryenAndInt64(),
                OrI64 => ffi::BinaryenOrInt64(),
                XorI64 => ffi::BinaryenXorInt64(),
                ShlI64 => ffi::BinaryenShlInt64(),
                ShrUI64 => ffi::BinaryenShrUInt64(),
                ShrSI64 => ffi::BinaryenShrSInt64(),
                RotLI64 => ffi::BinaryenRotLInt64(),
                RotRI64 => ffi::BinaryenRotRInt64(),
                EqI64 => ffi::BinaryenEqInt64(),
                NeI64 => ffi::BinaryenNeInt64(),
                LtSI64 => ffi::BinaryenLtSInt64(),
                LtUI64 => ffi::BinaryenLtUInt64(),
                LeSI64 => ffi::BinaryenLeSInt64(),
                LeUI64 => ffi::BinaryenLeUInt64(),
                GtSI64 => ffi::BinaryenGtSInt64(),
                GtUI64 => ffi::BinaryenGtUInt64(),
                GeSI64 => ffi::BinaryenGeSInt64(),
                GeUI64 => ffi::BinaryenGeUInt64(),
                AddF32 => ffi::BinaryenAddFloat32(),
                SubF32 => ffi::BinaryenSubFloat32(),
                MulF32 => ffi::BinaryenMulFloat32(),
                DivF32 => ffi::BinaryenDivFloat32(),
                CopySignF32 => ffi::BinaryenCopySignFloat32(),
                MinF32 => ffi::BinaryenMinFloat32(),
                MaxF32 => ffi::BinaryenMaxFloat32(),
                EqF32 => ffi::BinaryenEqFloat32(),
                NeF32 => ffi::BinaryenNeFloat32(),
                LtF32 => ffi::BinaryenLtFloat32(),
                LeF32 => ffi::BinaryenLeFloat32(),
                GtF32 => ffi::BinaryenGtFloat32(),
                GeF32 => ffi::BinaryenGeFloat32(),
                AddF64 => ffi::BinaryenAddFloat64(),
                SubF64 => ffi::BinaryenSubFloat64(),
                MulF64 => ffi::BinaryenMulFloat64(),
                DivF64 => ffi::BinaryenDivFloat64(),
                CopySignF64 => ffi::BinaryenCopySignFloat64(),
                MinF64 => ffi::BinaryenMinFloat64(),
                MaxF64 => ffi::BinaryenMaxFloat64(),
                EqF64 => ffi::BinaryenEqFloat64(),
                NeF64 => ffi::BinaryenNeFloat64(),
                LtF64 => ffi::BinaryenLtFloat64(),
                LeF64 => ffi::BinaryenLeFloat64(),
                GtF64 => ffi::BinaryenGtFloat64(),
                GeF64 => ffi::BinaryenGeFloat64(),
            }
        }
    }
}

// TODO: Host

pub struct FnType {
    raw: ffi::BinaryenFunctionTypeRef,
}

pub struct FnRef {
    inner: ffi::BinaryenFunctionRef,
}

/// Type of the values. These can be found on the stack and
/// in the local vars.
#[derive(Copy, Clone)]
pub enum ValueTy {
    I32,
    I64,
    F32,
    F64,
}

pub struct Ty(Option<ValueTy>);

impl Ty {
    pub fn none() -> Ty {
        Ty(None)
    }

    pub fn value(ty: ValueTy) -> Ty {
        Ty(Some(ty))
    }
}

impl From<ValueTy> for ffi::BinaryenType {
    fn from(ty: ValueTy) -> ffi::BinaryenType {
        unsafe {
            match ty {
                ValueTy::I32 => ffi::BinaryenInt32(),
                ValueTy::I64 => ffi::BinaryenInt64(),
                ValueTy::F32 => ffi::BinaryenFloat32(),
                ValueTy::F64 => ffi::BinaryenFloat64(),
            }
        }
    }
}

impl From<Ty> for ffi::BinaryenType {
    fn from(ty: Ty) -> ffi::BinaryenType {
        match ty.0 {
            Some(ty) => ty.into(),
            None => unsafe { ffi::BinaryenNone() },
        }
    }
}

pub struct Expr {
    _module_ref: Rc<InnerModule>,
    raw: ffi::BinaryenExpressionRef,
}

impl Expr {
    pub fn from_raw(module: &Module, raw: ffi::BinaryenExpressionRef) -> Expr {
        Expr {
            _module_ref: Rc::clone(&module.inner),
            raw,
        }
    }

    pub unsafe fn to_raw(&self) -> ffi::BinaryenExpressionRef {
        self.raw
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Literal {
    I32(u32),
    I64(u64),
    F32(f32),
    F64(f64),
}

impl From<Literal> for ffi::BinaryenLiteral {
    fn from(literal: Literal) -> ffi::BinaryenLiteral {
        unsafe {
            match literal {
                Literal::I32(v) => ffi::BinaryenLiteralInt32(v as i32),
                Literal::I64(v) => ffi::BinaryenLiteralInt64(v as i64),
                Literal::F32(v) => ffi::BinaryenLiteralFloat32(v),
                Literal::F64(v) => ffi::BinaryenLiteralFloat64(v),
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RelooperBlockId(usize);

pub struct Relooper {
    raw: ffi::RelooperRef,
    blocks: Vec<ffi::RelooperBlockRef>,
}

impl Relooper {
    pub fn new() -> Relooper {
        Relooper {
            raw: unsafe { ffi::RelooperCreate() },
            blocks: Vec::new(),
        }
    }

    pub fn add_block(&mut self, expr: Expr) -> RelooperBlockId {
        let raw = unsafe { ffi::RelooperAddBlock(self.raw, expr.raw) };
        let index = self.blocks.len();
        self.blocks.push(raw);

        RelooperBlockId(index)
    }

    pub fn render(self, module: &Module, entry_id: RelooperBlockId, label_helper: u32) -> Expr {
        let entry = self.blocks[entry_id.0];
        let raw = unsafe {
            ffi::RelooperRenderAndDispose(self.raw, entry, label_helper as _, module.inner.raw)
        };
        Expr::from_raw(module, raw)
    }

    pub fn add_branch(
        &mut self,
        from: RelooperBlockId,
        to: RelooperBlockId,
        condition: Option<Expr>,
        code: Option<Expr>,
    ) {
        let from_block = self.blocks[from.0];
        let to_block = self.blocks[to.0];

        unsafe {
            let condition_ptr = condition.map_or(ptr::null_mut(), |e| e.raw);
            let code_ptr = code.map_or(ptr::null_mut(), |e| e.raw);
            ffi::RelooperAddBranch(from_block as _, to_block as _, condition_ptr, code_ptr)
        }
    }
}

impl Default for Relooper {
    fn default() -> Relooper {
        Relooper::new()
    }
}

// see https://github.com/WebAssembly/binaryen/blob/master/test/example/c-api-hello-world.c
#[test]
fn test_hello_world() {
    let mut module = Module::new();

    let params = &[ValueTy::I32, ValueTy::I32];
    let iii = module.add_fn_type(Some("iii".into()), params, Ty::value(ValueTy::I32));

    let x = module.get_local(0, ValueTy::I32);
    let y = module.get_local(1, ValueTy::I32);
    let add = module.binary(BinaryOp::AddI32, x, y);

    let _adder = module.add_fn("adder".into(), &iii, &[], add);

    assert!(module.is_valid());
    module.print();
}
