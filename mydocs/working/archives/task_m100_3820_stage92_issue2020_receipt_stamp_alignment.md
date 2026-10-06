---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 92 — 복학원서 접수증 날인 앵커 회귀 분석

## 전 단계에서 이월된 사실

Stage 91의 HWP5-origin HWPX RowBreak 보정은 아래 focused gate를 통과했다.

- `issue_1939_hwp5_origin_hwpx_strict_render_diff_is_stable` — 82쪽, 구조와 최대 변위
  1px 계약을 함께 확인, `1 passed; 0 failed`.
- `issue_1891_hwp5_origin_hwpx_export_reparse_keeps_page_count` — `1 passed; 0 failed`.
- `issue_2430_cell_rewrap_threshold` — 수정 직전 `2 passed; 0 failed`.

다만 final `CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 cargo test
--profile release-test --tests`는 exit `101`로 끝났다. 실패는
`issue_2020_bokhak_receipt_seal_line_and_stamp_align` 하나이며, 같은 integration binary의
나머지 세 test는 통과했다.

## 관측값과 결함 경계

`samples/복학원서.hwp` p1의 receipt table (`pi=16`)에서 빨간 도장 원은 contract 범위에
남아 있다.

| 항목 | 기대 계약 | 현재 관측 |
| --- | --- | --- |
| 도장 원 좌상단/크기 | x=609–616, y=948–954, 87–92×82–88px | 중심 `(656.1, 993.1)` — 통과 |
| `㊞`의 원 대비 중심 차 | x축 15–28px, y축 ±8px | text 중심 `(560.1, 996.1)`, x축 96.0px — 실패 |

즉 표 전체나 원 도형이 이동한 것이 아니라, 원 내부 좌측에 있어야 하는 `㊞` glyph만 왼쪽의
날짜/텍스트 anchor로 재배치됐다. 이는 `receipt_date_stamp_shift_px` 같은 고정 shift를 다시
넓히거나 test 상한을 완화해 해결할 문제가 아니다.

## 관련 코드 경로

`F081C`는 TAC filler로 폭 측정에서는 0폭, paint에서는 숨기며(`composer.rs`), 같은 문단의
`F012B`는 display text `(인)`으로 확장된다. 접수증 전용 가시 선은
`layout.rs::tac_receipt_filler_prefix`와 `push_tac_receipt_seal_line`이 생성한다. 반면 이번
실패의 `㊞`은 해당 합성 line이 아니라 receipt table 내부의 별도 text run이다.

원인 대조 결과, Stage 82가 p81의 14pt `한양신명조` 공백을 411/1024em으로 보정하면서
**글꼴명만** 조건으로 삼은 것이 직접 원인이다. 접수증 날짜 줄은 같은 원명이지만 10pt이고,
기준 PDF에서는 일반 반각(512/1024em) 공백을 사용한다. 기존 전역 보정은 날짜 앞의 공백
약 73개를 각 약 1.3px씩 줄여 `㊞`만 약 96px 왼쪽으로 옮겼다. 도장 원·receipt table은
정상이라 RowBreak/표 fragment 보정과 인과관계가 없다.

`mydocs/manual/webhwpctrl_compat_development.md`와 `tools/hwpctrl_compat/README.md`도
함께 확인했다. HWPCTRL의 COM/fixture oracle은 API 호출·저장 결과 호환성에만 사용하며, 이
renderer의 문자 advance·PDF 배치는 한컴 PDF가 정답이다. 따라서 HWPCTRL API, fixture, ledger를
건드리지 않고 HWP 저장 글자모양의 **face + point size** 문맥으로 measurement 보정을 제한한다.

따라서 다음을 서로 분리한다.

1. Stage 91의 HWP5-origin RowBreak profile 확장이 `pi=16` 수평 text anchor에 실제로 닿았는지,
2. parent commit에서도 같은 `㊞` x좌표가 재현되는지(기존 baseline/test의 불일치 가능성),
3. date run과 stamp run의 저장 char-position/paragraph offset이 layout 중 어디에서 합쳐지는지.

## 원인 확정

한컴 PDF의 text bbox에서 `㊞`의 x는 472.064160pt, 96dpi 기준 약 **629.4px**다.
기존 release-test SVG의 x=554.56px 및 원 중심 대비 96.0px 차이는 PDF와 맞지 않아,
기존 #2020 contract가 잘못됐다는 가설은 배제했다.

원인은 `text_measurement.rs`의 `한양신명조` U+0020 보정을 글꼴명만으로 전체 글자
크기에 적용한 것이다. p81의 14pt 중첩 표에서 검증된 411/1024em space advance가 접수증의
10pt 날짜 문단에도 적용되면서 다수의 저장 공백이 누적 축소되었다. 따라서 마지막 `㊞` run만
왼쪽으로 크게 이동했고, 인접 도장 원 도형은 정상 위치에 남았다.

수정은 PDF로 확인된 적용 조건을 보존해 `한양신명조` **14pt(56/3px)**에만
411/1024em을 적용하고, 10pt 접수증은 한컴 PDF와 같은 일반 반각(0.5em) 공백으로
되돌리는 것이다. 글꼴 전체의 폭 정책을 바꾸거나 receipt 전용 offset을 다시 도입하지 않는다.

Stage 91은 이 원인이 아니다. 원 HWP에서는 확장 전후의 stored-pagination predicate가 모두
참이고, receipt table은 TAC 배치라 RowBreak의 non-TAC tail-fit 경로에도 들어가지 않는다.

## 수정 뒤 검증

- `issue_2430_cell_rewrap_threshold` — `2 passed; 0 failed`.
- `issue_2020` — `4 passed; 0 failed`; receipt의 도장 원/`㊞` 상대 x·y, TAC filler,
  FSC reference page contracts를 함께 재확인했다.
- `text_measurement::tests::issue_3820_hanyang_shinmyeongjo_space_is_14pt_only`가
  14pt는 411/1024em, 10pt는 0.5em이라는 경계를 unit contract로 고정한다.

다음 단계는 이 변경을 포함해 전체 `cargo test --profile release-test --tests`의 최종
exit code와 summary를 확인하는 것이다.

## 전체 회귀 실행 결과

전용 target으로 실행한 전체 명령은 최종 exit `101`이었다.

```text
CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --tests
```

`issue_2020`은 포함된 integration binary에서 통과했으며, 실패는 마지막 lib 단계의 아래 두
기존 증분 페이지네이션 contract다.

- `wasm_api::tests::issue2214_scoped_cache_coherence_preserves_transient_pagination`
  — HWPX input 56의 flow signal 기대값 불일치.
- `wasm_api::tests::issue2424_resumable_delete_commits_only_after_final_fragment`
  — HWPX 삭제 뒤 fifth line cut이 유지됨.

이들은 접수증의 10pt 공백 측정과 독립된 편집/캐시 경로다. 성공으로 가장하지 않고,
Stage 93에서 개별 재현과 현재 measurement 보정 전후 비교를 수행한다.

## 검증 순서

1. `한양신명조` 14pt만 411/1024em을 사용하고, 다른 크기는 기존 일반 반각 공백을 유지하는
   unit test를 추가한다.
2. #2020 receipt stamp, Stage 82 p81/p82 owner, Stage 91 #1939 strict를 순차 실행한다.
3. 세 focused gate가 통과한 뒤 최종 전체 release-test를 다시 실행한다. PDF, fixture, baseline은
   변경하지 않는다.

## 완료 기준

- `㊞`이 한컴 PDF의 도장 원 내부 좌측에 재배치되어 #2020의 x/y contract를 통과한다.
- F081C가 SVG에 노출되지 않고 `(인)` 합성 선 및 post-marker 계약을 유지한다.
- Stage 91 #1939 strict gate와 #2430 cell-rewrap gate를 포함한 전체 release-test summary가
  `0 failed`다.
