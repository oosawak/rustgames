; =============================================================================
; LINEBOY for MSX1 — Z80 Assembly
; pasmo --msx lineboy.asm lineboy.rom
;
; Controls: Left/Right arrows = move, Space = jump
; Goal: collect all 10 stars, avoid 3 enemies
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
        ; Player
PL_X    EQU     $E000           ; player X (0-240, pixel)
PL_Y    EQU     $E001           ; player Y (0-176, pixel)
PL_VY   EQU     $E002           ; vertical velocity (signed, Q4)
PL_GNDQ EQU     $E003           ; grounded flag (0/1)
PL_DIR  EQU     $E004           ; facing dir (0=right,1=left)

        ; Enemy table: 3 enemies × 3 bytes (X, Y, dir)
EN_TBL  EQU     $E010           ; $E010,$E011,$E012 = enemy0; +3=enemy1, +6=enemy2

        ; Item table: 10 items × 2 bytes (X, Y), 0xFF=collected
IT_TBL  EQU     $E030           ; 20 bytes

        ; Game state
GM_ST   EQU     $E050           ; 0=title,1=play,2=gameover,3=clear
SCORE   EQU     $E051           ; items collected
FRAME   EQU     $E052           ; frame counter (8-bit)
PREV_K  EQU     $E053           ; previous keyboard state for edge detect

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

; =============================================================================
; Palette: TMS9918A 15 colors (index 1–15)
; We map our 4 game colors to these indices
; 0=transparent,1=black,2=green,3=light green,
; 4=blue,5=light blue,6=red,7=cyan,
; 8=orange,9=light orange,10=yellow,11=light yellow,
; 12=dark green,13=purple,14=gray,15=white
; =============================================================================
COL_BG      EQU 1   ; black background
COL_GROUND  EQU 2   ; green ground
COL_PLAYER  EQU 15  ; white player
COL_ENEMY   EQU 8   ; orange enemy
COL_ITEM    EQU 10  ; yellow star
COL_TEXT    EQU 15  ; white text

; =============================================================================
; START
; =============================================================================
START:
        DI
        LD      SP, $F380       ; safe stack in RAM
        EI                      ; enable interrupts (needed for BIOS)

        ; Init VDP
        CALL    INIT_VDP

        ; Init game state
        CALL    INIT_GAME

        ; Title screen
        CALL    SHOW_TITLE
        CALL    WAIT_SPACE

        ; Main game loop
MAIN_LOOP:
        ; Wait for VSYNC by polling VDP status bit 7
WAIT_VBL:
        IN      A, ($99)        ; read VDP status register
        AND     $80             ; check VBLANK flag (bit 7)
        JR      Z, WAIT_VBL    ; wait until VBLANK

        LD      A, (FRAME)
        INC     A
        LD      (FRAME), A

        CALL    READ_INPUT
        CALL    UPDATE_PLAYER
        CALL    UPDATE_ENEMIES
        CALL    CHECK_ITEMS
        CALL    CHECK_ENEMY_COLL
        CALL    DRAW_FRAME
        CALL    DRAW_HUD

        LD      A, (GM_ST)
        CP      2
        JP      Z, GAME_OVER_SCR
        CP      3
        JP      Z, CLEAR_SCR

        JP      MAIN_LOOP

; =============================================================================
; INIT_VDP — set Screen 1 (Graphic 1) mode
; =============================================================================
INIT_VDP:
        LD      HL, VDPREGS
        LD      B, VDPREGS_END - VDPREGS
        LD      C, 0
IVDP_LP:
        ; write register: first data, then $80|reg
        LD      A, (HL)
        OUT     (VDPCTL), A
        LD      A, $80
        OR      C
        OUT     (VDPCTL), A
        INC     HL
        INC     C
        DJNZ    IVDP_LP

        ; Clear VRAM
        CALL    VRAM_CLEAR

        ; Load font into pattern gen ($0000)
        CALL    LOAD_FONT

        ; Fill name table with spaces ($1800)
        CALL    CLEAR_NAMETABLE

        ; Set color table: all chars = COL_TEXT on COL_BG ($2000)
        CALL    SET_COLORS

        ; Load sprite patterns into VRAM
        CALL    LOAD_SPRITES

        RET

; =============================================================================
; VRAM_CLEAR — fill all 16KB VRAM with $00
; =============================================================================
VRAM_CLEAR:
        ; set VRAM addr 0
        XOR     A
        OUT     (VDPCTL), A
        LD      A, $40
        OUT     (VDPCTL), A
        ; write 16384 zeros
        LD      BC, 16384
        XOR     A
VC_LP:
        OUT     (VDPDATA), A
        DEC     BC
        LD      A, B
        OR      C
        JR      NZ, VC_LP
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

SET_VRAM_READ_ADDR:
        LD      A, L
        OUT     (VDPCTL), A
        LD      A, H
        AND     $3F
        OUT     (VDPCTL), A
        RET

; =============================================================================
; CLEAR_NAMETABLE — fill $1800-$1AFF with space (char $20)
; =============================================================================
CLEAR_NAMETABLE:
        LD      HL, $1800
        CALL    SET_VRAM_ADDR
        LD      BC, 768         ; 32×24 chars
        LD      A, $20
CNT_LP:
        OUT     (VDPDATA), A
        DEC     BC
        LD      A, B
        OR      C
        JR      NZ, CNT_LP
        RET

; =============================================================================
; SET_COLORS — color table at $2000: 32 entries, each = fg|bg<<4
; =============================================================================
SET_COLORS:
        LD      HL, $2000
        CALL    SET_VRAM_ADDR
        LD      B, 32
        LD      A, COL_TEXT
        SLA     A
        SLA     A
        SLA     A
        SLA     A
        OR      COL_BG          ; $F1 (white on black)
SC_LP:
        OUT     (VDPDATA), A
        DJNZ    SC_LP
        RET

; =============================================================================
; LOAD_FONT — copy minimal ASCII font into pattern table at $0000
; Each char is 8 bytes. We only need 0-9, A-Z, space, !, /
; For simplicity we embed a tiny 1-bit font.
; =============================================================================
LOAD_FONT:
        LD      HL, $0000
        CALL    SET_VRAM_ADDR
        LD      HL, FONT_DATA
        LD      BC, FONT_DATA_END - FONT_DATA
LF_LP:
        LD      A, (HL)
        OUT     (VDPDATA), A
        INC     HL
        DEC     BC
        LD      A, B
        OR      C
        JR      NZ, LF_LP
        RET

; =============================================================================
; INIT_GAME — reset all game variables
; =============================================================================
INIT_GAME:
        ; Player at x=24, y=120 (ground is y=168)
        LD      A, 24
        LD      (PL_X), A
        LD      A, 120
        LD      (PL_Y), A
        XOR     A
        LD      (PL_VY), A
        LD      (PL_GNDQ), A
        LD      (PL_DIR), A
        LD      (GM_ST), A
        LD      (SCORE), A
        LD      (FRAME), A
        LD      (PREV_K), A

        ; Enemies: x=60,y=160  x=120,y=160  x=180,y=160
        LD      HL, EN_TBL
        LD      (HL), 60
        INC     HL
        LD      (HL), 160
        INC     HL
        LD      (HL), 0
        INC     HL              ; dir=right
        LD      (HL), 120
        INC     HL
        LD      (HL), 160
        INC     HL
        LD      (HL), 1
        INC     HL              ; dir=left
        LD      (HL), 180
        INC     HL
        LD      (HL), 160
        INC     HL
        LD      (HL), 0            ; dir=right

        ; Items: 10 stars spread across the screen
        LD      HL, IT_TBL
        ; (x, y) pairs
        LD      A, 30
        LD      (HL), A
        INC     HL
        LD      A, 130
        LD      (HL), A
        INC     HL
        LD      A, 55
        LD      (HL), A
        INC     HL
        LD      A, 100
        LD      (HL), A
        INC     HL
        LD      A, 80
        LD      (HL), A
        INC     HL
        LD      A, 145
        LD      (HL), A
        INC     HL
        LD      A, 105
        LD      (HL), A
        INC     HL
        LD      A, 115
        LD      (HL), A
        INC     HL
        LD      A, 130
        LD      (HL), A
        INC     HL
        LD      A, 130
        LD      (HL), A
        INC     HL
        LD      A, 155
        LD      (HL), A
        INC     HL
        LD      A, 100
        LD      (HL), A
        INC     HL
        LD      A, 175
        LD      (HL), A
        INC     HL
        LD      A, 140
        LD      (HL), A
        INC     HL
        LD      A, 200
        LD      (HL), A
        INC     HL
        LD      A, 115
        LD      (HL), A
        INC     HL
        LD      A, 220
        LD      (HL), A
        INC     HL
        LD      A, 130
        LD      (HL), A
        INC     HL
        LD      A, 240
        LD      (HL), A
        INC     HL
        LD      A, 110
        LD      (HL), A

        RET

; =============================================================================
; READ_INPUT — read keyboard matrix
; Row 8: space=0, cursor keys in row 8
; MSX keyboard matrix:
;   Row 8: right,down,up,left,del,ins,home,space  (bit 0=right ... bit 7=space)
; SNSMAT: call with A=row, returns bits (0=pressed)
; =============================================================================
ROW_CURSOR EQU 8
BIT_RIGHT  EQU 0
BIT_LEFT   EQU 2
BIT_SPACE  EQU 7

        ; We'll store: bit0=right, bit1=left, bit2=space (1=pressed this frame)
INP_CUR EQU $E060               ; current keys pressed

READ_INPUT:
        LD      A, ROW_CURSOR
        CALL    SNSMAT
        ; SNSMAT returns 0 for pressed — invert
        CPL
        LD      (INP_CUR), A
        RET

; helper: test if key bit B is pressed in INP_CUR
; returns Z set if NOT pressed
KEY_PRESSED:   ; B = bit number
        LD      A, (INP_CUR)
        AND     A               ; check bit B
        ; caller must do BIT instruction
        RET

; =============================================================================
; UPDATE_PLAYER
; =============================================================================
GRAV_NUM EQU    2               ; fixed-point: velocity in 1/4 pixels
GRAV_MAX EQU    20
JUMP_V   EQU    $F4             ; -12 in signed byte (Q2 → -3 pixels/frame)
GROUND_Y EQU    168             ; ground y pixel

UPDATE_PLAYER:
        ; Gravity: vy += 1 (Q2: add 1 = 0.25 px/frame²), cap at GRAV_MAX
        LD      A, (PL_VY)
        ADD     A, GRAV_NUM
        CP      GRAV_MAX
        JR      C, UP_NOGRAVCLIP
        LD      A, GRAV_MAX
UP_NOGRAVCLIP:
        LD      (PL_VY), A

        ; Jump: if space pressed AND grounded
        LD      A, (INP_CUR)
        BIT     BIT_SPACE, A
        JR      Z, UP_NOJUMP
        LD      A, (PL_GNDQ)
        OR      A
        JR      Z, UP_NOJUMP
        ; jump!
        LD      A, JUMP_V       ; negative velocity
        LD      (PL_VY), A
        XOR     A
        LD      (PL_GNDQ), A
UP_NOJUMP:

        ; Horizontal movement
        LD      A, (INP_CUR)
        BIT     BIT_RIGHT, A
        JR      Z, UP_CHK_LEFT
        ; move right
        LD      A, (PL_X)
        ADD     A, 2
        CP      249             ; max x = 255-8+1
        JR      C, UP_SET_X_R
        LD      A, 248
UP_SET_X_R:
        LD      (PL_X), A
        LD      A, 0
        LD      (PL_DIR), A
        JR      UP_MOVE_Y
UP_CHK_LEFT:
        BIT     BIT_LEFT, A
        JR      Z, UP_MOVE_Y
        ; move left
        LD      A, (PL_X)
        SUB     2
        JR      NC, UP_SET_X_L
        XOR     A
UP_SET_X_L:
        LD      (PL_X), A
        LD      A, 1
        LD      (PL_DIR), A

UP_MOVE_Y:
        ; Apply vertical velocity (VY is Q2: divide by 4 for pixels)
        LD      A, (PL_VY)
        ; Check sign
        BIT     7, A
        JR      NZ, UP_VY_NEG
        ; positive: move down by A/4
        SRA     A
        SRA     A
        LD      B, A
        LD      A, (PL_Y)
        ADD     A, B
        JR      UP_CLAMP_Y
UP_VY_NEG:
        ; negative: treat as signed
        ; A is negative, SRA keeps sign
        SRA     A
        SRA     A
        ; A is now negative delta
        LD      B, A
        LD      A, (PL_Y)
        ADD     A, B            ; A = PL_Y + negative = move up
        JR      NC, UP_FLOOR   ; if overflow/underflow
        CP      0
        JR      NC, UP_CLAMP_Y
        XOR     A               ; clamp to 0
UP_CLAMP_Y:
UP_FLOOR:
        ; check ground collision
        CP      GROUND_Y
        JR      C, UP_AIRBORNE
        ; landed
        LD      A, GROUND_Y
        LD      (PL_Y), A
        XOR     A
        LD      (PL_VY), A
        LD      A, 1
        LD      (PL_GNDQ), A
        RET
UP_AIRBORNE:
        LD      (PL_Y), A
        XOR     A
        LD      (PL_GNDQ), A
        RET

; =============================================================================
; UPDATE_ENEMIES — 3 enemies patrol left/right on ground
; =============================================================================
UPDATE_ENEMIES:
        LD      HL, EN_TBL
        LD      B, 3
UE_LOOP:
        PUSH    BC
        PUSH    HL
        ; HL→X, HL+1→Y, HL+2→dir
        LD      A, (HL)         ; X
        INC     HL
        LD      C, (HL)         ; Y (unused, always GROUND_Y)
        INC     HL
        LD      D, (HL)         ; dir
        DEC     HL
        DEC     HL              ; back to X
        LD      E, A            ; E = X

        LD      A, D
        OR      A
        JR      NZ, UE_LEFT
        ; moving right
        LD      A, E
        ADD     A, 1
        CP      240
        JR      C, UE_STORE
        ; hit right wall → reverse
        LD      A, 240
        LD      (HL), A
        INC     HL
        INC     HL
        LD      (HL), 1         ; dir = left
        JR      UE_NEXT
UE_LEFT:
        LD      A, E
        SUB     1
        JR      NC, UE_STORE
        XOR     A               ; clamp to 0
        LD      (HL), A
        INC     HL
        INC     HL
        LD      (HL), 0         ; dir = right
        JR      UE_NEXT
UE_STORE:
        LD      (HL), A         ; store new X
UE_NEXT:
        POP     HL
        LD      DE, 3
        ADD     HL, DE          ; next enemy
        POP     BC
        DJNZ    UE_LOOP
        RET

; =============================================================================
; CHECK_ITEMS — detect player touching stars
; =============================================================================
CHECK_ITEMS:
        LD      HL, IT_TBL
        LD      B, 10
CI_LOOP:
        PUSH    BC
        PUSH    HL
        LD      A, (HL)
        CP      $FF             ; already collected?
        JR      Z, CI_NEXT
        ; check distance: |PL_X - IX| < 8 && |PL_Y - IY| < 8
        LD      C, A            ; C = item X
        LD      A, (PL_X)
        SUB     C
        CALL    ABS_A
        CP      8
        JR      NC, CI_NEXT     ; too far in X
        INC     HL
        LD      C, (HL)         ; C = item Y
        DEC     HL
        LD      A, (PL_Y)
        SUB     C
        CALL    ABS_A
        CP      8
        JR      NC, CI_NEXT     ; too far in Y
        ; collected!
        LD      (HL), $FF       ; mark X as collected
        INC     HL
        LD      (HL), $FF       ; mark Y
        DEC     HL
        LD      A, (SCORE)
        INC     A
        LD      (SCORE), A
        CP      10
        JR      C, CI_NEXT
        ; all collected → CLEAR
        LD      A, 3
        LD      (GM_ST), A
CI_NEXT:
        POP     HL
        LD      DE, 2
        ADD     HL, DE
        POP     BC
        DJNZ    CI_LOOP
        RET

; ABS_A: returns |A| in A (A is treated as signed)
ABS_A:
        OR      A
        RET     P               ; positive → done
        NEG
        RET

; =============================================================================
; CHECK_ENEMY_COLL — player vs enemies
; =============================================================================
CHECK_ENEMY_COLL:
        LD      HL, EN_TBL
        LD      B, 3
CEC_LOOP:
        PUSH    BC
        PUSH    HL
        LD      A, (HL)         ; enemy X
        LD      C, A
        LD      A, (PL_X)
        SUB     C
        CALL    ABS_A
        CP      8
        JR      NC, CEC_NEXT
        INC     HL
        LD      A, (HL)         ; enemy Y
        LD      C, A
        LD      A, (PL_Y)
        SUB     C
        CALL    ABS_A
        CP      8
        JR      NC, CEC_NEXT
        ; collision → gameover
        LD      A, 2
        LD      (GM_ST), A
CEC_NEXT:
        POP     HL
        LD      DE, 3
        ADD     HL, DE
        POP     BC
        DJNZ    CEC_LOOP
        RET

; =============================================================================
; DRAW_FRAME — draw all sprites via sprite attribute table at $1B00
; Sprite attr: Y, X, pattern, color (4 bytes each, max 32 sprites)
; =============================================================================
DRAW_FRAME:
        ; Set sprite attr table addr ($1B00)
        LD      HL, $1B00
        CALL    SET_VRAM_ADDR

        ; Sprite 0: Player (color = COL_PLAYER, pattern 0)
        LD      A, (PL_Y)
        OUT     (VDPDATA), A    ; Y
        LD      A, (PL_X)
        OUT     (VDPDATA), A    ; X
        LD      A, 0
        OUT     (VDPDATA), A    ; pattern
        LD      A, COL_PLAYER
        OUT     (VDPDATA), A    ; color

        ; Sprites 1-3: Enemies
        LD      HL, EN_TBL
        LD      B, 3
DF_EN_LOOP:
        PUSH    BC
        LD      A, (HL)         ; X
        LD      C, A
        INC     HL
        LD      A, (HL)         ; Y
        INC     HL
        INC     HL              ; skip dir
        ; write Y first then X
        OUT     (VDPDATA), A    ; Y
        LD      A, C
        OUT     (VDPDATA), A    ; X
        LD      A, 1            ; pattern 1 (enemy sprite)
        OUT     (VDPDATA), A
        LD      A, COL_ENEMY
        OUT     (VDPDATA), A
        POP     BC
        DJNZ    DF_EN_LOOP

        ; Sprites 4-13: Items (stars), skip collected ones
        LD      HL, IT_TBL
        LD      B, 10
        LD      D, 4            ; sprite index (unused here, just sequential)
DF_IT_LOOP:
        PUSH    BC
        LD      A, (HL)
        CP      $FF
        JR      Z, DF_IT_SKIP
        ; visible item
        LD      C, A            ; C = item X
        INC     HL
        LD      A, (HL)         ; A = item Y
        DEC     HL
        OUT     (VDPDATA), A    ; Y
        LD      A, C
        OUT     (VDPDATA), A    ; X
        LD      A, 2            ; pattern 2 (star sprite)
        OUT     (VDPDATA), A
        LD      A, COL_ITEM
        OUT     (VDPDATA), A
        JR      DF_IT_CONT
DF_IT_SKIP:
        ; invisible: put sprite off-screen (Y=$D0=208)
        LD      A, $D0
        OUT     (VDPDATA), A
        XOR     A
        OUT     (VDPDATA), A
        OUT     (VDPDATA), A
        OUT     (VDPDATA), A
        INC     HL
DF_IT_CONT:
        INC     HL
        POP     BC
        DJNZ    DF_IT_LOOP

        ; Terminate sprite list
        LD      A, $D0
        OUT     (VDPDATA), A

        ; Load sprite patterns (only first time, but harmless every frame for simplicity)
        ; Actually we'll do it once in INIT_VDP via LOAD_SPRITES
        RET

; =============================================================================
; DRAW_HUD — print "STARS: N/10" at top of screen using name table
; =============================================================================
DRAW_HUD:
        ; Position: row 0, col 0 → name table $1800
        LD      HL, $1800
        CALL    SET_VRAM_ADDR
        ; Print "STARS:"
        LD      HL, STR_STARS
        CALL    PRINT_STR_VRAM
        ; Print score digit
        LD      A, (SCORE)
        ADD     A, '0'
        OUT     (VDPDATA), A
        ; Print "/10"
        LD      HL, STR_OF10
        CALL    PRINT_STR_VRAM

        ; Draw ground line at row 21 (y=168 → char row = 168/8 = 21)
        LD      HL, $1800 + 21*32
        CALL    SET_VRAM_ADDR
        LD      B, 32
DH_GND:
        LD      A, $DB          ; block char (or use $7E)
        OUT     (VDPDATA), A
        DJNZ    DH_GND
        RET

PRINT_STR_VRAM:
        LD      A, (HL)
        OR      A
        RET     Z
        OUT     (VDPDATA), A
        INC     HL
        JR      PRINT_STR_VRAM

STR_STARS:  DB  "STARS:", 0
STR_OF10:   DB  "/10", 0

; =============================================================================
; SHOW_TITLE
; =============================================================================
SHOW_TITLE:
        CALL    CLEAR_NAMETABLE
        ; Row 8, col 10 → offset = 8*32+10 = 266 = $010A
        LD      HL, $1800 + 8*32 + 9
        CALL    SET_VRAM_ADDR
        LD      HL, STR_TITLE
        CALL    PRINT_STR_VRAM
        LD      HL, $1800 + 12*32 + 6
        CALL    SET_VRAM_ADDR
        LD      HL, STR_PRESS
        CALL    PRINT_STR_VRAM
        RET

STR_TITLE:  DB  "LINEBOY MSX", 0
STR_PRESS:  DB  "PRESS SPACE TO START", 0

; =============================================================================
; WAIT_SPACE — block until space pressed
; =============================================================================
WAIT_SPACE:
        LD      A, ROW_CURSOR
        CALL    SNSMAT
        CPL
        BIT     BIT_SPACE, A
        JR      Z, WAIT_SPACE
        ; debounce: wait until released
WS_REL:
        LD      A, ROW_CURSOR
        CALL    SNSMAT
        CPL
        BIT     BIT_SPACE, A
        JR      NZ, WS_REL
        CALL    INIT_GAME
        LD      A, 1
        LD      (GM_ST), A
        CALL    CLEAR_NAMETABLE
        RET

; =============================================================================
; GAME_OVER_SCR
; =============================================================================
GAME_OVER_SCR:
        CALL    CLEAR_NAMETABLE
        LD      HL, $1800 + 10*32 + 9
        CALL    SET_VRAM_ADDR
        LD      HL, STR_GAMEOVER
        CALL    PRINT_STR_VRAM
        LD      HL, $1800 + 13*32 + 6
        CALL    SET_VRAM_ADDR
        LD      HL, STR_PRESS
        CALL    PRINT_STR_VRAM
        CALL    WAIT_SPACE
        JP      MAIN_LOOP

STR_GAMEOVER: DB "GAME OVER", 0

; =============================================================================
; CLEAR_SCR
; =============================================================================
CLEAR_SCR:
        CALL    CLEAR_NAMETABLE
        LD      HL, $1800 + 10*32 + 10
        CALL    SET_VRAM_ADDR
        LD      HL, STR_CLEAR
        CALL    PRINT_STR_VRAM
        LD      HL, $1800 + 13*32 + 6
        CALL    SET_VRAM_ADDR
        LD      HL, STR_PRESS
        CALL    PRINT_STR_VRAM
        CALL    WAIT_SPACE
        JP      MAIN_LOOP

STR_CLEAR:  DB  "STAGE CLEAR!", 0

; =============================================================================
; LOAD_SPRITES — write sprite patterns into pattern gen at $3800
; Sprite 0: Player (8×8 box)
; Sprite 1: Enemy  (8×8 X shape)
; Sprite 2: Item   (8×8 star)
; =============================================================================
LOAD_SPRITES:
        LD      HL, $3800
        CALL    SET_VRAM_ADDR
        LD      HL, SPR_DATA
        LD      BC, SPR_DATA_END - SPR_DATA
LS_LP:
        LD      A, (HL)
        OUT     (VDPDATA), A
        INC     HL
        DEC     BC
        LD      A, B
        OR      C
        JR      NZ, LS_LP
        RET

; Sprite 0: Player — solid box
SPR_DATA:
        DB      %01111110
        DB      %01111110
        DB      %01111110
        DB      %01111110
        DB      %01111110
        DB      %01111110
        DB      %01111110
        DB      %01111110

; Sprite 1: Enemy — X shape
        DB      %11000011
        DB      %01100110
        DB      %00111100
        DB      %00011000
        DB      %00011000
        DB      %00111100
        DB      %01100110
        DB      %11000011

; Sprite 2: Item — star shape
        DB      %00011000
        DB      %00111100
        DB      %11111111
        DB      %00111100
        DB      %00111100
        DB      %11111111
        DB      %00111100
        DB      %00011000
SPR_DATA_END:

; =============================================================================
; FONT_DATA — minimal 5×7 font packed as 8 bytes/char for chars $20-$5A
; (space + uppercase + digits + some punctuation)
; We use a compact hand-coded font.
; =============================================================================
FONT_DATA:
; $00-$1F: control chars (all zeros, 32 chars × 8 bytes = 256 bytes)
        DS      256, 0

; $20 space
        DB      0,0,0,0,0,0,0,0
; $21 !
        DB      $18,$18,$18,$18,$18,$00,$18,$00
; $22 "
        DB      $66,$66,$66,$00,$00,$00,$00,$00
; $23 #
        DB      $66,$FF,$66,$66,$FF,$66,$00,$00
; $24-$2E misc (use simple patterns)
        DS      11*8, 0
; $2F /
        DB      $01,$02,$04,$08,$10,$20,$40,$00
; $30 0
        DB      $3C,$66,$6E,$76,$66,$66,$3C,$00
; $31 1
        DB      $18,$38,$18,$18,$18,$18,$7E,$00
; $32 2
        DB      $3C,$66,$06,$1C,$30,$60,$7E,$00
; $33 3
        DB      $3C,$66,$06,$1C,$06,$66,$3C,$00
; $34 4
        DB      $06,$1E,$36,$66,$7F,$06,$06,$00
; $35 5
        DB      $7E,$60,$7C,$06,$06,$66,$3C,$00
; $36 6
        DB      $3C,$66,$60,$7C,$66,$66,$3C,$00
; $37 7
        DB      $7E,$06,$0C,$18,$30,$30,$30,$00
; $38 8
        DB      $3C,$66,$66,$3C,$66,$66,$3C,$00
; $39 9
        DB      $3C,$66,$66,$3E,$06,$66,$3C,$00
; $3A :
        DB      0,$18,$18,0,$18,$18,0,0
; $3B-$3F misc
        DS      5*8, 0
; $40 @
        DS      8, 0
; $41 A
        DB      $18,$3C,$66,$66,$7E,$66,$66,$00
; $42 B
        DB      $7C,$66,$66,$7C,$66,$66,$7C,$00
; $43 C
        DB      $3C,$66,$60,$60,$60,$66,$3C,$00
; $44 D
        DB      $7C,$66,$66,$66,$66,$66,$7C,$00
; $45 E
        DB      $7E,$60,$60,$7C,$60,$60,$7E,$00
; $46 F
        DB      $7E,$60,$60,$7C,$60,$60,$60,$00
; $47 G
        DB      $3C,$66,$60,$6E,$66,$66,$3C,$00
; $48 H
        DB      $66,$66,$66,$7E,$66,$66,$66,$00
; $49 I
        DB      $7E,$18,$18,$18,$18,$18,$7E,$00
; $4A J
        DB      $06,$06,$06,$06,$06,$66,$3C,$00
; $4B K
        DB      $66,$6C,$78,$70,$78,$6C,$66,$00
; $4C L
        DB      $60,$60,$60,$60,$60,$60,$7E,$00
; $4D M
        DB      $63,$77,$7F,$6B,$63,$63,$63,$00
; $4E N
        DB      $66,$76,$7E,$7E,$6E,$66,$66,$00
; $4F O
        DB      $3C,$66,$66,$66,$66,$66,$3C,$00
; $50 P
        DB      $7C,$66,$66,$7C,$60,$60,$60,$00
; $51 Q
        DB      $3C,$66,$66,$66,$6E,$3C,$06,$00
; $52 R
        DB      $7C,$66,$66,$7C,$6C,$66,$66,$00
; $53 S
        DB      $3C,$66,$60,$3C,$06,$66,$3C,$00
; $54 T
        DB      $7E,$18,$18,$18,$18,$18,$18,$00
; $55 U
        DB      $66,$66,$66,$66,$66,$66,$3C,$00
; $56 V
        DB      $66,$66,$66,$66,$66,$3C,$18,$00
; $57 W
        DB      $63,$63,$63,$6B,$7F,$77,$63,$00
; $58 X
        DB      $66,$66,$3C,$18,$3C,$66,$66,$00
; $59 Y
        DB      $66,$66,$66,$3C,$18,$18,$18,$00
; $5A Z
        DB      $7E,$06,$0C,$18,$30,$60,$7E,$00
FONT_DATA_END:

; =============================================================================
; Pad to 16KB ROM size
; =============================================================================
        DS      16384 - ($ - $4000), $FF
