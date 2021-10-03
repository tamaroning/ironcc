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
    llvm_ty: LLVMTypeRef,
    llvm_val: LLVMValueRef,
}

impl VarInfo {
    pub fn new(llvm_ty: LLVMTypeRef, llvm_val: LLVMValueRef) -> VarInfo {
        VarInfo {
            llvm_ty: llvm_ty,
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
        let func = LLVMAddFunction(
            self.module,
            CString::new(func_name.as_str()).unwrap().as_ptr(),
            func_ty,
        );
        let bb_entry = LLVMAppendBasicBlock(func, CString::new("entry").unwrap().as_ptr());
        LLVMPositionBuilderAtEnd(self.builder, bb_entry);

        self.cur_func = Some(func);

        self.local_varmap.push(HashMap::new());
        // TODO: register arguments as local variables

        self.gen(&*body);
        self.local_varmap.pop();
    }

    pub unsafe fn gen(&mut self, ast: &AST) -> Option<LLVMValueRef> {
        match &ast {
            AST::Block(ref block) => self.gen_block(block),
            AST::UnaryOp(ref ast, ref op) => self.gen_unary_op(&**ast, &*op),
            AST::BinaryOp(ref lhs, ref rhs, ref op) => self.gen_binary_op(&**lhs, &**rhs, &*op),
            AST::Int(ref n) => self.make_int(*n as u64, false),
            AST::Return(None) => Some(LLVMBuildRetVoid(self.builder)),
            AST::Return(Some(ref val)) => self.gen_return(val),
            AST::Load(ref expr) => self.gen_load(expr),
            AST::Variable(ref name) => self.gen_var(name),
            AST::VariableDecl(ref ty, ref name, ref init_opt) => {
                self.gen_local_var_decl(ty, name, init_opt)
            }
            _ => None,
        }
    }

    pub unsafe fn gen_block(&mut self, block: &Vec<AST>) -> Option<LLVMValueRef> {
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
    ) -> Option<LLVMValueRef> {
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
        let var = LLVMBuildAlloca(
            builder,
            llvm_ty,
            CString::new(name.as_str()).unwrap().as_ptr(),
        );

        self.local_varmap.last_mut().unwrap()
            .insert(name.clone(), VarInfo::new(llvm_ty, var));

        // TODO: Is it OK not to go back to the function end?
        //LLVMPositionBuilderAtEnd(builder, entry_bb);

        // TODO: support initialization of variables

        Some(var)
    }

    pub unsafe fn gen_unary_op(&mut self, ast: &AST, op: &UnaryOps) -> Option<LLVMValueRef> {
        let res = match op {
            UnaryOps::Plus => self.gen(ast),
            UnaryOps::Minus => {
                let val = self.gen(ast).unwrap();
                let neg = LLVMBuildNeg(self.builder, val, CString::new("neg").unwrap().as_ptr());
                Some(neg)
            },
            _ => panic!("Unsupported unary op"),
        };
        res
    }

    pub unsafe fn gen_binary_op(
        &mut self,
        lhs: &AST,
        rhs: &AST,
        op: &BinaryOps,
    ) -> Option<LLVMValueRef> {
        // TODO: assign
        if let BinaryOps::Assign = op {
            return self.gen_assign(&inside_load(lhs), rhs);
        }

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

    pub unsafe fn gen_load(&mut self, var: &AST) -> Option<LLVMValueRef> {
        // TODO: support other types than AST::Variable
        match var {
            AST::Variable(ref name) => {
                let val = self.gen(var).unwrap();
                let ret = LLVMBuildLoad(
                    self.builder,
                    val,
                    CString::new("var".to_string()).unwrap().as_ptr(),
                );
                //let mut var_info = self.local_varmap.get(&name.clone()).unwrap();
                //*var_info.llvm_val = *ret;
                Some(ret)
            },
            _ => panic!("Error: AST::Load"),
        }
    }

    pub unsafe fn gen_var(&mut self, name: &String) -> Option<LLVMValueRef> {
        // TODO: support scope
        Some(self.local_varmap.last_mut().unwrap().get(name).unwrap().llvm_val)
    }

    pub unsafe fn gen_assign(&mut self, lhs: &AST, rhs: &AST) -> Option<LLVMValueRef> {
        let rhs_val = self.gen(rhs).unwrap();
        let dst = self.gen(lhs).unwrap();
        LLVMBuildStore(self.builder, rhs_val, dst);
        let load = LLVMBuildLoad(self.builder, dst, CString::new("load".to_string()).unwrap().as_ptr());
        Some(load)
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
