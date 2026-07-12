# #2015 Stage 2 findings — 부동 RowBreak 표 91.2px 오버플로우 근본원인 확정

- 이슈: #2015 / 브랜치: `fix/2015-saved-bounds-rowbreak-overflow`
- 범위: 소스 **무수정**. 오차 산식을 진단 트레이스로 확정 + 수정 후보/검증 판단.

## 1. 진단 수치 (RHWP_TABLE_DRIFT=1, pi=52)

```
첫 fragment: cursor_row=0 cont=false cur_h=806.0 table_avail=930.5
             host_before=0.0 vert_off=144.3 page_avail=0.0 avail_for_rows=0.0
연속:        cursor_row=2 cont=true  cur_h=0.0  page_avail=930.5 start_cut=[1]
```

`page_avail = table_avail − cur_h − vert_off = 930.5 − 806.0 − 144.3 = −19.8 → max(0)=0.0`

RHWP_CUT_DBG: row=2 를 `avail=0.0` 로 컷 → "진행보장 1유닛 강제소비" → 앵커(cur_h+vert_off=950.3px,
body 바닥 930.5 아래)에서 fragment 가 아래로 그려져 절대 1117.7px, body 바닥 1026.5px **초과 91.2px**.

## 2. 근본원인 — vert_offset 이중계상 (host pre-emit 상호작용)

`typeset.rs::pre_emit_visible_rowbreak_host_text`(#1811, ~12203):
```
let host_h = host_fmt.line_advances_sum(0..host_lines);  // ≈146px (4줄)
st.current_items.push(PartialParagraph{...});
st.current_height += host_h;                             // cur_h 를 para_start → para_start+host_h 로 전진
```

메인 분기(~13206 `vert_offset_overhead`, ~13264 `page_avail`):
```
page_avail = table_avail − st.current_height − vert_offset_overhead
           = table_avail − (para_start+host_h) − vert_off
```

- `vert_off`(144.3px)는 **para_start 기준** 표 오프셋.
- host 텍스트 pre-emit 가 이미 cur_h 를 para_start → para_start+host_h(≈146px)로 전진.
- 표의 참 위치 = para_start + vert_off. 현재 cur_h(=para_start+host_h) 기준 오프셋 = **vert_off − host_h ≈ 144.3 − 146 ≈ 0**.
- 그런데 산식은 full `vert_off` 를 다시 빼서 host_h 만큼 **이중차감** → page_avail 이 0 으로 붕괴, 앵커가 host_h 만큼 아래로 밀림.
- host_h ≈ vert_off (둘 다 para_start~표 사이 동일 구간)라 오차가 정확히 이 크기.

즉 **host 를 pre-emit 한 뒤에는 vert_offset_overhead 를 `max(0, vert_off − host_h)` 로 줄여야** 한다.
`ir-diff` 차이 0(포맷 공통)과도 정합 — 파싱이 아니라 typeset↔layout 배치 산식 문제.

## 3. 수정 후보

1. (typeset 예산) `vert_offset_overhead` 를 host pre-emit 시 `host_h` 만큼 감액
   (`st.pre_emitted_host_paras.contains(&para_idx)` 가드).
2. (layout 배치) 표 y 를 `cur_h + host_before + vert_off` 대신 pre-emit 케이스에서 동일 감액 —
   typeset 컷과 layout 배치가 **정의상 일치**해야 하므로 두 경로 동시 정합 필수(안 하면 fit 은 통과하나 그림은 여전히 초과).

## 4. 검증 한계 (판단 필요)

- 이 수정은 #1811 pre-emit + saved-bounds 코퍼스와 얽혀, 표 y 와 컷 경계가 함께 이동한다.
- 본 환경엔 시각 오라클 래스터 도구(`rsvg-convert`/`pdftoppm`) 부재 → visual sweep 픽셀 검증 불가.
- 프로젝트 방법론상 **시각 판정이 최종 게이트**이고 saved-bounds 다수 샘플 회귀가 필수.
- 따라서 엔진 편집은 (A) `cargo test` 전량 + 신규 #2015 불변식 + 전 샘플 `LAYOUT_OVERFLOW` 스캔으로
  1차 검증 후, 시각 판정은 오라클 보유 환경(메인테이너)에서 확정하는 절차를 권장.

## 5. 다음 단계 (Stage 3 예정)

`vert_offset_overhead` 감액 + layout 배치 정합 구현 → #2015 불변식 통과(`#[ignore]` 해제) →
`cargo test --profile release-test --tests` 전량 + 전 샘플 overflow 스캔 회귀 → 시각 판정.
발원지 ②(HWPX 인라인 표 합성 줄 피치)는 ① 안정화 후 별도 단계.
