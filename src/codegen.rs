extern crate llvm_sys as llvm;

use crate::node;

use std::ptr;
use std::ffi::CString;
use self::llvm::core::*;
use self::llvm::prelude::*;
use node::{AST, BinaryOp};

pub struct Codegen {
    context: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
}

impl Codegen {
    pub unsafe fn new(mod_name: &str) -> Codegen {
        let c_mod_name = CString::new(mod_name).unwrap();
        Codegen {
            context: LLVMContextCreate(),
            module: LLVMModuleCreateWithNameInContext(c_mod_name.as_ptr(), LLVMContextCreate()),
            builder: LLVMCreateBuilderInContext(LLVMContextCreate()),
        }
    }

    pub unsafe fn gen(&mut self) {
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

    pub unsafe fn make_int(&mut self, n: u64, is_unsigned: bool) -> LLVMValueRef {
        LLVMConstInt(LLVMInt32TypeInContext(self.context), n, if is_unsigned {1} else {0})
    }
}