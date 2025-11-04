use crate::ast::{Node, NodeKind};

pub fn gen_asm_from_lval(node: &Node) {
    if node.kind != NodeKind::LVar {
        panic!("代入の左辺値が変数ではありません");
    }
    println!("  mov rax, rbp");
    println!("  sub rax, {}", node.offset);
    println!("  push rax");
}

pub fn gen_asm_from_expr(node: &Node) {
    match node.kind {
        NodeKind::Return => {
            gen_asm_from_expr(node.lhs.as_ref().unwrap());
            println!("  pop rax");
            println!("  mov rsp, rbp");
            println!("  pop rbp");
            println!("  ret");
            return;
        }
        NodeKind::Num => {
            println!("  push {}", node.val);
            return;
        }
        NodeKind::LVar => {
            gen_asm_from_lval(node);
            println!("  pop rax");
            println!("  mov rax, [rax]");
            println!("  push rax");
            return;
        }
        NodeKind::Assign => {
            gen_asm_from_lval(node.lhs.as_ref().unwrap());
            gen_asm_from_expr(node.rhs.as_ref().unwrap());
            println!("  pop rdi");
            println!("  pop rax");
            println!("  mov [rax], rdi");
            println!("  push rdi");
            return;
        }
        _ => {}
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
        NodeKind::Rem => {
            println!("  cqo");
            println!("  idiv rdi");
            println!("  mov rax, rdx");
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
