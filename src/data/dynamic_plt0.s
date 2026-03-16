    .text
    
    auipc  t2, %pcrel_hi(gotplt)
    sub    t1, t1, t3               
    ld t3, %pcrel_lo(gotplt)(t2)    
    addi   t1, t1, -44 
    addi   t0, t2, %pcrel_lo(gotplt) 
    srli   t1, t1, 1 
    ld     t0, t0, 8          
    jr     t3
