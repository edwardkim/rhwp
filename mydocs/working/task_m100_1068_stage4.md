# Stage 4 보고서 — Task #1068: 진짜 근본 원인 규명 + 수정 + 회귀 검증

- 브랜치: `local/task1068`
- 수정 파일: `src/document_core/commands/document.rs` (1 곳)

## 승인된 재설계(L2750 carry)는 무효 — 진단으로 확정

Stage 3 에서 제시·승인된 재설계(**block 경로 L2750 carry 가드를 다행 TAC 표로 확장**)를
구현하던 중, 진단으로 그 전제가 틀렸음을 확정:

- `DIAG_1068` (para 567): `tac=true rows=14 table_total=860.7 available=941.1 cur_h=29.5
  fits_fresh=true`.
- `cur_h(29.5) + table_total(860.7) = 890.2 ≤ 941.1` → **L2736 에서 현재 페이지에 정상 fit**.
  L2750 은 도달조차 안 함 → carry 확장은 para 567 에 **영향 0** (실측: overflow 불변).

## 진짜 근본 원인 — 파서 후처리 over-inflation (`document.rs:283`)

HWPX XML 원본 대조 (`Contents/section0.xml`, para 567 linesegarray):
```
ls[0]: textpos=0,  vertsize=2200   ← 제목줄 ("제안(서) 평가항목 및 배점기준")
ls[1]: textpos=18, vertsize=63234  ← 표 줄 (treat_as_char 표, char 18)
```

그러나 파싱된 IR (`dump -s 0 -p 567`): `ls[0] lh=63234` (제목줄 lh 가 표 높이로 오염).
`th=2200` 은 정상 보존 → vertsize→line_height 매핑(`parse_lineseg_element`)은 정상,
**후처리가 오염**.

오염 지점 — `document_core/commands/document.rs` "HWPX TAC 표 lh 보정":
```rust
if let Some(seg) = para.line_segs.first_mut() {   // ← 무조건 첫 줄
    if seg.line_height < max_tac_h { seg.line_height = max_tac_h; }
}
```
의도: linesegarray 가 없어 기본 lh=100 단일 seg 만 생성된 경우 표 높이로 확대.
결함: **표가 둘째 줄 이후에 있는 문단**(제목줄 + 표줄)에서 무조건 `first_mut()`(제목줄)을
표 높이로 확대 → 제목줄 lh 2200 → 63234.

연쇄: 렌더러 `place_table_with_text:2548` 의 **lh 기반 표 줄 탐지**(find by line_height ≈
table height) 가 제목줄(idx 0)을 오매칭 → `pre_table_end_line=0` → 표 줄이 post-text 로
포함 → `PageItem::Table` + `PartialParagraph(표줄 포함)` 이중 그리기 → page used 1761px,
표 줄 y=1886 → **839px overflow**.

## 수정 — 보정 조건 정밀화 (최소 변경)

```rust
let already_covered = para.line_segs.iter().any(|s| s.line_height >= max_tac_h);
if !already_covered {
    if let Some(seg) = para.line_segs.first_mut() { ... }
}
```
이미 표 높이를 담은 LINE_SEG 가 있으면(한컴이 저장한 실제 linesegarray) 보정 생략.
linesegarray 가 없는 synthetic 단일 seg(lh=100) 케이스만 기존대로 확대 → 의도 보존.

라우팅·렌더러 미변경 → Stage 2 라우팅 정규화(반려, +44)·실험 B(블록 경로 attr↔treat_as_char
동일시: sample16/aift/mel-001 회귀)와 달리 **회귀 면 최소**.

## 검증 (전수)

- **타깃**: 제안요청서 para 567 **839px → 해소** (잔여 max 29px 는 #1068 무관 기존 드리프트).
- **회귀 후보 6 파일** (baseline → patched):

| 파일 | baseline | patched | 판정 |
|------|----------|---------|------|
| hwp3-sample11-hwpx | 0 | 0 | 불변 |
| tac-img-02.hwpx | 7 (max20) | 5 (max8.8) | 개선 |
| tac-img-02.hwp | 8 (max23.6) | 8 | 불변 |
| hwp3-sample16-hwp5 | 35 (max249) | 35 | 불변 |
| aift.hwpx | 5 (max18.2) | 5 (max9.6) | 개선 |
| mel-001.hwpx | 3 | 3 | 불변 |

- **전수 sweep** (samples 281 파일 중 hwp/hwpx, overflow 보유 97 파일):
  - baseline: 3057 lines / 382815px → patched: **3055 lines / 381115px** (−2 lines, −1700px, 회귀 0).
- **cargo test --release**: lib **1324 passed** + 통합 테스트 0 failed.
- **골든 SVG 8 종**: 8/8 통과 (form-002 / table-text / issue-157 / issue-267 / issue-147 /
  issue-617 / issue-677 / determinism).
- **clippy**: clean. **fmt**: clean (변경 파일).

## 다음
WASM 빌드(rhwp-studio 동기화) + 작업지시자 한컴 시각 판정 → 최종 보고서 + 이슈 클로즈.
