.intel_syntax noprefix
.text
.globl add
add:
  push rbp
  mov rbp, rsp
  sub rsp, 32
  mov [rbp-8], rdi
  mov [rbp-16], rsi
  mov [rbp-24], rdx
  mov [rbp-32], rcx
  lea rax, [rbp-8]
  push rax
  pop rax
  mov rax, [rax]
  push rax
  lea rax, [rbp-16]
  push rax
  pop rax
  mov rax, [rax]
  push rax
  pop rdi
  pop rax
  add rax, rdi
  push rax
  lea rax, [rbp-24]
  push rax
  pop rax
  mov rax, [rax]
  push rax
  pop rdi
  pop rax
  add rax, rdi
  push rax
  lea rax, [rbp-32]
  push rax
  pop rax
  mov rax, [rax]
  push rax
  pop rdi
  pop rax
  add rax, rdi
  push rax
  pop rax
  jmp .L.return.add
.L.return.add:
  mov rsp, rbp
  pop rbp
  ret
.globl main
main:
  push rbp
  mov rbp, rsp
  sub rsp, 32
  mov [rbp-8], rdi
  mov [rbp-16], rsi
  mov [rbp-24], rdx
  mov [rbp-32], rcx
  lea rax, [rbp-8]
  push rax
  pop rax
  mov rax, [rax]
  push rax
  lea rax, [rbp-16]
  push rax
  pop rax
  mov rax, [rax]
  push rax
  lea rax, [rbp-24]
  push rax
  pop rax
  mov rax, [rax]
  push rax
  lea rax, [rbp-32]
  push rax
  pop rax
  mov rax, [rax]
  push rax
  lea rax, [rbp-8]
  push rax
  push 1
  pop rdi
  pop rax
  mov [rax], rdi
  push rdi
  lea rax, [rbp-16]
  push rax
  push 2
  pop rdi
  pop rax
  mov [rax], rdi
  push rdi
  lea rax, [rbp-24]
  push rax
  push 3
  pop rdi
  pop rax
  mov [rax], rdi
  push rdi
  lea rax, [rbp-32]
  push rax
  push 4
  pop rdi
  pop rax
  mov [rax], rdi
  push rdi
  lea rax, [rbp-32]
  push rax
  pop rax
  mov rax, [rax]
  push rax
  lea rax, [rbp-24]
  push rax
  pop rax
  mov rax, [rax]
  push rax
  lea rax, [rbp-16]
  push rax
  pop rax
  mov rax, [rax]
  push rax
  lea rax, [rbp-8]
  push rax
  pop rax
  mov rax, [rax]
  push rax
  pop rdi
  pop rsi
  pop rdx
  pop rcx
  mov rax, rsp
  and rax, 15
  jnz .L.call.1
  mov rax, 0
  call add
  jmp .L.end.1
.L.call.1:
  sub rsp, 8
  mov rax, 0
  call add
  add rsp, 8
.L.end.1:
  push rax
  pop rax
  jmp .L.return.main
.L.return.main:
  mov rsp, rbp
  pop rbp
  ret
.section .note.GNU-stack,"",@progbits

