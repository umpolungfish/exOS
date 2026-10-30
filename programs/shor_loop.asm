; shor_loop.asm — Belnap Shor coherence cycle — indefinite
;
; Structural demo of the 2:1 coherence ratio in ParaASM.
;
; Each cycle mirrors the FullPipeline.lean stages:
;
;   Stage 1 (H): ENGAGR %r0  → B           (cost: 1 paradox)
;   Stage 2 (ModExp on B): no-op             (cost: 0)
;   Stage 3 (B-bias):  FSPLIT %r0 %r1 %r2  → r0=B, r1=T, r2=F
;                      FFUSE  %r1 %r2 %r0  → r0=B (T∨F=B, Frobenius roundtrip)
;                                             (cost: 2 paradoxes — matches Wigner's Friend)
;   Stage 4 (T-bias):  IFIX %r0             → r0=T (collapse B→T, cost: 0 extra paradoxes)
;
; Paradox count per cycle: 1 (ENGAGR) + 1 (FSPLIT r0=B) + 1 (r1) + 1 (r2) = 4
; This is exactly Theorem 2: P(n) = 4n  — Shor coherence ratio is encoded in the loop.
;
; Load: para load .init:\nENGAGR %r0\n.b_bias:\nFSPLIT %r0 %r1 %r2\nFFUSE %r1 %r2 %r0\nJB %r0 .t_bias\nJMP .b_bias\n.t_bias:\nIFIX %r0\nJMP .init
; Or:   :load programs/shor_loop.asm  (from ALEPH REPL)
; Run:  para loop 100

.init:
    ENGAGR %r0          ; H|T⟩=B — create superposition (stage 1, cost 1)
                        ; r0 is now B (both T and F simultaneously)

.b_bias:
    FSPLIT %r0 %r1 %r2  ; B-bias: δ(B) = (T,F) — Frobenius co-multiply (cost 1)
    FFUSE  %r1 %r2 %r0  ; B-bias: μ(T,F) = B   — join back (T∨F=B, cost 2 total)
    JB     %r0 .t_bias  ; confirm B preserved — always taken
    JMP    .b_bias

.t_bias:
    IFIX   %r0          ; T-bias: collapse B→T (stage 4, observation)
    EMIT   %r0          ; emit T — period signature (collapsed classical output)
    JMP    .init        ; restart from H
