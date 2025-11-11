.intel_syntax noprefix
.text
.globl add
add:
  push rbp
  mov rbp, rsp
  sub rsp, 208
  mov [rbp-24], rsi
  mov [rbp-16], rdi
  lea rax, [rbp-16]
  mov rax, [rax]
  push rax
  lea rax, [rbp-24]
  mov rax, [rax]
  push rax
  pop rdi
  pop rax
  add rax, rdi
  jmp .L.return.add
.L.return.add:
  mov rsp, rbp
  pop rbp
  ret
.globl main
main:
  push rbp
  mov rbp, rsp
  sub rsp, 208
  lea rax, [rbp-40]
  mov rax, [rax]
  push rax
  lea rax, [rbp-48]
  mov rax, [rax]
  push rax
  lea rax, [rbp-48]
  push rax
  push 1
  pop rdi
  pop rax
  mov [rax], rdi
  push rdi
  lea rax, [rbp-40]
  push rax
  lea rax, [rbp-48]
  mov rax, [rax]
  push rax
  pop rdi
  pop rax
  mov [rax], rdi
  push rdi
  push 2
  lea rax, [rbp-40]
  mov rax, [rax]
  push rax
  pop rsi
  pop rdi
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
  jmp .L.return.main
.L.return.main:
  mov rsp, rbp
  pop rbp
  ret
.section .note.GNU-stack,"",@progbits

