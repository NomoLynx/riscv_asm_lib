    .text

    auipc   t3, %pcrel_hi(printf)
    ld      t3, %pcrel_lo(printf)(t3)
    jalr    t1, t3
    nop
