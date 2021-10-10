use std::io::{self};
use std::io::Write as IoWrite;
use std::fmt;
use std::str::Chars;
use std::iter::Peekable;
use MathToken::*;
use MathError::*;


fn main() {
    loop {
        print!(">> ");
        io::stdout().flush().expect("Cannot flush stdin?");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("failed to read stdin.");
        if let "exit" = &input.trim()[..] {
            println!("Goodbye.");
            return;
        }
        let result = Tokens::handle(input);
        match result {
            Ok(float) => println!("{:.3}", float),
            Err(error) => println!("{}", error),
        }
    }
}

#[derive(Copy, Clone)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    ParOpen,
    ParClose,
}
impl Op {
    fn from_char(ch: char) -> Self {
        match ch {
            '+' => Op::Add,
            '-' => Op::Sub,
            '*' => Op::Mul,
            '/' => Op::Div,
            '(' => Op::ParOpen,
            ')' => Op::ParClose,
            _ => unreachable!()
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            Op::ParOpen | Op::ParClose => 0,
            Op::Add | Op::Sub => 1,
            Op::Mul | Op::Div => 2
        }
    }

    fn call(&self, x: f64, y: f64) -> f64 {
        match self {
            Op::Add => x + y,
            Op::Sub => x - y,
            Op::Mul => x * y,
            Op::Div => x / y,
            _ => unreachable!()
        }
    }
}
impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Op::Add => '+',
            Op::Sub => '-',
            Op::Mul => '*',
            Op::Div => '/',
            Op::ParOpen => '(',
            Op::ParClose => ')',
        })
    }
}


#[derive(Copy, Clone)]
enum MathToken {
    Num(f64),
    Oper(Op)
}
impl fmt::Display for MathToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Num(float) => write!(f, "Num({:.3})", float),
            Oper(oper) => write!(f, "Op({})", oper),
        }
    }
}


struct Tokens(Vec<MathToken>);
impl Tokens {
    fn parse(input: &str) -> Result<Tokens, MathError> {
        let mut chars = input.chars().peekable();
        let mut tokens = Vec::new();

        loop {
            match chars.peek() {
                Some('0'..='9' | '.') => tokens.push(Num(Tokens::parse_num(&mut chars)?)),
                Some('+'|'-'|'*'|'/'|'('|')') => tokens.push(Oper(Op::from_char(chars.next().unwrap()))),
                Some(chr @ '=') | Some(chr) if chr.is_whitespace() => { chars.next().unwrap(); },
                Some(_) => return Err(UnsupportedCharError(chars.next().unwrap())),
                None => return Ok(Tokens(tokens))
            }
        }
    }

    fn parse_num(input: &mut Peekable<Chars>) -> Result<f64, MathError> {
        let mut buf = String::new();

        while matches!(input.peek(), Some('0'..='9' | '.')) {
            buf.push(input.next().unwrap());
        }
        return match buf.parse::<f64>() {
            Ok(float) => Ok(float),
            Err(_) => Err(ParseNumError(buf))
        }
    }

    fn shunting(self) -> Result<Tokens, MathError> {
        let mut op_stack: Vec<Op> = Vec::new();
        let mut out_queue: Vec<MathToken> = Vec::new();

        for token in &self.0 {
            match token {
                Num(_) => out_queue.push(*token),
                Oper(op @ Op::ParOpen) => op_stack.push(*op),
                Oper(Op::ParClose) => loop {
                    if op_stack.len() == 0 {
                        return Err(MissingParens(self));
                    } else if matches!(op_stack.last(), Some(Op::ParOpen)) {
                        op_stack.pop().unwrap();
                        break;
                    } else {
                        out_queue.push(Oper(op_stack.pop().unwrap()));
                    }
                }
                Oper(oper) => {
                    while let Some(&prev) = op_stack.last() {
                        if oper.precedence() < prev.precedence() {
                            out_queue.push(Oper(op_stack.pop().unwrap()));
                        } else {
                            break;
                        }
                    }
                    op_stack.push(*oper);
                }
            }
        }
        while let Some(oper) = op_stack.pop() {
            out_queue.push(Oper(oper));
        }
        Ok(Tokens(out_queue))
    }

    fn solve(self) -> Result<f64, MathError> {
        let mut stack: Vec<MathToken> = Vec::new();

        for token in &self.0 {
            match token {
                Num(_) => stack.push(*token),
                Oper(oper) => {
                    if stack.len() < 2 {
                        return Err(BadTokens("Not enough tokens to pop from stack.", self));
                    }
                    if let (Num(y), Num(x)) = (stack.pop().unwrap(), stack.pop().unwrap()) {
                        stack.push(Num(oper.call(x, y)));
                    } else { return Err(BadTokens("unreachable? operators were put on the queue.", self)); }
                }
            }
        }
        if stack.len() == 1 {
            if let Some(Num(float)) = stack.pop() {
                return Ok(float);
            } else {
                return Err(BadTokens("unreachable? operators were put on the queue while emptying operator stack.", self));
            }
        } else {
            return Err(BadTokens("Too many tokens left on stack.", self));
        }
    }

    fn handle(input: String) -> Result<f64, MathError> {
        Tokens::parse(&input.trim())?.shunting()?.solve()
    }
}
impl fmt::Display for Tokens {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut tokens = self.0.iter();
        if let Some(token) = tokens.next() {
            match token {
                Num(float) => write!(f, "{{{}", float)?,
                Oper(oper) => write!(f, "{}", oper)?,
            }
        }
        for token in tokens {
            match token {
                Num(float) => write!(f, ", {}", float)?,
                Oper(oper) => write!(f, ", {}", oper)?,
            }
        }
        write!(f, "{}", "}")
    }
}


enum MathError {
    ParseNumError(String),
    UnsupportedCharError(char),
    BadTokens(&'static str, Tokens),
    MissingParens(Tokens),
}
impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseNumError(string) => write!(f, "ParseNumError: `{}`", string),
            UnsupportedCharError(chr) => write!(f, "UnsupportedCharError: `{}`", chr),
            BadTokens(string, tokens) => write!(f, "BadTokens: {} ({})", tokens, string),
            MissingParens(tokens) => write!(f, "MissingParens: {}", tokens),
        }
    }
}
