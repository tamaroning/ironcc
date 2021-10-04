extern crate llvm_sys as llvm;
#[warn(unused_import_braces)]
use self::llvm::core::*;
use self::llvm::prelude::*;
use crate::node;
use crate::types;
/*
use self::llvm::execution_engine;
use self::llvm::target::*;
use llvm::execution_engine::LLVMCreateExecutionEngineForModule;
use llvm::execution_engine::LLVMDisposeExecutionEngine;
use llvm::execution_engine::LLVMGetFunctionAddress;
use llvm::execution_engine::LLVMLinkInMCJIT;
use std::mem;
*/
use node::{BinaryOps, UnaryOps, AST};
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;
use types::Type;

#[derive(Debug)]
pub struct VarInfo {
    ty: Type,
    llvm_val: LLVMValueRef,
}

impl VarInfo {
    pub fn new(ty: Type, llvm_val: LLVMValueRef) -> VarInfo {
        VarInfo {
            ty: ty,
            llvm_val: llvm_val,
        }
    }
}

pub struct Codegen {
    context: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    cur_func: Option<LLVMValueRef>,
    local_varmap: Vec<HashMap<String, VarInfo>>,
}

#[warn(temporary_cstring_as_ptr)]
pub unsafe fn cstr(s: &str) -> *const i8 {
    CString::new(s).unwrap().as_ptr()
}

pub unsafe fn inside_load(ast: &AST) -> &AST {
    match ast {
        AST::Load(node) => node,
        _ => panic!("Error: ast load"),
    }
}

impl Codegen {
    pub unsafe fn new(mod_name: &str) -> Codegen {
        let c_mod_name = CString::new(mod_name).unwrap();
        Codegen {
            context: LLVMContextCreate(),
            module: LLVMModuleCreateWithNameInContext(c_mod_name.as_ptr(), LLVMContextCreate()),
            builder: LLVMCreateBuilderInContext(LLVMContextCreate()),
            cur_func: None,
            local_varmap: Vec::new(),
        }
    }

    // TODO: convert type
    pub unsafe fn typecast(&mut self, val: LLVMValueRef, to: LLVMTypeRef) -> LLVMValueRef {
        let from = LLVMTypeOf(val);
        match LLVMGetTypeKind(from) {
            llvm::LLVMTypeKind::LLVMPointerTypeKind => match LLVMGetTypeKind(to) {
                llvm::LLVMTypeKind::LLVMIntegerTypeKind => {
                    return LLVMBuildPtrToInt(self.builder, val, to, cstr("cast"));
                }
                _ => panic!(),
            },
            _ => val,
        }
    }

    pub unsafe fn type_to_llvmty(&self, ty: &Type) -> LLVMTypeRef {
        match &ty {
            Type::Int => LLVMInt32Type(),
            Type::Ptr(basety) => LLVMPointerType(self.type_to_llvmty(&*basety), 0),
            Type::Func(ret_type, param_types, _) => LLVMFunctionType(
                self.type_to_llvmty(ret_type),
                || -> *mut LLVMTypeRef {
                    let mut param_llvm_types: Vec<LLVMTypeRef> = Vec::new();
                    for param_type in param_types {
                        param_llvm_types.push(self.type_to_llvmty(param_type));
                    }
                    param_llvm_types.as_mut_slice().as_mut_ptr()
                }(),
                param_types.len() as u32,
                0, // is variable number of args
            ),
            _ => panic!("Unsupported type"),
        }
    }

    pub unsafe fn gen_program(&mut self, program: Vec<AST>) {
        for top_level in program {
            match top_level {
                AST::FuncDef(func_ty, func_name, body) => {
                    self.gen_func_def(func_ty, func_name, body);
                }
                _ => panic!("Unsupported node type"),
            }
        }
        // Debug
        LLVMDisposeBuilder(self.builder);
        LLVMDumpModule(self.module);
        /*
        //JIT exec
        // build engine
        let mut ee = mem::uninitialized();
        let mut out = mem::zeroed();
        //LLVMLinkInMCJIT();
        LLVM_InitializeNativeTarget();
        LLVM_InitializeNativeAsmParser();
        LLVM_InitializeNativeAsmParser();
        //LLVMInitializeX

        LLVMCreateExecutionEngineForModule(&mut ee, self.module, &mut out);

        let addr = LLVMGetFunctionAddress(ee, b"main\0".as_ptr() as *const _);
        let f: extern "C" fn() -> i32 = mem::transmute(addr);

        println!("ret = {}", f());

        LLVMDisposeExecutionEngine(ee);
        LLVMContextDispose(self.context);
        */
    }

    pub unsafe fn gen_func_def(&mut self, func_ty: Box<Type>, func_name: String, body: Box<AST>) {
        let func_ty = self.type_to_llvmty(&func_ty);
        let func = LLVMAddFunction(self.module, cstr(func_name.as_str()), func_ty);
        let bb_entry = LLVMAppendBasicBlock(func, cstr("entry"));
        LLVMPositionBuilderAtEnd(self.builder, bb_entry);

        self.cur_func = Some(func);

        self.local_varmap.push(HashMap::new());
        // TODO: register arguments as local variables

        self.gen(&*body);
        println!("{:?}", self.local_varmap.last_mut().unwrap());
        self.local_varmap.pop();
    }

    pub unsafe fn gen(&mut self, ast: &AST) -> Option<(LLVMValueRef, Option<Type>)> {
        match &ast {
            AST::Block(ref block) => self.gen_block(block),
            AST::UnaryOp(ref ast, ref op) => self.gen_unary_op(&**ast, &*op),
            AST::BinaryOp(ref lhs, ref rhs, ref op) => self.gen_binary_op(&**lhs, &**rhs, &*op),
            AST::Int(ref n) => self.make_int(*n as u64, false),
            AST::Return(None) => Some((LLVMBuildRetVoid(self.builder), None)),
            AST::Return(Some(ref val)) => self.gen_return(val),
            AST::Load(ref expr) => self.gen_load(expr),
            AST::Variable(ref name) => self.gen_var(name),
            AST::VariableDecl(ref ty, ref name, ref init_opt) => {
                self.gen_local_var_decl(ty, name, init_opt)
            }
            _ => None,
        }
    }

    pub unsafe fn gen_block(&mut self, block: &Vec<AST>) -> Option<(LLVMValueRef, Option<Type>)> {
        // TODO: support scope
        for ast in block {
            self.gen(ast);
        }
        None
    }

    pub unsafe fn gen_local_var_decl(
        &mut self,
        ty: &Type,
        name: &String,
        init_opt: &Option<Box<AST>>,
    ) -> Option<(LLVMValueRef, Option<Type>)> {
        let func = self.cur_func.unwrap();
        let builder = LLVMCreateBuilderInContext(self.context);
        let entry_bb = LLVMGetEntryBasicBlock(func);
        // insert declaration at first position of the function
        let first_inst = LLVMGetFirstInstruction(entry_bb);
        if first_inst == ptr::null_mut() {
            LLVMPositionBuilderAtEnd(builder, entry_bb);
        } else {
            LLVMPositionBuilderBefore(builder, first_inst);
        }
        let llvm_ty = self.type_to_llvmty(ty);
        let var = LLVMBuildAlloca(builder, llvm_ty, cstr(name.as_str()));

        self.local_varmap
            .last_mut()
            .unwrap()
            .insert(name.clone(), VarInfo::new(ty.clone(), var));

        // TODO: Is it OK not to go back to the function end?
        //LLVMPositionBuilderAtEnd(builder, entry_bb);

        // TODO: support initialization of variables

        None
    }

    pub unsafe fn gen_unary_op(
        &mut self,
        ast: &AST,
        op: &UnaryOps,
    ) -> Option<(LLVMValueRef, Option<Type>)> {
        let res = match op {
            UnaryOps::Plus => self.gen(ast),
            UnaryOps::Minus => {
                let val = self.gen(ast).unwrap().0;
                let neg = LLVMBuildNeg(self.builder, val, cstr("neg"));
                Some((neg, Some(Type::Int)))
            }
            UnaryOps::Addr => self.gen(inside_load(ast)),
            UnaryOps::Deref => self.gen_load(ast),
            _ => panic!("Unsupported unary op"),
        };
        res
    }

    pub unsafe fn gen_binary_op(
        &mut self,
        lhs: &AST,
        rhs: &AST,
        op: &BinaryOps,
    ) -> Option<(LLVMValueRef, Option<Type>)> {
        // TODO: assign
        if let BinaryOps::Assign = op {
            return self.gen_assign(&inside_load(lhs), rhs);
        }

        let (lhs_val, lhs_ty) = self.gen(&*lhs).unwrap();
        let (rhs_val, rhs_ty) = self.gen(&*rhs).unwrap();
        let lhs_ty = lhs_ty.unwrap();
        let rhs_ty = rhs_ty.unwrap();

        // TODO: type casting
        // support double binary

        // debug
        println!("type");
        println!("l: {:?}", lhs_ty);
        println!("r: {:?}", rhs_ty);

        // TODO: support pointer support
        if matches!(&lhs_ty, Type::Ptr(_)) {
            return self.gen_ptr_binary_op(lhs_val, rhs_val, lhs_ty, op);
        } else if matches!(&rhs_ty, Type::Ptr(_)) {
            return self.gen_ptr_binary_op(lhs_val, rhs_val, rhs_ty, op);
        }

        //let casted_lhs = self.typecast(lhs_val, LLVMInt64Type());
        //let casted_rhs = self.typecast(rhs_val, LLVMInt64Type());

        // TODO: cast types to the same int type (i32, i64)
        self.gen_int_binary_op(&lhs_val, &rhs_val, lhs_ty, op)
    }

    pub unsafe fn gen_ptr_binary_op(
        &mut self,
        lhs_val: LLVMValueRef,
        rhs_val: LLVMValueRef,
        ty: Type,
        op: &BinaryOps,
    ) -> Option<(LLVMValueRef, Option<Type>)> {
        let mut numidx = vec![match *op {
            BinaryOps::Add => rhs_val,
            BinaryOps::Sub => LLVMBuildSub(
                self.builder,
                self.make_int(0, true).unwrap().0,
                rhs_val,
                cstr("sub"),
            ),
            _ => panic!(),
        }];
        let ret = LLVMBuildGEP(
            self.builder,
            lhs_val,
            numidx.as_mut_slice().as_mut_ptr(),
            1,
            cstr("add"),
        );
        Some((ret, Some(ty)))
    }

    pub unsafe fn gen_int_binary_op(
        &mut self,
        lhs_val: &LLVMValueRef,
        rhs_val: &LLVMValueRef,
        ty: Type,
        op: &BinaryOps,
    ) -> Option<(LLVMValueRef, Option<Type>)> {
        let res = match op {
            BinaryOps::Add => LLVMBuildAdd(self.builder, *lhs_val, *rhs_val, cstr("add")),
            BinaryOps::Sub => LLVMBuildSub(self.builder, *lhs_val, *rhs_val, cstr("sub")),
            BinaryOps::Mul => LLVMBuildMul(self.builder, *lhs_val, *rhs_val, cstr("mul")),
            BinaryOps::Div => LLVMBuildSDiv(self.builder, *lhs_val, *rhs_val, cstr("sdiv")),
            BinaryOps::Eq => LLVMBuildICmp(
                self.builder,
                llvm::LLVMIntPredicate::LLVMIntEQ,
                *lhs_val,
                *rhs_val,
                cstr("eql"),
            ),
            BinaryOps::Ne => LLVMBuildICmp(
                self.builder,
                llvm::LLVMIntPredicate::LLVMIntNE,
                *lhs_val,
                *rhs_val,
                cstr("ne"),
            ),
            BinaryOps::Lt => LLVMBuildICmp(
                self.builder,
                llvm::LLVMIntPredicate::LLVMIntSLT,
                *lhs_val,
                *rhs_val,
                cstr("lt"),
            ),
            BinaryOps::Le => LLVMBuildICmp(
                self.builder,
                llvm::LLVMIntPredicate::LLVMIntSLE,
                *lhs_val,
                *rhs_val,
                cstr("le"),
            ),

            _ => panic!("Unsupported bianry op"),
        };
        // TODO: lhs_ty is OK?
        Some((res, Some(ty)))
    }

    pub unsafe fn gen_load(&mut self, ast: &AST) -> Option<(LLVMValueRef, Option<Type>)> {
        // TODO: support other types than AST::Variable
        match ast {
            AST::Variable(ref name) => {
                let (val, ty) = self.gen(ast).unwrap();
                let ty = ty.unwrap();
                let ret = LLVMBuildLoad(self.builder, val, cstr("var"));
                match ty {
                    Type::Ptr(origin_ty) => Some((ret, Some(*origin_ty))),
                    _ => panic!(),
                }
            },
            _ => {
                let (val, ty) = self.gen(ast).unwrap();
                let ret = LLVMBuildLoad(self.builder, val, cstr("var"));
                Some((ret, Some(Type::Ptr(Box::new(ty.unwrap())))))
            },
        }
    }

    pub unsafe fn gen_var(&mut self, name: &String) -> Option<(LLVMValueRef, Option<Type>)> {
        // TODO: support scope
        let var_info = self.local_varmap.last_mut().unwrap().get(name).unwrap();
        Some((
            var_info.llvm_val,
            Some(Type::Ptr(Box::new(var_info.ty.clone()))),
        ))
    }

    pub unsafe fn gen_assign(
        &mut self,
        lhs: &AST,
        rhs: &AST,
    ) -> Option<(LLVMValueRef, Option<Type>)> {
        let (rhs_val, ty) = self.gen(rhs).unwrap();
        let (dst, dst_ty) = self.gen(lhs).unwrap();
        LLVMBuildStore(self.builder, rhs_val, dst);
        let load = LLVMBuildLoad(self.builder, dst, cstr("load"));
        // TODO: ty is OK?
        Some((load, dst_ty))
    }

    pub unsafe fn gen_return(&mut self, ast: &AST) -> Option<(LLVMValueRef, Option<Type>)> {
        let ret_val = self.gen(ast);
        let ret = LLVMBuildRet(self.builder, ret_val.unwrap().0);
        Some((ret, None))
    }

    pub unsafe fn make_int(
        &mut self,
        n: u64,
        is_unsigned: bool,
    ) -> Option<(LLVMValueRef, Option<Type>)> {
        Some((
            LLVMConstInt(LLVMInt32Type(), n, if is_unsigned { 1 } else { 0 }),
            Some(Type::Int),
        ))
    }
}
