---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1238/README.md
last_verified: 2026-07-16
---

# 조사 보고서 — Task #1238 Stage 1: 미주 between-notes margin 누락 원인

작성일: 2026-06-02
대상: #1238 — 14쪽 문22 제목이 직전 다줄 풀이에 붙음

## 1. 증상 (실측)

SVG 텍스트 y좌표를 열(2단) 분리해 제목 above-gap 측정:

| 제목 | 직전 줄 | above-gap | 정상? |
|------|---------|-----------|-------|
| 문22 | 문21 풀이 마지막(다줄) "…합은 이다." | **11.3px** | ✗ (붙음) |
| 문24 | "이다."(단일줄) | 37.8px | ✓ |
| 문25 | (단일줄) | 26.9px | ✓ |
| 문23 | (단일줄) | 35.3px | ✓ |

between_notes(미주 사이) = 1984 HWPUNIT ≈ 26px. 정상 제목은 line_height + 26px 확보,
문22 만 26px 누락(11.3px) → 직전이 **다줄 문단**일 때만 발생.

## 2. between-notes 적용 메커니즘 (진단 로그 실측)

`typeset.rs:2031-2061`. 임시 로그(`DBG_BN1238`)로 전 경계 실측:

```
between_notes=1984  prev_spacing=452  extra_gap=1532  pag_base=1984
```

- `pagination_gap = between_notes - BASE_FLOW_HU = 1984 - 1984 = 0` → **vpos_offset 미증가**.
- 즉 between-notes 는 **오직 `prev_para.line_segs.last_mut().line_spacing = 1984`** 주입으로만 적용.
  (직전 미주의 마지막 문단, 마지막 LINE_SEG 의 line_spacing 을 1984 로 덮어씀.)
- 모든 문제-경계가 이 한 메커니즘에 의존한다(단일/다줄 무관 주입은 동일).

→ **가설(extra_gap=0) 은 반증**. 주입은 항상 일어난다. 문제는 **render 가 주입값을 무시**하는 것.

## 3. 근본 원인 (특정)

`src/renderer/layout/paragraph_layout.rs`:

- `endnote_line_vpos_base`(L1372-1389): **다줄 미주 문단**(`end > start_line + 1`)에서만 설정.
  줄 위치를 stored LINE_SEG vpos 델타로 배치.
- 줄 advance(L4152-4167, 이 경로):

```rust
let trailing = if line_idx + 1 < end { line_spacing_px } else { 0.0 };  // ← 마지막 줄 trailing 버림
y + line_height + trailing + tac_picture_label_extra;
```

- **다줄 문단의 마지막 줄은 trailing = 0.0** → 주입된 last_seg.line_spacing(=1984) 이
  다음 문단(문22 제목) 시작 y 에 **반영되지 않음**.
- 단일줄 미주 문단은 이 경로를 타지 않고(L4173) `y += line_height + line_spacing_px` 로
  line_spacing(=주입 1984)을 포함 → between-notes 정상 확보.

→ **다줄 문단으로 끝나는 미주만 between-notes 가 누락**된다. (단일줄 끝은 정상.)

## 4. stored vpos 무관 확인

dump-pages: pi=631(문21 마지막) vpos=…400228, pi=632(문22) vpos=444925 (stored gap 큼).
그러나 render 는 다줄 경로에서 stored vpos 를 para 내부 델타로만 쓰고 **다음 문단 시작 y 는
이전 문단 마지막 줄 bottom(trailing 제외)** 으로 잡으므로 stored gap 은 반영 안 됨 → 11.3px.

## 5. 회귀 안전 게이트 (Stage 2 방향)

- 기존 시도(last_seg 무조건 trailing)는 issue_1139(3-09월)·1189 **이중 가산 회귀**.
- between_notes 주입은 typeset.rs 에서 **"다음 미주가 존재할 때 직전 미주의 마지막 문단"**
  에만 발생(`emitted_endnote_count > 0`). 따라서 **정확히 그 경우에만** trailing 복원:
  - 신규 헬퍼 `endnote_para_has_different_endnote_successor(para_index)` —
    `endnote_para_sources` 의 (section_index, para_index, control_index) 가 다음 local 과 다르면 true.
  - L4156: `else if 헬퍼 { line_spacing_px } else { 0.0 }`.
- 이 게이트는 **다른 미주가 뒤따르는 마지막 문단**(=주입 발생 위치)에만 trailing 을 복원 →
  같은 미주 내 문단·문서 마지막 미주는 무영향. 단일줄 경로(L4173)는 손대지 않음 → 이중가산 없음.

## 6. #1236(PR #1240) 와의 관계

- 본 건은 stream/devel 분기(아직 #1236 미머지). L4156 는 현재 `line_idx + 1 < end`.
- #1236 머지 시 같은 라인이 `... || same_endnote_successor` 로 바뀜(미주 **내부** 문단 trailing).
- 본 건 게이트는 **다른 미주 successor**(미주 **사이**) 로 의미가 직교 → 머지 시 OR 결합 가능.
  충돌 시 두 조건을 `||` 로 병합.

---

## 7. Stage 1 재개 (2026-06-02) — §5 가산 모델 반증 + min-gap 모델 특정

§5 의 "trailing 복원" 가산(additive) 접근은 **두 변형 모두 회귀 가드 4건을 깨뜨려 반증됨**:

| 시도 | 결과 |
|------|------|
| (A) blanket `endnote_para_has_different_endnote_successor` | 가드 4건 실패 |
| (B) `pagination_gap == 0` 집합 게이트 | **(A)와 완전 동일한 실패값** |

(B)가 (A)와 한 글자도 다르지 않은 결과 → **세 문서 모두 해당 경계에서 `pagination_gap == 0`**.
즉 `pagination_gap` 은 "수정 대상"과 "회귀 금지"를 **구분 못 하는 판별자**다. §2 의 단일 메커니즘
전제가 불완전했다.

### 7.1 실제 렌더 y 실측 (baseline, 코드 무변경)

`build_page_render_tree` + `min_para_text_y`/`max_para_content_bottom` (가드 테스트와 동일 측정):

| 문서 | 경계 (prev→문N) | prev_bottom | 문N top | **rendered gap** | 판정 |
|------|----------------|-------------|---------|------------------|------|
| 3-11월 (target) | 631(다줄)→632 문22 | 457.9 | 457.9 | **0.0** | 버그 (7mm 필요) |
| 3-09월 (guard) | 664(다줄)→665 문15 | 872.6 | 899.0 | **26.5** | 정상 (≈7mm) |
| 3-10월 (guard) | 573→574 문19 | 678.5 | 731.4 | **52.9** | 정상 (>7mm) |
| 3-10월 (guard) | 568(다줄)→569 문18 | 512.4 | 512.4 | **0.0** | §7.3 참조 |

7mm = 1984 HU ≈ **26.5px** (96dpi).

### 7.2 결론 — 가산(additive) 이 아니라 min-gap(클램프) 모델

between-notes 는 "마지막 줄에 trailing 을 **더한다**"가 아니라 **"미주 사이 간격을 최소 7mm 로
보장한다"**(자연 간격이 이미 7mm 이상이면 변경 없음). 가산 모델이 회귀한 이유:

- 3-09월: 26.5 + 26.5 = 53 → 과대 (테스트 [24,32] 위반).
- 3-10월: 52.9 + … = 누적 → overflow / q간격 과대.

min-gap 모델 = **`gap := max(natural_gap, 7mm)`**:

| 문서 | natural gap | max(·, 26.5) | 효과 |
|------|-------------|--------------|------|
| 3-11월 문22 | 0.0 | **26.5** | ✅ 수정 |
| 3-09월 문15 | 26.5 | 26.5 | 변경 없음 ✅ |
| 3-10월 문19 | 52.9 | 52.9 | 변경 없음 ✅ |

세 케이스 동시 정합. 가산이 실패한 지점을 정확히 회피한다.

### 7.3 잔여 리스크 — 3-10월 문18 (gap=0.0) + 누적 overflow

- 문18(pi=569)은 실제 미주-사이 경계인데 baseline gap=0.0. **문18 풀이가 빈 문단 4개
  (pi=570~573)뿐**이라, 3-10월 테스트는 인접 gap 이 아니라 **제목간 거리**(q18→q19 ∈ [205,235])
  만 검사 → 문18 블록 전체가 균일히 아래로 밀리면 거리 보존.
- 그러나 (a) min-gap 이 문18 을 7mm 밀 때 **한컴 PDF 에 실제 그 간격이 있는지**, (b) 결손 경계가
  여러 개면 **누적 밀림이 페이지 하단 overflow** 를 일으키는지 — **Stage 2 검증에서 가드 4건 +
  골든으로 직접 확인**해야 한다. 가드가 PDF 캘리브레이션값([205,235] 등)을 담고 있어 self-validating.

### 7.4 구현 지점 후보

가산 trailing(L4156)이 아니라, **새 미주의 첫 문단 진입 시** 시작 y 를
`max(incoming_y, prev_endnote_last_content_bottom + between_notes_px)` 로 클램프.
→ render(`paragraph_layout`)에 "직전 미주 마지막 content bottom" 상태 + "새 미주 첫 문단" 신호 필요.
typeset 이 새 미주 첫 문단 local idx 집합을 넘기는 방식(§5 (B)의 배선 재사용 가능). 상세는 impl v2.
