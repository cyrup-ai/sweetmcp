use rustpython_vm::{self as vm, Settings, scope::Scope};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use extism_pdk::*;
use serde_json::Value;
use sweetmcp_plugin_builder::prelude::*;
use sweetmcp_plugin_builder::{CallToolResult, Ready};

struct StoredVirtualMachine {
    interp: vm::Interpreter,
    scope: Scope,
}

impl StoredVirtualMachine {
    fn new() -> Self {
        let mut scope = None;
        let mut settings = Settings::default();
        settings.allow_external_library = false;

        let interp = vm::Interpreter::with_init(settings, |vm| {
            scope = Some(vm.new_scope_with_builtins());
        });

        StoredVirtualMachine {
            interp,
            scope: scope.expect("Scope should be initialized in Interpreter::with_init"),
        }
    }
}

thread_local! {
    static STORED_VMS: RefCell<HashMap<String, Rc<StoredVirtualMachine>>> = RefCell::default();
}

fn get_or_create_vm(id: &str) -> Rc<StoredVirtualMachine> {
    STORED_VMS.with(|cell| {
        let mut vms = cell.borrow_mut();
        if !vms.contains_key(id) {
            let stored_vm = StoredVirtualMachine::new();
            vms.insert(id.to_string(), Rc::new(stored_vm));
        }
        vms.get(id)
            .expect("VM should exist after insertion")
            .clone()
    })
}

/// Shell evaluation tool (currently using Python as placeholder)
struct ShellTool;

impl McpTool for ShellTool {
    const NAME: &'static str = "eval_shell";

    fn description(builder: DescriptionBuilder) -> DescriptionBuilder {
        builder
            .does("Execute shell commands in a sandboxed environment")
            .when("you need to run system commands for file operations or process management")
            .when("you need to execute shell scripts for automation tasks")
            .when("you need to perform system administration operations")
            .when("you need to chain commands with pipes and redirections")
            .when("you need to access environment variables and system information")
            .perfect_for("system automation, DevOps tasks, and command-line operations")
            .requires("Security warning - currently implemented incorrectly with Python. Requires proper shell sandbox implementation")
    }

    fn schema(builder: SchemaBuilder) -> Value {
        builder
            .required_string("code", "The shell command to execute")
            .build()
    }

    fn execute(args: Value) -> Result<CallToolResult, Error> {
        eval_python_as_shell(args)
    }
}

fn eval_python_as_shell(args: Value) -> Result<CallToolResult, Error> {
    if let Some(Value::String(code)) = args.get("code") {
        let stored_vm = get_or_create_vm("eval_python");

        let result = stored_vm.interp.enter(|vm| {
            match vm
                .compile(code, vm::compiler::Mode::Single, "<eval>".to_owned())
                .map_err(|err| vm.new_syntax_error(&err, Some(code)))
                .and_then(|code_obj| vm.run_code_obj(code_obj, stored_vm.scope.clone()))
            {
                Ok(output) => {
                    if !vm.is_none(&output) {
                        stored_vm
                            .scope
                            .globals
                            .set_item("last", output.clone(), vm)?;

                        match output.str(vm) {
                            Ok(s) => Ok(s.to_string()),
                            Err(e) => Err(e),
                        }
                    } else {
                        Ok("None".to_string())
                    }
                }
                Err(exc) => Err(exc),
            }
        });

        match result {
            Ok(output) => Ok(ContentBuilder::text(output)),
            Err(exc) => {
                let mut error_msg = String::new();
                stored_vm.interp.enter(|vm| {
                    vm.write_exception(&mut error_msg, &exc).unwrap_or_default();
                });
                Ok(ContentBuilder::error(error_msg))
            }
        }
    } else {
        Err(Error::msg("Please provide shell code to evaluate"))
    }
}

/// Create the plugin instance
fn plugin() -> McpPlugin<Ready> {
    mcp_plugin("eval_shell")
        .description("Shell command execution in sandboxed environment (currently using Python)")
        .tool::<ShellTool>()
        .serve()
}

// Generate standard MCP entry points
sweetmcp_plugin_builder::generate_mcp_functions!(plugin);
