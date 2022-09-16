use itertools::Itertools;
use crate::progop::*;

#[derive(Debug, Clone)]
pub enum OpTreeNode {
    Number(u8),
    Op(Box<OpTreeNodeOp>),
    Multi(Box<OpTreeNodeMulti>)
}

#[derive(Debug, Clone)]
pub struct OpTreeNodeOp {
    pub lhs: OpTreeNode,
    pub op: ProgOp,
    pub rhs: OpTreeNode
}

#[derive(Debug, Clone)]
pub struct OpTreeNodeMulti {
    pub op: ProgOp,
    pub terms: Vec<OpTreeNode>
}

impl OpTreeNode {

    pub fn new_number(n: u8) -> OpTreeNode {
        OpTreeNode::Number(n)
    }

    pub fn new_op(lhs: OpTreeNode, op: ProgOp, rhs: OpTreeNode) -> OpTreeNode {
        OpTreeNode::Op(Box::new(OpTreeNodeOp {
            op,
            lhs,
            rhs
        }))
    }

    fn new_multi(op: ProgOp, terms: Vec<OpTreeNode>) -> OpTreeNode {
        OpTreeNode::Multi(Box::new(OpTreeNodeMulti {
            op,
            terms
        }))
    }

    pub fn colour(&self, numbers: &[u32], colour: bool, brackets: bool) -> String {
        match self {
            OpTreeNode::Number(n) => ProgOp::Number(*n).colour(colour, numbers),
            OpTreeNode::Op(op) => {
                let lhs_str = op.lhs.colour(numbers, colour, false);
                let op_str = op.op.colour(colour, numbers);
                let rhs_str = op.rhs.colour(numbers, colour, true);

                if brackets {
                    format!("({} {} {})", lhs_str, op_str, rhs_str)
                } else {
                    format!("{} {} {}", lhs_str, op_str, rhs_str)
                }
            }
            OpTreeNode::Multi(m) => {
                let join = format!(" {} ", m.op.colour(colour, numbers));
                format!("{}", m.terms.iter().map(|t| t.colour(numbers, colour, true)).join(&join))
            }
        }
    }

    pub fn simplify(self) -> Self {
        let mut cur = self;

        cur = cur.group_ops();

        cur
    }

    fn group_ops(&self) -> OpTreeNode {
        match &self {
            OpTreeNode::Number(_) => {
                self.clone()
            }
            OpTreeNode::Multi(m) => {
                OpTreeNode::process_terms(m.op, &m.terms)
            }
            OpTreeNode::Op(o) => {
                if let Some(terms) = OpTreeNode::collapse(o.op, &*o) {
                    OpTreeNode::process_terms(o.op, &terms)
                } else {
                    let lhs = o.lhs.group_ops();
                    let rhs = o.rhs.group_ops();
                    OpTreeNode::new_op(lhs, o.op, rhs)
                }
            }
        }
    }

    fn collapse(op: ProgOp, node: &OpTreeNodeOp) -> Option<Vec<OpTreeNode>> {
        let mut term_ptrs = Vec::new();

        if OpTreeNode::collapse_int(op, node, &mut term_ptrs) {
            Some(term_ptrs.iter().map(|&t| t.clone()).collect())
        } else {
            None
        }
    }

    fn collapse_int<'a>(op: ProgOp, node: &'a OpTreeNodeOp, term_ptrs: &mut Vec<&'a OpTreeNode>) -> bool {
        let mut collapsed = false;

        match &node.lhs {
            OpTreeNode::Number(_) => {
                term_ptrs.push(&node.lhs)
            }
            OpTreeNode::Op(next) => {
                if next.op == op {
                    OpTreeNode::collapse_int(op, next, term_ptrs);
                    collapsed = true;
                } else {
                    term_ptrs.push(&node.lhs)
                }
            }
            OpTreeNode::Multi(m) => {
                if m.op == op { panic!("Unexpected!")}
                term_ptrs.push(&node.lhs)
            }
        }

        match op {
            ProgOp::OpAdd | ProgOp::OpMul => {
                match &node.rhs {
                    OpTreeNode::Number(_) => {
                        term_ptrs.push(&node.rhs)
                    }
                    OpTreeNode::Op(next) => {
                        if next.op == op {
                            OpTreeNode::collapse_int(op, next, term_ptrs);
                            collapsed = true;
                        } else {
                            term_ptrs.push(&node.rhs)
                        }
                    }
                    OpTreeNode::Multi(m) => {
                        if m.op == op { panic!("Unexpected!")}
                        term_ptrs.push(&node.rhs)
                    }
                }
            }
            _ => {
                term_ptrs.push(&node.rhs);
            }
        }

        collapsed
    }

    fn process_terms(terms_op: ProgOp, in_terms: &Vec<OpTreeNode>) -> OpTreeNode {
        let out_terms = in_terms.iter().map(|t| {
            t.group_ops()
        }).collect();

        OpTreeNode::new_multi(terms_op, out_terms)
    }

}

// Tests

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn simplify_add() {
        let mut program = Program::new(2);

        program.push(ProgOp::Number(0));
        program.push(ProgOp::Number(1));
        program.push(ProgOp::Number(2));
        program.push(ProgOp::Number(3));
        program.push(ProgOp::OpAdd);
        program.push(ProgOp::OpAdd);
        program.push(ProgOp::OpAdd);

        // 0 1 2 3 + + +
        let op_tree = program.op_tree();
        let simplified = op_tree.clone().simplify();

        println!("{:?}", op_tree);
        println!("{:?}", simplified);

        assert_eq!(simplified.colour(&[1, 2, 3, 4], false, false), "1 + 2 + 3 + 4");
    }

    #[test]
    fn simplify_mul() {
        let mut program = Program::new(2);

        // 0 1 2 3 * * *
        program.push(ProgOp::Number(0));
        program.push(ProgOp::Number(1));
        program.push(ProgOp::Number(2));
        program.push(ProgOp::Number(3));
        program.push(ProgOp::OpMul);
        program.push(ProgOp::OpMul);
        program.push(ProgOp::OpMul);

        let op_tree = program.op_tree();
        let simplified = op_tree.clone().simplify();

        println!("{:?}", op_tree);
        println!("{:?}", simplified);

        assert_eq!(simplified.colour(&[1, 2, 3, 4], false, false), "1 × 2 × 3 × 4");
    }

    #[test]
    fn simplify_sub_1() {
        let mut program = Program::new(2);

        // 0 1 2 3 - - -
        program.push(ProgOp::Number(0));
        program.push(ProgOp::Number(1));
        program.push(ProgOp::Number(2));
        program.push(ProgOp::Number(3));
        program.push(ProgOp::OpSub);
        program.push(ProgOp::OpSub);
        program.push(ProgOp::OpSub);

        let op_tree = program.op_tree();
        let simplified = op_tree.clone().simplify();

        println!("{:?}", op_tree);
        println!("{:?}", simplified);

        assert_eq!(simplified.colour(&[1, 2, 3, 4], false, false), "1 - (2 - (3 - 4))");
    }

    #[test]
    fn simplify_sub_2() {
        let mut program = Program::new(2);

        // 0 1 - 2 - 3 -
        program.push(ProgOp::Number(0));
        program.push(ProgOp::Number(1));
        program.push(ProgOp::OpSub);
        program.push(ProgOp::Number(2));
        program.push(ProgOp::OpSub);
        program.push(ProgOp::Number(3));
        program.push(ProgOp::OpSub);

        let op_tree = program.op_tree();
        let simplified = op_tree.clone().simplify();

        println!("{:?}", op_tree);
        println!("{:?}", simplified);

        assert_eq!(simplified.colour(&[1, 2, 3, 4], false, false), "1 - 2 - 3 - 4");
    }

    #[test]
    fn simplify_div_1() {
        let mut program = Program::new(2);

        // 0 1 2 3 / / /
        program.push(ProgOp::Number(0));
        program.push(ProgOp::Number(1));
        program.push(ProgOp::Number(2));
        program.push(ProgOp::Number(3));
        program.push(ProgOp::OpDiv);
        program.push(ProgOp::OpDiv);
        program.push(ProgOp::OpDiv);

        let op_tree = program.op_tree();
        let simplified = op_tree.clone().simplify();

        println!("{:?}", op_tree);
        println!("{:?}", simplified);

        assert_eq!(simplified.colour(&[1, 2, 3, 4], false, false), "1 / (2 / (3 / 4))");
    }

    #[test]
    fn simplify_div_2() {
        let mut program = Program::new(2);

        // 0 1 / 2 / 3 /
        program.push(ProgOp::Number(0));
        program.push(ProgOp::Number(1));
        program.push(ProgOp::OpDiv);
        program.push(ProgOp::Number(2));
        program.push(ProgOp::OpDiv);
        program.push(ProgOp::Number(3));
        program.push(ProgOp::OpDiv);

        let op_tree = program.op_tree();
        let simplified = op_tree.clone().simplify();

        println!("{:?}", op_tree);
        println!("{:?}", simplified);

        assert_eq!(simplified.colour(&[1, 2, 3, 4], false, false), "1 / 2 / 3 / 4");
    }

    #[test]
    fn simplify_3() {
        let mut program = Program::new(3);

        // 25 5 100 + -
        program.push(ProgOp::Number(0));
        program.push(ProgOp::Number(1));
        program.push(ProgOp::Number(2));
        program.push(ProgOp::OpAdd);
        program.push(ProgOp::OpSub);

        let op_tree = program.op_tree();
        let simplified = op_tree.clone().simplify();

        println!("{:?}", op_tree);
        println!("{:?}", simplified);

        assert_eq!(simplified.colour(&[1, 2, 3, 4], false, false), "1 - (2 + 3)");
    }

    #[test]
    fn simplify_4() {
        let mut program = Program::new(3);

        // 25 5 - 100 +
        program.push(ProgOp::Number(0));
        program.push(ProgOp::Number(1));
        program.push(ProgOp::OpSub);
        program.push(ProgOp::Number(2));
        program.push(ProgOp::OpAdd);

        let op_tree = program.op_tree();
        let simplified = op_tree.clone().simplify();

        println!("{:?}", op_tree);
        println!("{:?}", simplified);

        assert_eq!(simplified.colour(&[1, 2, 3, 4], false, false), "1 - 2 + 3");
    }

}