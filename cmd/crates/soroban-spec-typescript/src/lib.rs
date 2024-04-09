#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::missing_panics_doc
)]

use std::{fs, io};

use crate::types::Type;
use itertools::Itertools;
use sha2::{Digest, Sha256};
use hcnet_xdr::curr::{Limits, ScSpecEntry, WriteXdr};

use types::Entry;

use soroban_spec::read::{from_wasm, FromWasmError};

pub mod boilerplate;
mod types;
pub mod wrapper;

#[derive(thiserror::Error, Debug)]
pub enum GenerateFromFileError {
    #[error("reading file: {0}")]
    Io(io::Error),
    #[error("sha256 does not match, expected: {expected}")]
    VerifySha256 { expected: String },
    #[error("parsing contract spec: {0}")]
    Parse(hcnet_xdr::curr::Error),
    #[error("getting contract spec: {0}")]
    GetSpec(FromWasmError),
}

pub fn generate_from_file(
    file: &str,
    verify_sha256: Option<&str>,
) -> Result<String, GenerateFromFileError> {
    // Read file.
    let wasm = fs::read(file).map_err(GenerateFromFileError::Io)?;

    // Produce hash for file.
    let sha256 = Sha256::digest(&wasm);
    let sha256 = format!("{sha256:x}");

    if let Some(verify_sha256) = verify_sha256 {
        if verify_sha256 != sha256 {
            return Err(GenerateFromFileError::VerifySha256 { expected: sha256 });
        }
    }

    // Generate code.
    let json = generate_from_wasm(&wasm).map_err(GenerateFromFileError::GetSpec)?;
    Ok(json)
}

pub fn generate_from_wasm(wasm: &[u8]) -> Result<String, FromWasmError> {
    let spec = from_wasm(wasm)?;
    let json = generate(&spec);
    Ok(json)
}

fn generate_class(fns: &[Entry], spec: &[ScSpecEntry]) -> String {
    let method_types = fns.iter().map(entry_to_method_type).join("");
    let from_jsons = fns
        .iter()
        .filter_map(entry_to_name_and_return_type)
        .map(|(method, return_type)| format!("{method}: this.txFromJSON<{return_type}>"))
        .join(",\n        ");
    let spec = spec
        .iter()
        .map(|s| format!("\"{}\"", s.to_xdr_base64(Limits::none()).unwrap()))
        .join(",\n        ");
    format!(
        r#"export interface Client {{{method_types}
}}
export class Client extends ContractClient {{
  constructor(public readonly options: ContractClientOptions) {{
    super(
      new ContractSpec([ {spec} ]),
      options
    )
  }}
  public readonly fromJSON = {{
    {from_jsons}
  }}
}}"#,
    )
}

pub fn generate(spec: &[ScSpecEntry]) -> String {
    let mut collected: Vec<_> = spec.iter().map(Entry::from).collect();
    if !spec.iter().any(is_error_enum) {
        collected.push(Entry::ErrorEnum {
            doc: String::new(),
            name: "Error".to_string(),
            cases: vec![],
        });
    }
    let (fns, other): (Vec<_>, Vec<_>) = collected
        .into_iter()
        .partition(|entry| matches!(entry, Entry::Function { .. }));
    let top = other.iter().map(entry_to_method_type).join("\n");
    let bottom = generate_class(&fns, spec);
    format!("{top}\n\n{bottom}")
}

fn doc_to_ts_doc(doc: &str, method: Option<&str>) -> String {
    if let Some(method) = method {
        let doc = if doc.is_empty() {
            String::new()
        } else {
            format!("   *\n   * {}", doc.split('\n').join("\n   * "))
        };
        return format!(
            r#"/**
   * Construct and simulate a {method} transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.{doc}
   */"#
        );
    }

    if doc.is_empty() {
        return String::new();
    }

    let doc = doc.split('\n').join("\n * ");
    format!(
        r#"/**
 * {doc}
 */
"#
    )
}

fn is_error_enum(entry: &ScSpecEntry) -> bool {
    matches!(entry, ScSpecEntry::UdtErrorEnumV0(_))
}

const METHOD_OPTIONS: &str = r"{
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }";

pub fn entry_to_name_and_return_type(entry: &Entry) -> Option<(String, String)> {
    if let Entry::Function { name, outputs, .. } = entry {
        Some((name.to_owned(), outputs_to_return_type(outputs)))
    } else {
        None
    }
}
pub fn outputs_to_return_type(outputs: &[Type]) -> String {
    match outputs {
        [] => "null".to_owned(),
        [output] => type_to_ts(output),
        outputs => format!("readonly [{}]", outputs.iter().map(type_to_ts).join(", ")),
    }
}

#[allow(clippy::too_many_lines)]
pub fn entry_to_method_type(entry: &Entry) -> String {
    match entry {
        Entry::Function {
            doc,
            name,
            inputs,
            outputs,
            ..
        } => {
            let input_vals = inputs.iter().map(func_input_to_arg_name).join(", ");
            let input = (!inputs.is_empty())
                .then(|| {
                    format!(
                        "{{{input_vals}}}: {{{}}}, ",
                        inputs.iter().map(func_input_to_ts).join(", ")
                    )
                })
                .unwrap_or_default();
            let doc = doc_to_ts_doc(doc, Some(name));
            let return_type = outputs_to_return_type(outputs);
            format!(
                r#"
  {doc}
  {name}: ({input}options?: {METHOD_OPTIONS}) => Promise<AssembledTransaction<{return_type}>>
"#
            )
        }

        Entry::Struct { doc, name, fields } => {
            let docs = doc_to_ts_doc(doc, None);
            let fields = fields.iter().map(field_to_ts).join("\n  ");
            format!(
                r#"
{docs}export interface {name} {{
  {fields}
}}
"#
            )
        }

        Entry::TupleStruct { doc, name, fields } => {
            let docs = doc_to_ts_doc(doc, None);
            let fields = fields.iter().map(type_to_ts).join(",  ");
            format!("{docs}export type {name} = readonly [{fields}];")
        }

        Entry::Union { name, doc, cases } => {
            let doc = doc_to_ts_doc(doc, None);
            let cases = cases.iter().map(case_to_ts).join(" | ");

            format!(
                r#"{doc}export type {name} = {cases};
"#
            )
        }
        Entry::Enum { doc, name, cases } => {
            let doc = doc_to_ts_doc(doc, None);
            let cases = cases.iter().map(enum_case_to_ts).join("\n  ");
            let name = (name == "Error")
                .then(|| format!("{name}s"))
                .unwrap_or(name.to_string());
            format!(
                r#"{doc}export enum {name} {{
  {cases}
}}
"#,
            )
        }
        Entry::ErrorEnum { doc, cases, .. } => {
            let doc = doc_to_ts_doc(doc, None);
            let cases = cases
                .iter()
                .map(|c| format!("{}: {{message:\"{}\"}}", c.value, c.doc))
                .join(",\n  ");
            format!(
                r#"{doc}export const Errors = {{
  {cases}
}}"#
            )
        }
    }
}

fn enum_case_to_ts(case: &types::EnumCase) -> String {
    let types::EnumCase { name, value, .. } = case;
    format!("{name} = {value},")
}

fn case_to_ts(case: &types::UnionCase) -> String {
    let types::UnionCase { name, values, .. } = case;
    format!(
        "{{tag: \"{name}\", values: {}}}",
        type_to_ts(&Type::Tuple {
            elements: values.clone(),
        })
    )
}

fn field_to_ts(field: &types::StructField) -> String {
    let types::StructField { doc, name, value } = field;
    let doc = doc_to_ts_doc(doc, None);
    let type_ = type_to_ts(value);
    format!("{doc}{name}: {type_};")
}

pub fn func_input_to_ts(input: &types::FunctionInput) -> String {
    let types::FunctionInput { name, value, .. } = input;
    let type_ = type_to_ts(value);
    format!("{name}: {type_}")
}

pub fn func_input_to_arg_name(input: &types::FunctionInput) -> String {
    let types::FunctionInput { name, .. } = input;
    name.to_string()
}

pub fn parse_arg_to_scval(input: &types::FunctionInput) -> String {
    let types::FunctionInput { name, value, .. } = input;
    match value {
        types::Type::Address => format!("{name}: new Address({name})"),
        _ => name.to_string(),
    }
}

pub fn type_to_ts(value: &types::Type) -> String {
    match value {
        types::Type::U64 => "u64".to_owned(),
        types::Type::I64 => "i64".to_owned(),
        types::Type::U128 => "u128".to_owned(),
        types::Type::I128 => "i128".to_owned(),
        types::Type::U32 => "u32".to_owned(),
        types::Type::I32 => "i32".to_owned(),
        types::Type::Bool => "boolean".to_owned(),
        types::Type::Symbol | types::Type::String => "string".to_owned(),
        types::Type::Map { key, value } => {
            format!("Map<{}, {}>", type_to_ts(key), type_to_ts(value))
        }
        types::Type::Option { value } => format!("Option<{}>", type_to_ts(value)),
        types::Type::Result { value, .. } => {
            format!("Result<{}>", type_to_ts(value))
        }
        types::Type::Vec { element } => format!("Array<{}>", type_to_ts(element)),
        types::Type::Tuple { elements } => {
            if elements.is_empty() {
                "void".to_owned()
            } else {
                format!("readonly [{}]", elements.iter().map(type_to_ts).join(", "))
            }
        }
        types::Type::Custom { name } => name.clone(),
        // TODO: Figure out what js type to map this to. There is already an `Error_` one that
        // ahalabs have added in the bindings, so.. maybe rename that?
        types::Type::Val => "any".to_owned(),
        types::Type::Error { .. } => "Error_".to_owned(),
        types::Type::Address => "string".to_string(),
        types::Type::Bytes | types::Type::BytesN { .. } => "Buffer".to_string(),
        types::Type::Void => "void".to_owned(),
        types::Type::U256 => "u256".to_string(),
        types::Type::I256 => "i256".to_string(),
        types::Type::Timepoint => "Timepoint".to_string(),
        types::Type::Duration => "Duration".to_string(),
    }
}
