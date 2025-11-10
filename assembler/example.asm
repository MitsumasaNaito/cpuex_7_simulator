# フィボナッチは正常(bneのジャンプの幅は要調整)

addi $t0, $zero, 40
addi $t1, $zero, 0
addi $t2, $zero, 1
addi $t3, $zero, 2
loop: slt $at, $t0, $t3
bne $at, $zero, end
add $t4, $t1, $t2
add $t1, $t2, $zero
add $t2, $t4, $zero
addi $t3, $t3, 1
j loop
end: add $a0, $t2, $zero
addi $v0, $zero, 10
syscall

# ロード・ストアも正常
lw $ra, 100($v0)
sw $v0, 20($v1)
