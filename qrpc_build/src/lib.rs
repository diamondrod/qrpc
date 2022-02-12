//! qrpc_build parses proto files and generates Rust code of client methods called from q.

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use std::collections::HashSet;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Error, ErrorKind, Write};
use std::iter::{Iterator, Peekable};
use std::path::Path;
use std::str::Chars;

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Global Variables
//++++++++++++++++++++++++++++++++++++++++++++++++++//

//%% q %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Name of a q file to generate.
const TARGET_Q_FILE_NAME: &'static str = "../q/grpc_client_methods.q";

/// Header of q file.
const Q_FILE_HEADER: &'static str = r#"/
* @file grpc_client_methods.
* @overview This is an auto-generated file by qrpc_build crate based on user-specified proto files defining client methods to communicate with a gRPC server.
\
"#;

//%% Rust %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Neme of a Rust file to generate.
const TARGET_RUST_FILE_NAME: &'static str = "../qrpc/src/client/qrpc.rs";

/// Header of Rust file.
const RUST_FILE_HEADER: &'static str = r#"//! This is an auto-generated code by qrpc_build crate.

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use crate::message::{decode_message, encode_to_message, PROTO_FILE_DESCRIPTOR};
use kdbplus::api::*;
use once_cell::sync::Lazy;
use prost_reflect::DynamicMessage;
use std::sync::RwLock;
use super::ENDPOINT;
use tokio::runtime::Builder;
use tonic::Request;
"#;

/// Definition of error buffer.
const ERROR_BUFFER: &'static str = r#"
//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Global Variables
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Buffer to store error message until q returns it.
static ERROR_BUFFER: Lazy<RwLock<String>> = Lazy::new(||{
    RwLock::new(String::new())
});

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Implementation
//++++++++++++++++++++++++++++++++++++++++++++++++++//
"#;

/// Template of a response handler for an exported client methods called from q with an empty response.
const EMPTY_RESPONSE_HANDLER: &'static str = r#" Ok(_response) => {{
                        new_null()
                    }}"#;

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Macros
//++++++++++++++++++++++++++++++++++++++++++++++++++//

//%% q %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Template of defining enum message in q file.
/// # Parameters
/// - `source`: Name of a protobuf enum message.
/// - `elements`: List of field names of the enum message.
macro_rules! enum_template {
    () => {
        r#"
// Source of enum message {source}.
{source}: `{elements};
"#
    };
}

/// Template to load Rust function to q.
/// # Parameters
/// - `method`: Name of an RPC.
macro_rules! method_load_template {
    () => {
        r#"
// Load gRPC client method {method}.
.grpc.{method}: `libqrpc 2: (`{method}; 1);
"#
    };
}

//%% Rust %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Lines to import client type and necessary messages.
/// # Parameters
/// - `package`: Package name of proto file.
/// - `snake_case_service`: Snake case name of the service.
/// - `service`: Service name.
/// - `messages`: Comma delimited message types to use.
macro_rules! import_template {
    () => {
        r#"
use super::proto::{package}::{snake_case_service}_client::{service}Client;
use super::proto::{package}::{{{messages}}};
"#
    };
}

/// Template of a response handler for an exported client methods called from q with a non-empty response.
/// # Parameters
/// - `fq_response_type`: Fully qualified response type name starting from package name.
/// - `response_type`: Response type.
macro_rules! non_empty_response_handler {
    () => {
r#" Ok(response) => {{
                        let message_descriptor = PROTO_FILE_DESCRIPTOR
                            .get_message_by_name("{fq_response_type}")
                            .unwrap();
                        let mut dynamic_message = DynamicMessage::new(message_descriptor.clone());
                        dynamic_message
                            .transcode_from::<{response_type}>(&response.into_inner())
                            .unwrap();
                        decode_message(&dynamic_message, message_descriptor.fields())
                    }}"#
    };
}

/// Template of exported client methods called from q.
/// # Parameters
/// - `method`: gRPC service request method.
/// - `client_name`: Client type in the form of [service]Clients.
/// - `fq_request_type`: Fully qualified request type name starting from package name.
/// - `request_type`: Request type.
/// - `response_handler`: Pre-built response handler with fully-qualified response type and response type.
macro_rules! method_template {
    () => {
        r#"
#[no_mangle]
pub extern "C" fn {method}_(message: K) -> K {{
    let message_descriptor = PROTO_FILE_DESCRIPTOR
        .get_message_by_name("{fq_request_type}")
        .unwrap();
    match encode_to_message(message_descriptor, message) {{
        Ok(dynamic_message) => {{
            let runtime = Builder::new_current_thread()
                .enable_time()
                .enable_io()
                .build()
                .unwrap();
            if let Ok(mut client) = runtime.block_on({client_name}::connect(
                ENDPOINT.read().expect("failed to get read lock").clone(),
            )) {{
                match runtime.block_on(client.{method}(Request::new(
                    dynamic_message.transcode_to::<{request_type}>().unwrap(),
                ))) {{
                    {response_handler}
                    Err(error) => {{
                        let mut buffer = ERROR_BUFFER.write().expect("failed to get write lock");
                        buffer.clear();
                        let null_terminated_error = format!("{{}}\0", error.message());
                        buffer.push_str(null_terminated_error.as_str());
                        new_error(buffer.as_str())
                    }}
                }}
            }} else {{
                new_error("failed to connect\0")
            }}
        }}
        Err(error) => new_error(error),
    }}
}}
"#
    };
}

/// Template of exported client methods called from q which uses `google.protobuf.Empty` as a request type.
/// # Parameters
/// - `method`: gRPC service request method.
/// - `client_name`: Client type in the form of [service]Clients.
/// - `response_handler`: Pre-built response handler with fully-qualified response type and response type.
macro_rules! empty_input_method_template {
    () => {
        r#"
#[no_mangle]
pub extern "C" fn {method}_(_message: K) -> K {{
    let runtime = Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap();
    if let Ok(mut client) = runtime.block_on({client_name}::connect(
        ENDPOINT.read().expect("failed to get read lock").clone(),
    )) {{
        match runtime.block_on(client.{method}(Request::new(()))) {{
            {response_handler}
            Err(error) => {{
                let mut buffer = ERROR_BUFFER.write().expect("failed to get write lock");
                buffer.clear();
                let null_terminated_error = format!("{}\0", error.message());
                buffer.push_str(null_terminated_error.as_str());
                new_error(buffer.as_str())
            }}
        }}
    }} else {{
        new_error("failed to connect\0")
    }}
}}
"#
    };
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Structs
//++++++++++++++++++++++++++++++++++++++++++++++++++//

//%% Tokenizer %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Token kinds to recognize in this crate.
#[derive(PartialEq)]
enum TokenKind {
    /// Initial token at initialization of semantic analyzer.
    Start,
    /// Marker token placed at the end of tokenized results.
    End,
    Package,
    Service,
    Rpc,
    Returns,
    Enum,
    Identifier,
    Number(u8),
    Equal,
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,
    Semicolon,
}

/// Token composed of a token kind and an optional value for identifier.
struct Token {
    /// Kind of a token.
    kind: TokenKind,
    /// name of an identifier.
    value: Option<String>,
}

/// Tokenizer of a subset of protobuf file.
struct Tokenizer<'a> {
    cursor: usize,
    input: Peekable<Chars<'a>>,
}

//%% SemanticAnalyzer %%//vvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Definition of RPC.
struct RpcDefinition {
    /// Method name.
    method: String,
    /// Request type.
    request: String,
    /// Response type.
    response: String,
}

/// Node of Abstract Syntax Tree.
enum Node {
    /// Package name.
    Package(String),
    /// Service definition.
    Service {
        /// Service name.
        name: String,
        /// Associated messages to this service.
        messages: Vec<String>,
        /// List of RPC definitions.
        rpcs: Vec<RpcDefinition>,
    },
    /// Enum message definition.
    Enum {
        /// Enum message name.
        name: String,
        /// Elements in the enum definition.
        elements: Vec<String>,
    },
}

/// Analyze a token stream and build Abstract Syntax Tree.
struct SemanticAnalyzer<'a> {
    /// Sequence of tokens to convert into AST.
    tokenizer: Tokenizer<'a>,
    /// Current token.
    token: Token,
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Implementation
//++++++++++++++++++++++++++++++++++++++++++++++++++//

//%% Tokenizer %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Start => write!(f, "[start]"),
            Self::End => write!(f, "[end]"),
            Self::Package => write!(f, "package"),
            Self::Identifier => write!(f, "[identifier]"),
            Self::Number(_) => write!(f, "[number]"),
            Self::Enum => write!(f, "enum"),
            Self::Rpc => write!(f, "rpc"),
            Self::Service => write!(f, "service"),
            Self::Returns => write!(f, "returns"),
            Self::Equal => write!(f, "="),
            Self::LeftParenthesis => write!(f, "("),
            Self::RightParenthesis => write!(f, ")"),
            Self::LeftBrace => write!(f, "{{"),
            Self::RightBrace => write!(f, "}}"),
            Self::Semicolon => write!(f, ";"),
        }
    }
}

impl Token {
    /// General constructor.
    fn new(kind: TokenKind, value: Option<String>) -> Self {
        Self { kind, value }
    }
}

impl<'a> Tokenizer<'a> {
    /// Build line tokenizer.
    fn new(line: &'a str) -> Self {
        Self {
            cursor: 0,
            input: line.trim().chars().peekable(),
        }
    }

    /// Proceed the cursor by 1.
    fn advance(&mut self) -> Option<char> {
        self.cursor += 1;
        self.input.next()
    }

    /// Skip whitespaces.
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.input.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Tokenize identifier and reserved keywords.
    fn tokenize_identifier(&mut self) -> io::Result<Token> {
        let mut identifier = String::new();
        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        Ok(identifier_to_token(identifier))
    }

    /// Tokenize an integer number.
    fn tokenize_number(&mut self) -> io::Result<Token> {
        let mut number = 0;
        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_digit() {
                number += 10 * number + (ch as u8 - 48);
                self.advance();
            } else {
                break;
            }
        }
        Ok(Token::new(TokenKind::Number(number), None))
    }

    /// Tokenize a line.
    fn next_token(&mut self) -> io::Result<Token> {
        while let Some(ch) = self.input.peek() {
            if ch.is_whitespace() {
                self.skip_whitespace();
            } else if ch.is_ascii_alphabetic() {
                return self.tokenize_identifier();
            } else if ch.is_ascii_digit() {
                return self.tokenize_number();
            } else {
                match ch {
                    '(' => {
                        self.advance();
                        return Ok(Token::new(TokenKind::LeftParenthesis, None));
                    }
                    ')' => {
                        self.advance();
                        return Ok(Token::new(TokenKind::RightParenthesis, None));
                    }
                    '{' => {
                        self.advance();
                        return Ok(Token::new(TokenKind::LeftBrace, None));
                    }
                    '}' => {
                        self.advance();
                        return Ok(Token::new(TokenKind::RightBrace, None));
                    }
                    ';' => {
                        self.advance();
                        return Ok(Token::new(TokenKind::Semicolon, None));
                    }
                    '=' => {
                        self.advance();
                        return Ok(Token::new(TokenKind::Equal, None));
                    }
                    _ => {
                        return Err(Error::new(
                            ErrorKind::Unsupported,
                            format!("unrecognized character: {}", ch),
                        ))
                    }
                }
            }
        }
        Ok(Token::new(TokenKind::End, None))
    }
}

//%% SemanticAnalyzer %%//vvvvvvvvvvvvvvvvvvvvvvvvvvv/

impl<'a> SemanticAnalyzer<'a> {
    /// Initialize semantic analyzer and set the firsst token to the internal current token.
    fn new(input: &'a str) -> io::Result<Self> {
        let mut instance = Self {
            tokenizer: Tokenizer::new(input),
            token: Token::new(TokenKind::Start, None),
        };
        instance.token = instance.tokenizer.next_token()?;
        Ok(instance)
    }

    /// Consume the current token if its kind matches the passed one.
    fn consume_token(&mut self, kind: TokenKind) -> io::Result<Option<String>> {
        if self.token.kind == kind {
            if kind == TokenKind::Identifier {
                // Get underlying value for identifier
                let identifier = self.token.value.clone();
                self.token = self.tokenizer.next_token()?;
                Ok(identifier)
            } else {
                self.token = self.tokenizer.next_token()?;
                Ok(None)
            }
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                format!("expected {}, got {}", kind, self.token.kind),
            ))
        }
    }

    /// Parse RPC definition.
    /// ```text
    /// rpc method(request) returns (response)
    /// ```
    /// # Parameters
    /// - `messages`: Set of message names associated with a service.
    fn parse_rpc(&mut self, messages: &mut HashSet<String>) -> io::Result<RpcDefinition> {
        self.consume_token(TokenKind::Rpc)?;
        // Get method name
        let method = self
            .consume_token(TokenKind::Identifier)?
            .expect("method name does not exist");
        self.consume_token(TokenKind::LeftParenthesis)?;
        // Get request type
        let request = self
            .consume_token(TokenKind::Identifier)?
            .expect("request type does not exist");
        if request.as_str() != "google.protobuf.Empty" {
            messages.insert(request.clone());
        }
        self.consume_token(TokenKind::RightParenthesis)?;
        self.consume_token(TokenKind::Returns)?;
        self.consume_token(TokenKind::LeftParenthesis)?;
        // Get request type
        let response = self
            .consume_token(TokenKind::Identifier)?
            .expect("response type does not exist");
        if response.as_str() != "google.protobuf.Empty" {
            messages.insert(response.clone());
        }
        self.consume_token(TokenKind::RightParenthesis)?;
        self.consume_token(TokenKind::Semicolon)?;
        Ok(RpcDefinition {
            method,
            request,
            response,
        })
    }

    /// Parse enum field definition.
    /// ```text
    /// name = field_number;
    /// ```
    /// # Parameters
    /// - `field_index`: Field index in an enum message.
    fn parse_enum(&mut self, field_index: u8) -> io::Result<String> {
        let field = self
            .consume_token(TokenKind::Identifier)?
            .expect("field name does not exist");
        self.consume_token(TokenKind::Equal)?;
        self.consume_token(TokenKind::Number(field_index))?;
        self.consume_token(TokenKind::Semicolon)?;
        Ok(field)
    }

    /// Parse package statement or service definition.
    /// ```text
    /// package name;
    /// service name{
    ///   rpc method(request) returns (response);
    ///   rpc method(request) returns (response);
    /// }
    /// ```
    fn parse(&mut self) -> io::Result<Node> {
        match self.token.kind {
            TokenKind::Package => {
                // Package node
                self.consume_token(TokenKind::Package)?;
                let node = Node::Package(self.token.value.clone().unwrap());
                self.consume_token(TokenKind::Identifier)?;
                self.consume_token(TokenKind::Semicolon)?;
                Ok(node)
            }
            TokenKind::Service => {
                // Service node
                self.consume_token(TokenKind::Service)?;
                // Get service name
                let service = self
                    .consume_token(TokenKind::Identifier)?
                    .expect("service name does not exist");
                self.consume_token(TokenKind::LeftBrace)?;
                // Pare RPC definitions
                let mut rpc_definitions = Vec::new();
                let mut messages = HashSet::new();
                while self.token.kind == TokenKind::Rpc {
                    rpc_definitions.push(self.parse_rpc(&mut messages)?);
                }
                let mut messages_unique = messages.into_iter().collect::<Vec<String>>();
                messages_unique.sort();
                self.consume_token(TokenKind::RightBrace)?;
                Ok(Node::Service {
                    name: service,
                    messages: messages_unique,
                    rpcs: rpc_definitions,
                })
            }
            TokenKind::Enum => {
                // Enum node
                self.consume_token(TokenKind::Enum)?;
                // Get enum message name
                let enum_name = self
                    .consume_token(TokenKind::Identifier)?
                    .expect("enum name does not exist");
                self.consume_token(TokenKind::LeftBrace)?;
                let mut index = 0;
                let mut elements = Vec::new();
                while self.token.kind == TokenKind::Identifier {
                    elements.push(self.parse_enum(index)?);
                    index += 1;
                }
                self.consume_token(TokenKind::RightBrace)?;
                Ok(Node::Enum {
                    name: enum_name,
                    elements,
                })
            }
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "only package or service definitions are to be parsed",
            )),
        }
    }
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Private Functions
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Convert identifier into corresponding token.
fn identifier_to_token(identifier: String) -> Token {
    match identifier.as_str() {
        "package" => Token::new(TokenKind::Package, None),
        "service" => Token::new(TokenKind::Service, None),
        "rpc" => Token::new(TokenKind::Rpc, None),
        "returns" => Token::new(TokenKind::Returns, None),
        "enum" => Token::new(TokenKind::Enum, None),
        _ => Token::new(TokenKind::Identifier, Some(identifier)),
    }
}

/// Consume AST and convert it to code.
fn ast_to_code(
    rust_file_writer: &mut BufWriter<File>,
    q_file_writer: &mut BufWriter<File>,
    ast: Node,
    package: &mut String,
) -> io::Result<()> {
    match ast {
        Node::Package(pkg) => {
            // Store package name
            *package = pkg;
            Ok(())
        }
        Node::Service {
            name,
            messages,
            rpcs,
        } => {
            if package.is_empty() {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    "package name is not set",
                ))
            } else {
                let snake_case_service = camel_to_snake(name.as_str());
                // Write import lines.
                let import = format!(
                    import_template!(),
                    package = package.as_str(),
                    snake_case_service = snake_case_service.as_str(),
                    service = name.as_str(),
                    messages = messages.join(",")
                );
                rust_file_writer.write_all(import.as_bytes())?;

                // Write method code.
                for rpc in rpcs {
                    let method = match (rpc.request.as_str(), rpc.response.as_str()) {
                        ("google.protobuf.Empty", "google.protobuf.Empty") => {
                            format!(
                                empty_input_method_template!(),
                                method = rpc.method.to_lowercase(),
                                client_name = format!("{}Client", name),
                                response_handler = EMPTY_RESPONSE_HANDLER
                            )
                        }
                        ("google.protobuf.Empty", _) => {
                            let response_handler = format!(
                                non_empty_response_handler!(),
                                fq_response_type =
                                    [package.as_str(), rpc.response.as_str()].join("."),
                                response_type = rpc.response
                            );
                            format!(
                                empty_input_method_template!(),
                                method = rpc.method.to_lowercase(),
                                client_name = format!("{}Client", name),
                                response_handler = response_handler
                            )
                        }
                        (_, "google.protobuf.Empty") => {
                            format!(
                                method_template!(),
                                method = rpc.method.to_lowercase(),
                                client_name = format!("{}Client", name),
                                fq_request_type =
                                    [package.as_str(), rpc.request.as_str()].join("."),
                                request_type = rpc.request,
                                response_handler = EMPTY_RESPONSE_HANDLER
                            )
                        }
                        _ => {
                            let response_handler = format!(
                                non_empty_response_handler!(),
                                fq_response_type =
                                    [package.as_str(), rpc.response.as_str()].join("."),
                                response_type = rpc.response
                            );
                            format!(
                                method_template!(),
                                method = rpc.method.to_lowercase(),
                                client_name = format!("{}Client", name),
                                fq_request_type =
                                    [package.as_str(), rpc.request.as_str()].join("."),
                                request_type = rpc.request,
                                response_handler = response_handler
                            )
                        }
                    };
                    rust_file_writer.write_all(method.as_bytes())?;

                    // Write a line to load Rust function.
                    let method_load_line =
                        format!(method_load_template!(), method = rpc.method.to_lowercase());
                    q_file_writer.write_all(method_load_line.as_bytes())?;
                }
                Ok(())
            }
        }
        Node::Enum { name, elements } => {
            // Write enum definition
            let elements = elements.join("`");
            let enum_definition = format!(enum_template!(), source = name, elements = elements);
            q_file_writer.write_all(enum_definition.as_bytes())
        }
    }
}

/// Convert camel case name to snake case name.
fn camel_to_snake(camel: &str) -> String {
    let mut chars = camel.chars().peekable();
    let mut snake = String::new();
    // It is assured that input is not empty.
    snake.push((chars.next().unwrap() as u8 + 32) as char);
    while let Some(&ch) = chars.peek() {
        chars.next();
        if ch.is_uppercase() {
            match chars.peek() {
                Some(ch_next) if ch_next.is_uppercase() => {
                    // Capital in a sequence of capitals.
                    snake.push((ch as u8 + 32) as char);
                }
                Some(ch_next) if ch_next.is_lowercase() => {
                    // Start of new word
                    // Insert '_'
                    snake.push('_');
                    snake.push((ch as u8 + 32) as char);
                }
                _ => (),
            }
        } else {
            snake.push(ch);
        }
    }
    snake
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Interface
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Generate Rust code of gRPC client methods called from q.
pub fn generate_code(files: &[impl AsRef<Path>], includes: &[impl AsRef<Path>]) -> io::Result<()> {
    // Open q target file.
    let q_output = OpenOptions::new()
        .read(false)
        .write(true)
        .truncate(true)
        .create(true)
        .open(TARGET_Q_FILE_NAME)?;
    let mut q_file_writer = BufWriter::new(q_output);

    // Write a header of q file.
    q_file_writer.write_all(Q_FILE_HEADER.as_bytes())?;

    // Open Rust target file.
    let rust_output = OpenOptions::new()
        .read(false)
        .write(true)
        .truncate(true)
        .create(true)
        .open(TARGET_RUST_FILE_NAME)?;
    let mut rust_file_writer = BufWriter::new(rust_output);

    // Write headers.
    rust_file_writer.write_all(RUST_FILE_HEADER.as_bytes())?;

    // Write definition of error buffer.
    rust_file_writer.write_all(ERROR_BUFFER.as_bytes())?;

    // Read inputs and check service related information.
    files
        .iter()
        .map(|file| {
            // Package name. Only one package name exists per file.
            let mut package = String::new();
            // String to parse service definition.
            let mut service_definition = String::new();
            // String to parse enum definition.
            let mut enum_definition = String::new();
            // Track service definition scope.
            let mut in_service_definition = false;
            // Track enum message definition scope.
            let mut in_enum_definition = false;
            // Try to open file and parse.
            let found = includes
                .iter()
                .map(|include_path| {
                    if let Ok(input) = OpenOptions::new()
                        .read(true)
                        .write(false)
                        .create(false)
                        .open(include_path.as_ref().join(file))
                    {
                        let mut line = String::new();
                        let mut reader = BufReader::new(input);
                        // Read each line and find package name, service name and associated messages.
                        while let Ok(num_bytes) = reader.read_line(&mut line) {
                            if num_bytes == 0 {
                                // Reached EOF
                                break;
                            }
                            if line.trim().starts_with("package") {
                                let mut analyzer = SemanticAnalyzer::new(line.as_str())?;
                                let ast = analyzer.parse()?;
                                ast_to_code(
                                    &mut rust_file_writer,
                                    &mut q_file_writer,
                                    ast,
                                    &mut package,
                                )?;
                                line.clear();
                            } else if line.trim().starts_with("service") {
                                in_service_definition = true;
                                service_definition += line.as_str();
                                line.clear();
                            } else if line.trim().starts_with("rpc") {
                                service_definition += line.as_str();
                                line.clear();
                            } else if line.trim().starts_with("enum") {
                                in_enum_definition = true;
                                enum_definition += line.as_str();
                                line.clear();
                            } else if line.trim().starts_with("}") {
                                if in_service_definition {
                                    // Closing brace of a service definition
                                    service_definition += line.as_str();
                                    // Analyze and generate code.
                                    let mut analyzer =
                                        SemanticAnalyzer::new(service_definition.as_str())?;
                                    let ast = analyzer.parse()?;
                                    ast_to_code(
                                        &mut rust_file_writer,
                                        &mut q_file_writer,
                                        ast,
                                        &mut package,
                                    )?;
                                    // Escape from service
                                    service_definition.clear();
                                    in_service_definition = false;
                                } else if in_enum_definition {
                                    // Closing brace of a enum definition
                                    enum_definition += line.as_str();
                                    // Analyze and generate code.
                                    let mut analyzer =
                                        SemanticAnalyzer::new(enum_definition.as_str())?;
                                    let ast = analyzer.parse()?;
                                    ast_to_code(
                                        &mut rust_file_writer,
                                        &mut q_file_writer,
                                        ast,
                                        &mut package,
                                    )?;
                                    // Escape from enum
                                    enum_definition.clear();
                                    in_enum_definition = false;
                                } else {
                                    // Nothing to do
                                }
                                line.clear();
                            } else {
                                if in_enum_definition
                                    && line.contains("=")
                                    && line.trim().ends_with(";")
                                {
                                    // Enum field definition `field = number;`
                                    enum_definition += line.as_str();
                                }
                                line.clear();
                            }
                        }
                        // File was found
                        Ok(true)
                    } else {
                        // File not found
                        Ok(false)
                    }
                })
                .collect::<io::Result<Vec<bool>>>()?;

            if found.iter().any(|result| *result) {
                Ok(())
            } else {
                Err(Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "{} was not found in any include directories",
                        file.as_ref().to_str().unwrap()
                    ),
                ))
            }
        })
        .collect::<io::Result<()>>()?;

    Ok(())
}
