extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};
use syn::punctuated::Punctuated;
use std::io::Write;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref STATE: std::sync::Mutex<Option<Binder>> = std::sync::Mutex::new(None);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArgType {
    ArgNum,
    ArgStr,
}

struct BindedFunction {
    name : String,
    help : String,
    args : Vec<ArgType>, 
    return_type : ArgType,
}

struct Binder {
    name : String,
    file_name : String,
    function_name_prefix : String,
    functions : Vec<BindedFunction>,
}

impl Binder {
    fn new(name : &str, file_name : &str, function_name_prefix : &str) -> Self {
        Self {
            name : name.to_owned(),
            file_name : file_name.to_owned(),
            function_name_prefix: function_name_prefix.to_owned(),
            functions : vec![],
        }
    }
    fn generate_xml(&self) -> String {
        let preamble = format!("\
            <extension> \
            <name>{}</name> \n ", &self.name);

        let static_metadata = "\
            <version>1.0.0</version> \n\
            <packageID></packageID> \n\
            <ProductID></ProductID> \n\
            <date>23/12/21</date> \n\
            <license>Free to use, also for commercial games.</license> \n\
            <description></description> \n\
            <helpfile></helpfile> \n\
            <installdir></installdir> \n\
            <classname></classname> \n\
            <androidclassname></androidclassname> \n\
            <sourcedir></sourcedir> \n\
            <androidsourcedir></androidsourcedir> \n\
            <macsourcedir></macsourcedir> \n\
            <maclinkerflags></maclinkerflags> \n\
            <maccompilerflags></maccompilerflags> \n\
            <androidinject></androidinject> \n\
            <androidmanifestinject></androidmanifestinject> \n\
            <iosplistinject></iosplistinject> \n\
            <androidactivityinject></androidactivityinject> \n\
            <gradleinject></gradleinject> \n\
            <iosSystemFrameworks/> \n\
            <iosThirdPartyFrameworks/> \n\
            <ConfigOptions> \n\
                <Config name=\"Default\"> \n\
                <CopyToMask>105553895358702</CopyToMask> \n\
                </Config> \n\
            </ConfigOptions> \n\
            <androidPermissions/> \n\
            <IncludedResources/> \n\
            ";

        let mut functions_xml = String::default();
        for function in &self.functions {
            functions_xml += &function.generate_function_xml(&self.function_name_prefix);
        }

        let files = format!("\
            <files> \n\
                <file> \n\
                <filename>{}</filename> \n\
                <origname>extensions\\{}</origname> \n\
                <init></init> \n\
                <final></final> \n\
                <kind>1</kind> \n\
                <uncompress>0</uncompress> \n\
                <ConfigOptions> \n\
                    <Config name=\"Default\"> \n\
                    <CopyToMask>9223372036854775807</CopyToMask> \n\
                    </Config> \n\
                </ConfigOptions> \n\
                <ProxyFiles/> \n\
                <functions> \n\
                    {} \n\
                </functions> \n\
                <constants/> \n\
                </file> \n\
            </files> \n\
            </extension> \n\
            ", &self.file_name, &self.file_name, functions_xml);

        let mut full = String::default();

        full += &preamble;
        full += static_metadata;
        full += &files;
        full
    }
}


impl BindedFunction {
    fn generate_function_xml(&self, base_name : &str) -> String {
        /*
        <function>
        <name>rope_get_node_y</name>
        <externalName>get_node_y</externalName>
        <kind>11</kind>
        <help></help>
        <returnType>2</returnType>
        <argCount>1</argCount>
        <args>
        <arg>2</arg>
        </args>
        </function>
        */

        let mut args_generated = String::from("<args>");
        for arg in &self.args {
            args_generated += &format!("<arg>{}</arg>", (if *arg == ArgType::ArgNum {2} else {1}));
        }
        args_generated += "</args>";

        let return_type = if self.return_type == ArgType::ArgNum {2} else {1};

        let full_name = format!("{}_{}", base_name, &self.name);


        let xx = format!("<function>\n\
            <name>{}</name>\n\
            <externalName>{}</externalName>\n\
            <kind>11</kind>\n\
            <help>{}</help>\n\
            <returnType>{}</returnType>\n\
            <argCount>{}</argCount>\n\
            {}\n\
            </function>",
            &full_name,
            &self.name,
            &self.help,
            return_type,
            &self.args.len(),
            args_generated);

        xx
    }
}

struct StartInput {
    lib_name : String,
    dll : String,
    prefix_name : String,
}

fn string_from_lit(lit : &syn::Lit) -> String {
    if let syn::Lit::Str(x) = lit {
        x.value()
    }
    else {
        panic!("Expected strings as input");
    }
}

impl syn::parse::Parse for StartInput {
    fn parse(input : syn::parse::ParseStream)  -> syn::Result<Self> {
         let vars = Punctuated::<syn::Lit, syn::Token![,]>::parse_terminated(input)?;

         //panic!("Vars len {}", vars.len());

         Ok(Self {
             lib_name: string_from_lit(&vars[0]),
             dll: string_from_lit(&vars[1]),
             prefix_name: string_from_lit(&vars[2]),
         })
    }
}

#[proc_macro]
pub fn gms_bind_start(input : TokenStream) -> TokenStream {
    //let input_cloned = input.clone();
    let args = parse_macro_input!(input as StartInput);

    let mut guard = STATE.lock().unwrap();
    *guard = Some(Binder::new(&args.lib_name, &args.dll, &args.prefix_name));

    //input
    TokenStream::default()
}

#[proc_macro]
pub fn gms_bind_end(input : TokenStream) -> TokenStream {
    let guard = STATE.lock().unwrap();
    let state_ref = (*guard).as_ref().unwrap();
    let xml = state_ref.generate_xml();
    let path = format!("C:\\users\\daslocom\\tmp\\{}.xml", &state_ref.name);
    let mut file = std::fs::File::create(&path).unwrap();
    file.write(xml.as_bytes()).unwrap();
    file.flush().unwrap();

    input
}

#[proc_macro_attribute]
pub fn gms_bind(input : TokenStream, function_stream : TokenStream) -> TokenStream {
    //let parsed = syn::parse(function_stream).unwrap();
    let parsed = parse_macro_input!(function_stream as syn::ItemFn);
    //panic!("{:?}", parsed.sig.ident);

    let mut arg_types : Vec<ArgType> = vec![];

    for input in &parsed.sig.inputs {
        // Assume we can have two forms of argument
        // &str
        // f64
        if let (syn::FnArg::Typed(arg)) = input {
            match *arg.ty {
                syn::Type::Reference(_) | syn::Type::Ptr(_) => {
                    arg_types.push(ArgType::ArgStr);
                }
                _ => {
                    arg_types.push(ArgType::ArgNum);
                }
            }
        }
    }

    let mut return_type = ArgType::ArgNum;
    if let syn::ReturnType::Type(_, boxed_type) = &parsed.sig.output {
        match *boxed_type.clone() {
            syn::Type::Reference(_) | syn::Type::Ptr(_) => {
                return_type = ArgType::ArgStr;
            }
            _ => {
                return_type = ArgType::ArgNum;
            }
        }
    }

    //panic!("{:?}", arg_types);
    let function = BindedFunction {
        name : String::from(parsed.sig.ident.to_string()),
        help : "".to_owned(),
        args : arg_types,
        return_type,
    };

    let mut guard = STATE.lock().unwrap();
    let state_ref = (*guard).as_mut().unwrap();
    state_ref.functions.push(function);

    TokenStream::from(quote!(#parsed))
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
