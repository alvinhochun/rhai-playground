use crate::codemirror;
use js_sys::RegExp;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub struct RhaiMode {
    indent_unit: u32,
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct State {
    token_state: rhai::TokenizeState,
    unclosed_bracket_count: i32,
    line_indent: u32,
    is_defining_identifier: bool,
}

thread_local! {
    static ELECTRIC_INPUT: RegExp = RegExp::new("^\\s*[}\\])]$", "").into();
    static LINE_COMMENT: JsValue = JsValue::from_str("//");
}

#[wasm_bindgen]
impl RhaiMode {
    #[wasm_bindgen(constructor)]
    pub fn new(indent_unit: u32) -> Self {
        Self { indent_unit }
    }

    #[wasm_bindgen(js_name = startState)]
    pub fn start_state(&self) -> State {
        State {
            token_state: rhai::TokenizeState {
                include_comments: true,
                ..Default::default()
            },
            unclosed_bracket_count: 0,
            line_indent: 0,
            is_defining_identifier: false,
        }
    }

    #[wasm_bindgen(js_name = copyState)]
    pub fn copy_state(&self, state: &State) -> State {
        state.clone()
    }

    pub fn token(
        &self,
        stream: codemirror::StringStream,
        state: &mut State,
    ) -> Result<Option<String>, JsValue> {
        token(stream, state)
    }

    // #[wasm_bindgen(js_name = blankLine)]
    // pub fn blank_line(&self, state: &mut State) -> Result<(), JsValue> {
    //     Ok(())
    // }

    pub fn indent(&self, state: &mut State, text_after: String) -> JsValue {
        indent(self, state, text_after)
            .map(JsValue::from)
            .unwrap_or_else(|| codemirror::CODEMIRROR_PASS.clone())
    }

    #[wasm_bindgen(getter, js_name = electricInput)]
    pub fn electric_input(&self) -> RegExp {
        ELECTRIC_INPUT.with(|v| v.clone())
    }

    #[wasm_bindgen(getter, js_name = lineComment)]
    pub fn line_comment(&self) -> JsValue {
        LINE_COMMENT.with(|v| v.clone())
    }
}

struct StreamAdapter {
    stream: codemirror::StringStream,
}

impl rhai::InputStream for StreamAdapter {
    fn get_next(&mut self) -> Option<char> {
        self.stream.next()
    }

    fn peek_next(&mut self) -> Option<char> {
        self.stream.peek()
    }
}

fn token(stream: codemirror::StringStream, state: &mut State) -> Result<Option<String>, JsValue> {
    if stream.sol() {
        state.line_indent = stream.indentation();
        state.unclosed_bracket_count = 0;
    }

    let (next_token, _) = rhai::get_next_token(
        &mut StreamAdapter { stream },
        &mut state.token_state,
        &mut rhai::Position::default(),
    )
    .ok_or_else(|| "Failed to get next token")?;
    match &next_token {
        rhai::Token::LeftBrace
        | rhai::Token::LeftBracket
        | rhai::Token::LeftParen
        | rhai::Token::MapStart => {
            if state.unclosed_bracket_count < 0 {
                state.unclosed_bracket_count = 0;
            }
            state.unclosed_bracket_count += 1;
        }
        rhai::Token::RightBrace | rhai::Token::RightBracket | rhai::Token::RightParen => {
            state.unclosed_bracket_count -= 1;
        }
        _ => {}
    };
    let res = match &next_token {
        rhai::Token::IntegerConstant(_) => "number",
        rhai::Token::FloatConstant(_) => "number",
        rhai::Token::Identifier(_) => {
            if state.is_defining_identifier {
                "def"
            } else {
                "variable"
            }
        }
        rhai::Token::CharConstant(_) => "string-2",
        rhai::Token::StringConst(_) => "string",
        rhai::Token::LeftBrace => "bracket",
        rhai::Token::RightBrace => "bracket",
        rhai::Token::LeftParen => "bracket",
        rhai::Token::RightParen => "bracket",
        rhai::Token::LeftBracket => "bracket",
        rhai::Token::RightBracket => "bracket",
        rhai::Token::Plus => "operator",
        rhai::Token::UnaryPlus => "operator",
        rhai::Token::Minus => "operator",
        rhai::Token::UnaryMinus => "operator",
        rhai::Token::Multiply => "operator",
        rhai::Token::Divide => "operator",
        rhai::Token::Modulo => "operator",
        rhai::Token::PowerOf => "operator",
        rhai::Token::LeftShift => "operator",
        rhai::Token::RightShift => "operator",
        rhai::Token::SemiColon => "operator",
        rhai::Token::Colon => "operator",
        rhai::Token::DoubleColon => "operator",
        rhai::Token::Comma => "operator",
        rhai::Token::Period => "operator",
        rhai::Token::MapStart => "bracket",
        rhai::Token::Equals => "operator",
        rhai::Token::True => "builtin",
        rhai::Token::False => "builtin",
        rhai::Token::Let => "keyword",
        rhai::Token::Const => "keyword",
        rhai::Token::If => "keyword",
        rhai::Token::Else => "keyword",
        rhai::Token::While => "keyword",
        rhai::Token::Loop => "keyword",
        rhai::Token::For => "keyword",
        rhai::Token::In => "keyword",
        rhai::Token::LessThan => "operator",
        rhai::Token::GreaterThan => "operator",
        rhai::Token::LessThanEqualsTo => "operator",
        rhai::Token::GreaterThanEqualsTo => "operator",
        rhai::Token::EqualsTo => "operator",
        rhai::Token::NotEqualsTo => "operator",
        rhai::Token::Bang => "operator",
        rhai::Token::Pipe => "operator",
        rhai::Token::Or => "operator",
        rhai::Token::XOr => "operator",
        rhai::Token::Ampersand => "operator",
        rhai::Token::And => "operator",
        rhai::Token::Fn => "keyword",
        rhai::Token::Continue => "keyword",
        rhai::Token::Break => "keyword",
        rhai::Token::Return => "keyword",
        rhai::Token::Throw => "keyword",
        rhai::Token::PlusAssign => "operator",
        rhai::Token::MinusAssign => "operator",
        rhai::Token::MultiplyAssign => "operator",
        rhai::Token::DivideAssign => "operator",
        rhai::Token::LeftShiftAssign => "operator",
        rhai::Token::RightShiftAssign => "operator",
        rhai::Token::AndAssign => "operator",
        rhai::Token::OrAssign => "operator",
        rhai::Token::XOrAssign => "operator",
        rhai::Token::ModuloAssign => "operator",
        rhai::Token::PowerOfAssign => "operator",
        rhai::Token::Private => "keyword",
        rhai::Token::Import => "keyword",
        rhai::Token::Export => "keyword",
        rhai::Token::As => "keyword",
        rhai::Token::Comment(_) => "comment",
        rhai::Token::LexError(e) => {
            console::log_1(&JsValue::from_str(&format!("LexError: {}", e)));
            "error"
        }
        rhai::Token::EOF => return Ok(None),
    };
    match &next_token {
        rhai::Token::Fn
        | rhai::Token::Let
        | rhai::Token::Const
        | rhai::Token::As
        | rhai::Token::For => {
            state.is_defining_identifier = true;
        }
        rhai::Token::Comment(_) => {}
        _ => {
            state.is_defining_identifier = false;
        }
    };
    Ok(Some(res.to_owned()))
}

fn indent(mode: &RhaiMode, state: &State, text_after: String) -> Option<u32> {
    let should_dedent = || {
        text_after
            .trim_start()
            .starts_with(['}', ']', ')'].as_ref())
    };
    if state.unclosed_bracket_count > 0 {
        if should_dedent() {
            Some(state.line_indent)
        } else {
            Some(state.line_indent + mode.indent_unit)
        }
    } else {
        if should_dedent() {
            Some(state.line_indent.saturating_sub(mode.indent_unit))
        } else {
            None
        }
    }
}