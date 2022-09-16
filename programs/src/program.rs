use crate::progop::*;
use crate::op_tree::*;
use colored::*;
use itertools::Itertools;

/// Holds a single RPN program
#[derive(Eq, PartialEq)]
pub struct Program {
    instructions: Vec<ProgOp>
}

impl Program {

    /// Creates a new program
    pub fn new(num_cnt: usize) -> Self {
        Program {
            instructions: Vec::with_capacity(num_cnt + (num_cnt - 1))
        }
    }

    /// Adds an instruction to the program
    pub fn push(&mut self, op: ProgOp) {
        self.instructions.push(op);
    } 

    // Returns the numer of operators in the program
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Runs the program with a given set of numbers and preallocated stack
    pub fn run(&self, numbers: &[u32], stack: &mut Vec<u32>) -> Result<u32, ProgErr> {
        stack.clear();

        for op in &self.instructions {
            match op {
                ProgOp::Number(x) => stack.push(numbers[*x as usize]),
                ProgOp::OpAdd => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();

                    stack.push(n2 + n1);
                },
                ProgOp::OpSub => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();

                    if n2 < n1 {
                        return Err(ProgErr::Negative)
                    }

                    let int = n2 - n1;

                    if int == 0 {
                        return Err(ProgErr::Zero)
                    }

                    stack.push(int);
                },
                ProgOp::OpMul => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();

                    if n1 == 1 || n2 == 1 {
                        return Err(ProgErr::Mul1)
                    }

                    let int = n2 * n1;

                    if int == 0 {
                        return Err(ProgErr::Zero)
                    }

                    stack.push(int);
                },
                ProgOp::OpDiv => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();

                    if n1 == 0 {
                        return Err(ProgErr::DivZero)
                    }

                    if n1 == 1 {
                        return Err(ProgErr::Div1)
                    }

                    if n2 % n1 != 0 {
                        return Err(ProgErr::NonInteger)
                    }

                    stack.push(n2 / n1);
                },
            }
        }

        Ok(stack.pop().unwrap())
    }

    /// Returns false if the program contains a calculation which is commutative and would be covered by another program
    pub fn commutative_filter(&self) -> bool {
        let mut op_tree = self.op_tree();
        op_tree = op_tree.simplify();

        Program::commutative_filter_walk(&op_tree)
    }

    fn commutative_filter_walk(node: &OpTreeNode) -> bool {
        match node {
            OpTreeNode::Number(_) => {}
            OpTreeNode::Op(o) => {
                // Check the operator is commutative
                match o.op {
                    ProgOp::OpAdd | ProgOp::OpMul => {
                        // Add or multiply operator
                        match o.lhs {
                            OpTreeNode::Number(ln) => {
                                // Number on the left
                                match o.rhs {
                                    OpTreeNode::Number(rn) => {
                                        // Number on the right
                                        // Always go from smallest to biggest
                                        if ln > rn { return false; }
                                    }
                                    _ => ()
                                }
                            }
                            _ => ()
                        }
                    },
                    _ => ()
                }

                if !Program::commutative_filter_walk(&o.lhs) { return false; }
                if !Program::commutative_filter_walk(&o.rhs) { return false; }
            }
            OpTreeNode::Multi(o) => {
                // Check the operator is commutative
                match o.op {
                    ProgOp::OpAdd | ProgOp::OpMul => {
                        // Add or multiply operator
                        let mut cur_n = None;
                        let mut got_expr = false;

                        for t in &o.terms {
                            match t {
                                OpTreeNode::Number(n) => {
                                    if got_expr { return false; }
                                    if let Some(cur_n) = cur_n {
                                        // Always go from smallest to biggest
                                        if n < cur_n { return false; }
                                    }
                                    cur_n = Some(n);
                                }
                                OpTreeNode::Op(_) | OpTreeNode::Multi(_) => {
                                    got_expr = true
                                }
                            }
                        }
                    },
                    _ => ()
                }

            }
        }

        true
    }

    /// Returns the formatted steps of a program for a given set of numbers
    pub fn steps(&self, numbers: &[u32], colour: bool) -> Vec<String> {
        let mut result = Vec::new();
        let mut stack: Vec<u32> = Vec::with_capacity(numbers.len());
        let mut str_stack: Vec<String> = Vec::with_capacity(numbers.len());

        let oper = |str: &str| -> String {
            if colour { str.dimmed().to_string() }
            else { str.to_string() }
        };

        let card = |c: u32| -> String {
            if colour { c.to_string().on_blue().to_string() }
            else { c.to_string() }
        };

        for op in &self.instructions {
            match op {
                ProgOp::Number(x) => {
                    stack.push(numbers[*x as usize]);
                    str_stack.push(card(numbers[*x as usize]));
                },
                ProgOp::OpAdd => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    let n1_str = str_stack.pop().unwrap();
                    let n2_str = str_stack.pop().unwrap();
                    let ans = n2 + n1;
                    let ans_str = ans.to_string();
                    result.push(format!("{} {} {} {} {}", n2_str, oper("+"), n1_str, oper("="), ans_str));
                    stack.push(ans);
                    str_stack.push(ans_str);
                },
                ProgOp::OpSub => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    let n1_str = str_stack.pop().unwrap();
                    let n2_str = str_stack.pop().unwrap();
                    let ans = n2 - n1;
                    let ans_str = ans.to_string();
                    result.push(format!("{} {} {} {} {}", n2_str, oper("-"), n1_str, oper("="), ans_str));
                    stack.push(ans);
                    str_stack.push(ans_str);
                },
                ProgOp::OpMul => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    let n1_str = str_stack.pop().unwrap();
                    let n2_str = str_stack.pop().unwrap();
                    let ans = n2 * n1;
                    let ans_str = ans.to_string();
                    result.push(format!("{} {} {} {} {}", n2_str, oper("×"), n1_str, oper("="), ans_str));
                    stack.push(ans);
                    str_stack.push(ans_str);
                },
                ProgOp::OpDiv => {
                    let n1 = stack.pop().unwrap();
                    let n2 = stack.pop().unwrap();
                    let n1_str = str_stack.pop().unwrap();
                    let n2_str = str_stack.pop().unwrap();
                    let ans = n2 / n1;
                    let ans_str = ans.to_string();
                    result.push(format!("{} {} {} {} {}", n2_str, oper("/"), n1_str, oper("="), ans_str));
                    stack.push(ans);
                    str_stack.push(ans_str);
                },
            }
        }

        result
    }

    /// Build an operator tree for the program
    pub fn op_tree(&self) -> OpTreeNode {
        let mut stack: Vec<OpTreeNode> = Vec::new();

        for op in self.instructions.iter() {
            match *op {
                ProgOp::Number(n) => stack.push(OpTreeNode::new_number(n)),
                _ => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    stack.push(OpTreeNode::new_op(lhs, *op, rhs))
                }
            }
        }

        stack.pop().unwrap()
    }

    /// Converts the RPN program to infix equation
    pub fn equation(&self, numbers: &[u32], colour: bool) -> String {
        self.op_tree().colour(numbers, colour, false)
    }

    /// Converts the RPN program to a string for a given set of numbers
    pub fn rpn(&self, numbers: &[u32], colour: bool) -> String {
        let oper = |str: &str| -> String {
            if colour { str.dimmed().to_string() }
            else { str.to_string() }
        };

        let card = |c: u32| -> String {
            if colour { c.to_string().on_blue().to_string() }
            else { c.to_string() }
        };

        self.instructions.iter().map(|&i| {
            match i {
                ProgOp::Number(n) => card(numbers[n as usize]),
                ProgOp::OpAdd => oper("+"),
                ProgOp::OpSub => oper("-"),
                ProgOp::OpMul => oper("×"),
                ProgOp::OpDiv => oper("/"),
            }
        }).join(" ")
    }

}

/// Errors generated by RPN program run
#[derive(Debug, Eq, PartialEq)]
pub enum ProgErr {
    Zero,       // Program generated a zero intermediate result
    Negative,   // Program generated a negative intermediate result
    DivZero,    // Program encountered a division by zero
    NonInteger, // Program encountered a non-integer intermediate result
    Mul1,       // Program encountered multiply by 1 (noop)
    Div1,       // Program encountered divide by 1 (noop)
}
