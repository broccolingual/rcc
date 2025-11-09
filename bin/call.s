.intel_syntax noprefix
.globl main
main:
  push rbp
  mov rbp, rsp
  sub rsp, 208
  mov rax, rsp
  and rax, 15
  jnz .Lcall1
  mov rax, 0
  call foo
  jmp .Lend1
.Lcall1:
  sub rsp, 8
  mov rax, 0
  call foo
  add rsp, 8
.Lend1:
  push rax
  pop rax
  mov rsp, rbp
  pop rbp
  ret
.section .note.GNU-stack,"",@progbits
