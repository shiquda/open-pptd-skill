//! Lightweight LaTeX macro preprocessor.
//!
//! Slide formulas routinely define short aliases (`\newcommand{\R}{\mathbb{R}}`).
//! Those definitions are expanded before parsing, so a user-defined macro never
//! reaches the "unsupported LaTeX command" diagnostics and never leaks into the
//! deck as raw markup.
//!
//! Supported definitions:
//!
//! * `\newcommand{\name}[n]{body}` / `\newcommand\name[n]{body}`
//! * `\newcommand{\name}[n][default]{body}` — `\name[…]` may then be omitted
//! * `\renewcommand` (the macro must exist) and `\providecommand` (no-op when it does)
//! * `\def\name{body}` and `\def\name#1#2{body}`
//!
//! Scoping is per formula: each `<latex>` body (or `\(…\)` span) is compiled on
//! its own, so a macro defined in one of them is not visible in the next.

use std::collections::HashMap;

use super::MathError;

/// Maximum number of macro expansions performed while compiling one formula.
const MAX_EXPANSIONS: usize = 4096;

/// Maximum nesting depth of macro expansions (`\a` expanding to `\a` …).
const MAX_DEPTH: usize = 32;

#[derive(Debug, Clone)]
struct Macro {
    nargs: usize,
    default: Option<String>,
    body: String,
}

/// Expand every macro definition and use in `src`, and strip the definitions.
pub(super) fn expand_macros(src: &str) -> Result<String, MathError> {
    let (source, definitions) = collect_definitions(src)?;
    if definitions.is_empty() {
        return Ok(source);
    }
    let mut expander = Expander {
        definitions: &definitions,
        expansions: 0,
    };
    expander.expand(&source, 0)
}

struct Expander<'a> {
    definitions: &'a HashMap<String, Macro>,
    expansions: usize,
}

impl Expander<'_> {
    fn expand(&mut self, src: &str, depth: usize) -> Result<String, MathError> {
        if depth > MAX_DEPTH {
            return Err(MathError::MacroError(
                "macro expansion nested too deeply (recursive macro?)".to_string(),
            ));
        }
        let chars: Vec<char> = src.chars().collect();
        let mut out = String::with_capacity(src.len());
        let mut i = 0;
        while i < chars.len() {
            if chars[i] != '\\' {
                out.push(chars[i]);
                i += 1;
                continue;
            }
            let (name, after_name) = read_command(&chars, i);
            let Some(mac) = self.definitions.get(&name).cloned() else {
                out.extend(chars[i..after_name].iter());
                i = after_name;
                continue;
            };
            self.expansions += 1;
            if self.expansions > MAX_EXPANSIONS {
                return Err(MathError::MacroError(format!(
                    "macro expansion limit reached while expanding `{name}` (recursive macro?)"
                )));
            }
            let (args, after_args) = read_arguments(&chars, after_name, &mac, &name)?;
            let body = substitute(&mac, &args, &name)?;
            out.push_str(&self.expand(&body, depth + 1)?);
            i = after_args;
        }
        Ok(out)
    }
}

/// Read the control sequence starting at `start` (which points at `\`).
///
/// Returns the command including its backslash and the index just past it.
fn read_command(chars: &[char], start: usize) -> (String, usize) {
    let mut i = start + 1;
    if i >= chars.len() {
        return ("\\".to_string(), chars.len());
    }
    if chars[i].is_ascii_alphabetic() {
        while i < chars.len() && chars[i].is_ascii_alphabetic() {
            i += 1;
        }
    } else {
        i += 1;
    }
    (chars[start..i].iter().collect(), i)
}

/// Skip whitespace, returning the index of the next significant character.
fn skip_whitespace(chars: &[char], start: usize) -> usize {
    let mut i = start;
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }
    i
}

/// Read a `{…}` group, returning its content and the index just past `}`.
fn read_group(chars: &[char], start: usize) -> Result<(String, usize), MathError> {
    let mut depth = 0usize;
    let mut i = start;
    let mut content = String::new();
    while i < chars.len() {
        match chars[i] {
            '{' => {
                depth += 1;
                if depth > 1 {
                    content.push('{');
                }
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((content, i + 1));
                }
                content.push('}');
            }
            c => content.push(c),
        }
        i += 1;
    }
    Err(MathError::SyntaxError(
        "unclosed '{' in macro definition or argument".to_string(),
    ))
}

/// Read a `[…]` group, returning its content and the index just past `]`.
///
/// `start` points at the opening `[`.
fn read_bracket(chars: &[char], start: usize) -> Result<(String, usize), MathError> {
    let mut i = start + 1;
    let mut content = String::new();
    while i < chars.len() {
        if chars[i] == ']' {
            return Ok((content, i + 1));
        }
        content.push(chars[i]);
        i += 1;
    }
    Err(MathError::SyntaxError(
        "unclosed '[' in macro definition".to_string(),
    ))
}

/// Read one macro argument: a `{…}` group, a control sequence, or one character.
fn read_argument(chars: &[char], start: usize) -> Result<(String, usize), MathError> {
    if chars[start] == '{' {
        return read_group(chars, start);
    }
    if chars[start] == '\\' {
        return Ok(read_command(chars, start));
    }
    Ok((chars[start].to_string(), start + 1))
}

/// Collect the `nargs` arguments of a macro call starting at `start`.
fn read_arguments(
    chars: &[char],
    start: usize,
    mac: &Macro,
    name: &str,
) -> Result<(Vec<String>, usize), MathError> {
    let mut args = Vec::new();
    let mut i = start;

    // A first argument written as `[…]` is the LaTeX optional argument, which
    // only exists when the macro declared a default; otherwise it may be omitted.
    if mac.nargs > 0 && mac.default.is_some() {
        let next = skip_whitespace(chars, i);
        if next < chars.len() && chars[next] == '[' {
            let (arg, after) = read_bracket(chars, next)?;
            args.push(arg);
            i = after;
        } else if let Some(default) = &mac.default {
            args.push(default.clone());
        }
    }

    while args.len() < mac.nargs {
        let next = skip_whitespace(chars, i);
        if next >= chars.len() {
            return Err(MathError::MacroError(format!(
                "`{name}` expects {} argument(s), found {}",
                mac.nargs,
                args.len()
            )));
        }
        let (arg, after) = read_argument(chars, next)?;
        args.push(arg);
        i = after;
    }
    Ok((args, i))
}

/// Replace `#1`…`#9` in a macro body with the arguments of the call.
fn substitute(mac: &Macro, args: &[String], name: &str) -> Result<String, MathError> {
    let chars: Vec<char> = mac.body.chars().collect();
    let mut out = String::with_capacity(mac.body.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '#' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        match chars.get(i + 1) {
            Some('#') => {
                out.push('#');
                i += 2;
            }
            Some(digit) if digit.is_ascii_digit() && *digit != '0' => {
                let index = digit.to_digit(10).unwrap() as usize;
                if index > args.len() {
                    return Err(MathError::MacroError(format!(
                        "`{name}` references #{index} but takes {} argument(s)",
                        mac.nargs
                    )));
                }
                out.push_str(&args[index - 1]);
                i += 2;
            }
            _ => {
                return Err(MathError::MacroError(format!(
                    "`{name}` has a stray '#' in its definition"
                )));
            }
        }
    }
    Ok(out)
}

/// Split `src` into the formula with all definitions removed and the macro table.
fn collect_definitions(src: &str) -> Result<(String, HashMap<String, Macro>), MathError> {
    let chars: Vec<char> = src.chars().collect();
    let mut source = String::with_capacity(src.len());
    let mut definitions: HashMap<String, Macro> = HashMap::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '\\' {
            source.push(chars[i]);
            i += 1;
            continue;
        }
        let (name, after_name) = read_command(&chars, i);
        match name.as_str() {
            "\\newcommand" | "\\renewcommand" | "\\providecommand" => {
                let (macro_name, mac, after) = parse_newcommand(&chars, after_name, &name)?;
                match name.as_str() {
                    "\\newcommand" if definitions.contains_key(&macro_name) => {
                        return Err(MathError::MacroError(format!(
                            "`{macro_name}` is already defined (use \\renewcommand)"
                        )));
                    }
                    "\\renewcommand" if !definitions.contains_key(&macro_name) => {
                        return Err(MathError::MacroError(format!(
                            "`{macro_name}` is not defined (use \\newcommand)"
                        )));
                    }
                    "\\providecommand" if definitions.contains_key(&macro_name) => {}
                    _ => {
                        definitions.insert(macro_name, mac);
                    }
                }
                i = after;
            }
            "\\def" => {
                let (macro_name, mac, after) = parse_def(&chars, after_name)?;
                definitions.insert(macro_name, mac);
                i = after;
            }
            _ => {
                source.extend(chars[i..after_name].iter());
                i = after_name;
            }
        }
    }
    Ok((source, definitions))
}

/// Parse `{\name}[nargs][default]{body}` (or `\name…`) after `\newcommand`.
fn parse_newcommand(
    chars: &[char],
    start: usize,
    command: &str,
) -> Result<(String, Macro, usize), MathError> {
    let mut i = skip_whitespace(chars, start);
    let macro_name = if i < chars.len() && chars[i] == '{' {
        let (group, after) = read_group(chars, i)?;
        i = after;
        group.trim().to_string()
    } else if i < chars.len() && chars[i] == '\\' {
        let (name, after) = read_command(chars, i);
        i = after;
        name
    } else {
        return Err(MathError::MacroError(format!(
            "`{command}` expects a macro name"
        )));
    };
    if !macro_name.starts_with('\\') || macro_name.len() < 2 {
        return Err(MathError::MacroError(format!(
            "`{command}` expects a macro name starting with a backslash, found `{macro_name}`"
        )));
    }

    i = skip_whitespace(chars, i);
    let mut nargs = 0usize;
    if i < chars.len() && chars[i] == '[' {
        let (count, after) = read_bracket(chars, i)?;
        nargs = count.trim().parse::<usize>().map_err(|_| {
            MathError::MacroError(format!("`{macro_name}` has a non-numeric argument count `{count}`"))
        })?;
        if nargs > 9 {
            return Err(MathError::MacroError(format!(
                "`{macro_name}` declares {nargs} arguments; at most 9 are supported"
            )));
        }
        i = after;
    }

    let mut default = None;
    i = skip_whitespace(chars, i);
    if nargs > 0 && i < chars.len() && chars[i] == '[' {
        let (value, after) = read_bracket(chars, i)?;
        default = Some(value);
        i = after;
    }

    i = skip_whitespace(chars, i);
    if i >= chars.len() || chars[i] != '{' {
        return Err(MathError::MacroError(format!(
            "`{macro_name}` is missing its replacement text"
        )));
    }
    let (body, after) = read_group(chars, i)?;
    Ok((
        macro_name,
        Macro {
            nargs,
            default,
            body,
        },
        after,
    ))
}

/// Parse `\name#1#2{body}` after `\def`.
fn parse_def(chars: &[char], start: usize) -> Result<(String, Macro, usize), MathError> {
    let i = skip_whitespace(chars, start);
    if i >= chars.len() || chars[i] != '\\' {
        return Err(MathError::MacroError(
            "`\\def` expects a macro name".to_string(),
        ));
    }
    let (macro_name, mut i) = read_command(chars, i);

    // The parameter text may only be `#1#2…`; delimited parameters (`\def\x#1,{}`)
    // are not supported.
    let mut nargs = 0usize;
    loop {
        let next = skip_whitespace(chars, i);
        if next >= chars.len() || chars[next] == '{' {
            i = next;
            break;
        }
        if chars[next] == '#' {
            let digit = chars.get(next + 1).copied();
            match digit {
                Some(d) if d.is_ascii_digit() && d != '0' => {
                    nargs += 1;
                    i = next + 2;
                }
                _ => {
                    return Err(MathError::MacroError(format!(
                        "`{macro_name}` has a malformed parameter text"
                    )));
                }
            }
        } else {
            return Err(MathError::MacroError(format!(
                "`{macro_name}` uses a delimited parameter text, which is not supported"
            )));
        }
    }
    if nargs > 9 {
        return Err(MathError::MacroError(format!(
            "`{macro_name}` declares {nargs} parameters; at most 9 are supported"
        )));
    }
    if i >= chars.len() || chars[i] != '{' {
        return Err(MathError::MacroError(format!(
            "`{macro_name}` is missing its replacement text"
        )));
    }
    let (body, after) = read_group(chars, i)?;
    Ok((
        macro_name,
        Macro {
            nargs,
            default: None,
            body,
        },
        after,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expand(src: &str) -> String {
        expand_macros(src).expect("macro expansion should succeed")
    }

    #[test]
    fn newcommand_without_arguments() {
        assert_eq!(expand(r"\newcommand{\R}{\mathbb{R}} x \in \R"), r" x \in \mathbb{R}");
    }

    #[test]
    fn newcommand_with_arguments() {
        assert_eq!(
            expand(r"\newcommand{\norm}[1]{\left|#1\right|} \norm{x}"),
            r" \left|x\right|"
        );
        assert_eq!(
            expand(r"\newcommand{\pair}[2]{(#1, #2)} \pair{a}{b}"),
            r" (a, b)"
        );
    }

    #[test]
    fn newcommand_default_argument() {
        assert_eq!(
            expand(r"\newcommand{\norm}[2][2]{\|#1\|_{#2}} \norm{x}"),
            r" \|2\|_{x}"
        );
        assert_eq!(
            expand(r"\newcommand{\norm}[2][2]{\|#1\|_{#2}} \norm[3]{x}"),
            r" \|3\|_{x}"
        );
    }

    #[test]
    fn def_and_parameterised_def() {
        assert_eq!(expand(r"\def\vx{\mathbf{x}} \vx"), r" \mathbf{x}");
        assert_eq!(expand(r"\def\pair#1#2{#1+#2} \pair ab"), r" a+b");
    }

    #[test]
    fn redefinition_rules() {
        assert_eq!(expand(r"\newcommand{\R}{a}\renewcommand{\R}{b} \R"), " b");
        assert_eq!(
            expand(r"\newcommand{\R}{a}\providecommand{\R}{b} \R"),
            " a"
        );
        let err = expand_macros(r"\newcommand{\R}{a}\newcommand{\R}{b} \R").unwrap_err();
        assert!(matches!(err, MathError::MacroError(_)), "{err:?}");
        let err = expand_macros(r"\renewcommand{\R}{b} \R").unwrap_err();
        assert!(matches!(err, MathError::MacroError(_)), "{err:?}");
    }

    #[test]
    fn nested_expansion() {
        assert_eq!(
            expand(r"\newcommand{\R}{\mathbb{R}}\newcommand{\Rn}{\R^n} \Rn"),
            r" \mathbb{R}^n"
        );
    }

    #[test]
    fn recursive_macro_is_reported() {
        let err = expand_macros(r"\def\a{\a} \a").unwrap_err();
        assert!(
            err.to_string().contains("expansion"),
            "expected an expansion error, got {err}"
        );
    }

    #[test]
    fn missing_argument_is_reported() {
        let err = expand_macros(r"\newcommand{\x}[2]{#1#2} \x{a}").unwrap_err();
        assert!(matches!(err, MathError::MacroError(_)), "{err:?}");
    }

    #[test]
    fn undefined_macro_is_left_for_the_parser() {
        assert_eq!(expand(r"\newcommand{\R}{\mathbb{R}} \foo"), r" \foo");
    }

    #[test]
    fn delimited_parameters_are_rejected() {
        let err = expand_macros(r"\def\x#1,#2{#1#2} \x a,b").unwrap_err();
        assert!(err.to_string().contains("delimited"), "{err}");
    }
}
