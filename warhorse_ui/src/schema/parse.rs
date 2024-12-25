use logos::{Lexer, Logos};
use std::{collections::HashMap, error::Error, fmt};
use std::iter::Peekable;
use quote::__private::TokenStream;
use quote::{format_ident, quote};
use rust_format::{Formatter, RustFmt};

pub fn generate_rust_code(inputs: &[&str]) -> Result<String, Box<dyn Error>> {
    if inputs.is_empty() {
        return Err("No input found".into());
    }

    // Parse all inputs and collect their ASTs
    let mut all_schemas: Vec<SchemaDefinition> = Vec::new();

    for input in inputs {
        let mut parser = Parser::new(input);
        let schemas = parser.parse()?;
        all_schemas.extend(schemas);
    }

    // Check for duplicate schema names
    let mut seen_names = HashMap::new();
    for schema in &all_schemas {
        if let Some(first_occurrence) = seen_names.get(&schema.name) {
            return Err(format!("Duplicate schema name '{}' found. First defined at line {:?}",
                               schema.name, first_occurrence).into());
        }
        seen_names.insert(&schema.name, schema);
    }

    // Generate the Rust code
    let enum_variants: Vec<_> = all_schemas.iter().map(|schema| {
        let name = format_ident!("{}", schema.name);
        let mut fields: Vec<_> = schema.fields.iter().collect();
        // Sort fields by name for consistent ordering
        fields.sort_by(|a, b| a.0.cmp(b.0));

        let fields = fields.into_iter().map(|(field_name, field_type)| {
            let field_ident = format_ident!("{}", field_name);
            let rust_type = value_kind_to_rust_type(field_type);
            quote! { #field_ident: #rust_type }
        });

        quote! {
            #name {
                #(#fields,)*
            }
        }
    }).collect();

    let enum_name = format_ident!("Widget");    
    let feed_data_match_arms: Vec<_> = all_schemas.iter().map(|schema| {
        let name = format_ident!("{}", schema.name);
        let mut field_refs = Vec::new();
        let mut field_updates = Vec::new();

        for (field_name, field_type) in &schema.fields {
            let field_ident = format_ident!("{}", field_name);
            field_refs.push(quote! { ref mut #field_ident });
            
            match field_type {
                ValueKind::String => {
                    field_updates.push(quote! {
                        if let Some(new_val) = data.get(#field_name).and_then(|v| v.as_str()) {
                            *#field_ident = new_val.to_string();
                        }
                    });
                }
                ValueKind::Number => {
                    field_updates.push(quote! {
                        if let Some(new_val) = data.get(#field_name).and_then(|v| v.as_f64()) {
                            *#field_ident = new_val;
                        }
                    });
                }
                ValueKind::Bool => {
                    field_updates.push(quote! {
                        if let Some(new_val) = data.get(#field_name).and_then(|v| v.as_bool()) {
                            *#field_ident = new_val;
                        }
                    });
                }
                ValueKind::Schema(_) => {
                    field_updates.push(quote! {
                        if let Some(new_val) = data.get(#field_name) {
                            #field_ident.feed_data(new_val);
                        }
                    });
                }
                ValueKind::Array(_) => {
                    field_updates.push(quote! {
                        if let Some(arr) = data.get(#field_name).and_then(|v| v.as_array()) {
                            #field_ident.clear();
                            for item in arr {
                                let mut child = Self::default();
                                child.feed_data(item);
                                #field_ident.push(Box::new(child));
                            }
                        }
                    });
                }
            }
        }

        quote! {
            Self::#name { #(#field_refs,)* } => {
                #(#field_updates)*
            }
        }
    }).collect();

    let tokens = quote! {
        pub mod generated {
            #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
            pub enum #enum_name {
                #(#enum_variants),*,
                /// A container widget that holds other widgets.
                Container {
                    id: String,
                    inner: Vec<Box<Widget>>,
                },
                /// A widget that repeats the block for each item in a list.
                ForEach(String, Box<Widget>),
            }

            impl #enum_name {
                pub fn feed_data(&mut self, data: &warhorse_ui::serde_json::Value) {
                    match self {
                        #(#feed_data_match_arms),*,
                        Self::Container { ref mut id, ref mut inner } => {
                            if let Some(new_id) = data.get("id").and_then(|v| v.as_str()) {
                                *id = new_id.to_string();
                            }
                            if let Some(arr) = data.get("inner").and_then(|v| v.as_array()) {
                                inner.clear();
                                for item in arr {
                                    let mut child = Self::default();
                                    child.feed_data(item);
                                    inner.push(Box::new(child));
                                }
                            }
                        },
                        Self::ForEach(ref key, block) => {
                            if let Some(data) = data.get(key.as_str()) {
                                if let Some(items) = data.as_array() {
                                    for item in items {
                                        let mut block = block.clone();
                                        block.feed_data(item);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            impl Default for #enum_name {
                fn default() -> Self {
                    Self::Container {
                        id: String::default(),
                        inner: Vec::default(),
                    }
                }
            }
        }
    };

    Ok(RustFmt::default().format_str(&tokens.to_string())?)
}

fn value_kind_to_rust_type(value_kind: &ValueKind) -> TokenStream {
    match value_kind {
        ValueKind::String => quote!(String),
        ValueKind::Number => quote!(f64),
        ValueKind::Bool => quote!(bool),
        ValueKind::Schema(_name) => {
            quote!(Box<Widget>)
        },
        ValueKind::Array(inner_type) => {
            let inner = value_kind_to_rust_type(inner_type);
            quote!(Vec<#inner>)
        }
    }
}

#[derive(Debug)]
struct SchemaDefinition {
    name: String,
    fields: HashMap<String, ValueKind>,
}

#[derive(Debug, Clone, PartialEq)]
enum ValueKind {
    String,
    Number,
    Bool,
    Schema(String),
    Array(Box<ValueKind>),
}

#[derive(Debug)]
struct ParseError {
    line: usize,
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error at line {}: {}", self.line, self.message)
    }
}

impl Error for ParseError {}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\f\r]+")]  // Skip whitespace including carriage return
#[logos(skip r"//[^\n]*")]    // Skip line comments
enum Token {
    #[token("\n")]
    NewLine,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[token("Bool")]
    Bool,

    #[token("String")]
    String,

    #[token("Number")]
    Number,

    #[regex("[A-Za-z][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
}

struct Parser<'a> {
    lexer: Peekable<Lexer<'a, Token>>,
    line_number: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser {
            lexer: Token::lexer(input).peekable(),
            line_number: 1,
        }
    }

    fn parse(&mut self) -> Result<Vec<SchemaDefinition>, Box<dyn Error>> {
        let mut elements = Vec::new();

        while self.peek_token().is_ok() {
            match self.peek_token()? {
                Token::NewLine => {
                    self.next_token()?;
                    self.line_number += 1;
                }
                _ => {
                    elements.push(self.parse_struct()?);
                }
            }
        }

        if elements.is_empty() {
            return Err(self.make_error("No elements found"));
        }

        Ok(elements)
    }

    fn parse_struct(&mut self) -> Result<SchemaDefinition, Box<dyn Error>> {
        let mut fields = HashMap::new();
        let name = self.parse_identifier()?;
        self.expect(Token::LBrace)?;

        loop {
            match self.peek_token()? {
                Token::RBrace => {
                    self.next_token()?;
                    break;
                }
                Token::NewLine => {
                    self.next_token()?;
                    self.line_number += 1;
                    continue;
                }
                _ => {
                    let field_name = self.parse_identifier()?;
                    self.expect(Token::Colon)?;
                    let field_type = self.parse_type()?;

                    // Check next token after the type
                    match self.peek_token()? {
                        Token::RBrace => {
                            fields.insert(field_name.clone(), field_type.clone());
                            continue;  // Let the outer loop handle the RBrace
                        }
                        Token::Comma => {
                            self.next_token()?;  // Consume the comma
                            fields.insert(field_name.clone(), field_type.clone());
                        }
                        Token::NewLine => {
                            fields.insert(field_name.clone(), field_type.clone());
                            self.next_token()?;  // Consume the newline
                            self.line_number += 1;

                            // Check if next token after newline is RBrace
                            if let Token::RBrace = self.peek_token()? {
                                continue;  // Let the outer loop handle the RBrace
                            }
                        }
                        _ => return Err(self.make_error("Expected comma, newline, or closing brace after field")),
                    }
                }
            }
        }

        if fields.is_empty() {
            return Err(self.make_error("Empty struct definition"));
        }

        Ok(SchemaDefinition { name, fields })
    }

    fn parse_identifier(&mut self) -> Result<String, Box<dyn Error>> {
        match self.next_token()? {
            Token::Identifier(name) => Ok(name),
            _ => Err(self.make_error("Expected identifier")),
        }
    }

    fn parse_type(&mut self) -> Result<ValueKind, Box<dyn Error>> {
        let base_type = match self.next_token()? {
            Token::String => ValueKind::String,
            Token::Number => ValueKind::Number,
            Token::Bool => ValueKind::Bool,
            Token::Identifier(name) => ValueKind::Schema(name),
            _ => return Err(self.make_error("Expected type")),
        };

        // Check for array notation
        match self.peek_token()? {
            Token::LBracket => {
                self.next_token()?; // Consume '['
                self.expect(Token::RBracket)?; // Expect ']'
                Ok(ValueKind::Array(Box::new(base_type)))
            }
            _ => Ok(base_type),
        }
    }

    fn next_token(&mut self) -> Result<Token, Box<dyn Error>> {
        match self.lexer.next() {
            Some(Ok(token)) => Ok(token),
            Some(Err(_)) => Err(self.make_error("Lexer error")),
            None => Err(self.make_error("Unexpected end of input")),
        }
    }

    fn peek_token(&mut self) -> Result<Token, Box<dyn Error>> {
        match self.lexer.peek() {
            Some(Ok(token)) => Ok(token.clone()),
            Some(Err(_)) => Err(self.make_error("Lexer error")),
            None => Err(self.make_error("Unexpected end of input")),
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), Box<dyn Error>> {
        let token = self.next_token()?;
        if token == expected {
            Ok(())
        } else {
            Err(self.make_error(&format!(
                "Expected {:?}, found {:?}",
                expected, token
            )))
        }
    }

    fn make_error(&self, message: &str) -> Box<dyn Error> {
        Box::new(ParseError {
            line: self.line_number,
            message: message.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use quote::quote;

    #[test]
    fn test_generate_rust_code() -> Result<(), Box<dyn Error>> {
        let inputs = &[
            r#"
                User {
                    id: Number,
                    name: String,
                    active: Bool,
                }
            "#,
                r#"
                Message {
                    content: String,
                    id: Number,
                    recipients: User[],
                    sender: User,
                }
            "#,
        ];

        let generated = generate_rust_code(inputs)?;
        let expected = RustFmt::default().format_str(&quote! {
            pub mod generated {
                #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
                pub enum Widget {
                    User {
                        active: bool,
                        id: f64,
                        name: String,
                    },
                    Message {
                        content: String,
                        id: f64,
                        recipients: Vec<Box<Widget>>,
                        sender: Box<Widget>,
                    },
                    /// A widget that repeats its children for each item in a list.
                    ForEach(String, Box<Widget>),
                }

                impl Widget {
                    pub fn feed_data(&mut self, data: &warhorse_ui::serde_json::Value) {
                        match self {
                            Self::User { ref mut id, ref mut text, .. } => {
                                if let Some(new_id) = data.get("id").and_then(|v| v.as_str()) {
                                    *id = new_id.to_string();
                                }
                                if let Some(new_text) = data.get("text").and_then(|v| v.as_str()) {
                                    *text = new_text.to_string();
                                }
                            }
                            Self::Message { ref mut id, ref mut text, .. } => {
                                if let Some(new_id) = data.get("id").and_then(|v| v.as_str()) {
                                    *id = new_id.to_string();
                                }
                                if let Some(new_text) = data.get("text").and_then(|v| v.as_str()) {
                                    *text = new_text.to_string();
                                }
                            }
                            Self::ForEach(key, block) => {
                                if let Some(data) = data.get(key) {
                                    if let Some(items) = data.as_array() {
                                        for item in items {
                                            let mut block = block.clone();
                                            block.feed_data(item);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }.to_string())?;

        assert_eq!(generated, expected);

        // Test error cases
        let duplicate_inputs = &[
            r#"User { id: Number }"#,
            r#"User { name: String }"#,
        ];

        assert!(generate_rust_code(duplicate_inputs).is_err(),
                "Should error on duplicate schema names");

        // Empty input
        let empty_inputs: &[&str] = &[];
        assert!(generate_rust_code(empty_inputs).is_err(),
                "Should error on empty input");

        Ok(())
    }

    #[test]
    fn test_parse_struct() -> Result<(), Box<dyn Error>> {
        let schema = r#"
            Button { id : Number, text: String }
            Container {
                id: Number,
                children: Button[],
            }
        "#;

        let mut parser = Parser::new(schema);
        let elements = parser.parse()?;
        assert_eq!(elements.len(), 2);

        let button = &elements[0];
        assert_eq!(button.name, "Button");
        assert_eq!(button.fields.len(), 2);
        assert_eq!(button.fields.get("id"), Some(&ValueKind::Number));
        assert_eq!(button.fields.get("text"), Some(&ValueKind::String));

        let container = &elements[1];
        assert_eq!(container.name, "Container");
        assert_eq!(container.fields.len(), 2);
        assert_eq!(container.fields.get("id"), Some(&ValueKind::Number));
        assert_eq!(
            container.fields.get("children"),
            Some(&ValueKind::Array(Box::new(ValueKind::Schema("Button".to_string()))))
        );

        Ok(())
    }
}
