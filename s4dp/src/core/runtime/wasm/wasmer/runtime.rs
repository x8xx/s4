macro_rules! new_runtime {
    ($wasm: expr, $entry_func_name: expr, $argc: expr, $( $native_func_name: ident, $native_func: ident ),*) => {
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
            let args = crate::core::memory::array::Array::<wasmer::Value>::new($argc);
            crate::core::runtime::wasm::runtime::Runtime {
                store,
                module,
                import_object,
                instance,
                entry_func_name: $entry_func_name,
                args
            }
        }
    }
}

macro_rules! set_runtime_args_i64  {
    ($runtime: expr, $index: expr, $value: expr) => {
        $runtime.args[$index] = wasmer::Value::I64($value);
    }
}

macro_rules! set_runtime_args_i32  {
    ($runtime: expr, $index: expr, $value: expr) => {
        $runtime.args[$index] = wasmer::Value::I32($value);
    }
}


macro_rules! call_runtime_i64 {
    ($runtime: expr) => {
        $runtime.instance.exports.get_function(&$runtime.entry_func_name).unwrap().call($runtime.args.as_slice()).unwrap()[0].unwrap_i64()
    }
}

// pub fn new() {
//     fn host_func() -> i32 {
//         0
//     }
//     let data: Vec<u8> = Vec::new();
//     let mut runtime = new_runtime!(&data, "parser".to_string(),  3, host_func, host_func);
//     runtime_set_args_i64!(runtime, 0, 512);
//     runtime_set_args_i32!(runtime, 1, 256);
//     call_runtime!(runtime);
// }


pub struct Runtime {
    pub store: wasmer::Store,
    pub module: wasmer::Module,
    pub import_object: wasmer::ImportObject,
    pub instance: wasmer::Instance,
    pub entry_func_name: String,
    pub args: crate::core::memory::array::Array<wasmer::Value>,
}

pub(crate) use new_runtime;
pub(crate) use set_runtime_args_i64;
pub(crate) use set_runtime_args_i32;
pub(crate) use call_runtime_i64;
