# 최종 결과보고서 — Task #1853

## 이슈

[#1853](https://github.com/edwardkim/rhwp/issues/1853) — 같은 문단 float 스택 이월
규칙(#1831/PR #1844) 회귀: tac 캡션·페이지-절대 앵커 과잉 포착으로 표 통째-이월 +1쪽 (3건)

## 결론

`preceded_by_same_para_float` 술어를 같은 문단의 **진짜 flow 스택 float**으로 한정하여
PR #1844 이 5,200건 서베이에서 유발한 표 통째-이월 +1쪽 회귀 3건을 해소했다. 별표4(2448877)
정합과 전 테스트는 무회귀.

## 근본 원인

PR #1844(merge `a987ae10`)의 `src/renderer/typeset.rs` `preceded_by_same_para_float`
술어가 `para_index` 만 비교하고 `ci`·`tac`·앵커를 구분하지 않아, 표 자신의 `tac=true` 캡션
상자(156767631 ci=1, 78842 ci=0)나 `vert=용지`(페이지-절대) 앵커 상자(3143097 pi2 의 상자
22개)까지 "선행 형제 float" 로 오분류 → 분할 가능한 본체 표를 통째 다음 쪽으로 이월 → +1쪽.

## 수정

선행 항목의 소스 컨트롤(`para.controls[control_index]`)을 조회해 기존 헬퍼
`is_para_topbottom_float`(`!tac && TopAndBottom && vert=Para`)인 표만 선행 float 로 센다.
- tac 캡션 상자 제외 → 156767631·78842 해소
- vert=용지(Paper/Page) 절대 앵커 제외 → 3143097 해소
- 별표4 표1(tac=false·TopAndBottom·vert=Para)은 그대로 인정 → 정합 유지

이월 발동 조건·prefill·advance 로직은 불변. 술어를 좁히기만 하므로 정상 분할되던 문서는
영향 없고 과잉 이월되던 문서만 정상 분할로 복귀한다.

## 검증

| 검증 | 결과 |
|---|---|
| 156767631 (MATCH→PAGE_DELTA 직접 회귀) | 6→**5** (MATCH 복원, 한글 5) |
| 78842 | 53→**52** (한글 52) |
| 3143097 | 4→**3** (devel 수준; 한글 1 정합은 페이지-절대 앵커 별도 과제) |
| 별표4(2448877)·float-stack-defer 캘리브레이션 | 각 2쪽 불변 |
| issue_1853 신규 게이트 2건 | 통과 (구 바이너리 53쪽·캡션쪽 본체 없음으로 유효성 확인) |
| 기존 float/표 테스트 | issue_1156/1488/1510/1549/1639/1663/1748 통과 |
| lib 단위 2073건 / clippy | 0 실패 / 0 warning |
| 통합 테스트 | svg_snapshot 5건(CRLF-only 노이즈, 내용 동일) 외 전부 통과 |

## 잔여/후속

- 3143097 의 한글 1쪽 정합(pi=2 의 `vert=용지` 페이지-절대 앵커 개체를 앵커 문단 쪽에
  배치)은 본 타스크 범위 밖. rhwp 가 페이지-절대 앵커 개체를 flow 로 쌓는 별도 갈래로,
  별도 이슈 후보.
- PAGE_DELTA ±1(134건)·PI_MISMATCH 단일 플립(72건)의 razor-thin 높이 계통은 별개 과제.

## 산출물

- 소스: `src/renderer/typeset.rs`
- 게이트: `samples/issue1853_caption_precedes_body_split.hwpx` + `tests/issue_1853.rs`
- 문서: `plans/task_m100_1853.md`, `plans/task_m100_1853_impl.md`,
  `working/task_m100_1853_stage1.md`, 본 보고서, `pr/pr_1844_review.md`(사후 발견 기록)
- 서베이 데이터: `output/poc/survey_pipage_pr1844/master.tsv`(5,200행)
