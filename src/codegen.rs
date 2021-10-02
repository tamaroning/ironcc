extern crate llvm_sys as llvm;
use crate::node;
use crate::types;
use node::{AST, BinaryOps, UnaryOps};
use types::Type;
use std::ptr;
use std::ffi::CString;
use self::llvm::core::*;
use self::llvm::prelude::*;

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
                    let mut param_llvm_types : Vec<LLVMTypeRef> = Vec::new();
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
        let func_t = LLVMFunctionType(
            int_t,
            ptr::null_mut(),
            0,
            0,
        );
        let main_func = LLVMAddFunction(self.module, CString::new("main").unwrap().as_ptr(), func_t);
        let bb_entry = LLVMAppendBasicBlock(main_func, CString::new("entry").unwrap().as_ptr());

        LLVMPositionBuilderAtEnd(self.builder, bb_entry);
        LLVMBuildRet(self.builder, self.make_int(0, false));
        LLVMDumpModule(self.module);
    }

    pub unsafe fn gen_program(&mut self, program: Vec<AST>) {
        for top_level in program {
            match top_level {
                AST::FuncDef(func_ty, func_name, body) => {
                    self.gen_func_def(func_ty, func_name,  body);
                },
                _ => panic!("Unsupported node type"),
            }
        }
        LLVMDumpModule(self.module);
    }

    pub unsafe fn gen_func_def(
        &mut self,
        func_ty: Box<Type>,
        func_name: String,
        body: Box<AST>,
    ) {
        let func_ty = self.type_to_llvmty(&func_ty);
        let func = LLVMAddFunction(
            self.module,
            CString::new(func_name.as_str()).unwrap().as_ptr(),
            func_ty,
        );

        let bb_entry = LLVMAppendBasicBlock(func, CString::new("entry").unwrap().as_ptr());
        LLVMPositionBuilderAtEnd(self.builder, bb_entry);
    }

    pub unsafe fn make_int(&mut self, n: u64, is_unsigned: bool) -> LLVMValueRef {
        LLVMConstInt(LLVMInt32TypeInContext(self.context), n, if is_unsigned {1} else {0})
    }
}