# Task #3032 — HWP3 footnote_between_margin IR 배선 (PR #3036 인수·보완 보고서)

`Closes #3032`. kevin9327의 PR #3036 발견을 메인테이너가 인수해 적용처·스케일을 정정한 기록.

## 경위

- PR #3036: `doc_info` offset 108 "각주와 각주 사이의 간격"(`footnote_between_margin`)이
  파싱만 되고 IR 미배선임을 발견, **미주(endnote_shape)** `raw_unknown` 에 raw 값 배선.
- CI 실패: `issue_1692` PDF 쪽범위 골든(43쪽 마지막 미주 58→53). 메인테이너 시각 대조로
  미주 간격이 벌어진 것을 확인. 기여자는 스펙·바이트·소비처 추적 코멘트 회신(성실 조사,
  단 ×4 누락·적용처 판단이 실측과 상이).
- 현 devel(#3225 시그니처 통일)과 CONFLICTING 상태이기도 하여 close+통합 경로로 인수.

## 판정 근거 — 한컴 자체 변환이 정답지

작업지시자 확인: `samples/SO-SUEOP.hwpx` 는 같은 .hwp 를 **한컴 편집기에서 열어 HWPX 로
저장**한 파일 — 한컴 암묵지가 명시된 1차 권위 자료다. 실측:

| HWPX | betweenNotes | aboveLine | belowLine |
|---|---:|---:|---:|
| footNotePr(각주) | **284** (=71×4) | 852 (=213×4) | 568 (=142×4) |
| endNotePr(미주) | **0** | 864(기본) | 576(기본) |

- **×4 확정**: HWP3 hunit → HWPUNIT 은 이웃 필드(#2772/#3054)와 동일한 ×4.
- **적용처 확정**: offset 108 은 **각주 전용**. 미주는 0 — 원 PR 의 endnote 배선이 골든을
  깬 이유가 이것이며, 실제로 endnote ×4(284) 배선 실험도 골든 red 를 재현했다.

## 구현

- `fixup` 의 section_def 조립부에서 `footnote_shape.raw_unknown =
  footnote_between_margin.saturating_mul(4)` 배선 (`src/parser/hwp3/mod.rs`).
- `hwp3_default_endnote_shape()` 는 raw_unknown 배선 없이 0 유지(사유 주석).
- 단위 테스트 `issue_3032_footnote_between_margin_wires_footnote_shape_only`:
  SO-SUEOP 실파싱으로 각주 284·미주 0 동시 검증.
- 커밋 authorship 은 kevin9327 유지(인수 크레딧).

## 검증

- 단위 테스트 + `issue_1692` 골든 11/11 (HWP3·HWPX 양 포맷).
- 전체 release-test **4005/4005**, fmt clean, Docker wasm 빌드 성공.
- **시각 판정(작업지시자, 2026-07-25) 통과**: devel↔적용판 42·43쪽 픽셀 diff **0**
  (2,005,644px 전수, native-skia scale 1.5 동일 폰트셋) — 미주 정답지 보존 증명.
  각주 경로는 저장소 HWP3 샘플에 각주 문서가 없어 잠복(발화 시 한컴 오라클 값과 일치).

## 파생 효과·발견

- HWP↔HWPX 동일 문서의 studio 렌더 차이 한 축 해소(footnote betweenNotes 284=284 정합).
- 시각 판정 중 별도 결함 2건 발견·등록: #3302(1쪽 그림 미표시), #3303(문단 테두리
  '없음' 오렌더 — bde926e4b 연관 점검 필요).

## 후속

- PR #3036 은 본 통합 merge 후 검토 결과·크레딧 코멘트와 함께 close.
- HWP3 각주 실문서 확보 시 각주 간격 시각 실측(작업지시자 한컴 제작 제안 유지).
