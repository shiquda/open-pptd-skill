//! Native OOXML OMML math compiler and decompiler.
//!
//! Converts LaTeX math formulas into native OOXML OMML (`<a14:m>`)
//! structures and reconstructs LaTeX formulas from OMML elements during reverse
//! compilation.
//!
//! Unsupported LaTeX commands result in a hard [`MathError::UnsupportedCommand`]
//! so that raw LaTeX never leaks into the presentation.

use std::fmt::Write as _;
use xmltree::Element as XmlEl;

mod macros;

use macros::expand_macros;

/// Errors produced during LaTeX math compilation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MathError {
    /// An unsupported or unknown LaTeX command was encountered.
    #[error("unsupported LaTeX command: {0}")]
    UnsupportedCommand(String),

    /// The LaTeX formula syntax is malformed (unbalanced braces, unclosed delimiter, etc.).
    #[error("LaTeX math syntax error: {0}")]
    SyntaxError(String),

    /// A macro definition is invalid or its expansion cannot be completed.
    #[error("LaTeX macro error: {0}")]
    MacroError(String),
}

/// Escape text content per XML 1.0 specifications for `<m:t>` nodes.
fn xml_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Supported symbols & commands
// ---------------------------------------------------------------------------

/// Greek letters, operators, and relations mapped to Unicode characters.
fn symbol_char(cmd: &str) -> Option<&'static str> {
    Some(match cmd {
        // Greek lowercase
        "alpha" => "α",
        "beta" => "β",
        "gamma" => "γ",
        "delta" => "δ",
        "epsilon" | "varepsilon" => "ε",
        "zeta" => "ζ",
        "eta" => "η",
        "theta" => "θ",
        "vartheta" => "ϑ",
        "iota" => "ι",
        "kappa" => "κ",
        "lambda" => "λ",
        "mu" => "μ",
        "nu" => "ν",
        "xi" => "ξ",
        "pi" => "π",
        "varpi" => "ϖ",
        "rho" => "ρ",
        "varrho" => "ϱ",
        "sigma" => "σ",
        "varsigma" => "ς",
        "tau" => "τ",
        "upsilon" => "υ",
        "phi" => "φ",
        "varphi" => "ϕ",
        "chi" => "χ",
        "psi" => "ψ",
        "omega" => "ω",
        // Greek uppercase
        "Gamma" => "Γ",
        "Delta" => "Δ",
        "Theta" => "Θ",
        "Lambda" => "Λ",
        "Xi" => "Ξ",
        "Pi" => "Π",
        "Sigma" => "Σ",
        "Upsilon" => "Υ",
        "Phi" => "Φ",
        "Psi" => "Ψ",
        "Omega" => "Ω",
        // Relations & operators
        "le" | "leq" => "≤",
        "ge" | "geq" => "≥",
        "neq" | "ne" => "≠",
        "approx" => "≈",
        "sim" => "∼",
        "simeq" => "≃",
        "propto" => "∝",
        "equiv" => "≡",
        "mid" => "∣",
        "in" => "∈",
        "notin" => "∉",
        "subset" => "⊂",
        "subseteq" => "⊆",
        "supset" => "⊃",
        "supseteq" => "⊇",
        "cup" => "∪",
        "cap" => "∩",
        "times" => "×",
        "cdot" => "·",
        "pm" => "±",
        "mp" => "∓",
        "div" => "÷",
        "circ" => "∘",
        "bullet" => "•",
        "ast" => "∗",
        "star" => "⋆",
        "oplus" => "⊕",
        "otimes" => "⊗",
        "infty" => "∞",
        "partial" => "∂",
        "nabla" => "∇",
        "forall" => "∀",
        "exists" => "∃",
        "nexists" => "∄",
        "neg" | "lnot" => "¬",
        "land" | "wedge" => "∧",
        "lor" | "vee" => "∨",
        "to" | "rightarrow" => "→",
        "leftarrow" => "←",
        "Rightarrow" => "⇒",
        "Leftarrow" => "⇐",
        "leftrightarrow" => "↔",
        "Leftrightarrow" => "⇔",
        "mapsto" => "↦",
        "uparrow" => "↑",
        "downarrow" => "↓",
        "ldots" | "dots" => "…",
        "cdots" => "⋯",
        "ddots" => "⋱",
        "vdots" => "⋮",
        "prime" => "′",
        "angle" => "∠",
        "perp" => "⊥",
        "parallel" => "∥",
        "emptyset" => "∅",
        "therefore" => "∴",
        "because" => "∵",
        "ell" => "ℓ",
        "hbar" => "ℏ",
        "imath" => "ı",
        "jmath" => "ȷ",
        "Re" => "ℜ",
        "Im" => "ℑ",
        "aleph" => "ℵ",
        _ => return None,
    })
}

/// Upright mathematical functions / operators.
fn is_upright_func(cmd: &str) -> bool {
    matches!(
        cmd,
        "sin"
            | "cos"
            | "tan"
            | "arcsin"
            | "arccos"
            | "arctan"
            | "sinh"
            | "cosh"
            | "tanh"
            | "log"
            | "ln"
            | "exp"
            | "lg"
            | "max"
            | "min"
            | "argmax"
            | "argmin"
            | "arg"
            | "det"
            | "dim"
            | "ker"
            | "gcd"
            | "deg"
            | "sup"
            | "inf"
            | "hom"
            | "sec"
            | "csc"
            | "cot"
    )
}

/// Big operators with limits.
fn big_operator_info(cmd: &str) -> Option<(&'static str, &'static str)> {
    Some(match cmd {
        "sum" => ("∑", "undOvr"),
        "prod" => ("∏", "undOvr"),
        "int" => ("∫", "subSup"),
        "oint" => ("∮", "subSup"),
        "iint" => ("∬", "subSup"),
        "iiint" => ("∭", "subSup"),
        "lim" => ("lim", "undOvr"),
        _ => return None,
    })
}

/// Delimiter mapping for `\left` / `\right`.
fn delimiter_chr(cmd: &str) -> Option<&'static str> {
    Some(match cmd {
        "(" => "(",
        ")" => ")",
        "[" => "[",
        "]" => "]",
        "{" | "brace" => "{",
        "}" => "}",
        "|" | "vert" => "|",
        "Vert" => "‖",
        "langle" => "⟨",
        "rangle" => "⟩",
        "lfloor" => "⌊",
        "rfloor" => "⌋",
        "lceil" => "⌈",
        "rceil" => "⌉",
        "." => "",
        _ => return None,
    })
}

/// Accents mapping.
fn accent_chr(cmd: &str) -> Option<&'static str> {
    Some(match cmd {
        "bar" | "overline" => "¯",
        "hat" => "̂",
        "vec" => "⃗",
        "tilde" => "̃",
        "dot" => "˙",
        "ddot" => "¨",
        _ => return None,
    })
}

/// Blackboard-bold letters for `\mathbb{…}`.
fn mathbb(letter: char) -> char {
    match letter {
        'A' => '𝔸',
        'B' => '𝔹',
        'C' => 'ℂ',
        'D' => '𝔻',
        'E' => '𝔼',
        'F' => '𝔽',
        'G' => '𝔾',
        'H' => 'ℍ',
        'I' => '𝕀',
        'J' => '𝕁',
        'K' => '𝕂',
        'L' => '𝕃',
        'M' => '𝕄',
        'N' => 'ℕ',
        'O' => '𝕆',
        'P' => 'ℙ',
        'Q' => 'ℚ',
        'R' => 'ℝ',
        'S' => '𝕊',
        'T' => '𝕋',
        'U' => '𝕌',
        'V' => '𝕎',
        'W' => '𝕎',
        'X' => '𝕏',
        'Y' => '𝕐',
        'Z' => 'ℤ',
        '0' => '𝟘',
        '1' => '𝟙',
        '2' => '𝟚',
        '3' => '𝟛',
        '4' => '𝟜',
        '5' => '𝟝',
        '6' => '𝟞',
        '7' => '𝟟',
        '8' => '𝟠',
        '9' => '𝟡',
        other => other,
    }
}

/// Spacing commands.
fn spacing_str(cmd: &str) -> Option<&'static str> {
    Some(match cmd {
        "," => "\u{2009}",          // thin space
        ";" => "\u{2005}",          // thick space
        ":" => "\u{2005}",          // medium space
        "quad" => "\u{2003}",       // em space
        "qquad" => "\u{2003}\u{2003}", // double em space
        " " => " ",
        "!" => "",                  // negative thin space
        _ => return None,
    })
}

/// Check if a command is supported by our LaTeX math subset.
fn is_supported_command(cmd: &str) -> bool {
    symbol_char(cmd).is_some()
        || is_upright_func(cmd)
        || big_operator_info(cmd).is_some()
        || accent_chr(cmd).is_some()
        || spacing_str(cmd).is_some()
        || matches!(
            cmd,
            "frac"
                | "dfrac"
                | "tfrac"
                | "sqrt"
                | "left"
                | "right"
                | "begin"
                | "end"
                | "mathbb"
                | "mathrm"
                | "text"
                | "operatorname"
                | "{"
                | "}"
                | "_"
                | "&"
                | "%"
                | "#"
                | "$"
                | "\\"
                | "|"
                | "vert"
                | "Vert"
                | "langle"
                | "rangle"
                | "lfloor"
                | "rfloor"
                | "lceil"
                | "rceil"
        )
}

/// Everything that would stop `pptd build` from compiling a LaTeX formula:
/// unsupported commands and invalid macro definitions.
///
/// Macro definitions are expanded first, so a formula that defines its own
/// aliases is reported clean. Each returned string is a user-facing message.
pub fn latex_problems(src: &str) -> Vec<String> {
    let expanded = match expand_macros(src) {
        Ok(expanded) => expanded,
        Err(err) => return vec![err.to_string()],
    };
    let mut problems = Vec::new();
    let chars: Vec<char> = expanded.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' {
            // Comment until newline
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if chars[i] == '\\' {
            i += 1;
            if i >= chars.len() {
                break;
            }
            let start = i;
            let command = if chars[i].is_ascii_alphabetic() {
                while i < chars.len() && chars[i].is_ascii_alphabetic() {
                    i += 1;
                }
                chars[start..i].iter().collect::<String>()
            } else {
                let single: String = chars[start..=start].iter().collect();
                i += 1;
                single
            };
            if !is_supported_command(&command) {
                let problem = format!("unsupported LaTeX command `\\{command}`");
                if !problems.contains(&problem) {
                    problems.push(problem);
                }
            }
        } else {
            i += 1;
        }
    }
    problems
}

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum MathNode {
    Text(String),
    Symbol(String),
    UprightText(String),
    /// A `{…}` group. It is kept as a node instead of being flattened into text
    /// so that the structure inside survives: `{\frac{a}{b}}` must stay a
    /// fraction, not collapse into the characters `a/b`.
    Group(Vec<MathNode>),
    Frac {
        num: Vec<MathNode>,
        den: Vec<MathNode>,
    },
    Sqrt {
        deg: Option<Vec<MathNode>>,
        body: Vec<MathNode>,
    },
    Sup {
        base: Box<MathNode>,
        sup: Vec<MathNode>,
    },
    Sub {
        base: Box<MathNode>,
        sub: Vec<MathNode>,
    },
    SubSup {
        base: Box<MathNode>,
        sub: Vec<MathNode>,
        sup: Vec<MathNode>,
    },
    Delimited {
        beg: String,
        end: String,
        body: Vec<MathNode>,
    },
    Nary {
        chr: String,
        lim_loc: String,
        sub: Option<Vec<MathNode>>,
        sup: Option<Vec<MathNode>>,
        body: Vec<MathNode>,
    },
    Matrix {
        rows: Vec<Vec<Vec<MathNode>>>,
    },
    /// A multi-line equation (`align`, `aligned`, `gather`, `gathered`, `split`,
    /// `eqnarray`): rows of cells, each cell boundary being an alignment point.
    ///
    /// `display_only` marks the environments that are only valid as display
    /// math (`align`, `gather`, `eqnarray`), as opposed to the `…ed` variants
    /// that may appear inside a line of text.
    Aligned {
        rows: Vec<Vec<Vec<MathNode>>>,
        display_only: bool,
    },
    Accent {
        chr: String,
        body: Vec<MathNode>,
    },
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            chars: src.chars().collect(),
            pos: 0,
            _marker: std::marker::PhantomData,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if c.is_whitespace() {
                self.pos += 1;
            } else if c == '%' {
                while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    fn parse_list(&mut self, stop_chars: &[char]) -> Result<Vec<MathNode>, MathError> {
        let mut list = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.chars.len() {
                break;
            }
            if let Some(c) = self.peek() {
                if stop_chars.contains(&c) {
                    break;
                }
            }

            let node = self.parse_primary()?;
            // Check for attached script `^` or `_`
            let node = self.attach_scripts(node)?;
            list.push(node);
        }
        Ok(list)
    }

    fn attach_scripts(&mut self, base: MathNode) -> Result<MathNode, MathError> {
        self.skip_whitespace();
        let c = self.peek();
        if c == Some('^') {
            self.next_char();
            let sup = self.parse_arg()?;
            self.skip_whitespace();
            if self.peek() == Some('_') {
                self.next_char();
                let sub = self.parse_arg()?;
                Ok(MathNode::SubSup {
                    base: Box::new(base),
                    sub,
                    sup,
                })
            } else {
                Ok(MathNode::Sup {
                    base: Box::new(base),
                    sup,
                })
            }
        } else if c == Some('_') {
            self.next_char();
            let sub = self.parse_arg()?;
            self.skip_whitespace();
            if self.peek() == Some('^') {
                self.next_char();
                let sup = self.parse_arg()?;
                Ok(MathNode::SubSup {
                    base: Box::new(base),
                    sub,
                    sup,
                })
            } else {
                Ok(MathNode::Sub {
                    base: Box::new(base),
                    sub,
                })
            }
        } else {
            Ok(base)
        }
    }

    fn parse_arg(&mut self) -> Result<Vec<MathNode>, MathError> {
        self.skip_whitespace();
        if self.peek() == Some('{') {
            self.next_char();
            let res = self.parse_list(&['}'])?;
            if self.peek() == Some('}') {
                self.next_char();
            } else {
                return Err(MathError::SyntaxError("unclosed '{'".to_string()));
            }
            Ok(res)
        } else {
            let node = self.parse_primary()?;
            Ok(vec![node])
        }
    }

    /// Parse the operand of a big operator: a `{…}` group or a single atom.
    fn parse_operand(&mut self) -> Result<Vec<MathNode>, MathError> {
        self.skip_whitespace();
        match self.peek() {
            None | Some('}') | Some(']') | Some(')') | Some('&') => Ok(Vec::new()),
            Some('{') => self.parse_arg(),
            _ => {
                let node = self.parse_primary()?;
                let node = self.attach_scripts(node)?;
                Ok(vec![node])
            }
        }
    }

    fn parse_verbatim_text(&mut self) -> Result<String, MathError> {
        self.skip_whitespace();
        if self.peek() != Some('{') {
            return Err(MathError::SyntaxError("expected '{' after \\text".into()));
        }
        self.next_char();
        let mut depth = 1usize;
        let mut buf = String::new();
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            self.pos += 1;
            if c == '{' {
                depth += 1;
                buf.push(c);
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    return Ok(buf);
                }
                buf.push(c);
            } else {
                buf.push(c);
            }
        }
        Err(MathError::SyntaxError("unclosed '{' in text".into()))
    }

    fn parse_primary(&mut self) -> Result<MathNode, MathError> {
        self.skip_whitespace();
        let Some(c) = self.peek() else {
            return Err(MathError::SyntaxError("unexpected end of formula".into()));
        };

        if c == '{' {
            self.next_char();
            let inner = self.parse_list(&['}'])?;
            if self.peek() == Some('}') {
                self.next_char();
            } else {
                return Err(MathError::SyntaxError("unclosed '{'".to_string()));
            }
            return Ok(MathNode::Group(inner));
        }

        if c == '\\' {
            self.next_char();
            if self.pos >= self.chars.len() {
                return Err(MathError::SyntaxError("trailing backslash".into()));
            }
            let start = self.pos;
            if self.chars[self.pos].is_ascii_alphabetic() {
                while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_alphabetic() {
                    self.pos += 1;
                }
                let name: String = self.chars[start..self.pos].iter().collect();
                self.parse_command(&name)
            } else {
                let single = self.chars[self.pos];
                self.pos += 1;
                let s: String = single.to_string();
                self.parse_command(&s)
            }
        } else {
            self.pos += 1;
            Ok(MathNode::Text(c.to_string()))
        }
    }

    fn parse_command(&mut self, name: &str) -> Result<MathNode, MathError> {
        if let Some(sym) = symbol_char(name) {
            return Ok(MathNode::Symbol(sym.to_string()));
        }
        if is_upright_func(name) {
            return Ok(MathNode::UprightText(name.to_string()));
        }
        if let Some((chr, lim_loc)) = big_operator_info(name) {
            // Check for limits `_` and `^`
            let mut sub = None;
            let mut sup = None;
            loop {
                self.skip_whitespace();
                let c = self.peek();
                if c == Some('_') && sub.is_none() {
                    self.next_char();
                    sub = Some(self.parse_arg()?);
                } else if c == Some('^') && sup.is_none() {
                    self.next_char();
                    sup = Some(self.parse_arg()?);
                } else {
                    break;
                }
            }
            // TeX gives a big operator the single atom that follows it as its
            // operand: `\sum_{i=1}^n A + \sum_{j=1}^m B` is the first sum of `A`
            // plus the second sum, not one sum swallowing everything after it.
            let body = self.parse_operand()?;
            return Ok(MathNode::Nary {
                chr: chr.to_string(),
                lim_loc: lim_loc.to_string(),
                sub,
                sup,
                body,
            });
        }
        if let Some(achr) = accent_chr(name) {
            let body = self.parse_arg()?;
            return Ok(MathNode::Accent {
                chr: achr.to_string(),
                body,
            });
        }
        if let Some(sp) = spacing_str(name) {
            return Ok(MathNode::Text(sp.to_string()));
        }

        match name {
            "frac" | "dfrac" | "tfrac" => {
                let num = self.parse_arg()?;
                let den = self.parse_arg()?;
                Ok(MathNode::Frac { num, den })
            }
            "sqrt" => {
                self.skip_whitespace();
                let deg = if self.peek() == Some('[') {
                    self.next_char();
                    let d = self.parse_list(&[']'])?;
                    if self.peek() == Some(']') {
                        self.next_char();
                    } else {
                        return Err(MathError::SyntaxError("unclosed '[' in sqrt".into()));
                    }
                    Some(d)
                } else {
                    None
                };
                let body = self.parse_arg()?;
                Ok(MathNode::Sqrt { deg, body })
            }
            "left" => {
                self.skip_whitespace();
                let delim = self.parse_delimiter()?;
                let body = self.parse_list_until_right()?;
                let end_delim = self.parse_delimiter()?;
                Ok(MathNode::Delimited {
                    beg: delim,
                    end: end_delim,
                    body,
                })
            }
            "begin" => {
                let env = self.parse_verbatim_text()?;
                self.parse_environment(&env)
            }
            "mathbb" => {
                let text = self.parse_arg_as_text()?;
                let res: String = text.chars().map(mathbb).collect();
                Ok(MathNode::Text(res))
            }
            "mathrm" | "operatorname" => {
                let text = self.parse_arg_as_text()?;
                Ok(MathNode::UprightText(text))
            }
            "text" => {
                let text = self.parse_verbatim_text()?;
                Ok(MathNode::UprightText(text))
            }
            "{" => Ok(MathNode::Text("{".into())),
            "}" => Ok(MathNode::Text("}".into())),
            "_" => Ok(MathNode::Text("_".into())),
            "&" => Ok(MathNode::Text("&".into())),
            "%" => Ok(MathNode::Text("%".into())),
            "#" => Ok(MathNode::Text("#".into())),
            "$" => Ok(MathNode::Text("$".into())),
            "\\" => Ok(MathNode::Text("\n".into())),
            "vert" => Ok(MathNode::Text("|".into())),
            "Vert" => Ok(MathNode::Text("‖".into())),
            "langle" => Ok(MathNode::Text("⟨".into())),
            "rangle" => Ok(MathNode::Text("⟩".into())),
            _ => Err(MathError::UnsupportedCommand(format!("\\{name}"))),
        }
    }

    fn parse_arg_as_text(&mut self) -> Result<String, MathError> {
        let nodes = self.parse_arg()?;
        Ok(render_nodes_text(&nodes))
    }

    fn parse_delimiter(&mut self) -> Result<String, MathError> {
        self.skip_whitespace();
        let Some(c) = self.next_char() else {
            return Err(MathError::SyntaxError("expected delimiter after \\left/\\right".into()));
        };
        if c == '\\' {
            let start = self.pos;
            if self.pos < self.chars.len() && self.chars[self.pos].is_ascii_alphabetic() {
                while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_alphabetic() {
                    self.pos += 1;
                }
                let name: String = self.chars[start..self.pos].iter().collect();
                delimiter_chr(&name)
                    .map(str::to_string)
                    .ok_or_else(|| MathError::UnsupportedCommand(format!("\\{name}")))
            } else if self.pos < self.chars.len() {
                let single = self.chars[self.pos];
                self.pos += 1;
                let s = single.to_string();
                delimiter_chr(&s)
                    .map(str::to_string)
                    .ok_or_else(|| MathError::UnsupportedCommand(format!("\\{s}")))
            } else {
                Err(MathError::SyntaxError("trailing backslash in delimiter".into()))
            }
        } else {
            let s = c.to_string();
            delimiter_chr(&s)
                .map(str::to_string)
                .ok_or_else(|| MathError::SyntaxError(format!("unrecognized delimiter '{c}'")))
        }
    }

    fn parse_list_until_right(&mut self) -> Result<Vec<MathNode>, MathError> {
        let mut list = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.chars.len() {
                return Err(MathError::SyntaxError("expected \\right".into()));
            }
            if self.peek() == Some('\\') {
                let start = self.pos;
                self.next_char();
                let cmd_start = self.pos;
                while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_alphabetic() {
                    self.pos += 1;
                }
                let name: String = self.chars[cmd_start..self.pos].iter().collect();
                if name == "right" {
                    break;
                }
                // Rewind and parse command
                self.pos = start;
            }
            let node = self.parse_primary()?;
            let node = self.attach_scripts(node)?;
            list.push(node);
        }
        Ok(list)
    }

    fn parse_environment(&mut self, env: &str) -> Result<MathNode, MathError> {
        if let Some(display_only) = aligned_environment(env) {
            let rows = self.parse_grid(env)?;
            return Ok(MathNode::Aligned { rows, display_only });
        }
        match env {
            "matrix" | "pmatrix" | "bmatrix" | "vmatrix" | "cases" => {
                let rows = self.parse_grid(env)?;

                let matrix_node = MathNode::Matrix { rows };
                match env {
                    "pmatrix" => Ok(MathNode::Delimited {
                        beg: "(".into(),
                        end: ")".into(),
                        body: vec![matrix_node],
                    }),
                    "bmatrix" => Ok(MathNode::Delimited {
                        beg: "[".into(),
                        end: "]".into(),
                        body: vec![matrix_node],
                    }),
                    "vmatrix" => Ok(MathNode::Delimited {
                        beg: "|".into(),
                        end: "|".into(),
                        body: vec![matrix_node],
                    }),
                    "cases" => Ok(MathNode::Delimited {
                        beg: "{".into(),
                        end: "".into(),
                        body: vec![matrix_node],
                    }),
                    _ => Ok(matrix_node),
                }
            }
            other => Err(MathError::UnsupportedCommand(format!("\\begin{{{other}}}"))),
        }
    }

    /// Parse the body of a grid environment into rows of cells.
    fn parse_grid(&mut self, env: &str) -> Result<Vec<Vec<Vec<MathNode>>>, MathError> {
        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        let mut current_cell = Vec::new();

        loop {
            self.skip_whitespace();
            if self.pos >= self.chars.len() {
                return Err(MathError::SyntaxError(format!(
                    "unclosed environment \\begin{{{env}}}"
                )));
            }
            if self.peek() == Some('&') {
                self.next_char();
                current_row.push(std::mem::take(&mut current_cell));
                continue;
            }
            if self.peek() == Some('\\') {
                let start = self.pos;
                self.next_char();
                let cmd_start = self.pos;
                if self.pos < self.chars.len() && self.chars[self.pos].is_ascii_alphabetic() {
                    while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_alphabetic() {
                        self.pos += 1;
                    }
                    let name: String = self.chars[cmd_start..self.pos].iter().collect();
                    if name == "end" {
                        let end_env = self.parse_verbatim_text()?;
                        if end_env != env {
                            return Err(MathError::SyntaxError(format!(
                                "mismatched \\end{{{end_env}}} for \\begin{{{env}}}"
                            )));
                        }
                        current_row.push(current_cell);
                        if !current_row.is_empty() {
                            rows.push(current_row);
                        }
                        break;
                    }
                    if name == "cr" {
                        current_row.push(std::mem::take(&mut current_cell));
                        rows.push(std::mem::take(&mut current_row));
                        continue;
                    }
                    // Rewind and parse regular command
                    self.pos = start;
                } else if self.pos < self.chars.len() && self.chars[self.pos] == '\\' {
                    self.next_char();
                    current_row.push(std::mem::take(&mut current_cell));
                    rows.push(std::mem::take(&mut current_row));
                    continue;
                } else {
                    self.pos = start;
                }
            }

            let node = self.parse_primary()?;
            let node = self.attach_scripts(node)?;
            current_cell.push(node);
        }

        Ok(rows)
    }
}

/// Environments that compile to `<m:eqArr>`; the flag marks the ones that are
/// only valid as display math (the `…ed` variants may be used inline).
fn aligned_environment(env: &str) -> Option<bool> {
    Some(match env {
        "align" | "align*" | "gather" | "gather*" | "eqnarray" | "eqnarray*" => true,
        "aligned" | "gathered" | "split" => false,
        _ => return None,
    })
}

fn render_nodes_text(nodes: &[MathNode]) -> String {
    let mut s = String::new();
    for node in nodes {
        match node {
            MathNode::Text(t) | MathNode::Symbol(t) | MathNode::UprightText(t) => s.push_str(t),
            MathNode::Group(inner) => s.push_str(&render_nodes_text(inner)),
            MathNode::Frac { num, den } => {
                s.push_str(&render_nodes_text(num));
                s.push('/');
                s.push_str(&render_nodes_text(den));
            }
            MathNode::Sqrt { body, .. } => {
                s.push_str(&render_nodes_text(body));
            }
            MathNode::Sup { base, sup } => {
                s.push_str(&render_nodes_text(&[base.as_ref().clone()]));
                s.push('^');
                s.push_str(&render_nodes_text(sup));
            }
            MathNode::Sub { base, sub } => {
                s.push_str(&render_nodes_text(&[base.as_ref().clone()]));
                s.push('_');
                s.push_str(&render_nodes_text(sub));
            }
            MathNode::SubSup { base, sub, sup } => {
                s.push_str(&render_nodes_text(&[base.as_ref().clone()]));
                s.push('_');
                s.push_str(&render_nodes_text(sub));
                s.push('^');
                s.push_str(&render_nodes_text(sup));
            }
            MathNode::Delimited { beg, end, body } => {
                s.push_str(beg);
                s.push_str(&render_nodes_text(body));
                s.push_str(end);
            }
            MathNode::Nary { body, .. } => {
                s.push_str(&render_nodes_text(body));
            }
            MathNode::Matrix { rows } => {
                for r in rows {
                    for c in r {
                        s.push_str(&render_nodes_text(c));
                    }
                }
            }
            MathNode::Aligned { rows, .. } => {
                for (r_idx, row) in rows.iter().enumerate() {
                    if r_idx > 0 {
                        s.push(' ');
                    }
                    for cell in row {
                        s.push_str(&render_nodes_text(cell));
                    }
                }
            }
            MathNode::Accent { body, .. } => {
                s.push_str(&render_nodes_text(body));
            }
        }
    }
    s
}

// ---------------------------------------------------------------------------
// OMML Emitter
// ---------------------------------------------------------------------------

fn emit_run(out: &mut String, text: &str, upright: bool) {
    if text.is_empty() {
        return;
    }
    out.push_str("<m:r>");
    if upright {
        out.push_str("<m:rPr><m:sty m:val=\"p\"/></m:rPr>");
    }
    out.push_str("<a:rPr lang=\"en-US\"><a:latin typeface=\"Cambria Math\"/></a:rPr>");
    out.push_str("<m:t>");
    out.push_str(&xml_escape(text));
    out.push_str("</m:t></m:r>");
}

fn emit_nodes(out: &mut String, nodes: &[MathNode]) {
    let mut text_buf = String::new();
    let mut text_upright = false;

    let flush_text = |out: &mut String, buf: &mut String, upright: bool| {
        if !buf.is_empty() {
            emit_run(out, buf, upright);
            buf.clear();
        }
    };

    for node in nodes {
        match node {
            MathNode::Text(t) => {
                if text_upright {
                    flush_text(out, &mut text_buf, text_upright);
                    text_upright = false;
                }
                text_buf.push_str(t);
            }
            MathNode::Symbol(s) => {
                if text_upright {
                    flush_text(out, &mut text_buf, text_upright);
                    text_upright = false;
                }
                text_buf.push_str(s);
            }
            MathNode::UprightText(u) => {
                if !text_upright {
                    flush_text(out, &mut text_buf, text_upright);
                    text_upright = true;
                }
                text_buf.push_str(u);
            }
            _ => {
                flush_text(out, &mut text_buf, text_upright);
                match node {
                    MathNode::Frac { num, den } => {
                        out.push_str("<m:f><m:fPr><m:type m:val=\"bar\"/></m:fPr><m:num>");
                        emit_nodes(out, num);
                        out.push_str("</m:num><m:den>");
                        emit_nodes(out, den);
                        out.push_str("</m:den></m:f>");
                    }
                    MathNode::Sqrt { deg, body } => {
                        out.push_str("<m:rad><m:radPr>");
                        if let Some(d) = deg {
                            out.push_str("<m:degHide m:val=\"0\"/></m:radPr><m:deg>");
                            emit_nodes(out, d);
                            out.push_str("</m:deg><m:e>");
                        } else {
                            out.push_str("<m:degHide m:val=\"1\"/></m:radPr><m:deg/><m:e>");
                        }
                        emit_nodes(out, body);
                        out.push_str("</m:e></m:rad>");
                    }
                    MathNode::Sup { base, sup } => {
                        out.push_str("<m:sSup><m:e>");
                        emit_nodes(out, &[base.as_ref().clone()]);
                        out.push_str("</m:e><m:sup>");
                        emit_nodes(out, sup);
                        out.push_str("</m:sup></m:sSup>");
                    }
                    MathNode::Sub { base, sub } => {
                        out.push_str("<m:sSub><m:e>");
                        emit_nodes(out, &[base.as_ref().clone()]);
                        out.push_str("</m:e><m:sub>");
                        emit_nodes(out, sub);
                        out.push_str("</m:sub></m:sSub>");
                    }
                    MathNode::SubSup { base, sub, sup } => {
                        out.push_str("<m:sSubSup><m:e>");
                        emit_nodes(out, &[base.as_ref().clone()]);
                        out.push_str("</m:e><m:sub>");
                        emit_nodes(out, sub);
                        out.push_str("</m:sub><m:sup>");
                        emit_nodes(out, sup);
                        out.push_str("</m:sup></m:sSubSup>");
                    }
                    MathNode::Delimited { beg, end, body } => {
                        out.push_str("<m:d><m:dPr>");
                        let _ = write!(out, "<m:begChr m:val=\"{}\"/>", xml_escape(beg));
                        let _ = write!(out, "<m:endChr m:val=\"{}\"/>", xml_escape(end));
                        out.push_str("</m:dPr><m:e>");
                        emit_nodes(out, body);
                        out.push_str("</m:e></m:d>");
                    }
                    MathNode::Nary {
                        chr,
                        lim_loc,
                        sub,
                        sup,
                        body,
                    } => {
                        // `\lim` is a function name, not an operator: ST_Char only
                        // accepts a single character, so `<m:chr m:val="lim"/>` is
                        // invalid OMML and renderers show a replacement glyph.
                        if chr == "lim" && sup.is_none() {
                            out.push_str("<m:func><m:funcPr/><m:fName><m:limLow><m:e>");
                            emit_run(out, "lim", true);
                            out.push_str("</m:e><m:lim>");
                            if let Some(s) = sub {
                                emit_nodes(out, s);
                            }
                            out.push_str("</m:lim></m:limLow></m:fName><m:e>");
                            emit_nodes(out, body);
                            out.push_str("</m:e></m:func>");
                            continue;
                        }
                        out.push_str("<m:nary><m:naryPr>");
                        let _ = write!(out, "<m:chr m:val=\"{}\"/>", xml_escape(chr));
                        let _ = write!(out, "<m:limLoc m:val=\"{lim_loc}\"/>");
                        if sub.is_none() {
                            out.push_str("<m:subHide m:val=\"1\"/>");
                        }
                        if sup.is_none() {
                            out.push_str("<m:supHide m:val=\"1\"/>");
                        }
                        out.push_str("</m:naryPr><m:sub>");
                        if let Some(s) = sub {
                            emit_nodes(out, s);
                        }
                        out.push_str("</m:sub><m:sup>");
                        if let Some(s) = sup {
                            emit_nodes(out, s);
                        }
                        out.push_str("</m:sup><m:e>");
                        emit_nodes(out, body);
                        out.push_str("</m:e></m:nary>");
                    }
                    MathNode::Matrix { rows } => {
                        out.push_str("<m:m>");
                        for row in rows {
                            out.push_str("<m:mr>");
                            for cell in row {
                                out.push_str("<m:e>");
                                emit_nodes(out, cell);
                                out.push_str("</m:e>");
                            }
                            out.push_str("</m:mr>");
                        }
                        out.push_str("</m:m>");
                    }
                    MathNode::Group(inner) => emit_nodes(out, inner),
                    MathNode::Aligned { rows, .. } => {
                        // An equation array: one <m:e> per line, with the alignment
                        // point written as an `&` run — the same encoding Word uses
                        // in its linear format and Microsoft's MML2OMML.XSL emits.
                        out.push_str("<m:eqArr><m:eqArrPr><m:baseJc m:val=\"center\"/></m:eqArrPr>");
                        for row in rows {
                            out.push_str("<m:e>");
                            for (idx, cell) in row.iter().enumerate() {
                                if idx > 0 {
                                    out.push_str("<m:r><m:t>&amp;</m:t></m:r>");
                                }
                                emit_nodes(out, cell);
                            }
                            out.push_str("</m:e>");
                        }
                        out.push_str("</m:eqArr>");
                    }
                    MathNode::Accent { chr, body } => {
                        out.push_str("<m:acc><m:accPr>");
                        let _ = write!(out, "<m:chr m:val=\"{}\"/>", xml_escape(chr));
                        out.push_str("</m:accPr><m:e>");
                        emit_nodes(out, body);
                        out.push_str("</m:e></m:acc>");
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    flush_text(out, &mut text_buf, text_upright);
}

/// Convert a LaTeX math formula into an OOXML OMML `<a14:m>` XML fragment.
///
/// Macro definitions (`\newcommand`, `\def`, …) are expanded first.
///
/// If `display` is `true`, emits display math:
/// `<a14:m><m:oMathPara><m:oMathParaPr><m:jc m:val="centerGroup"/></m:oMathParaPr><m:oMath>…</m:oMath></m:oMathPara></a14:m>`.
///
/// If `display` is `false`, emits inline math:
/// `<a14:m><m:oMath>…</m:oMath></a14:m>`.
pub fn latex_to_omml(src: &str, display: bool) -> Result<String, MathError> {
    let expanded = expand_macros(src)?;
    let mut parser = Parser::new(&expanded);
    let nodes = parser.parse_list(&[])?;
    if !display && has_display_only_environment(&nodes) {
        return Err(MathError::SyntaxError(
            "the align/gather/eqnarray environments need a paragraph of their own (display math)"
                .to_string(),
        ));
    }

    let mut inner = String::new();
    emit_nodes(&mut inner, &nodes);

    let mut out = String::new();
    out.push_str("<a14:m>");
    if display {
        out.push_str("<m:oMathPara><m:oMathParaPr><m:jc m:val=\"centerGroup\"/></m:oMathParaPr><m:oMath>");
        out.push_str(&inner);
        out.push_str("</m:oMath></m:oMathPara>");
    } else {
        out.push_str("<m:oMath>");
        out.push_str(&inner);
        out.push_str("</m:oMath>");
    }
    out.push_str("</a14:m>");
    Ok(out)
}

/// Whether the formula is a display-only environment (`align`, `gather`, `eqnarray`).
fn has_display_only_environment(nodes: &[MathNode]) -> bool {
    nodes.iter().any(|node| match node {
        MathNode::Aligned { display_only, .. } => *display_only,
        MathNode::Group(inner) => has_display_only_environment(inner),
        _ => false,
    })
}

/// Whether the formula uses an environment that must own its paragraph
/// (`align`, `gather`, `eqnarray`).
///
/// The validator uses this to mirror the display/inline decision the compiler
/// makes, so `pptd check` reports the same problem `pptd build` would.
pub fn requires_display_paragraph(src: &str) -> bool {
    let Ok(expanded) = expand_macros(src) else {
        return false;
    };
    let mut parser = Parser::new(&expanded);
    match parser.parse_list(&[]) {
        Ok(nodes) => has_display_only_environment(&nodes),
        Err(_) => false,
    }
}

/// A plain-text segment with optional baseline shift for fallback rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathSegment {
    pub text: String,
    pub baseline: Option<i32>,
}

/// Convert a LaTeX formula into plain-text fallback segments with baseline shifts.
///
/// Used for `<mc:Fallback>` in mixed-content paragraphs where office viewers like
/// LibreOffice do not parse `<a14:m>` inline math.
pub fn latex_to_fallback_segments(src: &str) -> Result<Vec<MathSegment>, MathError> {
    let expanded = expand_macros(src)?;
    let mut parser = Parser::new(&expanded);
    let nodes = parser.parse_list(&[])?;
    let mut raw = Vec::new();
    nodes_to_fallback(&nodes, None, &mut raw);
    Ok(coalesce_segments(raw))
}

fn is_single_token(nodes: &[MathNode]) -> bool {
    if nodes.len() != 1 {
        return false;
    }
    match &nodes[0] {
        MathNode::Text(s) => s.chars().count() == 1 || s.chars().all(|c| c.is_ascii_digit()),
        MathNode::Symbol(_) => true,
        MathNode::UprightText(s) => s.chars().count() == 1,
        MathNode::Group(inner) => is_single_token(inner),
        _ => false,
    }
}

fn nodes_to_fallback(
    nodes: &[MathNode],
    current_baseline: Option<i32>,
    out: &mut Vec<MathSegment>,
) {
    for node in nodes {
        match node {
            MathNode::Text(s) | MathNode::Symbol(s) | MathNode::UprightText(s) => {
                out.push(MathSegment {
                    text: s.clone(),
                    baseline: current_baseline,
                });
            }
            MathNode::Frac { num, den } => {
                let num_single = is_single_token(num);
                let den_single = is_single_token(den);
                if !num_single {
                    out.push(MathSegment {
                        text: "(".into(),
                        baseline: current_baseline,
                    });
                }
                nodes_to_fallback(num, current_baseline, out);
                if !num_single {
                    out.push(MathSegment {
                        text: ")".into(),
                        baseline: current_baseline,
                    });
                }
                out.push(MathSegment {
                    text: "/".into(),
                    baseline: current_baseline,
                });
                if !den_single {
                    out.push(MathSegment {
                        text: "(".into(),
                        baseline: current_baseline,
                    });
                }
                nodes_to_fallback(den, current_baseline, out);
                if !den_single {
                    out.push(MathSegment {
                        text: ")".into(),
                        baseline: current_baseline,
                    });
                }
            }
            MathNode::Sqrt { deg, body } => {
                out.push(MathSegment {
                    text: "√".into(),
                    baseline: current_baseline,
                });
                if let Some(d) = deg {
                    out.push(MathSegment {
                        text: "[".into(),
                        baseline: current_baseline,
                    });
                    nodes_to_fallback(d, current_baseline, out);
                    out.push(MathSegment {
                        text: "]".into(),
                        baseline: current_baseline,
                    });
                }
                let body_single = is_single_token(body);
                if !body_single {
                    out.push(MathSegment {
                        text: "(".into(),
                        baseline: current_baseline,
                    });
                }
                nodes_to_fallback(body, current_baseline, out);
                if !body_single {
                    out.push(MathSegment {
                        text: ")".into(),
                        baseline: current_baseline,
                    });
                }
            }
            MathNode::Sup { base, sup } => {
                nodes_to_fallback(std::slice::from_ref(base.as_ref()), current_baseline, out);
                nodes_to_fallback(sup, Some(30000), out);
            }
            MathNode::Sub { base, sub } => {
                nodes_to_fallback(std::slice::from_ref(base.as_ref()), current_baseline, out);
                nodes_to_fallback(sub, Some(-25000), out);
            }
            MathNode::SubSup { base, sub, sup } => {
                nodes_to_fallback(std::slice::from_ref(base.as_ref()), current_baseline, out);
                nodes_to_fallback(sub, Some(-25000), out);
                nodes_to_fallback(sup, Some(30000), out);
            }
            MathNode::Delimited { beg, end, body } => {
                out.push(MathSegment {
                    text: beg.clone(),
                    baseline: current_baseline,
                });
                nodes_to_fallback(body, current_baseline, out);
                out.push(MathSegment {
                    text: end.clone(),
                    baseline: current_baseline,
                });
            }
            MathNode::Nary {
                chr,
                lim_loc: _,
                sub,
                sup,
                body,
            } => {
                out.push(MathSegment {
                    text: chr.clone(),
                    baseline: current_baseline,
                });
                if let Some(s) = sub {
                    nodes_to_fallback(s, Some(-25000), out);
                }
                if let Some(s) = sup {
                    nodes_to_fallback(s, Some(30000), out);
                }
                if !body.is_empty() {
                    out.push(MathSegment {
                        text: " ".into(),
                        baseline: current_baseline,
                    });
                    nodes_to_fallback(body, current_baseline, out);
                }
            }
            MathNode::Matrix { rows } => {
                out.push(MathSegment {
                    text: "[".into(),
                    baseline: current_baseline,
                });
                for (r_idx, row) in rows.iter().enumerate() {
                    if r_idx > 0 {
                        out.push(MathSegment {
                            text: "; ".into(),
                            baseline: current_baseline,
                        });
                    }
                    for (c_idx, col) in row.iter().enumerate() {
                        if c_idx > 0 {
                            out.push(MathSegment {
                                text: ", ".into(),
                                baseline: current_baseline,
                            });
                        }
                        nodes_to_fallback(col, current_baseline, out);
                    }
                }
                out.push(MathSegment {
                    text: "]".into(),
                    baseline: current_baseline,
                });
            }
            MathNode::Group(inner) => nodes_to_fallback(inner, current_baseline, out),
            MathNode::Aligned { rows, .. } => {
                for (r_idx, row) in rows.iter().enumerate() {
                    if r_idx > 0 {
                        out.push(MathSegment {
                            text: "; ".into(),
                            baseline: current_baseline,
                        });
                    }
                    for cell in row {
                        nodes_to_fallback(cell, current_baseline, out);
                    }
                }
            }
            MathNode::Accent { chr, body } => {
                nodes_to_fallback(body, current_baseline, out);
                out.push(MathSegment {
                    text: chr.clone(),
                    baseline: current_baseline,
                });
            }
        }
    }
}

fn coalesce_segments(segs: Vec<MathSegment>) -> Vec<MathSegment> {
    let mut out: Vec<MathSegment> = Vec::new();
    for seg in segs {
        if seg.text.is_empty() {
            continue;
        }
        if let Some(last) = out.last_mut() {
            if last.baseline == seg.baseline {
                last.text.push_str(&seg.text);
                continue;
            }
        }
        out.push(seg);
    }
    out
}

// ---------------------------------------------------------------------------
// OMML to LaTeX Decompiler (for import.rs)
// ---------------------------------------------------------------------------

/// Convert an OMML `<m:oMath>` node back into LaTeX markup.
pub fn omml_to_latex(omath: &XmlEl) -> String {
    let mut s = String::new();
    decompile_children(&mut s, &omath.children);
    s
}

fn decompile_children(out: &mut String, children: &[xmltree::XMLNode]) {
    for node in children.iter().filter_map(|n| n.as_element()) {
        decompile_element(out, node);
    }
}

fn symbol_to_latex(c: char) -> Option<&'static str> {
    Some(match c {
        '≤' => "\\le ",
        '≥' => "\\ge ",
        '≠' => "\\neq ",
        '≈' => "\\approx ",
        '≡' => "\\equiv ",
        '∈' => "\\in ",
        '∉' => "\\notin ",
        '⊂' => "\\subset ",
        '⊆' => "\\subseteq ",
        '⊃' => "\\supset ",
        '⊇' => "\\supseteq ",
        '∪' => "\\cup ",
        '∩' => "\\cap ",
        '×' => "\\times ",
        '·' => "\\cdot ",
        '±' => "\\pm ",
        '∓' => "\\mp ",
        '÷' => "\\div ",
        '∞' => "\\infty ",
        '∂' => "\\partial ",
        '∇' => "\\nabla ",
        '∀' => "\\forall ",
        '∃' => "\\exists ",
        '→' => "\\to ",
        '←' => "\\leftarrow ",
        '⇒' => "\\Rightarrow ",
        '⇐' => "\\Leftarrow ",
        'α' => "\\alpha ",
        'β' => "\\beta ",
        'γ' => "\\gamma ",
        'δ' => "\\delta ",
        'ε' => "\\epsilon ",
        'ζ' => "\\zeta ",
        'η' => "\\eta ",
        'θ' => "\\theta ",
        'ϑ' => "\\vartheta ",
        'ι' => "\\iota ",
        'κ' => "\\kappa ",
        'λ' => "\\lambda ",
        'μ' => "\\mu ",
        'ν' => "\\nu ",
        'ξ' => "\\xi ",
        'π' => "\\pi ",
        'ϖ' => "\\varpi ",
        'ρ' => "\\rho ",
        'ϱ' => "\\varrho ",
        'σ' => "\\sigma ",
        'ς' => "\\varsigma ",
        'τ' => "\\tau ",
        'υ' => "\\upsilon ",
        'φ' => "\\phi ",
        'ϕ' => "\\varphi ",
        'χ' => "\\chi ",
        'ψ' => "\\psi ",
        'ω' => "\\omega ",
        'Γ' => "\\Gamma ",
        'Δ' => "\\Delta ",
        'Θ' => "\\Theta ",
        'Λ' => "\\Lambda ",
        'Ξ' => "\\Xi ",
        'Π' => "\\Pi ",
        'Σ' => "\\Sigma ",
        'Υ' => "\\Upsilon ",
        'Φ' => "\\Phi ",
        'Ψ' => "\\Psi ",
        'Ω' => "\\Omega ",
        'ℝ' => "\\mathbb{R}",
        'ℕ' => "\\mathbb{N}",
        'ℤ' => "\\mathbb{Z}",
        'ℚ' => "\\mathbb{Q}",
        'ℂ' => "\\mathbb{C}",
        'ℍ' => "\\mathbb{H}",
        'ℙ' => "\\mathbb{P}",
        _ => return None,
    })
}

fn first_child<'a>(el: &'a XmlEl, name: &str) -> Option<&'a XmlEl> {
    el.children
        .iter()
        .filter_map(|n| n.as_element())
        .find(|c| c.name == name)
}

fn decompile_element(out: &mut String, el: &XmlEl) {
    match el.name.as_str() {
        "f" => {
            let num = first_child(el, "num");
            let den = first_child(el, "den");
            out.push_str("\\frac{");
            if let Some(n) = num {
                decompile_children(out, &n.children);
            }
            out.push_str("}{");
            if let Some(d) = den {
                decompile_children(out, &d.children);
            }
            out.push('}');
        }
        "rad" => {
            let deg = first_child(el, "deg");
            let e = first_child(el, "e");
            let hide = first_child(el, "radPr")
                .and_then(|p| first_child(p, "degHide"))
                .and_then(|h| h.attributes.get("val"))
                .map(String::as_str);

            if let Some(deg_el) = deg.filter(|d| hide != Some("1") && !d.children.is_empty()) {
                out.push_str("\\sqrt[");
                decompile_children(out, &deg_el.children);
                out.push_str("]{");
            } else {
                out.push_str("\\sqrt{");
            }
            if let Some(body) = e {
                decompile_children(out, &body.children);
            }
            out.push('}');
        }
        "sSup" => {
            let e = first_child(el, "e");
            let sup = first_child(el, "sup");
            if let Some(base) = e {
                decompile_children(out, &base.children);
            }
            out.push_str("^{");
            if let Some(s) = sup {
                decompile_children(out, &s.children);
            }
            out.push('}');
        }
        "sSub" => {
            let e = first_child(el, "e");
            let sub = first_child(el, "sub");
            if let Some(base) = e {
                decompile_children(out, &base.children);
            }
            out.push_str("_{");
            if let Some(s) = sub {
                decompile_children(out, &s.children);
            }
            out.push('}');
        }
        "sSubSup" => {
            let e = first_child(el, "e");
            let sub = first_child(el, "sub");
            let sup = first_child(el, "sup");
            if let Some(base) = e {
                decompile_children(out, &base.children);
            }
            out.push_str("_{");
            if let Some(s) = sub {
                decompile_children(out, &s.children);
            }
            out.push_str("}^{");
            if let Some(s) = sup {
                decompile_children(out, &s.children);
            }
            out.push('}');
        }
        "nary" => {
            let chr = first_child(el, "naryPr")
                .and_then(|p| first_child(p, "chr"))
                .and_then(|c| c.attributes.get("val"))
                .map(String::as_str)
                .unwrap_or("∑");

            let op_name = match chr {
                "∑" => "\\sum",
                "∏" => "\\prod",
                "∫" => "\\int",
                "∮" => "\\oint",
                "lim" => "\\lim",
                other => other,
            };
            out.push_str(op_name);

            let sub_hide = first_child(el, "naryPr")
                .and_then(|p| first_child(p, "subHide"))
                .and_then(|h| h.attributes.get("val"))
                .map(String::as_str)
                == Some("1");
            let sup_hide = first_child(el, "naryPr")
                .and_then(|p| first_child(p, "supHide"))
                .and_then(|h| h.attributes.get("val"))
                .map(String::as_str)
                == Some("1");

            let sub = first_child(el, "sub").filter(|s| !sub_hide && !s.children.is_empty());
            let sup = first_child(el, "sup").filter(|s| !sup_hide && !s.children.is_empty());

            if let Some(s) = sub {
                out.push_str("_{");
                decompile_children(out, &s.children);
                out.push('}');
            }
            if let Some(s) = sup {
                out.push_str("^{");
                decompile_children(out, &s.children);
                out.push('}');
            }
            if let Some(e) = first_child(el, "e").filter(|e| !e.children.is_empty()) {
                out.push(' ');
                decompile_children(out, &e.children);
            }
        }
        "d" => {
            let beg = first_child(el, "dPr")
                .and_then(|p| first_child(p, "begChr"))
                .and_then(|c| c.attributes.get("val"))
                .map(String::as_str)
                .unwrap_or("(");
            let end = first_child(el, "dPr")
                .and_then(|p| first_child(p, "endChr"))
                .and_then(|c| c.attributes.get("val"))
                .map(String::as_str)
                .unwrap_or(")");

            let e = first_child(el, "e");
            let matrix = e.and_then(|body| first_child(body, "m"));
            if let Some(m) = matrix {
                let env = match (beg, end) {
                    ("(", ")") => "pmatrix",
                    ("[", "]") => "bmatrix",
                    ("|", "|") => "vmatrix",
                    ("{", "" | ".") => "cases",
                    _ => "",
                };
                if !env.is_empty() {
                    let _ = write!(out, "\\begin{{{env}}}");
                    decompile_matrix_rows(out, m);
                    let _ = write!(out, "\\end{{{env}}}");
                    return;
                }
            }

            let beg_str = match beg {
                "{" => "\\{",
                "}" => "\\}",
                "" => ".",
                "⟨" => "\\langle ",
                "⟩" => "\\rangle ",
                other => other,
            };
            let end_str = match end {
                "{" => "\\{",
                "}" => "\\}",
                "" => ".",
                "⟨" => "\\langle ",
                "⟩" => "\\rangle ",
                other => other,
            };
            let _ = write!(out, "\\left{beg_str}");
            if let Some(body) = e {
                decompile_children(out, &body.children);
            }
            let _ = write!(out, "\\right{end_str}");
        }
        "m" => {
            out.push_str("\\begin{matrix}");
            decompile_matrix_rows(out, el);
            out.push_str("\\end{matrix}");
        }
        "eqArr" => {
            // One <m:e> per line; alignment points are the `&` runs inside them.
            out.push_str("\\begin{aligned}");
            let rows: Vec<&XmlEl> = el
                .children
                .iter()
                .filter_map(|n| n.as_element())
                .filter(|c| c.name == "e")
                .collect();
            for (idx, row) in rows.iter().enumerate() {
                if idx > 0 {
                    out.push_str(" \\\\ ");
                }
                decompile_children(out, &row.children);
            }
            out.push_str("\\end{aligned}");
        }
        "acc" => {
            let chr = first_child(el, "accPr")
                .and_then(|p| first_child(p, "chr"))
                .and_then(|c| c.attributes.get("val"))
                .map(String::as_str)
                .unwrap_or("¯");
            let acc_cmd = match chr {
                "¯" => "\\overline",
                "̂" => "\\hat",
                "⃗" => "\\vec",
                "̃" => "\\tilde",
                "˙" => "\\dot",
                "¨" => "\\ddot",
                _ => "\\bar",
            };
            out.push_str(acc_cmd);
            out.push('{');
            if let Some(e) = first_child(el, "e") {
                decompile_children(out, &e.children);
            }
            out.push('}');
        }
        "r" => {
            let is_upright = first_child(el, "rPr")
                .and_then(|p| first_child(p, "sty"))
                .and_then(|s| s.attributes.get("val"))
                .map(String::as_str)
                == Some("p");

            let t = first_child(el, "t")
                .and_then(|t| t.get_text())
                .unwrap_or_default();

            if is_upright {
                if is_upright_func(&t) {
                    let _ = write!(out, "\\{t} ");
                } else {
                    let _ = write!(out, "\\text{{{t}}}");
                }
            } else {
                for c in t.chars() {
                    if let Some(sym) = symbol_to_latex(c) {
                        out.push_str(sym);
                    } else {
                        out.push(c);
                    }
                }
            }
        }
        _ => {
            decompile_children(out, &el.children);
        }
    }
}

fn decompile_matrix_rows(out: &mut String, m: &XmlEl) {
    let rows: Vec<&XmlEl> = m
        .children
        .iter()
        .filter_map(|n| n.as_element())
        .filter(|c| c.name == "mr")
        .collect();

    for (r_idx, row) in rows.iter().enumerate() {
        if r_idx > 0 {
            out.push_str(" \\\\ ");
        }
        let cells: Vec<&XmlEl> = row
            .children
            .iter()
            .filter_map(|n| n.as_element())
            .filter(|c| c.name == "e")
            .collect();
        for (c_idx, cell) in cells.iter().enumerate() {
            if c_idx > 0 {
                out.push_str(" & ");
            }
            decompile_children(out, &cell.children);
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frac_structure() {
        let omml = latex_to_omml(r"\frac{a+b}{c+d}", false).unwrap();
        assert!(omml.contains("<m:f>"));
        assert!(omml.contains("<m:fPr><m:type m:val=\"bar\"/></m:fPr>"));
        assert!(omml.contains("<m:num>"));
        assert!(omml.contains("<m:den>"));
        assert!(omml.contains("<m:t>a+b</m:t>"));
        assert!(omml.contains("<m:t>c+d</m:t>"));
    }

    #[test]
    fn test_sqrt_structure() {
        let omml = latex_to_omml(r"\sqrt{x^2+y^2}", false).unwrap();
        assert!(omml.contains("<m:rad>"));
        assert!(omml.contains("<m:degHide m:val=\"1\"/>"));
        assert!(omml.contains("<m:sSup>"));

        let omml_deg = latex_to_omml(r"\sqrt[3]{x}", false).unwrap();
        assert!(omml_deg.contains("<m:degHide m:val=\"0\"/>"));
        assert!(omml_deg.contains("<m:deg>"));
        assert!(omml_deg.contains("<m:t>3</m:t>"));
    }

    #[test]
    fn test_lim_becomes_function_not_invalid_nary_char() {
        let omml = latex_to_omml(r"\lim_{N \to \infty} \frac{1}{N}", true).unwrap();
        assert!(omml.contains("<m:func>"), "{omml}");
        assert!(omml.contains("<m:limLow>"), "{omml}");
        assert!(omml.contains("<m:t>lim</m:t>"), "{omml}");
        assert!(
            !omml.contains("m:val=\"lim\""),
            "lim must not be emitted as a nary character: {omml}"
        );
    }

    #[test]
    fn test_nary_sum_structure() {
        let omml = latex_to_omml(r"\sum_{i=1}^{n} i^2", true).unwrap();
        assert!(omml.contains("<a14:m><m:oMathPara>"));
        assert!(omml.contains("<m:nary>"));
        assert!(omml.contains("<m:chr m:val=\"∑\"/>"));
        assert!(omml.contains("<m:limLoc m:val=\"undOvr\"/>"));
        assert!(omml.contains("<m:sub>"));
        assert!(omml.contains("<m:sup>"));
        assert!(omml.contains("<m:e>"));
    }

    #[test]
    fn test_unsupported_command_error() {
        let err = latex_to_omml(r"\foo{x}", false).unwrap_err();
        match err {
            MathError::UnsupportedCommand(cmd) => {
                assert_eq!(cmd, r"\foo");
            }
            _ => panic!("expected UnsupportedCommand error"),
        }

        let problems = latex_problems(r"\frac{a}{b} + \foo{x} + \bar{y} + \baz");
        assert_eq!(
            problems,
            vec![
                "unsupported LaTeX command `\\foo`".to_string(),
                "unsupported LaTeX command `\\baz`".to_string()
            ]
        );
        assert!(latex_problems(r"\newcommand{\R}{\mathbb{R}} x \in \R").is_empty());
    }

    #[test]
    fn test_xml_escaping_in_text() {
        let omml = latex_to_omml(r"\text{a<b&c}", false).unwrap();
        assert!(omml.contains("a&lt;b&amp;c"));
        assert!(!omml.contains("<m:t>a<b&c</m:t>"));
    }

    #[test]
    fn test_matrix_and_delimiters() {
        let omml = latex_to_omml(r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}", false).unwrap();
        assert!(omml.contains("<m:d>"));
        assert!(omml.contains("<m:begChr m:val=\"(\"/>"));
        assert!(omml.contains("<m:endChr m:val=\")\"/>"));
        assert!(omml.contains("<m:m>"));
        assert!(omml.contains("<m:mr>"));
    }

    #[test]
    fn test_accents() {
        let omml = latex_to_omml(r"\vec{v} + \bar{x}", false).unwrap();
        assert!(omml.contains("<m:acc>"));
        assert!(omml.contains("<m:chr m:val=\"⃗\"/>"));
        assert!(omml.contains("<m:chr m:val=\"¯\"/>"));
    }

    #[test]
    fn test_roundtrip_decompile() {
        let cases = [
            r"\frac{a+b}{c+d}",
            r"\sqrt{x^{2}+y^{2}}",
            r"\sum_{i=1}^{n} i^{2}",
        ];
        for latex in cases {
            let omml = latex_to_omml(latex, false).unwrap();
            let wrapped = format!(
                "<root xmlns:a14=\"http://schemas.microsoft.com/office/drawing/2010/main\" xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">{omml}</root>"
            );
            let parsed = XmlEl::parse(wrapped.as_bytes()).unwrap();
            let m_el = first_child(&parsed, "m").unwrap();
            let omath = first_child(m_el, "oMath").unwrap();
            let decompiled = omml_to_latex(omath);
            // Recompiling the decompiled LaTeX should produce the exact same OMML
            let re_omml = latex_to_omml(&decompiled, false).unwrap();
            assert_eq!(omml, re_omml, "roundtrip failed for: {latex}");
        }
    }

    #[test]
    fn test_fallback_segments_fractions_and_radicals() {
        // Compound fraction: parentheses
        let segs = latex_to_fallback_segments(r"\frac{a+b}{c+d}").unwrap();
        let flat: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(flat, "(a+b)/(c+d)");

        // Simple fraction: no parentheses
        let segs = latex_to_fallback_segments(r"\frac{x}{y}").unwrap();
        let flat: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(flat, "x/y");

        // Compound radical: parentheses around radicand
        let segs = latex_to_fallback_segments(r"\sqrt{x^2+y^2}").unwrap();
        let flat: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(flat, "√(x2+y2)");
        // Check superscripts have baseline Some(30000)
        assert!(segs.iter().any(|s| s.text == "2" && s.baseline == Some(30000)));

        // Simple radical: no parentheses
        let segs = latex_to_fallback_segments(r"\sqrt{x}").unwrap();
        let flat: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(flat, "√x");

        // Radical with degree
        let segs = latex_to_fallback_segments(r"\sqrt[3]{x}").unwrap();
        let flat: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(flat, "√[3]x");

        // Summation with limits
        let segs = latex_to_fallback_segments(r"\sum_{i=1}^{n} i^2").unwrap();
        let flat: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(flat, "∑i=1n i2");
        assert!(segs.iter().any(|s| s.text == "i=1" && s.baseline == Some(-25000)));
        assert!(segs.iter().any(|s| s.text == "n" && s.baseline == Some(30000)));
    }

    #[test]
    fn test_group_preserves_structure() {
        let omml = latex_to_omml(r"{\frac{a}{b}} + c", false).unwrap();
        assert!(omml.contains("<m:f>"), "{omml}");
        assert!(!omml.contains("<m:t>a/b"), "{omml}");

        let nested = latex_to_omml(r"{x + \frac{a}{b}}", false).unwrap();
        assert!(nested.contains("<m:f>"), "{nested}");
        assert!(nested.contains("<m:t>x+</m:t>"), "{nested}");
    }

    #[test]
    fn test_nary_takes_one_operand() {
        let omml = latex_to_omml(r"\sum_{i=1}^{n} A + \sum_{j=1}^{m} B", true).unwrap();
        let first_close = omml.find("</m:nary>").expect("first sum");
        let second_open = omml[first_close..]
            .find("<m:nary>")
            .map(|i| i + first_close)
            .expect("second sum");
        assert!(second_open > first_close, "{omml}");
        assert!(omml[..first_close].contains("<m:t>A</m:t>"), "{omml}");
        assert!(!omml[..first_close].contains('B'), "{omml}");
    }

    #[test]
    fn test_align_environment() {
        let omml = latex_to_omml(r"\begin{align} a &= b \\ c &= d \end{align}", true).unwrap();
        assert!(omml.contains("<m:oMathPara>"), "{omml}");
        assert!(omml.contains("<m:eqArr>"), "{omml}");
        assert!(omml.contains("<m:baseJc m:val=\"center\"/>"), "{omml}");
        assert_eq!(omml.matches("<m:e>").count(), 2, "{omml}");
        assert!(omml.contains("<m:t>&amp;</m:t>"), "alignment point: {omml}");

        // `align` is a display environment and cannot be mixed into a line of text.
        let err = latex_to_omml(r"\begin{align} a &= b \end{align}", false).unwrap_err();
        assert!(err.to_string().contains("display"), "{err}");
    }

    #[test]
    fn test_aligned_environment_is_inline_capable() {
        let omml = latex_to_omml(r"\begin{aligned} a &= b \\ c &= d \end{aligned}", false).unwrap();
        assert!(omml.contains("<m:eqArr>"), "{omml}");
        assert!(!omml.contains("<m:oMathPara>"), "{omml}");
    }

    #[test]
    fn test_gather_has_no_alignment_points() {
        let omml = latex_to_omml(r"\begin{gather} a = b \\ c = d \end{gather}", true).unwrap();
        assert_eq!(omml.matches("<m:e>").count(), 2, "{omml}");
        assert!(!omml.contains("<m:t>&amp;</m:t>"), "{omml}");
    }

    #[test]
    fn test_eqnarray_alignment_points() {
        let omml = latex_to_omml(r"\begin{eqnarray} a &=& b \\ c &=& d \end{eqnarray}", true).unwrap();
        assert_eq!(omml.matches("<m:t>&amp;</m:t>").count(), 4, "{omml}");
    }

    #[test]
    fn test_macros_are_expanded_before_parsing() {
        let omml = latex_to_omml(
            r"\newcommand{\R}{\mathbb{R}}\def\half{\frac{1}{2}} \R^n + \half",
            true,
        )
        .unwrap();
        assert!(!omml.contains("newcommand"), "{omml}");
        assert!(omml.contains("ℝ"), "{omml}");
        assert!(omml.contains("<m:f>"), "{omml}");
        assert!(latex_problems(r"\newcommand{\R}{\mathbb{R}} \R").is_empty());

        let fallback = latex_to_fallback_segments(r"\newcommand{\half}[1]{\frac{#1}{2}} \half{x}")
            .expect("macros expand in fallback text too");
        let flat: String = fallback.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(flat, "x/2");
    }

    #[test]
    fn test_eqarr_roundtrip() {
        let latex = r"\begin{aligned} a &= b \\ c &= d \end{aligned}";
        let omml = latex_to_omml(latex, false).unwrap();
        let wrapped = format!(
            "<root xmlns:a14=\"http://schemas.microsoft.com/office/drawing/2010/main\" xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">{omml}</root>"
        );
        let parsed = XmlEl::parse(wrapped.as_bytes()).unwrap();
        let m_el = first_child(&parsed, "m").unwrap();
        let omath = first_child(m_el, "oMath").unwrap();
        let decompiled = omml_to_latex(omath);
        assert!(decompiled.contains("\\begin{aligned}"), "{decompiled}");
        assert!(decompiled.contains('&'), "{decompiled}");
        let re_omml = latex_to_omml(&decompiled, false).unwrap();
        assert_eq!(omml, re_omml, "roundtrip failed for: {latex}");
    }

    #[test]
    fn test_aligned_fallback_text() {
        let segs = latex_to_fallback_segments(r"\begin{aligned} a &= b \\ c &= d \end{aligned}")
            .expect("aligned is valid inline");
        let flat: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(flat, "a=b; c=d");
    }
}
