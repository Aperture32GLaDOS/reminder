use std::{fs::File, io::Read};
use proc_macro::{self, TokenStream, TokenTree};


fn parse_token_stream(input: TokenStream) -> Result<(String, String), TokenStream> {
    let all_tokens: Vec<TokenTree> = input.into_iter().collect();
    if all_tokens.len() != 3 {
        return Err("compile_error!(\"Embed file expects two arguments\")".parse().unwrap());
    }
    match all_tokens.first().unwrap() {
        TokenTree::Literal(x) => {
            // Assume it's a string
            let mut path = x.to_string();
            let chars: Vec<char> = path.chars().collect();
            if *chars.first().unwrap() != '"' || *chars.last().unwrap() != '"' {
                return Err("compile_error!(\"First argument should be a string literal\")".parse().unwrap());
            }
            path.remove(path.len() - 1);
            path.remove(0);
            let name_token = &all_tokens[2];
            let name;
            match name_token {
                TokenTree::Ident(y) => {
                    name = y.to_string();
                }
                _ => {
                    return Err("compile_error!(\"Second argument should be an identifier\")".parse().unwrap());
                }
            }
            return Ok((path, name));
        }
        _ => {
            return Err("compile_error!(\"First argument should be a string literal\")".parse().unwrap());
        }
    }
}

#[proc_macro]
pub fn embed_file(input: TokenStream) -> TokenStream {
    let parsed = parse_token_stream(input);
    let (file_path, var_name);
    match parsed {
        Ok(x) => {
            (file_path, var_name) = x;
        }
        Err(y) => {
            return y;
        }
    }
    let mut file = File::open(file_path).unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let mut array_contents: String = buffer.iter().fold(String::new(), |v, x| v + &(",".to_owned() + &x.to_string()));
    array_contents.remove(0);
    // Build up the embedded file
    let output: String = format!("const {}: [u8; {}] = [{}];", var_name, buffer.len(), array_contents);
    return output.parse().unwrap();
}

#[proc_macro]
pub fn embed_file_to_string(input: TokenStream) -> TokenStream {
    let parsed = parse_token_stream(input);
    let (file_path, var_name);
    match parsed {
        Ok(x) => {
            (file_path, var_name) = x;
        }
        Err(y) => {
            return y;
        }
    }
    let mut file = File::open(file_path).unwrap();
    let file_content: String;
    {
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        file_content = String::from_utf8_lossy(&buffer).to_string();
    }
    let file_content_escaped: String = file_content.escape_default().collect();
    // Build up the embedded file
    let output: String = format!("const {}: &str = \"{}\";", var_name, file_content_escaped);
    return output.parse().unwrap();
}
