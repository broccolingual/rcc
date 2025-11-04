use crate::ast::{Node, NodeKind};

pub fn gen_asm_from_expr(node: &Node) {
    if node.kind == NodeKind::Num {
        println!("  push {}", node.val);
        return;
    }

    gen_asm_from_expr(node.lhs.as_ref().unwrap());
    gen_asm_from_expr(node.rhs.as_ref().unwrap());

    println!("  pop rdi");
    println!("  pop rax");

    match node.kind {
        NodeKind::Add => println!("  add rax, rdi"),
        NodeKind::Sub => println!("  sub rax, rdi"),
        NodeKind::Mul => println!("  imul rax, rdi"),
        NodeKind::Div => {
            println!("  cqo");
            println!("  idiv rdi");
        }
        NodeKind::Eq => {
            println!("  cmp rax, rdi");
            println!("  sete al");
            println!("  movzb rax, al");
        }
        NodeKind::Ne => {
            println!("  cmp rax, rdi");
            println!("  setne al");
            println!("  movzb rax, al");
        }
        NodeKind::Lt => {
            println!("  cmp rax, rdi");
            println!("  setl al");
            println!("  movzb rax, al");
        }
        NodeKind::Le => {
            println!("  cmp rax, rdi");
            println!("  setle al");
            println!("  movzb rax, al");
        }
        _ => {}
    }
    println!("  push rax");
}
