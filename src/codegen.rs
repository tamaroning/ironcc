extern crate llvm_sys as llvm;
use self::llvm::core::*;
use self::llvm::prelude::*;
use crate::node;
use crate::types;
use node::{BinaryOps, UnaryOps, AST};
use std::ffi::CString;
use std::ptr;
use types::Type;

pub struct Codegen {
    context: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    cur_func: Option<LLVMValueRef>,
}

impl Codegen {
    pub unsafe fn new(mod_name: &str) -> Codegen {
        let c_mod_name = CString::new(mod_name).unwrap();
        Codegen {
            context: LLVMContextCreate(),
            module: LLVMModuleCreateWithNameInContext(c_mod_name.as_ptr(), LLVMContextCreate()),
            builder: LLVMCreateBuilderInContext(LLVMContextCreate()),
            cur_func: None,
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

    pub unsafe fn gen_test(&mut self) {
        let int_t = LLVMInt32TypeInContext(self.context);
        let func_t = LLVMFunctionType(int_t, ptr::null_mut(), 0, 0);
        let main_func =
            LLVMAddFunction(self.module, CString::new("main").unwrap().as_ptr(), func_t);
        let bb_entry = LLVMAppendBasicBlock(main_func, CString::new("entry").unwrap().as_ptr());

        LLVMPositionBuilderAtEnd(self.builder, bb_entry);
        LLVMBuildRet(self.builder, self.make_int(0 as u64, false).unwrap());
        LLVMDumpModule(self.module);
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
        LLVMDumpModule(self.module);
    }

    pub unsafe fn gen_func_def(&mut self, func_ty: Box<Type>, func_name: String, body: Box<AST>) {
        let func_ty = self.type_to_llvmty(&func_ty);
        let func = LLVMAddFunction(
            self.module,
            CString::new(func_name.as_str()).unwrap().as_ptr(),
            func_ty,
        );

        let bb_entry = LLVMAppendBasicBlock(func, CString::new("entry").unwrap().as_ptr());
        LLVMPositionBuilderAtEnd(self.builder, bb_entry);

        // TODO: register arguments as local variables

        self.gen(&*body);
    }

    pub unsafe fn gen(&mut self, ast: &AST) -> Option<LLVMValueRef> {
        match &ast {
            AST::Block(ref block) => self.gen_block(block),
            AST::BinaryOp(ref lhs, ref rhs, ref op) => self.gen_binary_op(&**lhs, &**rhs, &*op),
            AST::Int(ref n) => self.make_int(*n as u64, false),
            AST::Return(None) => Some(LLVMBuildRetVoid(self.builder)),
            AST::Return(Some(ref val)) => self.gen_return(val),
            _ => None,
        }
    }

    pub unsafe fn gen_block(&mut self, block: &Vec<AST>) -> Option<LLVMValueRef> {
        for ast in block {
            self.gen(ast);
        }
        None
    }

    pub unsafe fn gen_binary_op(
        &mut self,
        lhs: &AST,
        rhs: &AST,
        op: &BinaryOps,
    ) -> Option<LLVMValueRef> {
        // TODO: assign

        // TODO: type casting
        // support ptr binary, double binary

        self.gen_int_binary_op(lhs, rhs, op)
    }

    pub unsafe fn gen_int_binary_op(
        &mut self,
        lhs: &AST,
        rhs: &AST,
        op: &BinaryOps,
    ) -> Option<LLVMValueRef> {
        let lhs_val = self.gen(&*lhs).unwrap();
        let rhs_val = self.gen(&*rhs).unwrap();

        let res = match op {
            BinaryOps::Add => LLVMBuildAdd(
                self.builder,
                lhs_val,
                rhs_val,
                CString::new("add").unwrap().as_ptr(),
            ),
            BinaryOps::Sub => LLVMBuildSub(
                self.builder,
                lhs_val,
                rhs_val,
                CString::new("sub").unwrap().as_ptr(),
            ),
            BinaryOps::Mul => LLVMBuildMul(
                self.builder,
                lhs_val,
                rhs_val,
                CString::new("mul").unwrap().as_ptr(),
            ),
            BinaryOps::Div => LLVMBuildSDiv(
                self.builder,
                lhs_val,
                rhs_val,
                CString::new("sdiv").unwrap().as_ptr(),
            ),
            BinaryOps::Eq => LLVMBuildICmp(
                self.builder,
                llvm::LLVMIntPredicate::LLVMIntEQ,
                lhs_val,
                rhs_val,
                CString::new("eql").unwrap().as_ptr(),
            ),
            BinaryOps::Ne => LLVMBuildICmp(
                self.builder,
                llvm::LLVMIntPredicate::LLVMIntNE,
                lhs_val,
                rhs_val,
                CString::new("ne").unwrap().as_ptr(),
            ),
            BinaryOps::Lt => LLVMBuildICmp(
                self.builder,
                llvm::LLVMIntPredicate::LLVMIntSLT,
                lhs_val,
                rhs_val,
                CString::new("lt").unwrap().as_ptr(),
            ),
            BinaryOps::Le => LLVMBuildICmp(
                self.builder,
                llvm::LLVMIntPredicate::LLVMIntSLE,
                lhs_val,
                rhs_val,
                CString::new("le").unwrap().as_ptr(),
            ),

            _ => panic!("Unsupported bianry op"),
        };
        Some(res)
    }

    pub unsafe fn gen_return(&mut self, ast: &AST) -> Option<LLVMValueRef> {
        let ret_val = self.gen(ast);
        Some(LLVMBuildRet(self.builder, ret_val.unwrap()))
    }

    pub unsafe fn make_int(&mut self, n: u64, is_unsigned: bool) -> Option<LLVMValueRef> {
        Some(LLVMConstInt(
            LLVMInt32Type(),
            n,
            if is_unsigned { 1 } else { 0 },
        ))
    }
}
