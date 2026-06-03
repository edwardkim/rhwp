# Stage 1 보고서 — Task #1070: 조사 + 정밀 조건 도출

- 브랜치: `local/task1070` (stream/devel `be2a71c4` 기준)

## 근본 원인 확정

`place_table_with_text`(`typeset.rs:2517`) 의 `post_table_start`(2622) 산식:
```rust
} else if table.attr & 0x01 != 0 {   // HWP5 TAC → pre_end.max(1) (표줄 제외)
    pre_table_end_line.max(1)
} else if is_last_table && !is_first_table { 0 }
else { pre_table_end_line }          // ← HWPX TAC(비트0=0) 가 여기로
```
HWPX TAC 표는 `attr & 0x01 == 0` → 마지막 else → `post_table_start = pre_table_end_line(=0)`.
표줄(line0)이 post-text PartialParagraph(0..total_lines)에 포함 → 표줄 높이만큼 본문 줄이
하강 → overflow.

DIAG_1070 (재현):
```
pi=25 tac_attr=false total_lines=2 pre_end=0 post_start=0 ... includes_table_line=true
pi=51 tac_attr=false total_lines=2 pre_end=0 post_start=0 ... includes_table_line=true
```
HWP5 TAC(attr&0x01)는 `pre_end.max(1)` 로 이미 표줄을 제외 → 본 결함은 **HWPX 한정 갭**.

## 회귀 구조 규명 (실험)

`includes_table_line=true` 신호는 재현 파일뿐 아니라 #1068 회귀세트(sample16/aift/mel-001/
tac-img-02)의 **다수 단일줄(total_lines=1) TAC 표 문단**에서도 발동. 단순 blanket 확장이
회귀하는 이유.

### 실험 A — blanket (`attr&0x01 || treat_as_char`)
| 파일 | baseline | A |
|------|----------|---|
| 재현 3 | 472/348/348 | 5.6/7.4/7.4 ✅ |
| mel-001 | 3 | **8 (+5)** ❌ |
| tac-img-02.hwpx | 7 | **8 (+1)** ❌ |
| sample16-hwp5 | 35/249.5 | 20/253.3 (max +3.8) |

→ 단일줄(total_lines=1) TAC 표는 `post_start=pre_end.max(1)=1`, `total_lines(1)>1` 거짓 →
post-text 제거 → mel-001 회귀. 단일줄 표는 **건드리면 안 됨**.

### 실험 B — 정밀 조건 (`treat_as_char && total_lines > pre_end + 1`)
표줄 **다음에 실제 본문 줄이 있을 때만** 표줄 제외(단일줄 불변).
```rust
} else if table.attr & 0x01 != 0 {
    pre_table_end_line.max(1)
} else if table.common.treat_as_char && total_lines > pre_table_end_line + 1 {
    pre_table_end_line + 1
} else if is_last_table && !is_first_table { 0 }
else { pre_table_end_line };
```
| 파일 | baseline | B |
|------|----------|---|
| 재현 3 | 472/348/348 | **5.6/7.4/7.4** ✅ |
| sample11-hwpx | 0 | 0 |
| tac-img-02.hwpx | 7 | **6** ✅개선 |
| tac-img-02.hwp | 8 | 8 |
| sample16-hwp5 | 35 | **15** ✅개선 |
| aift.hwpx | 5 | **4** ✅개선 |
| mel-001.hwpx | 3 | 3 ✅불변 |

## 검증 (정밀 조건 B)

- **전수 sweep**(samples hwp/hwpx, overflow 97파일): baseline 3057 lines / 382815px →
  **3043 lines / 376707px** (−14 lines, −6108px, 회귀 0·개선).
- 골든 SVG **8/8**, cargo test --release lib **1324 passed**, clippy/fmt clean.

## 결론 / 다음
정밀 조건 B 가 재현 3파일 해소 + 4파일 개선 + 회귀 0 으로 검증됨. Stage 2(설계 페이퍼 검증
— tac_wrap_split/다중표/Square wrap 경계 모순 점검) → Stage 3(현 소스 확정) → Stage 4(공개
픽스처 회귀 가드 + 최종 검증).
