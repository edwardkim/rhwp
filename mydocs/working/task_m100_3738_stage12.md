---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 12 — native HWP5 각주 reset의 실제 영역 경계 보정

## 출발점

Stage 11 commit `1247793f0`은 HWP p67의 `FootnoteArea` reservation/paint 불일치와 footer
collision을 해소했다. 실제 HWP 쪽수는 224쪽으로 유지되고, HWPX도 224쪽인 반면 한컴오피스 2020
기준 PDF는 215쪽이다. 원본 HWP/HWPX, 기준 PDF, Stage 11 PNG는
[증적 보관 목록](../../pdf/pr3740/README.md)과
[Stage 11 visual sweep](task_m100_3738_stage11_visual_sweep.md)에 보관한다.

## 이번 Stage의 판정 순서

1. PDF와 rhwp의 physical page text/signature를 p68부터 순서대로 비교해, 동일 본문 anchor가 처음
   다른 physical page에 나타나는 지점을 찾는다.
2. 해당 page pair의 HWP stored `LINE_SEG`, table/picture/footnote ownership과 rhwp render tree를
   함께 대조한다. 단순 전체 쪽수 차이만으로 원인을 단정하지 않는다.
3. 독립적인 한 경로가 확인되면 그 형상만 보정하고 focused regression과 visual sweep을 남긴다.
4. 잔여가 있으면 커밋 후 새 Stage의 분석 문서로 분리한다.

Stage 9–11에서 복원한 p66 table 23 fragment 및 p67 각주 78–85 ownership은 이번 탐색의 기준점이며
새 변경으로 되돌리지 않는다.

## 1차 실측 — p30–p32와 p68–p69

- p30 문단 407은 표 8 직후 5줄 본문과 각주 29를 함께 가진 native HWP5 문단이다. 저장
  `LINE_SEG`의 line 3은 `vpos=0`이며, line 2의 bottom은 body height의 94%다. 기존 rhwp는
  다섯 줄을 모두 p30에 그려 line 3–4(`y=998.3/1025.0px`)가 `FootnoteArea y=990.1px`와
  겹쳤다. PDF는 앞 세 줄만 p30에 두고 나머지 두 줄을 p31로 넘긴다.
- p68은 PDF 하단의 그림 49를 rhwp가 p69 단독 페이지로 이월하는 별도 분기다. p68 이후의
  1쪽 shift를 만드는 독립 형상으로, p30–p32 각주/reset 경로와 한 수정으로 묶지 않는다.

## 기각한 1차 후보

native HWP5의 “각주-only control + body 하단 85% 이후 `vpos=0`”을 강제 page split으로 처리해
봤다. p30 신호는 잡았지만 같은 서명을 가진 다른 문단까지 분리돼 전체 HWP가 224→226쪽이 되고,
Stage 9 p66–p67 regression의 `page_count() <= 224`도 실패했다. 이 후보 코드는 커밋하지 않고
되돌린다.

## 구현 — reset 신호가 아니라 실제 첫 각주 영역과의 교차만 분리

`src/renderer/typeset.rs`에
`native_hwp5_first_footnote_overlap_break_line`을 추가해, 다음을 **동시에** 만족할 때에만
저장 reset 위치에서 문단 tail을 다음 physical page로 넘긴다.

1. native HWP5이고 현재 페이지의 첫 각주이며, 아직 예약된 각주 높이가 없다.
2. 문단 control이 단 하나의 `Footnote`이고 본문에 실제 글자가 있다.
3. reset 직전 줄의 visible bottom은 composer가 계산한 실제 `FootnoteArea` 상단 안에 남지만,
   그 줄의 trailing line-spacing은 그 상단을 넘는다.

각주 높이는 stored `LINE_SEG`가 아니라 renderer와 같은 `compose_paragraph` 결과의 line-height와
line-spacing을 합산한다. 마지막 각주 문단의 마지막 줄만 paint와 동일하게 trailing spacing을
더하지 않는다. 따라서 넓은 native-HWP5 reset 해석이 아니라, p30처럼 **보이는 본문이 실제 각주
영역을 침범하는** 경계에만 발동한다.

이 보정 뒤 p30은 `10년 후 71.7%`까지로 끝나고, p31은 `문제가 나타남` tail 뒤 `5. 독일`로
이어지며, p32는 `35>와 같이 점차 감소하는 추세임` 뒤 그림 35를 유지한다. Stage 9의 p66–p67
table-footnote fragment와 224쪽 page count도 focused regression으로 함께 유지했다.

## 검증 및 시각 증적

```bash
CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 \
  cargo test --profile release-test \
  --test issue_3738_rowbreak_table_footnote_fragment \
  --test issue_3738_hwp_caption_cell_alignment

CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 \
  cargo build --profile release-test --bin rhwp

python3 scripts/visual_sweep.py \
  --key issue3738-stage12-hwp-p030-p032-fixed \
  --hwp 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --pages 30-32 --dpi 144 \
  --rhwp-bin target/review-planet6897-20260802/release-test/rhwp
```

두 test executable에서 3개 test가 통과했고, sweep은 SVG/render tree 224쪽과 선택 raster p30–p32
3/3을 완료했다. 자동 structural 후보는 0건이며 overlay 평균 pixel match는 90.87502%, 평균 ink
proxy는 37.25284%다. 글꼴·차트 raster 차이를 포함하는 후자는 완료 단독 판정에 사용하지 않았다.
페이지별 근거와 사람 검토는 [Stage 12 visual sweep](task_m100_3738_stage12_visual_sweep.md)에 둔다.

## 잔여 분기 — p68 그림 49는 다음 Stage로 분리

p68의 문단 749는 그림 49와 caption을 담은 2×1 non-TAC `TopAndBottom` `RowBreak` 표다. 현재 p68
사용 높이는 `586.67px`이고, 첫 picture row의 보수적 필요량은 남은 budget보다 `2.6px` 크다고
계산돼 table 전체를 p69로 미룬다. 그러나 실제 렌더 geometry에서 table bottom은 `887.8px`,
`FootnoteArea` top은 `898.1px`로 약 `10.3px` 남는다. PDF는 그림 49를 p68 하단에 둔다.

이는 문단 reset이나 각주 paint 충돌이 아닌 RowBreak table의 near-fit safety 계산 문제다. 이 Stage에서
같이 넓히지 않았고, p68–p69 비교·입력 구조·render tree는 다음 커밋 뒤 Stage 13에서 독립적으로
분석한다. 따라서 전체 HWP/HWPX 224쪽과 PDF 215쪽의 차이를 해소했다고 주장하지 않는다.
