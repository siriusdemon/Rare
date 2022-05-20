// The S-expression parser
// 
// There are only two type in a S-expr, atom and list.
// atom can be: 
//      number  1 2 3 4 6 #b001 #o001 #x112 1.34 4.56 14.5
//      char    \c \space \n \newline \\
//      str     "hahah\n"           default format string 
//      vector  #(1 2 3) #((1 2 3) (1 2 3))
//      bool    true false
//      array   [1 2 3 4]
//      dict    {1: 2, 2: 3}    comma is optional for readable.


// list can be:
//      list    '(1 2 3 4)


// separator for manda
// () {} [] - list array dict
// '        - quote
// `        - quasiquote
// #        - complex data
// \        - char

// symbol terminal when parsing file
// ;        - comment
// ' '      - space
// \n       - newline


// Type of number
// u8, u16, u32, u64, u128
// i8, i16, i32, i64, i128
// f16, f32, f64, f128
//
// syntax
// normal: 123  343 1000,000,000 1.343
// science: 1e112, 2.343242E123213
// sign: -123 -3.14  1e-123 -1.3232e-232

use std::fmt;


fn seqs_to_string<E: fmt::Display>(seqs: impl Iterator<Item=E>, join: &str) -> String {
    let seqs: Vec<String> = seqs.map(|e| format!("{}", e)).collect();
    let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
    let seqs_s = seqs_ref.join(join);
    return seqs_s;
}

pub enum Expr {
    SInt { value: String, line: usize, col: usize}, 
    UInt { value: String, line: usize, col: usize},
    Float { value: String, line: usize, col: usize},
    Char { value: char, line: usize, col: usize},
    String { value: String, line: usize, col: usize},
    Symbol { value: String, line: usize, col: usize},
    List { value: Vec<Expr>, line: usize, col: usize},
    Vector { value: Vec<Expr>, line: usize, col: usize},
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::*;
        match self {
            UInt { value, line: _ , col: _ } => write!(f, "{}", value),
            SInt { value, line: _ , col: _ } => write!(f, "{}", value),
            Float { value, line: _ , col: _ } => write!(f, "{}", value),
            Char { value, line: _, col: _ } => write!(f, "\\{}", value),
            String { value, line: _, col: _ } => write!(f, "\"{}\"", value),
            Symbol { value, line: _, col: _ } => write!(f, "{}", value),
            List { value, line: _, col: _ } => write!(f, "({})", seqs_to_string(value.iter(), " ")),
            Vector { value, line: _, col: _ } => write!(f, "#({})", seqs_to_string(value.iter(), " ")),
        }
    }
}


// String Scanner -> Token Stream
pub struct Scanner {
    expr: Vec<char>,
}

fn is_valid_sym_char(c: char) -> bool {
    let number = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    let alphabet = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 
        'h', 'i', 'j', 'k', 'l', 'm', 'n', 
        'o', 'p', 'r', 's', 't', 'u', 'v',
        'w', 'x', 'y', 'z',
        'A', 'B', 'C', 'D', 'E', 'F', 'G',
        'H', 'I', 'J', 'K', 'L', 'M', 'N',
        'O', 'P', 'R', 'S', 'T', 'U', 'V',
        'W', 'X', 'Y', 'Z',
    ];
    let sign = ['+', '-', '*', '/', '=', '%', '!', '?', '>', '<', '^', '$', '_', '.'];
    number.contains(&c) || alphabet.contains(&c) || sign.contains(&c)
}

// symbol terminal
// character that can follow a symbol, but should not be treated as a part of the symbol.
// e.g. (a b c)
// 'c' is followed by ')' but ')' is not part of it.
fn is_sym_terminal(c: char) -> bool {
    let terminal = [
        '(', ')', '[', ']', '{', '}', 
        ';', '\n', ' ', ':',
    ];
    return terminal.contains(&c);
}

impl Scanner {
    pub fn new(expr: &str) -> Self {
        let expr: Vec<char> = expr.chars().collect();
        Self { expr }
    }

    pub fn scan(&self) -> Vec<Expr> {
        let mut tokens = vec![];
        let mut i = 0;
        let mut line = 1;
        let mut col = 1;
        while i < self.expr.len() {
            i = self.scan_expr(i, &mut line, &mut col, &mut tokens);
        }
        return tokens;
    }

    pub fn scan_expr(&self, mut i: usize, line: &mut usize, col: &mut usize, tokens: &mut Vec<Expr>) -> usize {
        let c = self.expr[i];
        match c {
            ' ' => {
                *col = *col + 1;
                i + 1   // skip ' '
            }
            '\n' => {
                *line = *line + 1;
                *col = 0;
                i + 1   // skip \n
            }
            ';' => {
                i += 1;
                while i < self.expr.len() && self.expr[i] != '\n' {
                    i += 1;
                }
                *line += 1;
                *col = 0;
                i + 1   // skip \n
            }
            '0' ..= '9' | '-' | '+' | '.' => self.scan_number(i, line, col, tokens),
            '#' => self.scan_hash(i, line, col, tokens),
            '(' => self.scan_list(i, line, col, tokens),
            '\\' => self.scan_char(i, line, col, tokens),
            '"' => self.scan_string(i, line, col, tokens),
            _e => self.scan_sym(i, line, col, tokens),
        }
    }

    fn scan_list(&self, mut i: usize, line: &mut usize, col: &mut usize, tokens: &mut Vec<Expr>) -> usize {
        i = i + 1;  // skip ( 
        let mut list = vec![];
        let oldline = *line;
        let oldcol = *col;
        while i < self.expr.len() && self.expr[i] != ')' {
            i = self.scan_expr(i, line, col, &mut list);
        }
        let tok = Expr::List {value: list, line: oldline, col: oldcol};
        tokens.push(tok);
        return i + 1; // skip )
    }

    fn scan_char(&self, mut i: usize, line: &mut usize, col: &mut usize, tokens: &mut Vec<Expr>) -> usize {
        let name_chars = ["newline", "space"];
        let mut sym = String::new();
        i = i + 1; // skip \
        while i < self.expr.len() && !is_sym_terminal(self.expr[i]) {
            sym.push(self.expr[i]);
            i = i + 1;
        }
        if sym.len() > 1 && !name_chars.contains(&sym.as_str()) {
            panic!("invalid symbol character at line {}, col {}", *line, *col);
        }

        let c: char = match sym.as_str() {
            "newline" => '\n',
            "space" => ' ',
            e => e.chars().collect::<Vec<char>>()[0],
        };

        let tok = Expr::Char {value: c, line: *line, col: *col};
        tokens.push(tok);
        *col = *col + sym.len() + 1;
        return i;
    }

    fn scan_sym(&self, mut i: usize, line: &mut usize, col: &mut usize, tokens: &mut Vec<Expr>) -> usize {
        let mut sym = String::new();
        while i < self.expr.len() && !is_sym_terminal(self.expr[i]) {
            if !is_valid_sym_char(self.expr[i]) {
                panic!("invalid symbol character at line {}, col {}", *line, *col);
            }
            sym.push(self.expr[i]);
            i = i + 1;
        }
        let newcol = *col + sym.len();
        let tok = Expr::Symbol {value: sym, line: *line, col: *col};
        tokens.push(tok);
        *col = newcol;
        return i;
    }

    // NOTE that string can not contain a \newline char. \n will be interpreted as \\ \n two chars.
    fn scan_string(&self, mut i: usize, line: &mut usize, col: &mut usize, tokens: &mut Vec<Expr>) -> usize {
        let mut sym = String::new();
        let oldcol = *col;
        let oldline = *line;
        i = i + 1;  // skip "
        *col += 1;
        while i < self.expr.len() && self.expr[i] != '"' {
            sym.push(self.expr[i]);
            if self.expr[i] == '\n' {
                *line += 1;
                *col = 0;
            } else {
                *col += 1;
            }
            i += 1;
        }
        if i == self.expr.len() {
            panic!("syntax error at line {}, col {}.", oldline, oldcol);
        }
        i += 1; // skip "
        *col += 1;
        let tok = Expr::String {value: sym, line: oldline, col: oldcol};
        tokens.push(tok);
        return i;
    }

    // hash type
    // uint: #xff #o777 #b111 
    // vector: #(1 2 3) #((1 23) (1 23))
    fn scan_hash(&self, mut i: usize, line: &mut usize, col: &mut usize, tokens: &mut Vec<Expr>) -> usize {
        let mut sym = String::new();
        i += 1;     // skip #
        match self.expr[i] {
            'x' | 'o' | 'b' => self.scan_hash_number(i, line, col, tokens),
            '(' => self.scan_hash_vector(i, line, col, tokens),
            _ => panic!("Invalid character after # at line {}, col {}", *line, *col),
        }
    }

    fn scan_hash_number(&self, mut i: usize, line: &mut usize, col: &mut usize, tokens: &mut Vec<Expr>) -> usize {
        let mut sym = String::new();
        let base = self.expr[i];
        i += 1; // skip base
        while i < self.expr.len() && !is_sym_terminal(self.expr[i]) {
            sym.push(self.expr[i]);
            i += 1;
        }
        let base2 = vec!['0', '1'];
        let base8 = vec!['0', '1', '2', '3', '4', '5', '6', '7'];
        let base16 = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
        let (valid_chars, base_n): (_, usize) = match base {'b' => (base2, 2), 'o' => (base8, 8), 'x' => (base16, 16), _e => unreachable!()};
        let mut sum: usize = 0; 
        for (j, c) in sym.chars().rev().enumerate() {
            match valid_chars.iter().position(|&x| x == c) {
                Some (idx) => sum = sum + base_n.wrapping_pow(j as u32) * idx,
                None => panic!("Invalid character when parse {} base number at line {} col {}", base_n, *line, *col),
            };
        }
        
        let e = Expr::UInt {value: sum.to_string(), line: *line, col: *col};
        tokens.push(e);
        *col += sym.len() + 2;  // # + base
        return i;
    }


    fn scan_hash_vector(&self, mut i: usize, line: &mut usize, col: &mut usize, tokens: &mut Vec<Expr>) -> usize {
        i = i + 1;  // skip ( 
        let mut list = vec![];
        let oldline = *line;
        let oldcol = *col;
        while i < self.expr.len() && self.expr[i] != ')' {
            i = self.scan_expr(i, line, col, &mut list);
        }
        let tok = Expr::Vector {value: list, line: oldline, col: oldcol};
        tokens.push(tok);
        return i + 1; // skip )
    }

    fn scan_number(&self, mut i: usize, line: &mut usize, col: &mut usize, tokens: &mut Vec<Expr>) -> usize {
        let mut sym = String::new();
        while i < self.expr.len() && !is_sym_terminal(self.expr[i]) {
            sym.push(self.expr[i]);
            i += 1;
        }

        let mut neg = false;
        let mut float = false;
        if !self.is_valid_number(&sym, &mut neg, &mut float) {
            panic!("Invalid number at line {} col {}", *line, *col);
        } 

        let oldcol = *col;
        *col += sym.len();
        let e = match (neg, float) {
            (false, false) => self.parse_pos_int(sym, *line, oldcol),
            (true, false)  => self.parse_neg_int(sym, *line, oldcol),
            (_, true)  => self.parse_float(sym, *line, oldcol),
        };
        tokens.push(e);
        return i;
    }

    /// table-base Finite-State-Machine
    // S0 {+/- -> S1, N -> S2, . -> S3}     -- init state
    // S1 { N  -> S2, . -> S3}              -- start with +/-
    // S2 {e/E -> S4, N -> S2, . -> S7}     -- number after +/-
    // S3 { N  -> S7}                       -- dot without number 
    // S4 {+/- -> S5, N -> S6}              -- e/E
    // S5 { N  -> S6}                       -- +/- after e/E
    // S6 { N  -> S6}                       -- number after (sign) e/E
    // S7 { N  -> S7, e/E -> S4}            -- number after dot
    //
    // Valid terminal states are $2 $6 $7
    fn is_valid_number(&self, numstr: &str, neg: &mut bool, float: &mut bool) -> bool {
        let mut state = 0;
        let valid_terminal = [2, 6, 7]; 
        const X: usize = 8;
        let transfers = [
            [1, 2, 3, X, X],
            [X, 2, 3, X, X],
            [X, 2, 7, 4, X],
            [X, 7, X, X, X],
            [5, 6, X, X, X],
            [X, 6, X, X, X],
            [X, 6, X, X, X],
            [X, 7, X, 4, X],
        ];
        for c in numstr.chars() {
            let cond = match c {
                '+' => 0,
                '-' => { if state == 0 { *neg = true; } 0 },
                '0' ..= '9' => 1,
                '.' => { *float = true; 2 },
                'e' | 'E' => { *float = true; 3 },
                _other => 4,
            };
            state = transfers[state][cond];
            if state == X {
                return false;
            }
        }
        return valid_terminal.contains(&state);
    }

    fn parse_float(&self, numstr: String, line: usize, col: usize) -> Expr {
        Expr::Float { value: numstr, line, col}
    }

    fn parse_pos_int(&self, numstr: String, line: usize, col: usize) -> Expr {
        Expr::UInt { value: numstr, line, col}
    }
    
    fn parse_neg_int(&self, numstr: String, line: usize, col: usize) -> Expr {
        Expr::SInt { value: numstr, line, col}
    }
}





#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scan_string() {
        let scanner = Scanner::new("\"hello world\"");
        let tokens = scanner.scan();
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            Expr::String { value, line: _, col: _} => assert_eq!(value, "hello world"),
            _e => assert!(false),
        };
    }

    #[test]
    fn scan_string2() {
        let scanner = Scanner::new("\"hello [?e1!e3e{world\"");
        let tokens = scanner.scan();
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            Expr::String { value, line: _, col: _} => assert_eq!(value, "hello [?e1!e3e{world"),
            _e => assert!(false),
        };
    }

    #[test]
    fn scan_char() {
        let scanner = Scanner::new(r"\a");
        let tokens = scanner.scan();
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            Expr::Char { value, line: _, col: _} => assert_eq!(value, &'a'),
            _e => assert!(false),
        };
    }

    #[test]
    fn scan_u32() {
        let scanner = Scanner::new("42");
        let tokens = scanner.scan();
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            Expr::UInt { value, line: _, col: _} => assert_eq!(value, "42"),
            _e => assert!(false),
        };
    }

    #[test]
    fn scan_u32_2() {
        let scanner = Scanner::new("-1");
        let tokens = scanner.scan();
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            Expr::SInt { value, line: _, col: _} => assert_eq!(value, "-1"),
            _e => assert!(false),
        };
    }

 
    #[test]
    fn scan_list() {
        let scanner = Scanner::new("(a b c (d e))");
        let tokens = scanner.scan();
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            Expr::List {value, line: _, col: _} => {
                match &value[0] {
                    Expr::Symbol {value: v, line: _, col: _} => {
                        assert_eq!(v.as_str(), "a");
                    }
                    _e => assert!(false),
                }
            }
            _e => assert!(false),
        }
    }
  
    #[test]
    fn scan_list2() {
        let scanner = Scanner::new("(a b c (d e))");
        let tokens = scanner.scan();
        let s = format!("{}", tokens[0]);
        assert_eq!(s.as_str(), "(a b c (d e))");
    }
    
    
    #[test]
    #[should_panic]
    fn scan_special1() {
        let scanner = Scanner::new("(a b c (d e))#");
        let tokens = scanner.scan();
    }

    // #[test]
    // #[should_panic]
    // fn scan_special2() {
    //     let scanner = Scanner::new("(a b c (d e))'");
    //     let tokens = scanner.scan();
    // }

    #[test]
    #[should_panic]
    fn scan_special3() {
        let scanner = Scanner::new("(a b c (d e))\\");
        let tokens = scanner.scan();
    }
 
    // #[test]
    // #[should_panic]
    // fn scan_special4() {
    //     let scanner = Scanner::new("(a b c (d e))`");
    //     let tokens = scanner.scan();
    // }
  
    #[test]
    fn scan_special5() {
        let scanner = Scanner::new("#(a f c)");
        let tokens = scanner.scan();
        let s = format!("{}", tokens[0]);
        assert_eq!(s.as_str(), "#(a f c)");
    }

    #[test]
    #[should_panic]
    fn scan_special6() {
        let scanner = Scanner::new("# (a f c)");
        let tokens = scanner.scan();
    }

    #[test]
    fn scan_number() {
        let scanner = Scanner::new("3.14");
        let tokens = scanner.scan();
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            Expr::Float {value, line: _, col: _} => assert_eq!(value, "3.14"),
            _e => assert!(false),
        }
    }

    #[test]
    fn scan_hash_number() {
        let scanner = Scanner::new("(#b10 #o17 #xff)");
        let tokens = scanner.scan();
        let s = format!("{}", tokens[0]);
        assert_eq!(s.as_str(), "(2 15 255)");
    }

    #[test]
    fn scan_vector() {
        let scanner = Scanner::new("#(1 2 3)");
        let tokens = scanner.scan();
        let s = format!("{}", tokens[0]);
        assert_eq!(s.as_str(), "#(1 2 3)");

        let scanner = Scanner::new("#(1 2 #(3))");
        let tokens = scanner.scan();
        let s = format!("{}", tokens[0]);
        assert_eq!(s.as_str(), "#(1 2 #(3))");
 
    }
}