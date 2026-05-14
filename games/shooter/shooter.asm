; =============================================================================
; SHOOTER for MSX1 — Vertical Scrolling Shooter Game (Z80 Assembly)
; pasmo --msx shooter.asm shooter.rom
;
; Controls: Left/Right arrows = move, Space = fire
; Goal: Destroy 100 enemies to clear the game
; =============================================================================

        ORG     $4000           ; MSX cartridge slot 1, page 1

; --- ROM header ---
        DB      'A','B'         ; cartridge ID
        DW      START           ; INIT
        DW      0               ; STATEMENT
        DW      0               ; DEVICE
        DW      0               ; TEXT
        DS      6, 0

; =============================================================================
; BIOS entry points
; =============================================================================
CHPUT   EQU     $00A2           ; print char in A
CHGET   EQU     $009F           ; wait key
POSIT   EQU     $00C6           ; set cursor (H=col, L=row)
KILBUF  EQU     $0156           ; clear keyboard buffer
SNSMAT  EQU     $0141           ; read keyboard matrix row
GTSTCK  EQU     $00D5           ; joystick
GTTRIG  EQU     $00D8           ; trigger

; VDP ports
VDPDATA EQU     $98
VDPCTL  EQU     $99

; PPI
PPIC    EQU     $AA

; Work area
JIFFY   EQU     $FC9E           ; frame counter
FORCLR  EQU     $F3E9           ; foreground color
BAKCLR  EQU     $F3EA           ; background color
BDRCLR  EQU     $F3EB           ; border color

; =============================================================================
; RAM layout (using $E000-$EFFF area)
; =============================================================================
        ; Player (triangle at bottom)
PL_X    EQU     $E000           ; player X (0-240, pixel)
PL_Y    EQU     $E001           ; player Y (fixed at 176)

        ; Bullets: max 4, each = (X, Y, active)
BUL_TBL EQU     $E010           ; 12 bytes (4 × 3)

        ; Enemies: max 16, each = (X, Y, type, active)
EN_TBL  EQU     $E040           ; 64 bytes (16 × 4)

        ; Game state
GM_ST   EQU     $E100           ; 0=title,1=play,2=gameover,3=clear
SCORE   EQU     $E101           ; enemies killed (8-bit, 0-100)
FRAME   EQU     $E102           ; frame counter (8-bit)
PREV_K  EQU     $E103           ; previous keyboard state for edge detect
SPAWN_T EQU     $E104           ; enemy spawn timer
DIFF    EQU     $E105           ; difficulty level (speed)

; =============================================================================
; VDP register initialisation table (Screen 1 / Graphic 1)
; =============================================================================
VDPREGS:
        ; R0: mode bits (Graphic 1 = 0x00)
        DB      $00
        ; R1: 16K, screen on, int on, sprites 8x8 ×1
        DB      $E0
        ; R2: name table   at $1800  → $1800>>10 = 6
        DB      $06
        ; R3: color table  at $2000  → $2000>>6  = $80
        DB      $80
        ; R4: pattern gen  at $0000  → $0000>>11 = 0
        DB      $00
        ; R5: sprite attr  at $1B00  → $1B00>>7  = $36
        DB      $36
        ; R6: sprite gen   at $3800  → $3800>>11 = 7
        DB      $07
        ; R7: border/text color (white on black)
        DB      $F1
VDPREGS_END:

; Palette colors
COL_BG      EQU 1   ; black background
COL_PLAYER  EQU 15  ; white player
COL_BULLET  EQU 10  ; yellow bullet
COL_ENEMY   EQU 8   ; orange enemy
COL_TEXT    EQU 15  ; white text

; =============================================================================
; SPRITE DATA (8×8×1 patterns in VRAM pattern table)
; Pattern 0: Player (triangle ▲)
; Pattern 1: Bullet (small square ■)
; Pattern 2-3: Enemy (falling square ◼)
; =============================================================================
SPRITE_PATTERNS:
        ; Pattern 0: Player triangle (▲)
        DB      $18, $3C, $7E, $FF, $FF, $7E, $3C, $18
        ; Pattern 1: Bullet (small yellow square)
        DB      $00, $00, $18, $18, $18, $18, $00, $00
        ; Pattern 2: Enemy type A (orange square)
        DB      $FF, $FF, $FF, $FF, $FF, $FF, $FF, $FF
        ; Pattern 3: Enemy type B (lighter orange)
        DB      $C3, $C3, $C3, $C3, $C3, $C3, $C3, $C3
SPRITE_PATTERNS_END:

; =============================================================================
; START: Main entry point
; =============================================================================
START:
        DI
        LD      SP, $DFFF

        ; Initialize VDP
        CALL    INIT_VDP
        CALL    CLEAR_VRAM
        CALL    LOAD_SPRITES
        CALL    CLEAR_NAMETABLE
        CALL    SET_COLORS
        CALL    INIT_GAME

        EI

        ; Main game loop
MAIN_LP:
        ; Wait for VBLANK
        LD      A, ($FFFF)        ; dummy read to prevent optimization
        HALT

        CALL    READ_INPUT
        CALL    UPDATE_PLAYER
        CALL    UPDATE_BULLETS
        CALL    UPDATE_ENEMIES
        CALL    SPAWN_ENEMY
        CALL    CHECK_COLLISIONS
        CALL    UPDATE_SPRITES
        CALL    DRAW_SCORE

        LD      A, (GM_ST)
        CP      3                 ; 3 = clear
        JR      NZ, MAIN_LP

        ; Game clear — hold for 2 seconds
        LD      B, 120            ; 120 frames = 2 seconds
CLEAR_LOOP:
        HALT
        DJNZ    CLEAR_LOOP
        JR      MAIN_LP

; =============================================================================
; INIT_VDP
; =============================================================================
INIT_VDP:
        LD      HL, VDPREGS
        LD      B, 8
        XOR     C
INIT_VDP_LP:
        LD      A, (HL)
        OUT     (VDPCTL), A
        LD      A, $80
        OR      C
        OUT     (VDPCTL), A
        INC     HL
        INC     C
        DJNZ    INIT_VDP_LP
        RET

; =============================================================================
; CLEAR_VRAM — zero out all 16KB VRAM
; =============================================================================
CLEAR_VRAM:
        XOR     A
        OUT     (VDPCTL), A
        LD      A, $40
        OUT     (VDPCTL), A
        LD      D, 64
CV_OUT:
        LD      B, 0
        XOR     A
CV_LP:
        OUT     (VDPDATA), A
        DJNZ    CV_LP
        DEC     D
        JR      NZ, CV_OUT
        RET

; =============================================================================
; LOAD_SPRITES — copy sprite patterns to VRAM ($3800)
; =============================================================================
LOAD_SPRITES:
        LD      HL, $3800
        CALL    SET_VRAM_ADDR
        LD      HL, SPRITE_PATTERNS
        LD      BC, SPRITE_PATTERNS_END - SPRITE_PATTERNS
LS_LP:
        LD      A, (HL)
        OUT     (VDPDATA), A
        INC     HL
        DEC     BC
        LD      A, B
        OR      C
        JR      NZ, LS_LP
        RET

; =============================================================================
; SET_VRAM_ADDR — set VRAM write address in HL
; =============================================================================
SET_VRAM_ADDR:
        LD      A, L
        OUT     (VDPCTL), A
        LD      A, H
        OR      $40
        OUT     (VDPCTL), A
        RET

; =============================================================================
; CLEAR_NAMETABLE — fill $1800-$1AFF with space
; =============================================================================
CLEAR_NAMETABLE:
        LD      HL, $1800
        CALL    SET_VRAM_ADDR
        LD      D, 3
CNT_OUT:
        LD      B, 0
        LD      A, $20
CNT_LP:
        OUT     (VDPDATA), A
        DJNZ    CNT_LP
        DEC     D
        JR      NZ, CNT_OUT
        RET

; =============================================================================
; SET_COLORS — initialize color table
; =============================================================================
SET_COLORS:
        LD      HL, $2000
        CALL    SET_VRAM_ADDR
        LD      B, 32
        LD      A, $F1            ; white on black
SC_LP:
        OUT     (VDPDATA), A
        DJNZ    SC_LP
        RET

; =============================================================================
; INIT_GAME — reset game variables
; =============================================================================
INIT_GAME:
        ; Player at center bottom
        LD      A, 120            ; center X
        LD      (PL_X), A
        LD      A, 176            ; near bottom
        LD      (PL_Y), A

        ; Clear bullets
        LD      HL, BUL_TBL
        LD      B, 4
IBG_BUL:
        LD      (HL), 0
        INC     HL
        LD      (HL), 0
        INC     HL
        LD      (HL), 0            ; inactive
        INC     HL
        DJNZ    IBG_BUL

        ; Clear enemies
        LD      HL, EN_TBL
        LD      B, 16
IBG_EN:
        LD      (HL), 0
        INC     HL
        LD      (HL), 0
        INC     HL
        LD      (HL), 0
        INC     HL
        LD      (HL), 0            ; inactive
        INC     HL
        DJNZ    IBG_EN

        XOR     A
        LD      (GM_ST), A         ; title state
        LD      (SCORE), A
        LD      (FRAME), A
        LD      (PREV_K), A
        LD      (SPAWN_T), A
        LD      (DIFF), A

        LD      (GM_ST), A         ; 1 = play state
        INC     A
        LD      (GM_ST), A

        RET

; =============================================================================
; READ_INPUT — read keyboard row 8 (cursor keys + space)
; =============================================================================
ROW_CURSOR EQU 8
BIT_RIGHT  EQU 0
BIT_LEFT   EQU 2
BIT_SPACE  EQU 7

INP_CUR    EQU $E060

READ_INPUT:
        LD      A, ROW_CURSOR
        CALL    SNSMAT
        CPL                        ; invert (SNSMAT returns 0 for pressed)
        LD      (INP_CUR), A
        RET

; =============================================================================
; UPDATE_PLAYER — move player left/right
; =============================================================================
UPDATE_PLAYER:
        LD      A, (GM_ST)
        CP      1                  ; 1 = play
        RET     NZ

        LD      A, (INP_CUR)

        ; Check left
        BIT     BIT_LEFT, A
        JR      Z, UPL_NL
        LD      A, (PL_X)
        SUB     2                  ; move left by 2 pixels
        CP      0
        JR      NC, UPL_LX
        LD      A, 0
UPL_LX:
        LD      (PL_X), A

UPL_NL:
        LD      A, (INP_CUR)
        BIT     BIT_RIGHT, A
        JR      Z, UPL_NR
        LD      A, (PL_X)
        ADD     A, 2               ; move right by 2 pixels
        CP      248                ; 248 = 256 - 8
        JR      C, UPL_RX
        LD      A, 248
UPL_RX:
        LD      (PL_X), A

UPL_NR:
        RET

; =============================================================================
; UPDATE_BULLETS — move active bullets upward
; =============================================================================
UPDATE_BULLETS:
        LD      HL, BUL_TBL
        LD      B, 4               ; 4 bullets

UB_LOOP:
        LD      A, (HL)            ; X
        PUSH    HL
        INC     HL
        LD      C, (HL)            ; Y
        INC     HL
        LD      D, (HL)            ; active flag
        POP     HL

        OR      A
        JR      Z, UB_NEXT         ; skip if X=0 (inactive)

        LD      A, C
        SUB     4                  ; move up by 4 pixels
        LD      C, A

        CP      0                  ; check if off-screen
        JR      NC, UB_ALIVE
        ; bullet off-screen: deactivate
        LD      (HL), 0            ; X = 0 (inactive)
        JR      UB_NEXT
UB_ALIVE:
        LD      (HL), 0            ; X = 0 (temp clear)
        INC     HL
        LD      (HL), C            ; update Y
        DEC     HL

UB_NEXT:
        INC     HL
        INC     HL
        INC     HL
        DJNZ    UB_LOOP
        RET

; =============================================================================
; UPDATE_ENEMIES — move active enemies downward
; =============================================================================
UPDATE_ENEMIES:
        LD      HL, EN_TBL
        LD      B, 16              ; 16 enemies

UEN_LOOP:
        PUSH    HL
        INC     HL
        LD      A, (HL)            ; Y
        INC     HL
        INC     HL
        LD      C, (HL)            ; active flag
        POP     HL

        OR      A
        JR      Z, UEN_NEXT        ; skip if X=0 (inactive)

        LD      A, (HL)            ; get type byte
        PUSH    HL
        INC     HL
        LD      A, (HL)            ; Y
        INC     HL

        ; difficulty: 1 + (DIFF >> 5) pixels per frame
        LD      C, A
        LD      A, (DIFF)
        SRL     A
        SRL     A
        SRL     A
        SRL     A
        SRL     A
        INC     A

        ADD     A, C               ; move down
        LD      C, A

        CP      192                ; check if off-screen
        JR      C, UEN_ALIVE
        ; enemy off-screen: deactivate
        POP     HL
        LD      (HL), 0            ; X = 0 (inactive)
        JR      UEN_NEXT
UEN_ALIVE:
        POP     HL
        INC     HL
        LD      (HL), C            ; update Y
        DEC     HL

UEN_NEXT:
        INC     HL
        INC     HL
        INC     HL
        INC     HL
        DJNZ    UEN_LOOP
        RET

; =============================================================================
; SPAWN_ENEMY — periodically spawn new enemies
; =============================================================================
SPAWN_ENEMY:
        LD      A, (GM_ST)
        CP      1                  ; 1 = play
        RET     NZ

        LD      A, (SCORE)
        CP      100                ; check if cleared
        RET     Z

        ; increment spawn timer
        LD      A, (SPAWN_T)
        INC     A
        LD      (SPAWN_T), A

        CP      30                 ; spawn every 30 frames
        RET     NZ

        XOR     A
        LD      (SPAWN_T), A

        ; find first inactive enemy slot
        LD      HL, EN_TBL
        LD      B, 16

SE_SEARCH:
        PUSH    HL
        PUSH    BC
        LD      A, (HL)
        POP     BC
        POP     HL
        OR      A
        JR      Z, SE_FOUND        ; found inactive slot

        INC     HL
        INC     HL
        INC     HL
        INC     HL
        DJNZ    SE_SEARCH
        RET                        ; no free slot

SE_FOUND:
        ; generate random X (0-240)
        LD      A, (FRAME)
        RRCA
        RRCA
        AND     $F0
        CP      248
        JR      C, SE_XOK
        LD      A, 240
SE_XOK:
        LD      (HL), A            ; X = random

        INC     HL
        LD      (HL), 0            ; Y = 0 (top)

        INC     HL
        LD      A, (DIFF)
        AND     $01
        LD      (HL), A            ; type = random (0 or 1)

        INC     HL
        LD      (HL), 1            ; active = 1

        RET

; =============================================================================
; CHECK_COLLISIONS — check bullet-enemy hits and player-enemy collisions
; =============================================================================
CHECK_COLLISIONS:
        LD      A, (GM_ST)
        CP      1                  ; 1 = play
        RET     NZ

        ; Check each bullet against each enemy
        LD      HL, BUL_TBL
        LD      B, 4

CC_BUL_LOOP:
        PUSH    HL
        LD      A, (HL)            ; bullet X
        PUSH    HL
        INC     HL
        LD      C, (HL)            ; bullet Y
        INC     HL
        LD      D, (HL)            ; bullet active
        POP     HL

        OR      A
        JR      Z, CC_BUL_NEXT     ; skip inactive bullet

        ; Check against enemies
        LD      HL, EN_TBL
        LD      D, 16

CC_EN_LOOP:
        PUSH    HL
        LD      E, (HL)            ; enemy X
        PUSH    HL
        INC     HL
        LD      H, (HL)            ; enemy Y
        INC     HL
        INC     HL
        LD      B, (HL)            ; enemy active
        POP     HL

        OR      B
        JR      Z, CC_EN_NEXT      ; skip inactive enemy

        ; Simple collision: if (abs(ex-bx) < 8 && abs(ey-by) < 8)
        LD      B, A               ; B = bullet X
        LD      A, E               ; A = enemy X
        SUB     B
        JR      C, CC_XPOS
        CP      8
        JR      NC, CC_EN_NEXT
        JR      CC_XHIT

CC_XPOS:
        NEG
        CP      8
        JR      NC, CC_EN_NEXT

CC_XHIT:
        LD      B, C               ; B = bullet Y
        LD      A, H               ; A = enemy Y
        SUB     B
        JR      C, CC_YPOS
        CP      8
        JR      NC, CC_EN_NEXT
        JR      CC_HIT

CC_YPOS:
        NEG
        CP      8
        JR      NC, CC_EN_NEXT

CC_HIT:
        ; Hit! Deactivate bullet and enemy
        POP     HL
        LD      (HL), 0            ; enemy X = 0 (inactive)

        LD      A, (SCORE)
        INC     A
        LD      (SCORE), A
        CP      100
        JR      NZ, CC_HIT_END
        LD      A, 3               ; game clear
        LD      (GM_ST), A
CC_HIT_END:

        JR      CC_BUL_NEXT        ; bullet done

CC_EN_NEXT:
        POP     HL
        INC     HL
        INC     HL
        INC     HL
        INC     HL
        DEC     D
        JR      NZ, CC_EN_LOOP

CC_BUL_NEXT:
        POP     HL
        INC     HL
        INC     HL
        INC     HL
        DJNZ    CC_BUL_LOOP

        RET

; =============================================================================
; UPDATE_SPRITES — update sprite attribute table ($1B00)
; =============================================================================
UPDATE_SPRITES:
        LD      HL, $1B00
        CALL    SET_VRAM_ADDR

        ; Player sprite at (PL_X, PL_Y)
        LD      A, (PL_Y)
        OUT     (VDPDATA), A
        LD      A, (PL_X)
        OUT     (VDPDATA), A
        LD      A, 0               ; pattern 0 (player triangle)
        OUT     (VDPDATA), A
        LD      A, $0F             ; white color
        OUT     (VDPDATA), A

        ; Bullets
        LD      HL, BUL_TBL
        LD      B, 4
UB_SPR:
        LD      A, (HL)            ; X
        PUSH    HL
        INC     HL
        LD      C, (HL)            ; Y
        INC     HL
        LD      D, (HL)            ; active
        INC     HL
        POP     HL

        OR      A
        JR      Z, UB_SPR_HIDE

        LD      A, C               ; Y
        OUT     (VDPDATA), A
        LD      A, (HL)            ; X
        OUT     (VDPDATA), A
        LD      A, 1               ; pattern 1 (bullet)
        OUT     (VDPDATA), A
        LD      A, $0A             ; yellow color
        OUT     (VDPDATA), A
        JR      UB_SPR_NEXT

UB_SPR_HIDE:
        LD      A, $D0             ; Y = 208 (off-screen)
        OUT     (VDPDATA), A
        LD      A, 0
        OUT     (VDPDATA), A
        LD      A, 1
        OUT     (VDPDATA), A
        LD      A, 0
        OUT     (VDPDATA), A

UB_SPR_NEXT:
        INC     HL
        INC     HL
        INC     HL
        DJNZ    UB_SPR

        ; Enemies
        LD      HL, EN_TBL
        LD      B, 16
UEN_SPR:
        LD      A, (HL)            ; X
        PUSH    HL
        INC     HL
        LD      C, (HL)            ; Y
        INC     HL
        LD      D, (HL)            ; type
        INC     HL
        LD      E, (HL)            ; active
        POP     HL

        OR      E
        JR      Z, UEN_SPR_HIDE

        LD      A, C               ; Y
        OUT     (VDPDATA), A
        LD      A, (HL)            ; X
        OUT     (VDPDATA), A
        LD      A, D               ; pattern (2 or 3)
        ADD     A, 2               ; offset from base 2
        OUT     (VDPDATA), A
        LD      A, $08             ; orange color
        OUT     (VDPDATA), A
        JR      UEN_SPR_NEXT

UEN_SPR_HIDE:
        LD      A, $D0             ; Y = 208 (off-screen)
        OUT     (VDPDATA), A
        LD      A, 0
        OUT     (VDPDATA), A
        LD      A, 2
        OUT     (VDPDATA), A
        LD      A, 0
        OUT     (VDPDATA), A

UEN_SPR_NEXT:
        INC     HL
        INC     HL
        INC     HL
        INC     HL
        DJNZ    UEN_SPR

        ; Terminate sprite list
        LD      A, $D0
        OUT     (VDPDATA), A

        RET

; =============================================================================
; DRAW_SCORE — draw score on screen at top-left
; =============================================================================
DRAW_SCORE:
        RET                        ; TODO: implement score display

        ; Pad ROM to 16KB (ends at $8000, 16384 bytes from $4000)
        DS      $8000 - $, 0

        END
