mod plugin;

use rustpython_vm::{self as vm, Settings, scope::Scope};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use extism_pdk::*;
use json::Value;
use plugin::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde_json::json;

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
        vms.get(id).expect("VM should exist after insertion").clone()
    })
}

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "eval_python" => eval_python(input),
        _ => Ok(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some(format!("Unknown tool: {}", input.params.name)),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        }),
    }
}

fn eval_python(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
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
            Ok(output) => Ok(CallToolResult {
                is_error: None,
                content: vec![Content {
                    annotations: None,
                    text: Some(output),
                    mime_type: Some("text/plain".to_string()),
                    r#type: ContentType::Text,
                    data: None,
                }],
            }),
            Err(exc) => {
                let mut error_msg = String::new();
                stored_vm.interp.enter(|vm| {
                    vm.write_exception(&mut error_msg, &exc).unwrap_or_default();
                });
                Ok(CallToolResult {
                    is_error: Some(true),
                    content: vec![Content {
                        annotations: None,
                        text: Some(error_msg),
                        mime_type: None,
                        r#type: ContentType::Text,
                        data: None,
                    }],
                })
            }
        }
    } else {
        Ok(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some("Please provide Python code to evaluate".into()),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        })
    }
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult{
        tools: vec![
            ToolDescription {
                name: "eval_shell".into(),
                description: "Execute shell commands in a sandboxed environment. Use this tool when you need to:
- Run system commands for file operations or process management
- Execute shell scripts for automation tasks
- Perform system administration operations
- Chain commands with pipes and redirections
- Access environment variables and system information
Perfect for system automation, DevOps tasks, and command-line operations. Note: Security warning - currently implemented incorrectly with Python. Requires proper shell sandbox implementation.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "The shell command to execute",
                        },
                    },
                    "required": ["code"],
                })
                .as_object()
                .expect("JSON schema should be valid object")
                .clone(),
            },
        ],
    })
}
