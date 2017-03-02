use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::c_char;

pub mod ext;

#[derive(Debug)]
enum AstNode {
    Tag(String),
    BinOp(Op, Box<AstNode>, Box<AstNode>),
    Grouping(Box<AstNode>),
}

#[derive(Debug, Copy, Clone)]
enum Op {
    Subtraction,
    Intersection,
    Union,
}

struct Context {
    nodes: Vec<AstNode>,
    ops:   Vec<Op>,
}

impl Context {
    pub fn new() -> Self {
        Context { nodes: vec![], ops: vec![] }
    }

    pub fn resolve(mut self) -> AstNode {
        while let Some(op) = self.ops.pop() {
            let rnode = self.nodes.pop().unwrap();
            let lnode = self.nodes.pop().unwrap();
            self.nodes.push(AstNode::BinOp(op, Box::new(lnode), Box::new(rnode)));
        }

        assert_eq!(self.nodes.len(), 1); self.nodes.pop().unwrap()
    }
}

pub fn build_query(query_str: &str) -> String {
    let mut suspended_ctx = vec![];

    let mut ctx = Context::new();
    let mut tag_buf = String::new();

    for token in query_str.chars() {
        match token {
            '+' => {
                ctx.ops.push(Op::Intersection);
                add_tag(&mut ctx, &mut tag_buf);
            },

            '-' => {
                ctx.ops.push(Op::Subtraction);
                add_tag(&mut ctx, &mut tag_buf);
            },

            '*' => {
                ctx.ops.push(Op::Union);
                add_tag(&mut ctx, &mut tag_buf);               
            },

            '(' => {
                let old_ctx = mem::replace(&mut ctx, Context::new());
                suspended_ctx.push(old_ctx);
            },

            ')' => {
                add_tag(&mut ctx, &mut tag_buf);

                // resume the suspended context ...
                let prev_ctx  = suspended_ctx.pop().unwrap();
                let group_ctx = mem::replace(&mut ctx, prev_ctx);
                ctx.nodes.push(AstNode::Grouping(Box::new(group_ctx.resolve())))
            },

            // not a token we recognize, assume it's a tag char.
            _ => tag_buf.push(token),
        }
    }

    add_tag(&mut ctx, &mut tag_buf);
    visit_ast_node(ctx.resolve())
}


fn visit_ast_node(node: AstNode) -> String {
    match node {
        AstNode::BinOp(Op::Subtraction, lhs, rhs) => {
            let lhs_frag = visit_ast_node(*lhs);
            let rhs_frag = visit_ast_node(*rhs);
            format!("{} EXCEPT {}", lhs_frag, rhs_frag)
        },

        AstNode::BinOp(Op::Intersection, lhs, rhs) => {
            let lhs_frag = visit_ast_node(*lhs);
            let rhs_frag = visit_ast_node(*rhs);
            format!("{} INTERSECT {}", lhs_frag, rhs_frag)
        },

        AstNode::BinOp(Op::Union, lhs, rhs) => {
            let lhs_frag = visit_ast_node(*lhs);
            let rhs_frag = visit_ast_node(*rhs);
            format!("{} UNION ({})", lhs_frag, rhs_frag)
        },

        AstNode::Grouping(inner) => format!("({})", visit_ast_node(*inner)),

        AstNode::Tag(ref tag_text) => entry_set(tag_text),
    }
}

fn entry_set(tag_name: &str) -> String {
    format!("SELECT entry_id FROM entries_tags
INNER JOIN tags ON tags.id = entries_tags.tag_id
WHERE tags.name = '{}'", tag_name)
}

fn add_tag(ctx: &mut Context, buf: &mut String) {
    if (buf.trim() == "") { return; } // lhs was not a tag!

    let tag_text = mem::replace(buf, String::new());
    let tag_node = AstNode::Tag(tag_text.trim().to_string());
    ctx.nodes.push(tag_node);
}

