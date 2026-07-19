---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1239/README.md
last_verified: 2026-07-16
---

# 조사 보고서 — Task #1239 Stage 1: 미주 인라인 수식 줄 병합 원인

작성일: 2026-06-02
대상: #1239 — 13쪽 문20 S= 블록 두 수식 줄 병합

## 1. 증상

문20 풀이 "S = ∫… = … = … = 17" 정렬 블록에서 PDF 는 5줄 분리인데 rhwp 는 줄2/줄3 사이가
비고(공백) 줄3에 두 수식("=∫₀²(-2t³+8t)dt", "=[-½t⁴+4t²]…")이 수평 병합.

## 2. 구조 (확인)

- S= 블록 = **단일 미주 문단 pi=602** (텍스트 "(빈)", composed 줄 runs=0 = 인라인 수식만).
- LINE_SEG 5개, vpos 균등(361142/363892/366642/369392/372239) — 정상.
- 인라인 수식(treat-as-char) tac_ci 별 배정 줄(`DBG_EQM2b` 진단):

| tac_ci | line_idx | tac_pos | 수식 |
|--------|----------|---------|------|
| 0,1 | 0 | 0 | S, =∫₀³ (1줄 정상) |
| 2 | 1 | 1 | =∫₀² (정상) |
| — | **2** | — | **빈 줄(공백)** |
| **3,4** | **3** | **2** | =∫₀²(-2t³+8t)dt + =[-½t⁴…] **(병합)** |
| 5 | 4 | 3 | =17 |

→ tac_ci 3·4 가 **같은 tac_pos=2** → 같은 줄(line 3). 2·3 이어야 정상.

## 3. 근본 원인 (특정)

`src/model/paragraph.rs::control_text_positions` (L848-868):

```rust
if next_off > current_off + char_width {
    let gap = next_off - current_off - char_width;
    let n_ctrls = gap / 8;            // 한 char-gap 의 컨트롤 수
    for _ in 0..n_ctrls {
        positions.push(i + 1);        // ← 모두 같은 position(i+1)
    }
}
```

- char_offsets 갭(8단위=1컨트롤)으로 컨트롤 위치를 복원. **한 갭에 여러 컨트롤이 있으면
  모두 같은 char position(i+1)** 을 받는다.
- 문20 S= 블록의 eq3·eq4 는 **사이에 텍스트 char 가 없는 연속 수식**(한컴이 LINE_SEG 경계로
  별도 줄에 둠, 텍스트 강제 줄나눔). char_offsets 갭은 둘을 한 위치(2)로 복원.
- 줄 배정 `tac_offsets_for_line`(`char_pos_in_line(pos, line.char_start, end)`)은 **char 위치
  기준**이라 같은 pos 의 두 수식을 같은 줄에 배정 → 병합. LINE_SEG 경계(텍스트 char 없는
  강제 줄나눔)를 보지 못함.

## 4. 핵심

**연속 인라인 수식이 한컴 LINE_SEG 로 별도 줄에 배치된 경우, rhwp 는 char 위치만으로 줄을
배정해 같은 줄로 병합한다.** (char_offsets 갭에 텍스트 char 가 없어 위치 구분 불가.)

## 5. 회귀 여부

- `control_text_positions` 는 **편집/커서/렌더 전반에서 공유**되는 핵심 함수 → 직접 수정은
  광범위 영향. char position 의미(편집 위치)는 유지하고, **줄 배정(렌더)에서 LINE_SEG 로
  연속 수식을 분배**하는 방향이 안전.

## 6. Stage 2 제안

- 줄 배정 단계(`tac_offsets_for_line`/렌더)에서, **같은 char position 의 연속 TAC 가 남은
  LINE_SEG 수보다 많을 때 LINE_SEG 경계로 분배**. 또는 composer 에서 인라인 수식의 LINE_SEG
  소속을 별도 계산(line_segs 의 vpos/text_start ↔ 수식 매핑).
- 일반 문단(연속 수식 1줄 배치가 정상인 경우) 회귀 방지 게이트 + 골든 스냅샷 가드.
