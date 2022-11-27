pub type RuntimeArgs = crate::core::memory::array::Array<wasmer::Value>;

macro_rules! new_runtime_args {
    ($argc: expr) => {
        crate::core::memory::array::Array::<wasmer::Value>::new($argc)
    }
}

macro_rules! new_runtime {
    ($wasm: expr, { $( $native_func_name: expr => $native_func: ident, )* }) => {
        {
            let store = wasmer::Store::default();
            #[cfg(feature="wasmer_llvm")]
            let store = {
                let llvm_compiler = wasmer_compiler_llvm::LLVM::default();
                wasmer::Store::new(&wasmer::Universal::new(llvm_compiler).engine())
            };
            let module = wasmer::Module::from_binary(&store, $wasm).unwrap();

            let linear_memory = wasmer::Memory::new(&store, wasmer::MemoryType::new(1, None, false)).unwrap();
            let mut import_object = wasmer::ImportObject::new();
            let mut env_obj = wasmer::Exports::new();
            env_obj.insert("__linear_memory", linear_memory);
            $(
                env_obj.insert($native_func_name, wasmer::Function::new_native(&store, $native_func));
            )*
            import_object.register("env", env_obj);

            let instance = wasmer::Instance::new(&module, &import_object).unwrap();
            crate::core::runtime::wasm::runtime::Runtime {
                store,
                module,
                import_object,
                instance,
            }
        }
    }
}

macro_rules! set_runtime_arg_i64  {
    ($runtime_args: expr, $index: expr, $value: expr) => {
        $runtime_args[$index] = wasmer::Value::I64($value);
    }
}

macro_rules! set_runtime_arg_i32  {
    ($runtime_args: expr, $index: expr, $value: expr) => {
        $runtime_args[$index] = wasmer::Value::I32($value);
    }
}


macro_rules! call_runtime {
    ($runtime: expr, $func_name: expr, $runtime_args: expr) => {
        $runtime.instance.exports.get_function($func_name).unwrap().call($runtime_args.as_slice()).unwrap()
    }
}

macro_rules! call_runtime_i32 {
    ($runtime: expr, $func_name: expr, $runtime_args: expr) => {
        $runtime.instance.exports.get_function($func_name).unwrap().call($runtime_args.as_slice()).unwrap()[0].unwrap_i32()
    }
}

pub struct Runtime {
    pub store: wasmer::Store,
    pub module: wasmer::Module,
    pub import_object: wasmer::ImportObject,
    pub instance: wasmer::Instance,
}

pub(crate) use new_runtime_args;
pub(crate) use new_runtime;
pub(crate) use set_runtime_arg_i64;
pub(crate) use set_runtime_arg_i32;
pub(crate) use call_runtime;
pub(crate) use call_runtime_i32;
