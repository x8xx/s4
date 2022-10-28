pub type RuntimeArgs = crate::core::memory::array::Array<wasmer::Value>;

macro_rules! new_runtime_args {
    ($argc: expr) => {
        crate::core::memory::array::Array::<wasmer::Value>::new($argc)
    }
}

macro_rules! new_runtime {
    ($wasm: expr, $entry_func_name: expr, $entry_args: expr, $( $native_func_name: ident, $native_func: ident ),*) => {
        {
            let llvm_compiler = wasmer_compiler_llvm::LLVM::default();
            let store = wasmer::Store::new(&wasmer::Universal::new(llvm_compiler).engine());
            let module = wasmer::Module::from_binary(&store, $wasm).unwrap();

            let linear_memory = wasmer::Memory::new(&store, wasmer::MemoryType::new(1, None, false)).unwrap();
            $(
                let $native_func_name = wasmer::Function::new_native(&store, $native_func);
             )*
            let import_object = wasmer::imports! {
                "env" => {
                    "__linear_memory" => linear_memory,
                    $("$native_func_name" => $native_func_name,)*
                },
            };

            let instance = wasmer::Instance::new(&module, &import_object).unwrap();
            crate::core::runtime::wasm::runtime::Runtime {
                store,
                module,
                import_object,
                instance,
                entry_func_name: $entry_func_name,
                args: &$entry_args
            }
        }
    }
}

macro_rules! set_runtime_args  {
    ($runtime: expr, $new_args: expr) => {
        $runtime.args = $new_args;
    }
}

macro_rules! set_runtime_arg_i64  {
    ($runtime: expr, $index: expr, $value: expr) => {
        $runtime.args[$index] = wasmer::Value::I64($value);
    }
}

macro_rules! set_runtime_arg_i32  {
    ($runtime: expr, $index: expr, $value: expr) => {
        $runtime.args[$index] = wasmer::Value::I32($value);
    }
}


macro_rules! call_runtime_i32 {
    ($runtime: expr) => {
        $runtime.instance.exports.get_function(&$runtime.entry_func_name).unwrap().call($runtime.args.as_slice()).unwrap()[0].unwrap_i32()
    }
}


pub struct Runtime<'a> {
    pub store: wasmer::Store,
    pub module: wasmer::Module,
    pub import_object: wasmer::ImportObject,
    pub instance: wasmer::Instance,
    pub entry_func_name: String,
    pub args: &'a RuntimeArgs,
}

pub(crate) use new_runtime_args;
pub(crate) use new_runtime;
pub(crate) use set_runtime_args;
pub(crate) use set_runtime_arg_i64;
pub(crate) use set_runtime_arg_i32;
pub(crate) use call_runtime_i32;
