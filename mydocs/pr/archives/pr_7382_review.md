---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-02
---

# PR #7382 리뷰 — 분할 표와 저장 각주 경계

## 최종 판정

**메인터너 보정 후 수용 가능.** 검토 브랜치는 `review/planet6897-7382-20260926`이며, 생산 코드 검증 후보는 `cd85bdf434f5c2522f2a24c3a59f3413f644a5a4`, base는 `02530b9ed567a44663edb26c65fb565c4a79f00d`입니다. 원 PR head `81a402179dc556cce781d844d4b9252be36ba8af` 자체의 승인이 아니라 체리픽·메인터너 보정 통합 후보의 판정입니다. [통합 PR #7505](https://github.com/edwardkim/rhwp/pull/7505)의 코드 후보 `b8da28d30e4bd6cddc0bccac55d22d53f623c5db` Full CI가 통과했습니다. 아래 문서 보완 trailing head의 게이트와 mergeability를 확인한 뒤 병합합니다.

- 최종 코드 전체 nextest: **10,229 PASS / 0 FAIL / 50 SKIP**, threads=8, release-test, locked, no-fail-fast, exit0. Native Skia lib·그림·직접 PDF 내보내기 3개 경로, doc tests, Native/WASM Clippy, workspace build/all-target Clippy, fmt와 suite/unit 정책 검사도 통과했습니다. generated harness를 재생성한 뒤 정책 검사를 재실행했으며 파생 파일은 커밋하지 않습니다.
- 원본 HWPX와 독립 한컴2024 PDF는 모두 **215쪽**입니다. Native/fresh WASM 전수215쪽 TSV의 최저는 **22쪽90.01587%**, **90% 미만0쪽**, 88쪽은 **98.62448%**입니다. 휴먼명조 TrueType을 확인한 [명시적 글꼴 환경](../assets/pr7382_20260926/stage325_font_environment.json), print·96dpi·전체 글꼴 임베드를 사용했습니다.
- 전수 래스터는 보정333에서 캡처했습니다. 이후 테스트 의미 교정과 동일 의미 Clippy 표현 교정 후 최종 코드로 Native SVG215개·render tree215개·fresh WASM SVG215개를 실제 재출력해 모두 byte 동일함을 확인했습니다. 최종 코드로22/66/67/88/163/164쪽은 Native/fresh WASM을 새로 캡처했고 양쪽 PNG도 같습니다. 이를 최종 head의215쪽 전체 재래스터라고 표현하지 않습니다.
- 전수 TSV는 보조값이며 단독 승인 근거가 아닙니다. 22/66/67/78/80/84/85/88/131/164/176/183/215쪽을 직접 판독했고 표 행·각주 수량·이어지는 본문과 쪽 소유를 확인했습니다. 84쪽 각주131–133,85쪽134–136,215쪽269–274가 PDF와 같습니다. 원 이슈의 다섯 추가 쪽 경계를 확인했습니다.
- 잔차:22쪽 작은 차트 격자/글리프 래스터 차이,164쪽 표 하단 테두리가 약13.3px 아래인 차이는 남습니다. 본문 누락·중복·쪽 소유 차이는 관찰하지 않았습니다. 완전 픽셀 동일이나 모든 입력의 피델리티 완성을 주장하지 않습니다.
- #7445에는 #7382를 실제 차단하던 대형·복합 입력의 해당 검사만 보류했고 원본·독립 PDF·정상 검사는 보존했습니다. 최종2건 실패는 기존 휴먼명조 TrueType 기대와 SVG 생략 배율의 의미를 독립 근거로 교정했습니다. 이 단계에서 새 Rust 검사 함수는 추가하지 않았습니다.
- [최종 검증](../assets/pr7382_20260926/stage338_final_validation.json), [lint·정책](../assets/pr7382_20260926/stage338_lint_validation.json), [출력 출처·한계](../assets/pr7382_20260926/stage338_final_provenance.json), [Native 전쪽 TSV](../assets/pr7382_20260926/stage338_native_whole_silhouette.tsv), [WASM 전쪽 TSV](../assets/pr7382_20260926/stage338_wasm_whole_silhouette.tsv).

![최종 코드88쪽 review](../assets/pr7382_20260926/stage338_native_088_review.png)
![최종 코드88쪽 standalone overlay](../assets/pr7382_20260926/stage338_native_088_overlay.png)

### 이전 단계의 판정 기록

아래 건수는 해당 단계에서 관찰한 결과이며 현재 실패 수가 아닙니다. 최신 진행은 위 요약과 각 보정의 실행 증거를 따릅니다.

**현재 진행**: 남은26개 원장은 보정178까지22건 처리/4건 대기이며, 이전 처리 항목인 #1749 HWP 재검토는 보정168에서 해결했습니다. 작은 문서는 현재 브랜치에서 보정하고, 전 문서 피델리티 복원이 필요한 실제 차단만 #7445로 이관합니다. 보정173 전체 nextest는42FAIL이며 이후 변경을 포함한 전수 재실행 전이므로 PR 준비는 미완료입니다.

**머지 보류.** 대상 PR은 사용자께서 확인하신 #7382이며, 현재 검토 브랜치는 `review/planet6897-7382-20260926`입니다. 통합 PR은 아직 생성하지 않았습니다. #6101·#7336의 생산 보정과 긍정 시각 증거는 [보정68](../assets/pr7382_20260926/stage68_validation.json)·[보정69](../assets/pr7382_20260926/stage69_validation.json)에 기록했습니다. 이는 현재 전체 회귀 통과나 원 PR의 최종 승인 근거를 대체하지 않습니다.

이전 실패38개는 과거 후보별로37개 처리/1개 사용자 승인 이관 기록이 있습니다. 유지37개를 최종 후보에서 각각 다시 실행해야 합니다. 사용자 범위 정정 후 정상 함수·corpus·renderer manifest·samples 입력을 복원했습니다. 복원 개별검사에서16개가 실제 실패했습니다. 각주 검사1개는 앞 단계에서 독립 근거로 교정했고, 나머지15개 중13함수를 실제 차단 범위로 보류하고 #6706은 실패한 쪽수 assertion만 보류해 위치·소유 검사를 정상 유지했습니다. #1749도 실패한 HWPX 컷 assertion만 보류하고 정상 부분을 유지해 당시 전용 실패 목록을 처리했습니다. 보정166의 재검증에서는 기존 HWP 컷1건이 다시 실패해 별도 재검토 대상으로 기록했고, 보정168에서 작은 HWP5쪽의 실제 저장 경계·빈 종료 공간·임베드 글꼴을 복원하여 해결했습니다. 별도 HWPX 첫 컷의 이전 보류는 보존합니다. corpus와 최종 검증은 진행 중입니다. 정상 검사와 모든 다른 입력·축은 유지했습니다. PII 혼합 검사에서 새로 확인한 쪽수 항목 한 개도 좁게 보류했으며 다른 다섯 입력은 유지·재실행했습니다.

보정125에서 text-overlap 전체16분할과 진안 정상2함수는18PASS입니다. 다른 축이나 전체 회귀의 통과를 뜻하지 않습니다.

보정110의 corpus/matrix 결과는63PASS/23FAIL의 진단 snapshot입니다. rowbreak HWPX의 실제 실패만 보정111에서 처리했고, 현재 전체 실패 함수 수는 전수 재실행 전 추정하지 않습니다. 낮은 점수라는 이유로 같은 문서의 정상 검사를 #7445에 일괄 이관하지 않습니다. [현재 계획](../assets/pr7382_20260926/remaining38_regression_plan.json)과 [입력별 보류 근거](../assets/issue7445/README.md)를 따릅니다.

남은 개별 실패를 분석한 뒤 최종 유지37개별 실행, 전체 `cargo nextest`(threads=8), Native Skia3 및 현재 생산 코드의 원 PR 전체 Native/fresh WASM 시각 검증을 완료해야 합니다. 과거968쪽 비교와 이전 전체 nextest는 현재 head 통과의 근거가 아닙니다. 직접 확인한 시험 문서 쪽번호 차이도 최종 판정에서 빠뜨리지 않습니다. 최신 원 PR head는 `81a402179dc556cce781d844d4b9252be36ba8af`로 재확인했습니다.

보정157에서 #1100은 정상 용지 PDF를 재산출한 뒤 현재 브랜치에서 개선했습니다. 기존3함수를 절대픽셀 핀 대신 쪽·문항 소속과 번호 보존으로 교정했고, 대조군 포함15PASS/단위2PASS·필수lint7단계·전체4쪽 Native/freshWASM gate를 통과했습니다. 원래60개 실패 목록 중34개 처리/26개 개별검토 대기입니다. 이는 최종 전체 실행으로 확정한 현재 실패 건수가 아니며, 통합PR 준비는 보류합니다. 아래 보정157 결과를 따릅니다.

## 접수와 provenance

- 원 PR: [#7382](https://github.com/edwardkim/rhwp/pull/7382), planet6897. 원 head `81a402179dc556cce781d844d4b9252be36ba8af`.
- 접수 당시 base `eb9142dd7c73297d555383d7d8434a470bdef26e`, 검토 branch `review/planet6897-7382-20260926`. 기존 reviewer jangster77 확인.
- 원 기능 `8e0f0249460a5243e38162340bc1c49d061ae72a` → `-x 5b2cc61cd`; 원 이미지 `fcba72b18483053c436388a8d88c64b81d67c2c0` → `-x a57d6430a`. source devel merge 81a는 체리픽하지 않았다.
- 관련 이슈 [#7379](https://github.com/edwardkim/rhwp/issues/7379). 부분 개선으로 전체 이슈를 종료하지 않는다.

## 독립 입력과 기대값

- [원본 HWPX](../../../samples/정책연구용역사업%20중간진도보고서%28살아있는%20간장%20기증자의%20의학적%20선별기준%20연구%29.hwpx), [한컴 2024 기준 PDF](../../../pdf/정책연구용역사업%20중간진도보고서%28살아있는%20간장%20기증자의%20의학적%20선별기준%20연구%29-hwpx-2024.pdf), 215쪽. 이미 저장소에 있는 동일 입력·기준을 사용한다.
- 기준 66쪽은 문단728 표의 머리행+본문4행(원 행0..4), 67쪽은 나머지2행(5..6). 원본 개체 높이11645HU는 첫5행의 저장 높이 합과 정확히 같다.
- 각주77은 앞 두 줄이66쪽, `Part 482(CONDITIONS...)`와 출처인 꼬리는67쪽이다. 원본 첫 각주 문단의 저장 vpos `0,1172,0`과 다음 문단의1172가 이 경계를 지시한다. 번호77은 꼬리에서 반복하지 않는다.
- 동일 보고서 HWP 대조군은 현재 제품 코드에서215쪽이며, 표728도66쪽0..5/67쪽5..7(exclusive)으로 나뉜다. 이 대조군을 HWPX 최종 시각 검증의 대용으로 쓰지 않는다.

## 원 변경의 재검토 결과

| 실행 | 결과 | 의미 |
| --- | --- | --- |
| base 제품+원 회귀2개 | 1PASS/1FAIL, exit100 | 66쪽에 표 없음으로 의도한 실패; 각주 없는 표는 정상 대조군 |
| 원 변경 적용 후 원 회귀2개 | 2PASS, exit0 | 표 존재·대략적인 높이만 검증 |
| 원 변경 후 실제 출력 | 217쪽 / PDF215쪽 | 부분 개선이며 전체 쪽수 계약 불충족 |
| 새 행·각주 소유 검사, 원 변경에서 실행 | 1FAIL, exit100 | 실제 첫 행 집합{0,1,2,3}, PDF 기대{0,1,2,3,4} |
| 원 변경 Native Visual Sweep66/67 | 82.17683% / 61.0948%, exit1 | 표 마지막 행 이월·각주77 누락, 재검토 gate. 글꼴 예외 없음 |

원 `register_body_footnote` 설명과 실제 표 셀 각주 경로가 다르다. 통째 배치 뒤에는 `controls/paragraph_flow.rs` → `notes::register_unqueued_table_cells` → `table::register_unqueued_table_footnote`가 모든 셀 각주를 등록한다. 진입에서 전체 각주 예약을 제거하면 통째 배치의 사전 fit도 바뀐다. 원본 높이 합 검사만으로 행 누락·중복이나 각주 분할을 입증할 수 없다.

## 메인터너 보정 1: 저장된 각주 경계를 분할 큐에 연결

사전 분석 → 코드/회귀 수정 → 실행 결과보고 → 개별 커밋 순서로 진행한다. 전체 수정이 준비되기 전에는 원 PR과 통합 후보 모두 보류다.

1. `entry.rs`는 base의 전체 각주 예약을 복원해 통째 fit 계약을 유지한다.
2. HWPX 파서는0에서 시작한 각주의 양수→0 재시작을 보존한다. 기존2344→0 연속줄 보정·all-zero HWP5 및 미주 경로는 대조군으로 확인한다.
3. `table.rs`의 유효 저장 줄/composer 일대일 각주 경계 판정을 미편집 HWPX에도 연결한다. split 실패 뒤 `prepare.rs`에서 비TAC·RowBreak·rowspan 없는 다행 표의 저장 각주 경계를 fragment queue가 소비한다.
4. `fragment/emit.rs`가 확정한 행·컷과 물리 높이를 먼저 예약하고 `table/footnotes.rs`가 marker를 가진 중간 조각에prefix, 다음 조각에tail을 등록한다. 숫자상 전체 각주가 fit해도 저장된 물리 경계를 지우지 않는다. fresh page 큐 소진·terminal·intra-row 및 marker가 없는 조각의 기존 계약은 유지한다.
5. [정식 회귀](../../../tests/cases/issue_7379_rowbreak_table_footnote_reservation.rs)는 최종 render tree에서 행 소유·각주77 앞뒤 내용·번호 중복·표/각주 비충돌을 직접 검사한다.

파서/큐 연결만 수행한 중간 후보는2PASS/1FAIL이었다. 행은4+2로 개선됐으나77번 꼬리도66쪽에 표시되어 새 검사로 실패했다. 이는 완료로 보고하지 않고 같은 범위에서 각주 등록 경로를 추가 보정했다. 첫 보정 후 집중3/3 PASS(0.176s), 정상 대조군9/9 PASS(0.565s), exit0을 확인했다. 대조군은2344→0 보정, all-zero 보존, #6495/#6545 미주 및 HWP→HWPX 왕복215쪽, #1937 표 각주를 포함한다. Native review/standalone overlay66·67쪽을 직접 판독했다. 행·각주 앞뒤 소유는 개선됐지만 표의 원점·본문/캡션 간격·각주 줄 위치가 다르고 실루엣88.1603%/73.99603%, exit1이다. 현재 출력219/PDF215쪽이며 승인하지 않는다. 전체 회귀·lint·fresh WASM은 아직 실행 완료로 보고하지 않는다. [단계1 검증](../assets/pr7382_20260926/stage1_validation.json)과 [Native manifest](../assets/pr7382_20260926/stage1_native_run_manifest.json)는 imported head+단계1 worktree diff의 진단 증거다. 최종 code head에서는 새로 캡처한다. 원 source의 issue_7379 PNG5개도 과거 증적이며 현재 수용 근거로 재사용하지 않는다.

![단계1 Native66 review](../assets/pr7382_20260926/stage1_native_review_066.png)
![단계1 Native67 review](../assets/pr7382_20260926/stage1_native_review_067.png)

첫 추가 쪽은 HWPX의77쪽 TAC 그림/캡션 표876이78쪽으로 밀리는 경계에서 발생한다. 이어885 대형 표의 셀 각주,1822 대형 단일 셀과 이후 경계가 남는다. 이를 글꼴이나 기존 차이로 면제하지 않고 한 경계씩 사전 분석·수정·전후 검사한다.

## 메인터너 보정 2: 표 원점과 캡션 종료 간격

단계1 `483cfb4f8`의 표 상단796.9/83.2px와 캡션166.1px, 뒤 본문206.1px는 PDF799.925/86.945/156.434/200.261px와 달랐다. 새 최종 좌표 회귀는 수정 전 첫 표 원점으로 FAIL(exit100)했고, 위 여백·캡션만 연결한 중간 후보는 뒤 본문196.533px로 FAIL했다. 측정에서 예약한 끝 바깥여백이 paint의 반환 높이에 빠진 경로도 같은 범위에서 보정했다.

- `host_spacing → fragment/budget → table_partial`의 위 바깥여백 조건을 공통 query로 연결했다. 열수는 바깥여백의 근거가 아니므로 기존 1열 한정을 제거했다. 중첩 frame과 이미 해결된 저장 원점의 중복 inset 방지는 유지한다.
- 저장된 단일 zero-width 줄이 단일 비TAC 표만 소유한 앵커일 때, 호스트 줄간격을 캡션 gap에 추가하지 않는다. `prepare`의 요구 높이와 `table_partial`의 실제 캡션 위치가 같은 결과를 소비한다. 보이지 않는 줄이라는 이유로 줄 상자를 제거하는 처리가 아니다.
- 아래 캡션이 끝난 뒤의 바깥여백을 예산·흐름 전진·paint 반환 높이에 함께 연결했다. 중간 컷의 행 예산에서 이 종료 간격을 미리 차감하지 않는다.
- 집중5/5 PASS(0.794s), 정상 대조군·PrEP·공통 앵커51/51 PASS(0.901s), exit0. 여백/캡션 간격 IR 변형3종도 행 소유·캡션 한 번·각주 lane 비충돌을 확인했으나 수정 전에도 통과하는 대조 검사다. 진단 trace상 세 변형은 terminal 수용 예산 안에 들어갔으므로 **공간 부족 후 재분할 분기의 실행 증거로 인정하지 않는다**. 그 분기는 미검증으로 남기고 다음 경계 검사에서 확인한다. 합성 IR은 한컴 출력의 대용이 아니다.
- 새 CLI의 전체 출력은219/PDF215쪽으로 불충족이다. Native66/67의 2px 실루엣은98.24785%/93.58939%, 선택 페이지 gate passed(exit0), 글꼴 예외 없음. 대표 review와 standalone overlay 두 쪽을 직접 판독해 행·각주 소유, 괘선·캡션·뒤 본문 위치를 확인했다. 각주 가로 공백·글자 폭과 얇은 괘선 차이는 남으며 pixel-perfect 일치라고 보고하지 않는다.

[단계2 검증](../assets/pr7382_20260926/stage2_validation.json)과 [Native manifest](../assets/pr7382_20260926/stage2_native_run_manifest.json)는 `483cfb4f8`+단계2 worktree diff의 진단 증거다. 최종 head의 fresh WASM·전체 회귀/lint는 아직 완료하지 않았다. 전체 쪽수가 다르므로 선택 페이지 통과에도 PR 생성·승인은 보류한다. 첫 추가 쪽 앞 표866의 행 내부 컷과 뒤 그림876의 여유 공간을 HWP 대조군/PDF와 연결해 다음 보정을 준비한다.

![단계2 Native66 review](../assets/pr7382_20260926/stage2_native_review_066.png)
![단계2 Native67 review](../assets/pr7382_20260926/stage2_native_review_067.png)

## 메인터너 보정 3: 저장 셀 내부 컷과 그림51의 물리 페이지

보정2 뒤 전체219쪽의 첫 추가 쪽을 역추적했다. 표866의 원본 row4 첫 셀은 `vpos=0,1620,3240,0` / `textpos=0,17,35,53`으로 첫 세 줄과 `투석을 시작하게 된 경우` 꼬리를 다른 물리 쪽에 저장한다. PDF76/77 및 HWP 대조군의 실제 컷 `[3,1,1]`이 같은 소유를 뒷받침한다. HWP 대조군215쪽을 HWPX 좌표·시각 일치의 대용으로 쓰지 않는다.

`existing footnote43.4px → entry available → prepare table_available → fragment/budget → row_cut → PartialTable → table_partial/FootnoteArea`를 대조했다. 기존 HWP5 예산324.4px와 달리 HWPX는40px 안전 여백을 더 빼284.4px만 주고, 행4의 첫 한 줄 후보가 고아 줄 검사에서 기각됐다. 파서에서 저장 reset을 삭제한 문제가 아니었다. 새 최종 tree 회귀는 수정 전 행4가 없는 것으로1FAIL(exit100,0.234s)했다.

- 기존 각주만 있고 자체 각주가 없는 비TAC T&B RowBreak 표에서, HWPX의 **한 셀 문단 안** 유효한0→양수→0 저장 경계에도 실제 각주 경계를 사용한다. 편집/reflow와 rowspan은 새 적용에서 제외한다. 문단마다 시작하는 local0만으로는 이 계약을 열지 않으며 일반40px 여백·고아 줄 임계값을 전역으로 완화하지 않았다.
- 첫 후보는 prefix/tail과 그림51의77쪽 소유를 복원했으나 캡션908.973/PDF913.379px로5PASS/1FAIL이었다. 표tail 위86.933/아래257.546px는 PDF86.945/257.799px와 가까웠다. Top 외부 캡션 표의 끝 바깥여백283HU=3.773px가 뒤 빈 문단 흐름에 빠진 것이므로, 세로 캡션 개체 종료의 예산·flow·paint 반환을 같은 query로 연결했다. 중간 컷·가로 캡션·외부 캡션 없는 표의 기존 계약은 유지한다.
- 최종 집중6/6 PASS(0.889s), 추가 좌표 검사와 정상 대조군을 함께 실행한75/75 PASS(1.742s), exit0. 대조군은 기존51개에 셀 단위 원자 분할/왕복, profile 독립 source frame, #6761의 확인된 저장 경계와 무효/다른 컷 반례를 더했다. 표866 앞 세 줄/꼬리의 내용·행 소유, 표tail 상·하단, 뒤 본문288.289px와 그림51 캡션913.379px, 표/각주 비충돌을 최종 tree에서 검사한다.
- 새 release-test CLI의 전체 출력은**218/PDF215쪽**이다. 표866은76쪽0..5+cut[3,1,1] /77쪽4..7+startCut[3,1,1], 그림876은77쪽으로 바뀌었다. Native76/77은96.49182%/98.35446%, 선택 gate passed(exit0), 글꼴 예외 없음. review와 standalone overlay 두 쪽을 직접 판독했다. 첫 표 조각의 얇은 괘선·셀 글자 위치, 각주 링크 색/폭 차이는 남으며 완전한 픽셀 일치를 주장하지 않는다.

[단계3 검증](../assets/pr7382_20260926/stage3_validation.json)과 [Native manifest](../assets/pr7382_20260926/stage3_native_run_manifest.json)는 `22d23872c`+단계3 worktree diff의 진단 증거다. 첫 추가 쪽을 해소했지만 표885의 대량 셀 각주, 표1822의 단일 셀 분할 및 이후 경계가 남는다. 전체 쪽수·최종 head full Rust/lint/Skia/fresh WASM·terminal 캡션 예산 부족 경계는 미해결/미검증이며 PR 생성·승인 보류다.

![단계3 Native76 review](../assets/pr7382_20260926/stage3_native_review_076.png)
![단계3 Native77 review](../assets/pr7382_20260926/stage3_native_review_077.png)

## 메인터너 보정 4: 다행 표의 셀 각주 큐와 확정된 표시 소유

보정3 뒤 표885의 셀 각주18개를 표 시작 전에 모두 예약해 표가79쪽으로 밀렸다. 독립 PDF78은 행0..3의 앞 조각과 기존각주105/106, 79는 행3 꼬리와 끝2행 및각주107~111, 80은 남은각주112~124와 뒤 본문이다. 새 최종 tree 검사는 수정 전78쪽 표 부재로 FAIL(exit100,0.176s)했다.

- 미편집/reflow 없는 direct HWPX 다행·비TAC T&B RowBreak·rowspan 없는 원표에도 셀 각주 큐를 연결했다. 각주 자체의 저장 reset은 prefix/tail 분할의 근거이며 보통 각주의 큐 참여를 제한하는 근거가 아니다. 통째 fit의 전체 각주 예약과 기존 Native/단일행 경로는 유지한다. 빈1×1 래퍼의 내부 행 컷을 바깥 각주 소유로 혼동하지 않는다.
- 추가 반례에서 아직 다음 조각 row6에 있는82번이 앞쪽66에 등록돼 FAIL(exit100,0.253s)했다. 행만 제한한 후보도 같은 행의 미소비 줄에 있는116번을 앞쪽에 등록해 FAIL(exit100,0.257s)했다. 실제 확정 `end_row/end_cut → 같은 폭·방향의 composed 줄 → cached CellUnit ordinal → col순 셀 컷`을 등록 상한으로 사용한다. 앞 조각에서 표시를 이미 소비한 밀린 각주는 허용한다. 두 합성 IR은 원래 저장 줄/표를 유지하고 각주 몸통 또는 주석 종류만 바꿔 용량과 소유를 분리했으며, 한컴 출력의 대용이 아니다. 최종 marker/footer 누락·중복과 물리 소유를 검사한다.
- 최종 집중·정상 대조군 **78/78 PASS(exit0,2.008s)**. 실제 표885 행/각주 소유, 앞서 보정한 표728/866과 그림51, 저장 reset 반례, 원자 컷·왕복·공통 앵커·PrEP를 함께 확인했다. 새 CLI 전체 출력은 **217/PDF215쪽**이다.
- Native78/79/80은 **87.97988% / 96.21953% / 95.66118%**(2px 실루엣; 원 JSON 참조), sweep exit1/re_review_required, 글꼴 예외 없음. 세 review와 standalone overlay를 직접 판독했다.78쪽 표 앞 조각의 아래선은960.947/PDF970.937px이고 셀 줄 위치도 다르다.79쪽 표 끝은 근접하지만 각주 가로 폭/색·일부 줄 위치 차이가 남는다.80쪽 본문과 각주 소유는 맞지만 링크 폭·색/일부 줄 위치 차이가 있다. 이 결과를 시각 개선 완료로 승격하지 않는다.

각주 개수>=8에 따른 기존 첫 조각 지연 및 terminal guard 예외도 새 HWPX 큐에 따라 들어왔다. 이를 일반 정책의 근거로 인정하지 않는다. 개수 예외를 배제한 후보는8PASS/1FAIL이었다. 표 끝960.947px에 각주 영역945.007px가 겹쳤다. fit에서 빈 footer band를 회수하지만 실제 각주 영역은 본문 하단에 고정하며, 기존 Body각주105의 빠른 추정이 내부 줄간격을 빠뜨리는 차이가 연결된다. `caption_extra` 누락도 코드 우려로 발견했지만 표885는 해당 값0이므로 이 표의 원인으로 보고하지 않는다. 실패 후보는 회수했고 **용량 계산·물리 끝점·기존 개수 예외의 일반화는 다음 개별 보정의 미해결 사항**으로 남긴다. 현재 커밋은 소유 보정의 독립 중간 결과이며 PR 수용 후보가 아니다.

[단계4 검증](../assets/pr7382_20260926/stage4_validation.json), [Native manifest](../assets/pr7382_20260926/stage4_native_run_manifest.json), [Native summary](../assets/pr7382_20260926/stage4_native_summary.json)는 `cd2203a07`+단계4 Rust diff를 고정한 진단 증거다. 최종 head full Rust/lint/Skia/fresh WASM은 아직 완료하지 않았다.

![단계4 Native78 review](../assets/pr7382_20260926/stage4_native_review_078.png)
![단계4 Native79 review](../assets/pr7382_20260926/stage4_native_review_079.png)
![단계4 Native80 review](../assets/pr7382_20260926/stage4_native_review_080.png)

## 남은 필수 게이트

- 전체 페이지 수215, 추가/누락 쪽의 첫 경계와 앞뒤 내용을 재검토한다. 선택66/67쪽의 통과만으로 이 조건을 면제하지 않는다.
- 최종 code head의 Native/fresh WASM Visual Sweep·대표 review/standalone overlay 직접 확인 및90% gate, 적용/비적용 경계 검사.
- cargo nextest threads8 전체 회귀, 모든Rust lint/WASM32/workspace all-target Clippy, workspace build, 고정base 정책 검사와 필요한 Skia/fresh WASM 검증.
- source-number review·오늘할일·대표 시각 증적을 같은 통합 PR에 포함하고 정확한 최신head CI/MERGEABLE/CLEAN 확인. 통합 owner reviewer 자동 지정 없음.
- 정상 merge 후 duration provenance·source supersede/범위에 맞는 issue 처리·devel 동기화·소유 output 정리. 로그는 ignored `output/pr-review/planet6897-7382-20260926/logs/`에만 저장하며 커밋하지 않는다.


## 메인터너 보정 5: 첫 저장 프레임의 빈 밴드와 셀 정렬

보정4 `45f40cf44`의78쪽 표25 끝960.947px는 PDF970.937px와 달랐다. 원본 HWPX table.sz.height33323HU=444.307px는 PDF 첫 프레임443.833px와 대응하나 내용 컷만 그린433.32px 상자는 빈 하단 밴드를 잃었다. HWP 대조군의 셀/줄 원본 메트릭은 같지만 그 현재 출력도 geometry가 달라 독립 좌표 기준으로 승격하지 않았다. 새 끝점 검사는 수정 전 FAIL(exit100); 같은 실행의 raw-source 진단 PASS는 결함 검출 검사로 세지 않는다.

- `saved_multirow_opening_frame_height → emit partial_height/end_row_height_override → table_partial row_heights/partial_table_height → 실제 셀/표 bbox`가 같은 원본 프레임을 소비한다. 미편집·reflow 없는 stored 비TAC T&B RowBreak, rowspan 없는 다행 표의 첫 빈 start_cut과 유효한 plain-text 문단 재시작 end_cut을 요구한다. emit은 객체전용 저장 앵커·래퍼 아님·기존 override 없음·실제 남은 예산 fit도 확인한다. 프레임 빈 밴드는 소비 유닛이 아니므로 다음 내용 tail에서 빼지 않는다. 본문보다 큰 수동1000px 프레임을 강제 수용하지 않는 정식 거부 대조군도 PASS했다.
- 첫 프레임 후보는80PASS였지만 Native78 직접 판독에서 셀 row3/col2가 Top으로 강제되는 차이를 발견했다. 원본 Center, 처음 세 문단8개 저장 줄의125.013px, 원본 padding 및 최종 cell bbox로 정한 기대839.427px에 실제833.813px가 FAIL(exit100,0.169s)했다. 같은 페이지 PDF 첫 prefix839.52px도 독립 확인했다.
- paint는 동일 query의 높이와 실제 partial_table_height가 일치하는 root 첫 프레임에만 원래 세로 정렬을 적용한다. 소비한 line_ranges의 높이로 계산하며 뒤 내용 전체를 중앙에 배치하지 않는다. 일반 예산 컷·continuation·rowspan·slice를 넘는 중첩 다중열 내용은 기존 계약을 유지한다. 첫 정렬 후보는 Top-only lazy composition 가정과 충돌해7PASS/5FAIL했다. 유한 프레임에서 Center/Bottom에 필요한 내용 높이는 전체 paint와 cursor probe 모두 계산하도록 수정하고 재검증했다.
- 최종 집중/정상 대조군81/81 PASS(exit0,2.951s). fresh CLI build exit0(1m54s), Native78/79/80의2px 실루엣97.8181%/96.21953%/95.66118%, 선택 gate passed(exit0), 글꼴 예외 없음. 세 review와 standalone overlay를 직접 확인했다. 표 하단·첫 셀 글줄과 후속 페이지 소유는 개선됐으나 얇은 괘선·본문 글자 및 각주 URL 폭 차이가 남으며 pixel-perfect라고 보고하지 않는다.

[단계5 검증](../assets/pr7382_20260926/stage5_validation.json)과 [Native manifest](../assets/pr7382_20260926/stage5_native_run_manifest.json)는45f40cf44+단계5 Rust/test diff를 고정한다. 전체217/PDF215쪽 차이로 재검토 상태이며 PR을 만들지 않는다. 각주 수 기반 예외/실제 footer 용량,174·175쪽 단일 셀1822의3쪽 분할, 뒤 추가 페이지, terminal caption 예산 실패 및 최종 전체 Rust/Skia/lint/fresh WASM은 남았다. 단계5를 먼저 독립 커밋하고 다음 경계 보정을 수행한다.

![단계5 Native78 review](../assets/pr7382_20260926/stage5_native_review_078.png)
![단계5 Native79 review](../assets/pr7382_20260926/stage5_native_review_079.png)
![단계5 Native80 review](../assets/pr7382_20260926/stage5_native_review_080.png)


## 메인터너 보정 6: 각주 큐와 실제 paint 영역의 용량 공유

`913a04326`에서 각주107·108만 남기고 다른 marker 슬롯을 빈 Endnote로 보존한 합성 IR은 수정 전 FAIL(exit100,0.296s)했다. 78쪽 표끝972.173px / 실제 각주위945.007px로 충돌했다. 이 반례는 각주 개수8개 임계값의 가정을 깨는 계약 검사이며 한컴 출력 일치 근거로 승격하지 않는다.

- `확정 body fragment/end_cut → 후보 FootnoteRef 목록 → estimate_footnote_area_height_with_metrics → body-bottom 물리 예산 → 수용 목록/동일 높이 예약 → layout_footnote_area`를 연결한다. 기존 Body 각주도 후보 목록에 포함해 실제 composed 줄간격·선·각주간 간격을 함께 측정한다. shape 기반 최종 paint 측정은 같은 helper를 호출하며 일반 출력의 기존 산식은 유지한다.
- 단일단 direct HWPX 큐는 빈 footer 밴드를 실제 paint보다 아래로 회수하지 않는다. 수용 뒤 정확한 영역 높이와 body-bottom 예약 상태를 현재 page에 기록하고 페이지 전환 시 초기화한다. 다음 본문의 available_height와 추가 각주 fit도 이 상태를 소비한다. 그림에서 이월된 note body의 별도 상태를 재사용하지 않아 저장 vpos 경로를 바꾸지 않는다.
- 새 HWPX 경로의 첫 조각 지연·terminal 수용은 각주 개수/선언 비율과 분리했다. 기존 native의 개수/guard 정책은 별도 근거 없이 바꾸지 않았다. 확대된 HWPX 큐는 단일단만 적용한다. 다단의 body-wide footnote 계약은 이번 신규 경로에서 비해당이며 그 검증을 완료했다고 보고하지 않는다. synchronous source와 resumed source는 같은 section 문단 목록을 전달한다. 현재 resumable 진입은 편집 상태를 요구하고 새 stored HWPX 큐는 편집/reflow를 제외하므로 해당 신규 큐의 resumed 실행을 주장하지 않는다.
- 집중13/13 PASS(exit0,1.502s), terminal 뒤 실제 본문 비충돌까지 추가한 최종 집중/정상 대조군85/85 PASS(exit0,3.023s). 기존 #1937/#4882/#6495/#6545 및 row-cut/앵커/PrEP 대조에 각주 줄높이 #5708·영역 폭 #6034를 포함했다.
- fresh CLI build exit0(1m52s), Native66/67/78/79/80 선택 gate passed(exit0),2px 실루엣98.24785%/93.58939%/97.8181%/96.21953%/95.66118%, 글꼴 예외 없음. 다섯 review 및 standalone overlay를 직접 판독했다. 표/각주 소유와 위치는 유지됐으며 얇은 괘선·각주 glyph/URL 폭 차이는 남는다.

[단계6 검증](../assets/pr7382_20260926/stage6_validation.json)과 [Native manifest](../assets/pr7382_20260926/stage6_native_run_manifest.json)는913a04326+단계6 Rust/test diff의 증거다. 전체217/PDF215쪽으로 PR 생성·승인은 계속 보류한다. 이 보정을 독립 커밋한 뒤 단일 셀1822의174~176쪽 분할과 이후 추가 페이지, terminal caption 예산 실패를 각각 해결한다. 최종 전체 회귀·lint·Skia·fresh WASM은 아직 완료하지 않았다.

![단계6 Native66 review](../assets/pr7382_20260926/stage6_native_review_066.png)
![단계6 Native67 review](../assets/pr7382_20260926/stage6_native_review_067.png)
![단계6 Native78 review](../assets/pr7382_20260926/stage6_native_review_078.png)
![단계6 Native79 review](../assets/pr7382_20260926/stage6_native_review_079.png)
![단계6 Native80 review](../assets/pr7382_20260926/stage6_native_review_080.png)


## 메인터너 보정 7: 단일 셀 첫 프레임·표시 소유·호스트 종료 간격

보정6 뒤 단일 셀 표1822는174/175/176의 세 조각으로 나뉘었다. 원본 마지막 줄43311+1000+padding282=44593HU와 다음 셀 문단의 vpos0은 첫 프레임의 정확한 컷13을 증언한다. 독립 PDF174에는그림66과224번 표시를 가진 끝 문장이,175에는나머지표·각주223..231·뒤제목이 있다. 소유 회귀는 수정 전174쪽그림66 부재로 FAIL(exit100)했다.

- `saved_single_cell_opening_frame_cut/height → queue preparation → row scan exact end_cut/physical demand → partial-table paint`가 같은 컷/프레임을 사용한다. 정상 padding과 저장 프레임의 유효성은 다른 계약이므로 malformed-padding 조건을 프레임 근거로 대신하지 않는다. 경계의 inline Footnote/Endnote marker는 원래 글줄에 속하며 block control은 허용하지 않는다. 직접 저장 HWPX·미편집/reflow 없음·비TAC RowBreak·1×1·정확한 저장 끝점과 reset·실제 예산 fit 조건을 유지한다. 임의 각주 개수/선언 비율로 이 경로를 선택하지 않는다.
- HorzColumn T&B RowBreak의 바깥 상단 여백은 단일/다행에 동일하게 적용한다. resolved 원점·중첩/다른 위치의 continuation은 기존 호출자 조건으로 중복 여백을 막는다. 첫 geometry 검사는429.827/PDF433.127px로 FAIL했다. 수정 후 네 표 끝점은통과했지만 뒤제목624.027/PDF641.061px로14PASS/1FAIL했다.
- 추적에서 terminal 뒤 호스트 trailing spacing1000HU와 바깥 아래283HU가 빠졌다. 같은 helper의17.107px를 pagination 종료와 실제 partial paint 종료에서 한 번 소비하고 bbox에는 더하지 않는다. 첫 후보는86PASS/1FAIL: 폭0인 PrEP 객체전용 앵커에도 밴드를 더해40쪽본문238.6/PDF220.864px가 됐다. 기존 `object_only_saved_table_anchor` 계약을 공유해 그 앵커에는 글줄 전진을 더하지 않도록 수정했다. 이는 문서ID/각주수 예외가 아니다.
- 최종 집중/정상 대조군87/87 PASS(exit0,3.257s), fresh CLI build exit0. 전체216/PDF215쪽으로 한 추가 쪽을 해소했다. Native174/175/176은99.85942%/92.46177%/79.50902%, selected sweep exit1/re_review_required, 글꼴 예외 없음. 세 review와 standalone overlay를 직접 확인했다.174의그림·첫표하단 및175의꼬리·뒤제목이 맞지만176의표위치·각주234 저장 경계가 다르다. 각주 glyph/폭과 얇은 괘선의 차이도 남는다.

[단계7 검증](../assets/pr7382_20260926/stage7_validation.json), [Native manifest](../assets/pr7382_20260926/stage7_native_run_manifest.json)는8786be62b+단계7 Rust/test diff를 고정한다. 이 단계는 독립 중간 보정이며 PR 생성/승인을 계속 보류한다.176/177 각주234,182쪽뒤문단1913의 추가 쪽,terminal caption 예산 실패,최종 full Rust/lint/Skia/fresh WASM을 다음 개별 단계에서 해결한다. 테스트 작성의 소수점 표기·Rust 이동 소유 컴파일 오류는 수정했으며 결함의 수정 전 FAIL 증거로 세지 않는다.

![단계7 Native174 review](../assets/pr7382_20260926/stage7_native_review_174.png)
![단계7 Native175 review](../assets/pr7382_20260926/stage7_native_review_175.png)
![단계7 Native176 review](../assets/pr7382_20260926/stage7_native_review_176.png)


## 메인터너 보정 8: 큐 없는 셀 각주의 저장 페이지 경계

원본 각주234의 두 저장 줄은 vpos0/0, flags393216/1441792다. 독립 PDF176은 번호와 첫 줄,177은 번호 없는 꼬리를 소유한다. 보정7의 최종 tree에서 꼬리가176쪽에 남아 새 회귀가 FAIL(exit100,0.191s)했다. 합성 TAG를 붙인 대조군은 같은 실행에서 PASS이며 결함 검출 증거가 아니다.

- `원본 stored/composed 줄 일대일 → 기존 reset query → register_unqueued_table_footnote_with_content_height → prefix/suffix FootnoteRef → 최종 FootnoteArea/TextLine`을 연결했다. table.rs의 셀 각주 수집은 이미 미편집 direct HWPX를 허용하지만 whole 표의 큐 없는 등록은 Native만 허용해 fragment 정보를 잃었다. 같은 저장 경계 판정을 큐 없는 등록에도 적용한다. 미편집 HWPX만 확대하며 합성/일대일이 깨진 줄은 기존 query가 거부하고 다단은 기존 조건으로 제외한다. Body30/240은 별도 등록 경로이므로 이번 수정의 해결 범위로 보고하지 않는다.
- 실제 prefix/꼬리의 페이지 소유·번호 누락/중복·뒤 각주235와 각주 첫 줄 좌표(PDF176949.169px/177996.209px)를 검사했다. 수동 합성0/0은 페이지 이월을 강제하지 않는 반례도 PASS했다. 집중17/17 PASS(exit0,1.773s), 관련 각주/왕복/PrEP 정상 대조를 포함한46/46 PASS(exit0,3.070s), fresh CLI build exit0(1m53s).
- Native176/177은85.24789%/90.47055%, sweep exit1/re_review_required, 글꼴 예외 없음. 두 review와 standalone overlay를 직접 확인했다.234 꼬리의 이월과 footer 위치가 개선됐지만176의 표1832 상단483.2/PDF486.508px 및177 표1843의 같은 여백 차이가 남는다. glyph/각주 폭 차이도 있으며 시각 통과로 승격하지 않는다.

[단계8 검증](../assets/pr7382_20260926/stage8_validation.json)과 [Native manifest](../assets/pr7382_20260926/stage8_native_run_manifest.json)는0c35e4458+단계8 Rust/test diff의 중간 증거다. 전체216/PDF215로 PR은 계속 보류한다. 그림67 뒤의 guide 중복 점유와 그림/캡션 위치는 HWP 대조군에서도 독립 PDF와 차이가 있어 HWP 쪽수215를 geometry 정답지로 삼지 않는다. 단일 셀 whole 표의 바깥여백·마지막 추가 페이지·terminal caption 예산 실패와 최종 필수 검증을 이어 해결한다.

![단계8 Native176 review](../assets/pr7382_20260926/stage8_native_review_176.png)
![단계8 Native177 review](../assets/pr7382_20260926/stage8_native_review_177.png)


## 메인터너 보정 9: 단일 셀 통째 표의 바깥 상단 여백

PDF176/177의 실제 괘선은 각각486.508~903.012px/539.729~689.965px다. 원본 표1832/1843은 같은 비TAC T&B HorzColumn·Left RowBreak, 양의 사방 균등 여백과 오프셋0을 가진다. `original_hwpx_column_rowbreak_equal_outer_margin_hu → fragment_outer_top_px → raw_top/lane_top → 최종 table bbox`에서 행 개수 조건이 단일 셀을 잘못 제외했다. 수정 전 좌표 회귀는483.16/PDF486.508px로 FAIL(exit100,0.170s)했다.

- 빈/무효 행은 제외하되 단일 셀도 기존 저장 바깥 상자 규칙을 소비한다. 다른 원점·오프셋·비균등 여백의 적용 조건을 이번 근거 없이 확대하지 않는다. partial 조각은 단계7부터 같은 행 개수에 관계없는 상단 여백을 소비하며, 이 단계는 whole 경로를 보정한다. 가로 여백의 일반성만으로 모든 세로 앵커를 바꾸지 않는다.
- 두 원본 표의 실제 상·하단과 기존 #6378 다행 표, #7063 가로 여백, PrEP 및 공통 앵커 대조군64/64 PASS(exit0,2.699s). fresh CLI build exit0. Native176/177은96.81055%/98.67443%, 선택 gate passed(exit0), 글꼴 예외 없음. 두 review 및 standalone overlay를 직접 확인했다. 표 외곽과 본문 위치가 개선됐으며 얇은 괘선·각주 glyph/폭 차이는 남는다.

[단계9 검증](../assets/pr7382_20260926/stage9_validation.json)과 [Native manifest](../assets/pr7382_20260926/stage9_native_run_manifest.json)는7f3b0ee8b+단계9 Rust/test diff의 증거다. 전체216/PDF215로 PR 생성·승인은 계속 보류한다. 다음은 그림67의 원본 저장 종료 사다리를 사용한 원점/flow/guide 공통 결과와 마지막 추가 페이지이며, terminal caption 예산 실패 및 최종 full Rust/lint/Skia/fresh WASM도 남는다.

![단계9 Native176 review](../assets/pr7382_20260926/stage9_native_review_176.png)
![단계9 Native177 review](../assets/pr7382_20260926/stage9_native_review_177.png)


## 메인터너 보정 10: 이월된 그림 표의 닫힌 저장 프레임

그림67 표1904의 높이50256HU와 바깥 위·아래 여백283HU를 합하면50822HU이며, 같은 PS의 빈 guide 사다리 뒤 일반 문단1910의 저장 vpos가 정확히50822로 다시 시작한다. 원본 저장본·한컴 PDF182의 표 상단86.945px, 캡션741.701px, 뒤 매독/기생충 본문787.461/894.021px, PDF183의 그림68이 독립 근거다. HWP 대조군도215쪽이지만 그림67의 원점은 잘못되어 그 출력으로 HWPX 좌표 기대값을 정하지 않았다.

- 수정 전 대상 좌표는123.4533px로 FAIL(exit100, 대상0.186s); 종료 사다리50822→51822HU를 바꾼 반례는 PASS다. 좌표 겹침만으로 guide 소유를 수용하지 않는다.
- `stored_table_frame_with_guides → query_closed_source_frame_placement → whole entry / prepare의 clean-deferral 후 실제 frame → fragment budget → emit commit → layout raw_top/최종 lane flow → section guide`를 연결했다. 미편집·미reflow 저장본, 단일 단의 새 fragment, 문단 상대 양수 offset, 유효한 단일 host/guide 사다리와 닫힌 외곽 상자, 실제 측정 높이와 선언 높이 일치, 실제 예산 fit일 때 같은 원점/점유 끝을 소비한다. guide는 실제 수용된 배치 결과가 존재할 때만 그 상자의 일부로 처리한다.
- 첫 후보는 normal whole entry에만 연결하여19/20 PASS였다. 실제 경로는 whole-fit 실패 후 split prepare에서 새 쪽으로 이월하고 scanner가 전체 행을 수용해 `PageItem::Table`을 방출했다. 이 경로의 prepared placement와 frame도 함께 갱신하여 후속 예산·paint가 옛 anchor를 다시 적용하지 않도록 했다. 실패 후보를 완료 증거로 사용하지 않는다.
- 최종 관련 앵커·바깥여백·빈 host 줄 간격 및 기존 대상 회귀71/71 PASS(exit0,2.650s), fresh CLI build exit0. 원본 전체215/PDF215쪽, 그림67 뒤 본문의 중복·이월 없음과183쪽 그림68을 검사했다. Native182/183의 선택 gate passed(exit0),99.87143%/99.91466%, 글꼴 예외 없음. 네 review/standalone overlay를 직접 확인했으며 그림 외곽·캡션·본문 배치는 맞고 얇은 선/glyph 차이는 남는다.

[단계10 검증](../assets/pr7382_20260926/stage10_validation.json)과 [Native manifest](../assets/pr7382_20260926/stage10_native_run_manifest.json)는9b0e7cece+단계10 Rust/test diff의 증거다. 전체 페이지 수 보류 사유는 해소했으나 캡션 예산 실패 경계, Body 각주30/240, 최종 full Rust/lint/Skia/fresh WASM과 전체 Native를 완료하기 전에는 PR 생성·승인하지 않는다.

![단계10 Native182 review](../assets/pr7382_20260926/stage10_native_review_182.png)
![단계10 Native183 review](../assets/pr7382_20260926/stage10_native_review_183.png)


## 메인터너 보정 11: 캡션 종료 예산과 실제 행 소유의 재스캔

이전 여백/간격3종은 끝 캡션이 새 페이지 예산 안에 들어가 실패 분기를 실행하지 않았다. 양쪽i16 속성의 유효 상한32700HU와 본문 높이20px 감소로 만든 반례에서 수정 전 예약100.7733px / 실제 중간 조각81.48px가 FAIL(exit100,summary1.871s)했다. 끝행만 뒤로 물리는 기존 emit은 `end_row`만 바꾸고 그 행 높이·컷을 그대로 예약했다. 마지막2행을 rowspan 소유 유닛으로 합치고 각주를 제거해 새 페이지에 들어가는 독립 IR 반례도 FAIL했고, 두 반례 실행은0PASS/2FAIL(exit100,1.990s)이었다. 과도한 캡션과 원본6개 각주가 함께 새 페이지에 들어가지 않는 초기 입력 및 작성 오류/컴파일 실패는 결함 검출 증거로 세지 않는다. 이 입력들은 알고리즘 계약이며 한컴 출력의 대용이 아니다.

- `RowBlockQuery의 마지막 소유 유닛 → 실제 caption/terminal margin 예산 → 동일 scan_block_table_split_rows의 prefix 재스캔 → consumed/end_row/양쪽 컷/override 일괄 반영 → emit/실제 bbox`를 연결했다. 마지막 행 숫자만 감소시키는 emit 분기를 제거했다. 앞 유닛이 없고 현재 페이지에 앞선 항목이 있으며 전체 유닛이 새 페이지에 들어가면, cursor·각주·내용을 소비하지 않고 다음 frame에서 재시도한다. 저장 양수 원점만 있는 빈 frame을 반복 이월하지 않는다. 새 페이지에도 들어가지 않는 객체의 기존 진행 fallback을 이번 검증의 해결 주장으로 바꾸지 않는다.
- 단일 마지막 행의 통째 진입은 선언19.5px만 믿고 캡션을 빠뜨리던 우회였다(수정 전 FAIL,summary0.481s; table top796.92/body83.16px). forward 문단 Top/Inside·객체전용 저장 앵커의 캡션 상자를 공통 배치 계획으로 만들고 whole fit, clean defer 이후 scanner, whole/partial paint가 같은 원점과 occupied bottom을 소비한다. 중간 조각은 중간 여백만, 마지막 조각은 종료 여백을 예약한다. 초기 공유 후보는42개 중4개 FAIL했으며, 중간 조각에 끝 여백을 미리 차감/등록한 오류를 같은 범위에서 수정했다.
- 위 캡션은 첫 예산에서 이미 차감한다. 첫 행 강제 수용이0px 예산을 우회한 새 반례가 FAIL(exit100,0.658s)했고, 동일 종료 검사에서 위 캡션을 이중 차감하지 않고 새 frame으로 이월하도록 고쳤다. 양수2250HU 오프셋 반례도86.9333/기대116.9333px로 FAIL(2PASS/1FAIL,exit100,0.690s)했다. 첫 유닛이 통째로 이월돼도 공통 계획을 다시 질의해 오프셋을 보존하며, 진짜 continuation이 소비한 앵커와 구분한다. 기존 `para_offset_consumed_by_page_break` 계약도 같이 소비한다. 절대 기준·중앙/하단 정렬·후향 앵커는 이 forward 문단 원점으로 해석할 수 없으므로 기존 위치 해석의 비대상 경로다. 해당 조합 전부의 한컴 출력 검증을 주장하지 않는다.
- 정식 반례는 모든 행을 한 번씩, 끝 rowspan을 함께, 캡션을 한 번, 실제 예약/paint 끝점을 확인한다. 단일 행의 위/아래 캡션 및30px 오프셋은 뒤 본문의 소유와 같은 페이지에서의 종료 후 비충돌도 검사한다. 별도로 기존 nested no-caption3개가 보정2의 비대상 인덱싱으로 panic한 것을 발견했다. HWPX top-level 캡션 여백 조건을 통과한 뒤에만 source paragraph를 읽도록 수정해 정상 nested 경로를 재검증했다.
- 최종 집중/정상 **44/44 PASS(exit0,3.597s,threads8)**. source7379 전체24개와 #6756/#6803 rowspan 컷, #7288 원자 행, #6024 continuation, #6837 nested row, #6599 nested caption, #5136/#6284 caption, #7390 PrEP와 #1937을 포함한다. fresh CLI build exit0(2m20s), 전체 **215/PDF215쪽**. Native66/67은 **98.24785% / 93.58939%**, 선택 gate passed(exit0), 글꼴 예외 없음. 두 review와 standalone overlay를 새 코드에서 직접 판독해 행·각주77 소유, 표 원점·캡션·뒤 본문 위치를 확인했다. 얇은 괘선·glyph/URL 폭·링크 색 차이는 남으며 완전 픽셀 일치로 보고하지 않는다.

동일 원본 HWPX의 저장 제품13.0.0.3901을 확인하고 engine2024/timeout1800으로 다시 변환했다. 새 PDF도215쪽이며 **215쪽 전체의 텍스트 줄/좌표가 기존 PDF와 정확히 같았다**.10쪽 BMP 배경224/235/255와 한컴 PDF JPEG234/242/255의 색 차이도 재현됐다. 변환 색상 원인은 미확정이며 renderer gamma나90% gate를 바꾸지 않았다. 기존 기준 PDF와 실패 증거를 유지한다. [같은 입력 PDF 좌표 대조](../assets/pr7382_20260926/stage11_same_input_pdf_geometry.json)는 재변환 hash/engine/job과 전쪽 좌표 결과를 남긴다.

[단계11 검증](../assets/pr7382_20260926/stage11_validation.json), [Native manifest](../assets/pr7382_20260926/stage11_native_manifest.json), [run manifest](../assets/pr7382_20260926/stage11_native_run_manifest.json)는 `015985403`+최종 단계11 Rust/test diff의 증거다. 다음 보정 전에 이 결과를 별도 커밋한다. 전체 Native 진단에서31/32·108·121 등 본문 tail과 각주 소유가 다른 경계를 확인했으며, Body각주30/240 및 그 외 낮은 페이지를 하나씩 처리한다. 최종 전체 회귀·Rust lint·정책·Skia·fresh WASM과 전체 시각 gate는 아직 완료하지 않았으며 PR을 만들지 않는다.

![단계11 Native66 review](../assets/pr7382_20260926/stage11_native_review_066.png)
![단계11 Native67 review](../assets/pr7382_20260926/stage11_native_review_067.png)


## 메인터너 보정 12: 본문 각주의 반복 페이지 시작 보존

### 사전 분석과 실제 소비 경로

원본 본문1865의 각주240은 `0/0/1172`를 저장한다. 독립 한컴2024 PDF178에는 번호와 첫 줄,179에는 `HTLV-1` 및 출처 꼬리 두 줄과 뒤 각주241/242가 있다. [원본 줄과 PDF 좌표](../assets/pr7382_20260926/stage12_independent_geometry.json)로 기대값을 고정했다. 파서가 두 번째0을1172로 덮고 Body 등록이 HWP5 경로에서만 저장 각주 reset을 소비하는 두 원인을 확인했다.

`section.rs::normalize_hwpx_note_line_vpos` → 동일 stored/composed 줄 대응을 확인하는 `native_hwp5_footnote_reset_fragments` → `body.rs::register_body_footnote`의 현재 prefix 예약/물리 page 전환/다음 suffix 예약 → 실제 `FootnoteArea` 배치로 연결한다. 편집하지 않은 HWPX도 표와 같은 저장 각주 경계를 사용한다. 합성 줄·저장 줄 부재는 physical split의 근거로 쓰지 않는다. 양수로 시작한2344/0의 연속줄 복원과 미주는 기존 계약을 유지한다. 본문 자체의 저장 reset과 충돌 경계는 이번 단계에서 변경하지 않았다.

### 수정 전후 실행과 판정

검증 producer는 `ec8a37a84` + Rust/test diff SHA256 `f3f0d1dc8da845d278f436bec045c5202fbdca8862abe638799ad5227a2544d9`다. [실행 증거](../assets/pr7382_20260926/stage12_validation.json)에 연결한다.

| 검사 | 결과 | 판정 |
| --- | --- | --- |
| 원본 parser/실제 각주 소유 수정 전 | 2FAIL, exit100,0.247s | `[0,1172,1172]`와178쪽 꼬리 조기 소비를 실제 검출 |
| 관련 원본·대조군 수정 후 | 35PASS, exit0,4.820s | 기존 표/각주·HWP5 왕복·미주·정규화 및 합성/저장줄 부재 반례 통과 |
| 새 CLI | build exit0,2m11s; 215/PDF215 | 중간 후보의 쪽수 계약 충족 |
| Native178/179 직접 비교 | 94.31041% /97.53225% | 번호/앞줄178, 꼬리179, 뒤 본문·각주 보존. glyph폭·URL색·일부 양쪽정렬 차이가 남아 픽셀 완전 일치를 주장하지 않음 |
| Native31/32 직접 비교 | 45.37615% /34.68853%, sweep 전체 exit1 | 각주30의 prefix/tail은 개선했으나 앞 본문407/421 소유·표/그림 원점 및32쪽 각주 구분선 차이가 남아 보류 |
| 전체 중간 Native audit | 215개 완료,50개90%미만, exit1 | [보정10 중간 보류 목록](../assets/pr7382_20260926/stage10_full_native_hold_inventory.json). 최종 head acceptance가 아님 |

추가 반례의 첫 작성에서 저장 줄 하나를 제거하면 composer 줄 수도 함께 줄어 실제 count mismatch가 아니었다(34PASS/1FAIL). 이를 회귀 검출로 세지 않고 저장 줄 부재 반례로 정정하여 재실행했다. 일반 저장/구성 줄 수 불일치 guard 자체를 이번 반례로 검증했다고 확대하지 않는다.

[Native manifest](../assets/pr7382_20260926/stage12_native_manifest.json)·[summary](../assets/pr7382_20260926/stage12_native_summary.json)·[metrics](../assets/pr7382_20260926/stage12_native_overlay_metrics.json). review178/179 및 standalone overlay178/179, review31/32를 직접 판독했다. 전체/fresh WASM/lint/최종 필수 검증은 아직 남았으며 PR 생성·승인은 보류다.

![Native178 review](../assets/pr7382_20260926/stage12_native_review_178.png)
![Native178 overlay](../assets/pr7382_20260926/stage12_native_overlay_178.png)
![Native179 review](../assets/pr7382_20260926/stage12_native_review_179.png)
![Native179 overlay](../assets/pr7382_20260926/stage12_native_overlay_179.png)
![남은 본문 소유31](../assets/pr7382_20260926/stage12_native_review_031.png)
![남은 본문 소유32](../assets/pr7382_20260926/stage12_native_review_032.png)


## 메인터너 보정 13: 실제 각주 예약과 본문 저장 경계 공유

### 사전 분석·소비 경로

보정12 뒤에도 본문407/421은 통째 항목으로 남아 저장 reset 뒤 줄을 앞쪽에 소비했다. [독립 PDF 줄 좌표](../assets/pr7382_20260926/stage13_independent_geometry.json)는407 꼬리 두 줄이31쪽83.141/109.861px,421 꼬리가32쪽83.141px에 있음을 입증한다. 단순 writer 좌표 되감김을 모두 물리 경계로 취급하지 않고, 미편집 단일 단 HWPX에 기존 nonsynthetic 경계/marker/실제 각주 예약 query를 연결한다.

`boundary.rs::native_hwp5_first_footnote_overlap_break_line`의 source 줄/marker·실제 FootnoteArea 투영 → `section/flow.rs`의 marker 등록 route 및 `paragraph.rs`의 강제 경계 선택 → 실제 `PartialParagraph` 컷 → `native_hwp5_body_footnote_tail_reset` 및 `body.rs`의 완료 prefix/현재 tail 등록 → 실제 본문과 뒤 표/그림 배치. 저장 각주 자체도 같은 경계를 입증한 경우, 기존 각주가 없는 marker 쪽이어도 첫 각주 collision route가 입증한 완료 prefix owner를 사용할 수 있게 했다. 기존 multi-note/native 및 일반 reset 경로를 일괄 확장하지 않았다.

### 실행 결과

Producer `4001f5b0c` + Rust/test diff SHA256 `ccca0de8ec46961ba9473f98e3e756a7cc25cd4dfbc9273730d285a5832205de`; [실행·실제 컷 증거](../assets/pr7382_20260926/stage13_validation.json).

| 검사 | 결과 | 의미 |
| --- | --- | --- |
| 수정 전 원본 본문2개 | 2FAIL, exit100,0.162s | 두 꼬리가 앞쪽에서 소비되는 의도한 원인으로 실패 |
| 수정 후 원본·대조군 | 38PASS, exit0,3.437s | 두 꼬리 좌표·앞 각주 번호·뒤 소유 보존 및 합성 되감김 비적용 |
| 실제 dump cuts | 407:30쪽0..3/31쪽3..5;421:31쪽0..4/32쪽4..5 | 저장 경계 뒤 내용의 누락/중복 없는 실제 항목 보존 |
| 새 CLI/쪽수 | build exit0,2m14s;215/PDF215 | 중간 쪽수 계약 충족 |
| Native30/31 직접 review/overlay | 96.40652% /97.33830% | 앞 본문 꼬리·제목·표 원점 복원. 글꼴/그림색/얇은 괘선 차이는 잔여 |
| Native32 직접 review/overlay | 81.60723%, sweep exit1 | 꼬리와 그림/표 원점은 개선; 뒤 문단 높이·이월 각주 구분선 차이가 남아 보류 |
| Native178/179 재캡처 | 94.31041% /97.53225% | 앞 단계 각주240 무회귀; 이 단계에서 직접 재판독을 반복했다고 확대하지 않음 |

첫 pre-run의421 문구는 제가 옮긴 `35%`가 원문 `<그림35>`와 달랐다. 작성 오류를 결함 증거로 세지 않고 동일 수정 전 코드에서 실제 뒤 문구로 정정하여 실패를 다시 확인했다.

[manifest](../assets/pr7382_20260926/stage13_native_manifest.json)·[summary](../assets/pr7382_20260926/stage13_native_summary.json)·[metrics](../assets/pr7382_20260926/stage13_native_overlay_metrics.json). 30/31/32의 review와 standalone overlay6개를 직접 판독했다.32쪽 구분선은 같은 보고서 HWP·HWPX 기준 PDF 모두 y1018.725px에서 확인되며 별도 후속 보정 대상으로 남긴다. 전체 최종/fresh WASM/lint는 미완료여서 PR 생성·승인 보류다.

![Native30 review](../assets/pr7382_20260926/stage13_native_review_030.png)
![Native30 overlay](../assets/pr7382_20260926/stage13_native_overlay_030.png)
![Native31 review](../assets/pr7382_20260926/stage13_native_review_031.png)
![Native31 overlay](../assets/pr7382_20260926/stage13_native_overlay_031.png)
![Native32 남은 차이](../assets/pr7382_20260926/stage13_native_review_032.png)
![Native32 overlay](../assets/pr7382_20260926/stage13_native_overlay_032.png)


## 사용자 요청: 추가한 영어 주석 전체 한글화

사전 범위는 이번 통합 branch의 base 대비 추가 설명 주석 전체다. 코드·회귀 검사20개 파일의 영어108줄을 한글로 바꿨다. 변경 전후 주석을 제외한 파일 내용이 바이트 단위로 같음을 검사했고, base 대비 추가 영어 설명 주석 잔여0개, `cargo fmt --all -- --check`·`git diff --check` exit0을 확인했다. [파일별 검증](../assets/pr7382_20260926/comment_translation_validation.json). 이는 설명 주석 변경이며 새 렌더링 검증 통과를 주장하지 않는다. 최종 head의 필수 검증은 후속 기능 보정 뒤 실행한다.


## 메인터너 보정 14: 이월 각주의 물리 쪽 구분선 보존

같은 원본 보고서의 HWP/HWPX 한컴2024 PDF32는 번호 없는30 꼬리 위에 `x94.509..283.528/y1018.725px` 구분선을 표시한다. [두 기준 출력의 선 좌표와 해시](../assets/pr7382_20260926/stage14_independent_geometry.json)로 독립 기대값을 정했다. 기존 조각 조회와 문서가 구분선·번호를 모두 첫 조각에만 표시한다고 가정한 원인을 수정했다.

`boundary.rs`의 두 줄/저장 reset 조각 플래그 생산 → `state/notes.rs`의 실제 예약·투영·영역 동기화 → `picture_footnote.rs`의 동일 `any(draw_separator)` 높이와 선 배치로 이어진다. 각 물리 쪽에서 구분선은 한 번만 예약·배치하며 번호는 앞 조각만 표시한다. 명시적으로 구분선을 생략하는 기존 플래그의 소비 계약은 유지하고, 실제 물리 각주 경계 query가 올바른 표시 값을 생산하게 했다. 본문 위치와 그림색 차이를 이 변경으로 해결했다고 확대하지 않는다.

Producer `477c9aeb8` + Rust/test diff SHA256 `3d707ab6f3dd52af178be2ab424c21a52c7e65d8ac71df1148058479b57662f7`; [전후 검사](../assets/pr7382_20260926/stage14_validation.json).

| 검사 | 결과 | 의미 |
| --- | --- | --- |
| 같은 HWP/HWPX 실제 꼬리, 수정 전 | 2FAIL, exit100,0.247s |32쪽 구분선 수0/기대1로 실제 누락 검출 |
| 수정 후 관련·정상 대조군 | 40PASS, exit0,3.443s | 구분선 좌표·꼬리 위치·번호 무반복, 꼬리+정상 각주에서 선1개, 기존 표/각주/미주/왕복 보존 |
| 새 CLI/쪽수 | build exit0,1m46s;215/PDF215 | 중간 쪽수 유지 |
| Native32 review/overlay 직접 판독 |81.85013%, sweep 전체 exit1 | 구분선 복원; 뒤 문단 위치·그림색 등 잔여로 보류 |
| Native179 review/overlay 직접 판독 |97.53225% | 꼬리와 뒤 정상 각주의 구분선 중복 없음, 앞 단계 무회귀 |
| Native31/178 재캡처 |97.33830% /94.31041% | 선택 지표 무회귀; 이 단계에서 직접 판독을 반복했다고 확대하지 않음 |

[manifest](../assets/pr7382_20260926/stage14_native_manifest.json)·[summary](../assets/pr7382_20260926/stage14_native_summary.json)·[metrics](../assets/pr7382_20260926/stage14_native_overlay_metrics.json). 전체/fresh WASM/최종 필수 검증은 미완료이며 PR 생성·승인 보류다. 추가·수정한 설명 주석은 한글로 작성했다.

![Native32 구분선 복원](../assets/pr7382_20260926/stage14_native_review_032.png)
![Native32 overlay](../assets/pr7382_20260926/stage14_native_overlay_032.png)
![Native179 정상 대조](../assets/pr7382_20260926/stage14_native_review_179.png)
![Native179 overlay](../assets/pr7382_20260926/stage14_native_overlay_179.png)


## 메인터너 보정 15: 실제 본문 앞 빈 줄의 점유 보존

동일 입력 PDF32의 뒤 본문 세 줄은821.061/847.621/874.341px다. 원본 빈 문단426의 저장53340→다음 본문55340HU는1000HU 줄높이+1000HU 간격,26.666px 점유를 입증한다. [독립 좌표](../assets/pr7382_20260926/stage15_independent_geometry.json). 측정과 줄 메트릭 생산은 정상이었으며, 구역 꼬리 흡수 경로가 다음 본문을 제목으로 추정해 그 뒤 문단의 쪽 경계를 앞당겨 적용한 것이 원인이었다.

`absorb_section_tail`의 경계 판단 → `hidden_empty_paras` 및 높이0 항목 → `layout.rs`의 숨김 반환 → 뒤 본문 최종 원점을 추적했다. 글이 있는 문단을 건너뛰는 잘못된 가정을 제거하고 빈 문단들 바로 뒤 첫 본문에서 관측한 reset만 안내 줄 흡수 근거로 유지했다. 문서 ID/숫자 조건을 추가하거나 줄 높이를 임의 보정하지 않았다.

Producer `00e210b06` + Rust/test diff SHA256 `bc7038e0bb950733809b6a8f63109dce27fd792f847fbec299a4c1bfc80ff9d8`; [명령·결과·소스 해시](../assets/pr7382_20260926/stage15_validation.json).

| 검사 | 결과 | 의미 |
| --- | --- | --- |
| 수정 전 실제 본문 좌표 |1FAIL, exit100,0.173s |807.693px vs PDF821.061px, 의도한13.333px 앞당김 검출 |
| 최종 소스 관련·정상 대조군 |43PASS, exit0,3.912s | 세 줄의 실제 위치, 앞선 표/각주/원본215쪽 왕복 및 국제고속선 HWP/HWPX242쪽·원본 reset 소유 보존 |
| 페이지 끝 빈 줄·표 조각 unit |4PASS, exit0,0.012s | 실제 끝 안내 줄 흡수와 prepared-state 생명주기 유지 |
| unit 정책·fmt |exit0 | 정책은 고정 base `eb9142dd7` 비교 |
| CLI·쪽수 |build exit0,1m56s;215/PDF215 | 최신 소스 출력,33쪽 첫 본문428 보존 |
| Native31/32/178/179 |97.33830/94.52341/94.31041/97.53225%, exit0 | 네 쪽 gate 통과, 글꼴 예외 없음 |

빈 줄 unit 첫 실행은 앞 보정에서 바꾼 필드의 옛 이름이 기존 테스트 초기화에 남아 테스트 전 빌드 실패(exit101)했다. 내 누락으로 기록하고 같은0.0 값의 올바른 필드로 고쳤다. 이를 빈 줄 결함 검출 증거로 세지 않으며 수정 후 unit4개와 최종 관련43개를 다시 통과시켰다. 추가·수정 설명 주석은 한글이며 통합 branch의 추가 영어 설명 주석 잔여0개다.

네 쪽의 review·standalone overlay8개를 직접 판독했다. 32쪽은 뒤 본문 위치가 복원되고 구분선·꼬리 각주가 남으며31/178/179의 기존 배치도 유지된다. 그림색·얇은 선·일부 글리프 잔차를 완전 일치로 보고하지 않는다. [manifest](../assets/pr7382_20260926/stage15_native_manifest.json)·[summary](../assets/pr7382_20260926/stage15_native_summary.json)·[metrics](../assets/pr7382_20260926/stage15_native_overlay_metrics.json). **선택 네 쪽 통과이며 전체/fresh WASM/최종 필수 검증 미완료, 다른 보류 페이지 해결 전 PR 생성 보류**다.

![Native32 빈 줄 뒤 본문 복원](../assets/pr7382_20260926/stage15_native_review_032.png)
![Native32 overlay](../assets/pr7382_20260926/stage15_native_overlay_032.png)


## 메인터너 보정 16: 저장 되감김 표의 온전한 행과 바깥 상자 공유

독립 한컴 PDF106의 표29는0..2행,107은3..7행이다. 앞 표 괘선670.947..986.121px, 뒤 표85.027..517.833px, 끝 캡션529.714px 및 뒤 본문592.901px를 [같은 원본 HWP/HWPX 두 출력](../assets/pr7382_20260926/stage16_independent_geometry.json)에서 확인했다. 내용 컷 높이가 실제 온전한 행보다 행마다11.733px 작게 예약돼 뒤 행까지 수용한 원인이다.

기존 HWP 계약과 같은 원본 되감김·일반 행·행 병합/셀 각주 없음 형상의 미편집 단단 HWPX에도 실제 온전한 행 높이를 연결했다. `whole_fit` 생산 → `prepare` 행 높이 → `RowBlockQuery` fit/소비 → 이어받기 plan → `budget/emit` → `table_partial` 실제 원점·점유 끝이 같은 결과를 사용한다. 새 물리 프레임에서 시작하는 온전한 이어받기 행은 바깥 위·아래 여백도 같은 plan으로 다시 연다. 행 내부 컷과 쪽 중간 원점은 이미 소유한 좌표를 유지한다. 문서 ID/새 수치 특례나 golden 완화를 추가하지 않았다. 기존 footer-local4px 값은 변경하지 않았다.

Producer `415b4b046` + Rust/test diff SHA256 `2ca3b8cc2b49496dd93d46d04e808aa35176f28dc6c9b422b6b783f4dd21dead`; [명령·결과·소스 해시](../assets/pr7382_20260926/stage16_validation.json).

| 검사 | 결과 | 의미 |
| --- | --- | --- |
| 수정 전 실제 두 회귀 |2FAIL, exit100,0.171s | 앞 행0..4/기대0..2,38.8px 표 초과 및 뒤 본문 조기 소비 |
| 첫 후보 |61PASS/1FAIL, exit100,3.206s | 행 소유와 꼬리 개선, 캡션1.88px·뒤 본문3.76px 차이 검출; 허용치 유지 후 공통 상자 보정 |
| 최종 관련·정상 대조군 |66PASS, exit0,4.009s | 행·괘선·캡션·뒤 본문·108꼬리 실제 검사, 패딩/원본 프레임/행 병합/중첩/원자 행과 앞 보정 무회귀 |
| 기존 Native·경계 unit |6PASS, exit0,0.227s | 원본 HWP 표 행 경계/그림 캡션, 빈 줄·조각 생명주기 보존 |
| unit 정책·fmt |exit0 | 고정 base `eb9142dd7` 비교 |
| CLI·쪽수 |build exit0,2m04s;215/PDF215 | 앞 표[0,3), 뒤[3,8), 본문1144[0,3)/[3,6) |
| Native106/107/108 |80.27049/85.40785/99.94123%, exit1 |108 개선,106/107 셀 글줄 잔여로 보류 |
| Native94/95 재캡처 수치 |98.91562/94.55592% | 같은 행 계약의 추가 수치; 이 단계에서 직접 판독했다고 확대하지 않음 |

첫 after 빌드는 계약명 변경 중 전달 구조체 한 곳을 빠뜨려 테스트 전 exit101로 실패했다. 내 오류를 바로잡고 전체 참조를 확인했으며 결함 검출 증거로 세지 않았다. 최종 source의6unit/66integration을 실행했다. 편집/텍스트 재조판 제외의 실제 편집 counter는 이번 단계에서 직접 실행하지 않았으며, 코드 보호 조건과 정상 대조군 통과를 실제 편집 검증으로 확대하지 않는다. 추가 설명 주석은 한글이고 추가 영어 설명 주석 잔여0개다.

106/107/108의 review·standalone overlay6개를 직접 판독했다. 표 외곽·캡션·뒤 본문 소유는 개선됐으나106/107 셀 안 글줄이 기준과 다른 위치에 있어 **gate는 보류**다. 글꼴 예외로 분류하지 않고 후속 셀 배치 보정에서 확인한다. [manifest](../assets/pr7382_20260926/stage16_native_manifest.json)·[summary](../assets/pr7382_20260926/stage16_native_summary.json)·[metrics](../assets/pr7382_20260926/stage16_native_overlay_metrics.json). 전체/fresh WASM/최종 필수 검증 미완료이며 PR 생성 보류다.

![Native106 남은 셀 글줄 차이](../assets/pr7382_20260926/stage16_native_review_106.png)
![Native107 남은 셀 글줄 차이](../assets/pr7382_20260926/stage16_native_review_107.png)
![Native108 본문 꼬리와 그림 복원](../assets/pr7382_20260926/stage16_native_review_108.png)
![Native108 overlay](../assets/pr7382_20260926/stage16_native_overlay_108.png)


## 메인터너 보정 17: 온전한 행의 실제 높이와 안 여백 공유

원본 표29의 셀 최소 높이는282HU지만 실제 첫 행은90.933px이며, 저장 안 여백은 위·아래 각각510HU(6.8px)다. 기존 배치가 최소 셀 높이로 다시 판단해 위 여백을0.933px로 축소했고, 첫 조각에서 예산에 예약한141HU 위 여백도 배치 원점에 전달되지 않았다. [동일 입력/PDF의 독립 글줄 좌표](../assets/pr7382_20260926/stage17_independent_geometry.json)는106쪽678.514px,107쪽91.794px다.

온전한 단일 행의 측정 높이 생산 → `table_partial` 실제 셀 상자 높이 → 공통 `resolve_cell_padding_at_height`의 축 선택·이상값 방어 → 실제 글줄 원점으로 연결했다. 행 내부 컷·높이 덮어쓰기·병합 셀 및 측정 높이를 사용하지 않는 중첩 경로는 기존 컷 계약을 유지한다. 첫 되감김 조각은 기존 `host_before_overhead + vert_offset_overhead`를 공통 배치 plan에 전달하고, 예산·확정·배치가 같은 원점과 아래 여백을 소비한다. 새 수치 특례나 허용치 완화는 없다. 이전 #7406의 관련 영어 설명 주석9줄도 한글로 변경했다.

Producer `bacbabc9e` + Rust/test diff SHA256 `d1abcad092e275a6d3bce3c8f8da9f05056f9e379f474feae37dd7c55bbc6627`; [실행·소스·잔여 범위](../assets/pr7382_20260926/stage17_validation.json).

| 검사 | 결과 | 의미 |
| --- | --- | --- |
| 수정 전 실제 좌표 회귀 |1FAIL, exit100,0.166s |106쪽 글줄670.767/기대678.514로 조기 배치 검출 |
| 수정 후 관련·정상 대조군 |76PASS, exit0,3.319s | 실제 셀 안 여백·독립 글줄 좌표, 조밀 표/음수·쓰레기 여백/중첩/병합/행 내부 컷 보존 |
| 라이브러리 대조군 |6PASS, exit0,0.225s | Native 저장 되감김 표·그림 캡션·빈 문단·조각 생명주기 보존 |
| CLI·쪽수 |build exit0,1m57s;215/PDF215 | 필수 페이지 수 일치 |
| Native106/107 |99.94111/99.02564% | 기존80.27049/85.40785%에서 개선, 셀 글줄과 표 경계를 함께 직접 판독 |
| Native94/95/108 |98.91562/94.55592/99.94123% | 선택 gate exit0, 글꼴 예외 없음; 모든 review/overlay 직접 판독 |
| fmt·추가 설명 주석 |exit0 / 영어0개 | 이 단계에서 추가·수정한 설명 주석은 한글 |

[manifest](../assets/pr7382_20260926/stage17_native_manifest.json)·[summary](../assets/pr7382_20260926/stage17_native_summary.json)·[metrics](../assets/pr7382_20260926/stage17_native_overlay_metrics.json). 선택5쪽의 review/standalone overlay10개를 직접 확인했다. 표 행·캡션·뒤 본문과108쪽 꼬리/그림의 누락·겹침은 보이지 않는다. 일부 글자 실루엣·획·간격과 그림 색은 남아 있으므로 완전 픽셀 일치를 주장하지 않는다. 전체 최종/fresh WASM/lint는 미완료여서 PR 생성·승인 보류다.

![Native106 셀 안 여백 복원](../assets/pr7382_20260926/stage17_native_review_106.png)
![Native106 overlay](../assets/pr7382_20260926/stage17_native_overlay_106.png)
![Native107 이어받기 셀](../assets/pr7382_20260926/stage17_native_review_107.png)
![Native107 overlay](../assets/pr7382_20260926/stage17_native_overlay_107.png)
![Native108 후속 내용](../assets/pr7382_20260926/stage17_native_review_108.png)
![Native108 overlay](../assets/pr7382_20260926/stage17_native_overlay_108.png)


## 사용자 요청 추가 반영: 이전 통합의 영어 설명 주석 한글화

이번 통합의108줄뿐 아니라 앞 #7406/#7366 통합이 시작된 `c80a8370a` 이후 추가 설명 주석도 다시 확인했다. 보정17에서 번역한 모델 설명9줄 외에13개 파일의21문단86줄을 한글로 바꿨다. [파일별 검증](../assets/pr7382_20260926/prior_comment_translation_validation.json)에서 주석을 제외한 파일 바이트가 전후 동일하며, 이 시작 base부터 현재 branch까지 추가 영어 설명 주석은0개다. 식별자와 코드 울타리 언어 표시는 유지했다. 주석만 바뀐 범위는 새 기능 검증 통과로 확대하지 않으며, 최종 기능 head의 필수 검증은 계속 진행한다.


## 메인터너 보정 18: 저장 앵커로 입증한 낮은 시작 본문 경계

원본 문단512의44000/46000/48000/0,516의62711..70711/0은 [한컴 PDF44/45의 첫 꼬리](../assets/pr7382_20260926/stage18_independent_geometry.json)와 대응한다. 문단512는 본문 높이의 약61%에서 시작해 기존70% 후보 조건에 막혔지만, 실제 저장 앵커와 현재 흐름은 일치했다. 이 비율이 물리 쪽 소유의 증거를 대신한 것이 원인이다.

`prepare_forced_page_boundary`는 HWPX의 소스·구성 줄 수 일치와 기존 세션 편집 플래그를 확인하고, 공통 `hwpx_saved_reset_fragment_matches_current_flow`에서 단단/일반 본문/비합성 저장 줄/단조 앞 조각/정확한0 reset/현재 앵커 일치를 입증한 경계를 생산한다. 이 결과를 기존 forced boundary → scan/whole-fit/split → 확정·실제 글줄 배치가 소비한다. Native HWP/HWP3의 기존 비율 조건을 광역 완화하지 않았다. 무효 앵커와 합성 줄은 새 경계로 승격하지 않는다.

세션 편집 플래그는 Native HWP5용이므로 HWPX 전역 편집 제외의 증거로 확대하지 않는다. 실제 HWPX 본문 편집은 원래 줄 태그를 보존하지만 내부0 reset을 연속 위치로 재조판한다. 처음에는 모든 줄이 구현 태그라고 잘못 가정해 내 대조 assertion이 실패했고, 실제 생성 계약과 꼬리 무중복으로 바로잡아 통과했다. 새 경계 결함 검출로 세지 않으며 편집 후 한컴 출력 일치로도 확대하지 않는다.

Producer `1216114cb` + 최종 Rust/test diff SHA256 `93502ea7a03e1dd4a3dbf9f6052074cfa2f17da90cb602818a914fd9d54f967c`; [실행·명령·주석 외 코드 동일성·잔여](../assets/pr7382_20260926/stage18_validation.json).87개 검사 뒤 함수의 설명 두 줄만 수정했으며, 당시 실제 diff로 소스를 복원해 현재 함수의 주석 외 바이트가 같음을 확인했다. 별도 추가한 실제 편집 대조군은 최종 코드에서 실행했다.

| 검사 | 결과 | 의미 |
| --- | --- | --- |
| 수정 전 실제 꼬리 회귀 |1FAIL, exit100,0.167s |44쪽 첫 꼬리 누락 검출 |
| 관련·정상 대조군 |87PASS, exit0,12.902s |44/45 실제 꼬리·뒤 표 원점, 합성/불일치 앵커 거절, #6761의315쪽/부분 되감김, Native/HWP3/각주/표 보존 |
| 실제 HWPX 편집 대조 |1PASS, exit0,0.215s | 연속 재조판 위치, 원본 컷 무재사용과 꼬리 무중복 |
| 라이브러리 대조군 |6PASS, exit0,0.196s | 저장 표·그림 캡션·빈 문단·조각 생명주기 |
| CLI·쪽수 |build exit0,1m49s;215/PDF215 | 페이지 수 일치 |
| Native44/45 |71.92862/98.61863% |55.45553/50.97961%에서 개선.44 표 위치는 보류 |
| Native43/46/106/107 |91.32182/97.24002/99.94111/99.02564% | 직접 판독, 앞 보정의 큰 배치 무회귀 |
| Native121/122 |98.11430/98.01917% | 본문 개선. 직접 판독에서 각주160 소유가 여전히 틀림을 확인하여 의미 검증 보류 |
| fmt·영어 주석 |exit0 / 추가 설명0개 | 한글 설명 준수 |

선택8쪽의 review/standalone overlay16개를 직접 확인했다. 자동 선택 gate는44쪽으로 `re_review_required`, sweep exit1이며 글꼴 예외는 없다.121의 각주는 PDF159/160,122는161인데 현재160이122로 이월된다. 자동98%도 이 소유 결함을 해소하지 않는다.43 각주의 일부 줄바꿈/간격 차이도 남는다.44 표 위치와 각주 소유는 다음 개별 보정 대상으로 분리하고, 전체 최종/fresh WASM/lint 미완료로 PR 생성·승인 보류를 유지한다.

[manifest](../assets/pr7382_20260926/stage18_native_manifest.json)·[summary](../assets/pr7382_20260926/stage18_native_summary.json)·[metrics](../assets/pr7382_20260926/stage18_native_overlay_metrics.json).

![Native44 수정 전](../assets/pr7382_20260926/stage18_before_native_review_044.png)
![Native44 본문 복원 및 표 잔여](../assets/pr7382_20260926/stage18_native_review_044.png)
![Native44 overlay](../assets/pr7382_20260926/stage18_native_overlay_044.png)
![Native45 수정 전](../assets/pr7382_20260926/stage18_before_native_review_045.png)
![Native45 꼬리 및 뒤 표 복원](../assets/pr7382_20260926/stage18_native_review_045.png)
![Native45 overlay](../assets/pr7382_20260926/stage18_native_overlay_045.png)
![Native121 각주160 잔여](../assets/pr7382_20260926/stage18_native_review_121.png)
![Native122 잘못 이월된 각주160](../assets/pr7382_20260926/stage18_native_review_122.png)


## 메인터너 보정 19: 빈 호스트 형제 표의 바깥 상자 공유

문단515의 두 ParaTop/TopAndBottom 표에는 각각283HU 바깥 여백이 있다. [동일 원본 PDF44의 괘선·캡션 좌표](../assets/pr7382_20260926/stage19_independent_geometry.json)로 기대값을 고정했다. `host_spacing::resolve`는 이 여백을 계산하지만 `empty_float::prepare`가 위여백을 쓰지 않고, 아래여백도 마지막 fit 면제와 함께 예약에서 빠뜨렸다. 첫 표와 둘째 표의 위치 차이가 누적된 원인이다. 음수 저장 offset을 화면에 맞춘 수치로 덮어쓰지 않았다.

HWPX 빈 호스트의 복수 자리차지 표 중 문단 상단/안쪽 정렬 경로에서 수평 lane 충돌 결과와 host spacing으로 `ParagraphFloatPlacement.table_top/occupied_bottom`을 생산한다. fit은 마지막 아래여백 면제를 유지하되, lane과 다음 표는 실제 점유 하단을 사용한다. `commit_empty_float_table`가 계획을 column metadata에 남기고 layout의 표 원점·실제 paint·후속 흐름은 그 동일 결과를 소비한다. 기존 단일 저장 앵커, Square 형제, 가시 호스트, 가운데·아래 정렬과 Native 경로의 계약은 유지한다. 다른 문서에 맞춘 수치·허용치 변경은 없다.

Producer `369b17e2b` + Rust/test diff SHA256 `5720b4e171e66ca36855cf155f5198a4623e7da629f389eff106db8e3affb04b`; [명령·정확한 전후 결과·잔여](../assets/pr7382_20260926/stage19_validation.json).

| 검사 | 결과 | 의미 |
| --- | --- | --- |
| 수정 전 실제 괘선·캡션 회귀 |1FAIL, exit100,0.179s | 첫 표 상단324.427 vs 독립PDF327.961 검출 |
| 대상 회귀 |40PASS, exit0,3.323s | 두 표 상·하단, 캡션·뒤 본문,45쪽 꼬리·뒤 표 보존 |
| 형제 표·여백 정상 대조군 |31PASS, exit0,0.493s | #6946/#6795 block·partial 형제, #2439/#2279/#1880/#2097 보존 |
| CLI·쪽수·fmt |build exit0,1m51s;215/PDF215;fmt exit0 | 동일 원본과 검증 코드 사용 |
| Native44 |99.68706% |71.92862%에서 개선, 큰 표·캡션 위치 차이 해소 |
| Native43/45/46 |91.32182/98.61863/97.24002% | 앞뒤 본문·표·그림 경계 직접 판독 |

준비 과정에서 suite 자동 배정이018→005로 바뀌어 첫 실행은0검사/exit4였다. 올바른005로 다시 실행한 위 수정 전 실패만 결함 검출로 인정한다. 설명 주석은 한글로 작성했다. 선택4쪽 review/standalone overlay8개를 직접 확인했으며 gate `passed`, sweep exit0, 글꼴 예외 없음이다. 표 괘선 농도, 일부 글자·각주 간격 차이는 남아 완전 일치로 보고하지 않는다.121/122의 각주160 소유와 전체 최종/fresh WASM/lint는 여전히 보류이며 아직 통합 PR을 만들지 않는다.

[manifest](../assets/pr7382_20260926/stage19_native_manifest.json)·[summary](../assets/pr7382_20260926/stage19_native_summary.json)·[metrics](../assets/pr7382_20260926/stage19_native_overlay_metrics.json).

![Native44 형제 표 바깥 여백 보정](../assets/pr7382_20260926/stage19_native_review_044.png)
![Native44 overlay](../assets/pr7382_20260926/stage19_native_overlay_044.png)
![Native45 뒤 본문·표 보존](../assets/pr7382_20260926/stage19_native_review_045.png)


## 메인터너 보정 20: 표시 쪽 본문 점유와 각주 소급 예약 공유

원본 문단1297은 첫 문장 뒤에 각주160 표시가 있고, 저장 앞7줄과 뒤3줄이 121/122쪽에 나뉜다. [원본 XML·동일 입력 HWP/HWPX 기준 PDF 좌표](../assets/pr7382_20260926/stage20_independent_geometry.json)에서 121쪽은 각주159/160, 122쪽은161만이다. 본문 컷은 맞았지만 통째 각주 소급 등록이 Native 전용이어서 HWPX의160을 꼬리 쪽122에 잘못 붙였다. 자동 점수98%도 이 소유 결함을 잡지 못했다.

기존 `native_hwp5_body_footnote_tail_reset`가 단단·단일 각주·비합성 표시와 양수→0 저장 경계 및 실제 꼬리 컷을 입증한 HWPX에 완료 표시 쪽 등록을 연결했다. 소급 등록은 기존 각주가 있는 쪽에 한정하며, 첫 각주·두 줄 충돌·별도 표시 reset의 우선순위를 유지한다. Native 등록에는 새 HWPX fit 조건을 적용하지 않는다.

공통 `plan_fragment`는 같은 `FormattedParagraph` 메트릭으로 흐름 전진량과 수용한 모든 줄 상자의 최대 점유 끝을 함께 생산한다. `commit_split_paragraph_fragment`가 `(문단, 끝 줄)`의 단 상대 점유 끝을 보존하고, `completed_body_fragment_note_fits`는 완료 단의 마지막 실제 조각을 확인한 뒤 그 끝과 단 원점, 기존 각주 영역·추가 예약량으로 공존을 판단한다. 예약 조회와 확정은 같은 `completed_page_note_added_height`를 소비하며 실제 본문·각주는 확정된 조각/영역에서 배치한다. 값이 없는 다단·데코 호스트는 새 경로로 승격하지 않는다. 음수 줄간격이 전진량을 줄여도 앞의 큰 줄 상자의 점유 끝을 잃지 않으며, 뒤 문단 간격은 본문 점유 끝으로 잘못 가산하지 않는다.

Producer `5375b1f33` + 최종 Rust/test diff SHA256 `481af03a6f15c74cdc470c25dc88b379a22e0d94fe284f0fe36a55d93649c472`; [명령·소스별 해시·전후 결과·잔여](../assets/pr7382_20260926/stage20_validation.json).

| 검사 | 결과 | 의미 |
| --- | --- | --- |
| 수정 전 실제 각주 소유 회귀 |1FAIL/1PASS, exit100,0.413s |159 상단1027.347 vs 독립PDF1011.729, 잘못된160 이월 검출 |
| 최종 관련·정상 대조군 |86PASS/1FAIL, exit100,5.771s | 대상45PASS, 정상 대조41PASS; 잔여1개는 아래 별도 캡션 결함 |
| 큰 각주·합성 저장 줄·큰 줄 상자 |PASS | 완료 쪽 강제 예약 거절. 수정 전에도 통과한 대조군을 결함 검출로 세지 않음 |
| Native HWP 동일 원본 |PASS |121/122의 기존 번호·좌표·본문 소유 보존 |
| CLI·쪽수 |build exit0,1m50s;215/PDF215 | 동일 입력, 최종 검증 코드 |
| fmt·source 단위 테스트 정책 |exit0;4205검사/298모듈 | 고정 base `eb9142dd7c` 대비 검사 |
| Native120/123 |99.67776/95.08258% | 앞뒤 표·본문·각주 직접 판독 |
| Native121/122 |98.96031/99.37751% |121에159/160,122에161만 표시됨을 직접 판독 |

점유 생산식의 임시 단위 계약은 줄 상자0..30/5..15px와 흐름 끝15px로 수정 전1FAIL(0.015s)→수정 후1PASS(0.014s)를 확인했다. 신규 `src` 단위 검사 정책이 거절해 진단 소스·로그는 `output`에 보존하고 제출 소스에서는 제거했다. 추가 실물 배치 반례는 `tests/cases/`에 두었다. 수동 큰 줄 반례 두 구성은 보완 전에도 통과해 최대 점유 생산식의 결함 검출 증거로 확대하지 않으며 정상 거절 대조로만 남긴다. 첫 suite 실행은 준비 후005→003 재배정을 놓친 내 오류로0검사/exit4였고, 위 실제 결과와 구분했다.

선택4쪽의 review/standalone overlay8개를 직접 확인했다. gate `passed`, sweep exit0, 글꼴 예외 없음이다. 글자 외곽·간격과123쪽 각주 줄바꿈 차이는 남아 완전 일치를 주장하지 않는다. [수정 전 manifest](../assets/pr7382_20260926/stage20_before_native_manifest.json)·[수정 후 manifest](../assets/pr7382_20260926/stage20_native_manifest.json)·[summary](../assets/pr7382_20260926/stage20_native_summary.json)·[metrics](../assets/pr7382_20260926/stage20_native_overlay_metrics.json).

정상 대조에서 Native HWP90의 표27 캡션 겹침을 발견했고, [보정20 전 정확한 커밋의 대조](../assets/pr7382_20260926/stage20_native90_prior_control.json)에서도 같은 검사1FAIL/exit100/0.208s로 재현했다. devel 실패나 이번 각주 변경의 회귀로 분류하지 않는다. 앞선 통합·보정에서 놓친 결함으로 다음 개별 보정에서 해결한다. 전체 최종 회귀·lint·fresh WASM·전체 시각 gate도 미완료여서 통합 PR 생성·승인 보류를 유지한다.

![121쪽 수정 전 각주160 누락](../assets/pr7382_20260926/stage20_before_native_review_121.png)
![121쪽 각주159·160 소유 복원](../assets/pr7382_20260926/stage20_native_review_121.png)
![121쪽 overlay](../assets/pr7382_20260926/stage20_native_overlay_121.png)
![122쪽 수정 전 각주160 잘못 이월](../assets/pr7382_20260926/stage20_before_native_review_122.png)
![122쪽 각주161만 보존](../assets/pr7382_20260926/stage20_native_review_122.png)
![122쪽 overlay](../assets/pr7382_20260926/stage20_native_overlay_122.png)


## 메인터너 보정 21: 선방출 Native 캡션과 첫 표 조각의 문단 기준 공유

[동일 원본 HWP·한컴 PDF90](../assets/pr7382_20260926/stage21_independent_geometry.json)의 표27은 캡션696.421..709.701px 뒤에718.095px에서 시작한다. 저장 문단962의 vpos46000HU, 표의 문단 기준 양수 오프셋1390HU와 바깥 위여백283HU가 이를 뒷받침한다. 앞 보정에서 첫 조각까지 넓힌 일반 흐름 원점은 선방출 호스트 전진량을 뺀 offset을 사용해700.267px에서 표를 칠했고, 이미 정상 위치에 있는 캡션과 겹쳤다. 단순히 Native를 fallback 대상에서 빼거나 paint에서 clamp하지 않고, 실제 문단 기준 좌표의 생산 결과를 마련했다.

`query_pre_emitted_caption_rowbreak_placement`는 기존 Native 번호 캡션 선방출 조건과 현재 단의 실제 `PartialParagraph` 소유, 비합성·연속 저장 줄, 미편집 단단/문단 상단 정렬을 확인한다. 저장 캡션 앵커를 현재 단 영역 좌표로 변환하고 signed offset·바깥 위여백으로 `ParagraphFloatPlacement`를 생산한다. 첫 조각 budget은 같은 table_top으로 가용 높이·컷을 고르고, commit은 같은 원점과 확정 조각 높이로 점유 끝을 기록한다. layout의 호스트 줄은 같은 stored_host_origin, 표는 resolved_table_top을 소비하므로 별도 para_start_y가 뒤에서 덮어쓰지 않는다. 새 쪽 이어받기는 이 first-only 계획을 재사용하지 않고 기존 실제 바깥여백·소유 계약을 유지한다. 앞 캡션이 현재 단에 없는 이월, 합성/편집/무효 저장 프레임과 다른 정렬은 이 계획으로 승격하지 않는다.

Producer `c34c15bbd` + 최종 Rust/test diff SHA256 `8ac3def6592910224db5c6445c7871333f6d7d328a35dd95109759394710337f`; [정확한 명령·해시·전후 결과](../assets/pr7382_20260926/stage21_validation.json).

| 검사 | 결과 | 의미 |
| --- | --- | --- |
| 수정 전 원본/양수 오프셋 정식 회귀 |2FAIL, exit100,0.498s | 원본 표 상단700.267 vs 독립PDF718.095 검출 |
| 최종 대상·정상 대조군 |89PASS, exit0,6.583s | 대상47와 정상42. 앞 단계의 Native90 캡션 실패도 PASS |
| 실제 표·뒤 내용 |PASS | 원본 첫 조각 하단995.711, 캡션 불겹침,90 관계 행/91 끝 행·캡션 무중복 |
| signed 양수 offset 변형 |PASS |500HU 증가에 표만6.667px 이동, 캡션 원점·뒤 행 소유 보존. 수동 좌표 계약 |
| CLI·쪽수 |build exit0,1m55s;HWP/HWPX215/PDF215 | 검증 코드와 동일 입력 |
| fmt·source 단위 테스트 정책 |exit0;4205검사/298모듈 | 고정 base `eb9142dd7c` 비교 |
| Native HWP90/91 |83.32319/96.28253% | 캡션·표 겹침 해소.90 본문 줄바꿈 차이는 보류 |
| Native HWPX106/107 |99.94111/99.02564% | 앞 보정의 공통 첫 표 여백·이어받기 배치 보존 |

새 Native HWP90/91과 HWPX106/107의 review/standalone overlay8개를 직접 확인했다.90은68.56281→83.32319%로 개선됐지만 앞 본문의 줄바꿈 차이가 남아 HWP gate `re_review_required`, sweep exit1이다. HWPX 선택 gate는 `passed`, exit0이다. 글꼴 예외를 쓰지 않고 본문 잔여 원인을 다음 개별 보정으로 조사한다. 표·캡션 부분 개선을 문서 전체 승인으로 확대하지 않으며 전체 최종 회귀·lints·fresh WASM·시각 gate 미완료로 통합 PR을 만들지 않는다.

[HWP manifest](../assets/pr7382_20260926/stage21_native_hwp_manifest.json)·[summary](../assets/pr7382_20260926/stage21_native_hwp_summary.json)·[metrics](../assets/pr7382_20260926/stage21_native_hwp_overlay_metrics.json), [HWPX manifest](../assets/pr7382_20260926/stage21_native_hwpx_manifest.json)·[metrics](../assets/pr7382_20260926/stage21_native_hwpx_overlay_metrics.json).

![Native90 수정 전 캡션 겹침](../assets/pr7382_20260926/stage21_before_native_hwp_review_090.png)
![Native90 캡션·표 보정 및 본문 잔여](../assets/pr7382_20260926/stage21_native_hwp_review_090.png)
![Native90 overlay](../assets/pr7382_20260926/stage21_native_hwp_overlay_090.png)
![Native91 끝 행·뒤 본문 보존](../assets/pr7382_20260926/stage21_native_hwp_review_091.png)
![HWPX106 원점·위여백 보존](../assets/pr7382_20260926/stage21_native_hwpx_review_106.png)
![HWPX107 이어받기·뒤 본문 보존](../assets/pr7382_20260926/stage21_native_hwpx_review_107.png)

## 메인터너 보정22 — 저장 목록 줄 원점과 뒤 표의 실제 각주 경계

- 사전 근거: Native957의 비합성 저장9줄 원점496HU/폭44856HU는 해소 여백으로 계산한496..45352HU와 같다. 기존 목록 원점 차단이0..44856HU로 바꾸어 정확 프레임 수용이 실패했고,90쪽꼬리8줄이7줄로 재조판됐다. 독립 PDF는 마지막269.861px/후행958 첫296.421px다. [독립 입력·좌표](../assets/pr7382_20260926/stage22_independent_geometry.json).
- 생산→소비: `ParagraphBox::for_stored_body_rows`가 가시·비합성·유효 원본 분할의 실제 여백 원점과 첫 저장 원점 일치를 확인해 물리 상자를 복원한다. 공통 composer와 증거 probe가 같은 상자를 받고, 기존 전체행 폭/원점·stale/controls/float 수용 검사는 유지한다. typeset 확정 컷과 paint는 같은 구성 줄을 소비한다. 편집·빈 목록·NO_LS·합성·다른 원점은 기존 경로다.
- 첫 후보는 Native 목표/빈 목록 커서 통과이나 HWPX216쪽 및19FAIL이었다. 본문 복원 후 뒤 표962가 실제 각주 영역 외40px를 이중 예약해 관계 행을 밀었다. 목록 조건을 Native 예외로 좁히지 않았다. 원본 HWPX의 같은 ordinary RowBreak/후속 저장 되감김/기존 각주/셀 각주·rowspan 없음 경로를 실제 각주 경계 소비에 연결했다. 편집·표 재조판·합성·다단은 새 HWPX 저장 계약에서 제외했다.

| 검사 | 실제 전후 결과 | 판정 |
| --- | --- | --- |
| Native90 원본 꼬리/후행 문단 | 1FAIL(exit100,0.205s):7≠8 → PASS | 충족 |
| HWPX90 괘선/관계 행/이어받기 | 첫 후보1FAIL(exit100,0.289s):하단931.947≠995.711 → PASS | 충족 |
| 집중·정상 대조군 | 최종96PASS/0FAIL(exit0,5.424s),threads8 | 충족 |
| 잘못된 폭·합성 목록/빈 목록 편집 커서 | 원본 줄 수용 거절·내용 보존 및 기존#1329/#5677 통과 | 충족 |
| fmt/소스 단위검사 정책 | exit0 / base eb9142dd7,4205검사·298모듈 | 충족 |
| 새 CLI / 쪽수 | exit0,1m45s / Native HWP215·HWPX215,각 PDF215 | 충족 |
| Native HWP90/91 | 94.09514%/96.28253%,gate passed,exit0 | 선택 자동 gate 충족 |
| Native HWPX90/91 | 94.09514%/95.99030%,gate passed,exit0 | 선택 자동 gate 충족 |
| 직접 판독 | 양 입력90/91 review·standalone overlay 8개 확인; 본문 꼬리·관계 행·캡션/괘선 개선 | 해당 보정 의미 충족 |
| 남은 직접 차이 | HWPX91의 캡션 각주142 누락, 목록 표식/본문 가로 시작 차이 | 미충족, 별도 후속 보정 |
| 전체 회귀·세 Clippy·fresh WASM·전체 시각 | 아직 실행 전 | 미검증 |

- [최종 source 해시·명령·결과](../assets/pr7382_20260926/stage22_validation.json). source producer `865d9e8605ae70e1dbb54d8566e462683d988725`, Rust/test diff SHA256 `87afc5898223467cf57df1dc05b84df50f09c642b142b0e71f4dc85bed706f1e`. 글꼴 예외와 허용치 변경 없음.
- [HWP90 review](../assets/pr7382_20260926/stage22_native_hwp_review_090.png) · [overlay](../assets/pr7382_20260926/stage22_native_hwp_overlay_090.png) · [HWP91 review](../assets/pr7382_20260926/stage22_native_hwp_review_091.png) · [overlay](../assets/pr7382_20260926/stage22_native_hwp_overlay_091.png).
- [HWPX90 review](../assets/pr7382_20260926/stage22_native_hwpx_review_090.png) · [overlay](../assets/pr7382_20260926/stage22_native_hwpx_overlay_090.png) · [HWPX91 review](../assets/pr7382_20260926/stage22_native_hwpx_review_091.png) · [overlay](../assets/pr7382_20260926/stage22_native_hwpx_overlay_091.png).
- 해결 범위는 저장 줄 소유와 실제 표 분할 경계다. 자동90% 이상을 캡션 각주 소유의 완료 증거로 삼지 않으며, 각주142 누락을 다음 개별 단계로 추적한다. 영어 설명 주석 추가 없음. 로그·진단·generated suite는 output/ignored 작업 증적이며 커밋하지 않는다.

## 메인터너 보정23 — 분할 표 캡션의 형제 각주 등록

- 원인: `section::controls → register_body_footnote(has_table=true)`에서 HWPX는 Native 전용 표 캡션 각주 분기와 `!has_table` 분기 모두 제외되어 각주 참조가 발행되지 않았다. 원본937/962/1000의 표 다음 형제 각주138/142/147은 셀 내부 각주가 아니다. [독립 입력·PDF 가시 글자 영역](../assets/pr7382_20260926/stage23_independent_geometry.json).
- 생산→소비: 원본 유효 HWPX의 단단·단일 번호 캡션 표·직후 단일 형제 각주를 기존 구조 판별에 연결한다. 실제 확정 `Table` 또는 끝 행/빈 끝 컷 `PartialTable`의 소유를 format 중립 `table_host_terminal_fragment_placement`로 확인한 뒤, 같은 composed content 높이의 fit 검사→FootnoteRef·예약 높이→최종 각주 배치를 소비한다. Native 기존 경로와 큰 각주의 기존 다음 쪽 수용 계약은 보존한다. 편집·무효 텍스트 분할·합성·표 재조판·다단은 새 원본 HWPX 경로에 승격하지 않는다.
- 신규 검사 첫 실행의 Native 좌표 assertion은 PDF 가시 글자 상단과 실제 논리 줄 상단을 같은 측정값으로 비교한 내 오류였다(918.16 vs920.368571). 제품 좌표를 clamp하지 않고 두 측정값을 구분해 기준 글자 상단의 실제 줄 상자 소속과 쪽 유일성을 검사했다. 교정된 수정 전 실행은 Native1PASS/HWPX3FAIL이며, 기존 golden/visual 허용치 변경은 없다.

| 검사 | 실제 결과 | 판정 |
| --- | --- | --- |
| 정식 원본 각주 소유 전후 | 1PASS/3FAIL(exit100,0.652s) → 세 HWPX 각주 및 Native 대조 통과 | 충족 |
| 집중·정상 대조군 | 96PASS/0FAIL(exit0,8.319s),threads8 | 충족 |
| fmt/소스 단위 정책 | exit0 / base eb9142dd7,4205검사·298모듈 | 충족 |
| 새 CLI / 원본 HWPX 쪽수 | exit0,1m58s /215쪽,PDF215쪽 | 충족 |
| Native HWPX86/87 | 97.15232%/98.92215% | 선택 gate 충족 |
| Native HWPX90/91 | 94.09514%/96.28253% | 선택 gate 충족 |
| Native HWPX94/95 | 98.91562%/97.85246% | 선택 gate 충족 |
| 직접 판독 | 영향6쪽의 review·standalone overlay12개; 각주138/142/147의 번호·본문 복원과 뒤 내용/표 경계 확인 | 해당 보정 의미 충족 |
| 편집·재조판·다단 확장 | 이번 원본 출력의 대용으로 주장하지 않음 | 비적용/미검증 |
| 전체 Native/fresh WASM·최종 회귀/lint | 최신 전수 inventory와 최종 검증을 이어서 실행할 단계 | 미검증 |

- [정확한 source 해시·명령·결과](../assets/pr7382_20260926/stage23_validation.json): producer `04b4162f2`, Rust/test diff SHA256 `f5f7e03e5be8678e2b3bb3b3bed6891ca8dc62f0968ef709f45a0b0cc9dd087d`. 자동 gate는 passed/exit0이며 글꼴 예외 없음.
- [87쪽 review](../assets/pr7382_20260926/stage23_native_hwpx_review_087.png) · [overlay](../assets/pr7382_20260926/stage23_native_hwpx_overlay_087.png), [91쪽 review](../assets/pr7382_20260926/stage23_native_hwpx_review_091.png) · [overlay](../assets/pr7382_20260926/stage23_native_hwpx_overlay_091.png), [95쪽 review](../assets/pr7382_20260926/stage23_native_hwpx_review_095.png) · [overlay](../assets/pr7382_20260926/stage23_native_hwpx_overlay_095.png).
- 남는 목록 표식/가로 시작·글자 메트릭 차이와 전체 전수 gate는 이 각주 등록 통과로 완료 처리하지 않는다. 영어 설명 주석 추가 없음. 모든 로그·진단·generated 파일은 output/ignored 증적이며 커밋하지 않는다.


## 메인터너 보정24 — 내부 저장 줄 앵커와 내용 없는 물리 첫 조각

### 사전 근거와 공통 소비 경로

- 그림7의 원본 문단246은 가시 저장5줄을 가지며, 원본 제어 UTF16 위치207은171..230 구간의 내부 줄(vpos12000HU)에 속한다. 기존 Native의 호스트 뒤 원점과 HWPX의 첫 줄 원점은 둘 다 잘못된 앵커였다. `12000+offset3618+outerTop283+height18534+outerBottom283=후행34718HU`로 닫히는 저장 프레임과 PDF 그림 상단296.794667px를 확인했다. 문서 번호나 좌표 상수를 구현 분기로 사용하지 않는다.
- `stored_control_line_indices → stored_interior_control_table_frame → whole_fit`은 유효 미편집 원본의 내부 줄 소유와 실제 측정 높이/저장 전체 프레임 일치를 확인해 공통 `ParagraphFloatPlacement`를 생산한다. fit 예산·최종 배치가 같은 table_top/occupied_bottom을 소비한다. 첫/끝 줄, 무효·합성·편집·표 재조판·다단은 새 저장 계약으로 승격하지 않는다.
- 그림8의 표250은 첫 조각13678HU 동안 내용 유닛을 소비하지 않지만 물리 공간을 점유한다. 첫 셀25619HU 전체를 무시하거나 첫 쪽에서 그림을 잘라 보이는 처리는 원인 해결이 아니다. 이어받기 요구 높이는 `max(25619-13678,17772+282)+1282+283+283=19902HU`이며 후행 저장 vpos와 정확히 닫힌다.
- `saved_picture_row_empty_opening_frame → prepare의 실제 각주 경계/수용 검사 → scan의 양수 높이·빈 컷[0] → emit의 남은 물리 높이 → budget/PartialTable paint`가 같은 계획을 소비한다. 첫/다음 조각의 위여백과 끝 조각의 아래여백도 같은 확정 소유에 연결한다. 첫 조각 각주7, 다음 조각 각주8/9/10 및 뒤 본문을 정식 최종 tree에서 검사한다.
- [독립 입력·HU·PDF 좌표](../assets/pr7382_20260926/stage24_independent_geometry.json), [대조군 단일 속성 변경·입력 해시](../assets/pr7382_20260926/stage24_input_provenance.json), [전체215쪽 대조 좌표](../assets/pr7382_20260926/stage24_all_page_controlled_geometry.json). 높이0은 각주8을 앞쪽으로 이동시키므로 원본 통과의 대용이 아니다. 높이40000은216쪽과 더 큰 이어받기로 선언 높이 전역 무시를 기각한다. noAdjust 변경은 원인이 아니었다.
- [한컴 가시 괘선 대조 PDF](../../../pdf/issue7379/liver7379-table250-visible-border-2024.pdf)는 두 셀의 같은 .12mm NONE 괘선만 SOLID로 바꾼 수동 대조군이다. 원본과215쪽 전체의 텍스트/그림 bbox가 정확히 같으며,12쪽 빈 괘선820.062663..1002.263997px와13쪽86.945312/327.321370/344.422689px가 물리 첫 조각·이어받기를 드러낸다. 대조군 출력을 원본 일치로 보고하지 않는다.

### 전후 결과와 남은 보류

| 검사 | 실제 결과 | 판정 |
| --- | --- | --- |
| 정확한 보정 전6fa 코드 + 최종 신규6개 회귀 | 0PASS/6FAIL,exit100,0.275s | 앵커·빈 조각·이어받기 결함 검출 |
| 최종 집중·정상 대조군 | 102PASS/0FAIL,exit0,6.337s,threads8 | 해당 소유/좌표/물리 높이 계약 충족 |
| fmt·새 CLI | 각각exit0 | 동일 Rust/test diff의 불변 CLI로 캡처 |
| manifest / source 단위 정책 | exit0 /4205검사·298모듈,base eb9142dd7 | fmt 뒤 prepare하고 별도 명령의 exit 확인 |
| Native HWP11/12/13/14 | 80.37557/99.76685/77.85867/82.03316% | exit1,re_review_required |
| Native HWPX11/12/13/14 | 68.96816/99.76685/77.05428/81.51575% | exit1,re_review_required |
| 직접 판독 | 두 형식4쪽 review·standalone overlay16개 | 12쪽 그림7/뒤 본문,13쪽 상단 그림8/캡션/각주 개선 |
| 남은 차이 | 13쪽 하단 표2·그림9 위치,11·14쪽 차이 | 미충족; 다음 개별 보정 |
| 최종 전체 회귀·Clippy·Skia·fresh WASM·전수 시각 | 미완료 | 미검증 |

[정확한 해시·명령·결과](../assets/pr7382_20260926/stage24_validation.json): producer `6fa58aef8813c185ce113754fbda60792db2a84e` + Rust/test diff SHA256 `5cdda382e642690e2a2728c11667e40eee305b55abbd8864c0718d748ae369a9`, 불변 CLI SHA256 `8477eeca956b269d3cc33608f2b355e048da784df2f99ef728c28587586b8b04`.
중간 후보의 ctrl_idx 누락 컴파일 실패는 수정 후 재검증했으며 결함 재현으로 세지 않는다. 첫 manifest 검사의 fmt 뒤 파생 drift도 prepare 후 별도 check에서 통과했다. 기존 baseline·golden·시각 허용치와 글꼴 예외를 변경하지 않았다. 추가한 설명 주석은 한글이다. 로그/output/generated suite는 커밋하지 않는다.

[보정 전 전체 Native inventory](../assets/pr7382_20260926/stage23_full_native_hold_inventory.json)는6fa의215쪽 전수 완료/32쪽90% 미만/exit1이다. 보정24 뒤 전수 통과로 바꾸어 보고하지 않는다. 단계16의 실제 TABLE 편집/재조판 반례도 최종 검증 전에 남아 있다.

[HWP manifest](../assets/pr7382_20260926/stage24_native_hwp_manifest.json) · [metrics](../assets/pr7382_20260926/stage24_native_hwp_overlay_metrics.json), [HWPX manifest](../assets/pr7382_20260926/stage24_native_hwpx_manifest.json) · [metrics](../assets/pr7382_20260926/stage24_native_hwpx_overlay_metrics.json).

![HWP12 그림7와 뒤 본문 복원](../assets/pr7382_20260926/stage24_native_hwp_review_012.png)
![HWP12 overlay](../assets/pr7382_20260926/stage24_native_hwp_overlay_012.png)
![HWPX13 그림8 복원과 하단 잔여](../assets/pr7382_20260926/stage24_native_hwpx_review_013.png)
![HWPX13 overlay](../assets/pr7382_20260926/stage24_native_hwpx_overlay_013.png)


## 메인터너 보정25 — 나란히 배치된 그림 표의 공통 바깥 프레임

### 독립 근거와 실제 소비 경로

- 원본 문단259의 비TAC Square/ParaTop/ColumnLeft 표2·그림9는 같은 유효 저장 줄40102HU를 공유한다. 기존 경로는 바깥 위/왼쪽 여백과 양수334HU 오프셋을 빠뜨렸다. [입력·대조군 provenance](../assets/pr7382_20260926/stage25_input_provenance.json).
- [한컴 가시 괘선 PDF](../../../pdf/issue7379/liver7379-table259-visible-border-2024.pdf)는 네 셀의 같은 .12mm NONE 괘선만 SOLID로 바꿨다. 원본과215쪽 전체 텍스트/그림 bbox가 정확히 같으며 표2 원점98.346670/625.395996px, 그림9 원점410.338664/620.921346px를 확인했다. [괘선 좌표](../assets/pr7382_20260926/stage25_border_pdf_geometry.json). 음수 오프셋을0으로 바꾼 별도 한컴 출력도 전체215쪽 bbox가 같아, 이 형제 표의 음수 값을 새 상단 앵커로 해석하지 않는다. [대조군](../assets/pr7382_20260926/stage25_offset_control_provenance.json) · [좌표](../assets/pr7382_20260926/stage25_offset_pdf_geometry.json).
- `stored_square_sibling_outer_frame → empty_float::prepare → ParagraphFloatPlacement(table_left/table_top/occupied_bottom) → layout::Table → table_layout 최종 원점`이 같은 확정 프레임을 소비한다. lane은 바깥 상자를, 실제 표는 그 안의 여백을 소비한다. 편집·합성·표 재조판·다단·외부 캡션은 원본 계약에 승격하지 않는다. 문서 번호·좌표 상수·paint clamp 분기는 추가하지 않았다.
- 첫 후보에서 최종 x를 inline override로 전달해 그림9의 가로 오프셋이 다시 더해졌다(722.61333px). 실제 최종 소비 지점에 공통 원점 쌍을 전달해 이중 적용을 수정했다. 양수 오프셋 수동 IR 변경의10px 이동 기대값은 독립 근거가 부족해 정식 검사에서 제외하고 진단 실패 기록을 보존했다. 해당 편집 계약은 미검증이며 관측 출력에 맞춘 assertion으로 통과시키지 않았다.

| 검사 | 실제 결과 | 판정 |
| --- | --- | --- |
| 정확한 b617 코드 + 원본 신규 좌표2개 | 0PASS/2FAIL,exit100,0.198s | 여백·원점 결함 검출 |
| 최종 집중·정상 대조군 | 131개128PASS/3FAIL,exit100,5.821s,threads8 | #6950 기존 회귀3개 미충족 |
| #6950 수정 전 b617 대조 | 25개22PASS/3FAIL,exit100,0.174s | 이번 Square 보정 전부터 있는 통합 회귀; 해결 필요 |
| 원본 HWP/HWPX·음수0 대조·합성 호스트 신규4개 | 모두PASS | 해당 원본 프레임·적용 제외 계약 충족 |
| fmt/manifest/소스 단위 정책 | exit0 /6196 static attrs /4205검사·298모듈,base eb9142dd7 | 충족 |
| 새 CLI·쪽수 | build exit0,1m49s /두 형식215쪽,PDF215쪽 | 충족 |
| Native HWP11/12/13/14 | 80.37557/99.76685/96.31316/82.03316% | exit1,re_review_required |
| Native HWPX11/12/13/14 | 68.96816/99.76685/94.95851/81.51575% | exit1,re_review_required |
| 직접 판독 | 두 형식4쪽 review·standalone overlay16개 | 13쪽 표2·그림9·캡션 복원,상단 그림8/각주 보존 |
| 남은 차이·필수 검증 | 11·14쪽 위치/크기,전수 검증·fresh WASM·전체 회귀/lint/Skia | 미충족/미검증 |

[정확한 source·diff·CLI 해시와 최종 명령](../assets/pr7382_20260926/stage25_validation.json). 최종 검사는 근거 없는 수동 IR 검사를 제외한131개 결과다. 이전132개 실행은 최종 검사로 재사용하지 않는다. 캡션의 PDF 가시 글자 상단과 저장HU 논리 줄 상단은 구분한다. 기존 baseline·golden·시각 허용치·글꼴 예외를 변경하지 않았다. 추가 주석과 이번에 손댄 기존 설명 주석은 한글로 바꿨다. 모든 로그는 output에 두며 커밋하지 않는다.

![HWP13 표2·그림9 복원](../assets/pr7382_20260926/stage25_native_hwp_review_013.png)
![HWP13 overlay](../assets/pr7382_20260926/stage25_native_hwp_overlay_013.png)
![HWPX13 표2·그림9 복원](../assets/pr7382_20260926/stage25_native_hwpx_review_013.png)
![HWPX13 overlay](../assets/pr7382_20260926/stage25_native_hwpx_overlay_013.png)

다음 개별 단계는 #6950의 객체 전용 문단에서 표 사이 간격과3쪽 계약을 복원한다. 보정19의 복수 TopAndBottom 원점 생산·예약·실제 배치를 최신 devel 대조와 연결해 확인하며, 기존 테스트를 완화하지 않는다.


## 메인터너 보정26 — 확정된 지연 좌표 기준의 마지막 줄 소비

### 사전 근거와 기대값 교정

- 현재 devel `eb9142dd7`에서 #6950의 기존25개는 모두 PASS(0.221s)다. 동일 원본 [한컴 PDF](../../../pdf/hwpx/20260909-para-table-2024.pdf)를직접확인하면 첫 표98.132..147.199px, 세 번째 표199.461..492.261px로 바깥 여백이 필요하다. devel의 첫 표94.5..143.6,세 번째180.9..474.0은위 여백·표 간 여백을 잃고 있다. 녹색 검사를 PDF 일치로 승격하지 않는다. [원본SHA·HU·PDF 좌표](../assets/pr7382_20260926/stage26_independent_geometry.json).
- 기존 ‘표 간격0’ 기대값은 PDF와 달랐다. 원본 각283HU의 앞 아래+뒤 위 여백 합7.546667px로 교정했으며 허용치0.02px는 유지했다. 관측 출력에 맞춰 허용치를 늘린 것이 아니며, 이 교정은 페이지 결함을 숨기지 않는다. 교정된 기존 검사도 수정 전에는4≠3쪽으로 실패했다.
- 추가 쪽의 원인은 마지막 예산 설명 줄(문단5,vpos68707HU,lh1200HU)이다. 실제 본문933.573333px 안에916.093333..932.093333px로 들어가며 PDF 가시 글자1010.2556..1026.2426px도 본문94.466667..1028.04px 안이다. 그러나 소비자가 확정 lazy 기준0을 버리고 첫 호스트30164를 다시 선택해 마지막 줄 소유의 증거를 거절했다. 4px 안전마진을 제거하거나 출력을 clamp하지 않는다.
- 실제 연결은 `section/vpos::vpos_snap_current_height → st.vpos_lazy_base → paragraph::prepare_forced_page_boundary → whole_fit의 저장줄 경계/fit → 실제 페이지 소유`다. page_base가 있으면 기존 우선, 없으면 이미 확정된 lazy_base, 둘 다 없으면 기존 첫 항목 fallback을 쓴다. 기존 source 유효성·흐름 일치·원본 쪽 끝 검사를 유지한다.

| 검사 | 실제 결과 | 판정 |
| --- | --- | --- |
| 정확한 보정 전ef572b3c9 + 최종26개 | 22PASS/4FAIL,exit100,0.173s | 쪽수·마지막줄소유결함검출 |
| 수정 후 #6950 전체 | 26/26PASS |3쪽·표여백·문단종료·마지막줄소유충족 |
| 확대12모듈 |140개138PASS/2FAIL,exit100,17.277s,threads8 | #6312 보류2개 |
| #6312 수정 전 불변CLI 대조 |0PASS/2FAIL,exit100,0.667s,365.7px | 보정26 전부터있는통합차이;다음단계해결 |
| fmt/manifest/source-unit |exit0/6197 static attrs/4205검사·298모듈,base eb9142dd7 |충족 |
| 새 CLI |release-test build exit0,1m36s |불변사본으로새캡처 |
| #6950 전체3쪽 |98.66178/99.1346/79.76857%,exit1 |3쪽위치차이로재검토 |
| #7382 HWPX43–46쪽 |91.32182/99.68706/98.61863/97.24002%,exit0 |선택gate충족;215쪽유지 |
| 직접 판독 |위7쪽 review·standalone overlay14개 +devel3쪽6개 |첫쪽여백·마지막줄복원,44쪽표19/20·캡션·뒤본문보존 |
| 전체최종검증 |전체회귀/세Clippy/Skia/fresh WASM/최신전수미완료 |미검증 |

[source·명령·해시·결과](../assets/pr7382_20260926/stage26_validation.json). 공유 캐시의 다른 checkout 라이브러리로 첫 검사가 빌드 실패(exit101)한 것은 결함 재현으로 세지 않았다. 캐시 삭제 없이 root 소스를 재빌드하고 같은 명령에서 의도한 페이지 결함 실패를 확인했다. 설명 주석은 한글로 바꿨고 base c80a8370a 이후 추가 영어 설명 주석은0개다. 로그·generated·output은 커밋하지 않는다. 최종 head 시각/WASM 증적은 이 중간 결과로 대체하지 않는다.

![#6950 첫쪽복원](../assets/pr7382_20260926/stage26_native_6950_review_001.png)
![#6950 첫쪽 overlay](../assets/pr7382_20260926/stage26_native_6950_overlay_001.png)
![#6950 3쪽남은위치차이](../assets/pr7382_20260926/stage26_native_6950_review_003.png)
![#7382 44쪽정상형제표보존](../assets/pr7382_20260926/stage26_native_hwpx_review_044.png)

#6950의3쪽하단표여백/원점과#6312의호스트줄·형제표차이는보류다. 기존 골든·실루엣 허용치·글꼴 예외를 변경하지 않으며 다음 개별 보정에서 독립 출력과 연결한다.

## 메인터너 보정27 — 가운데 정렬을 소유한 1×1 표의 물리 프레임

### 사전 분석

- #6950 원본3쪽의 문단29는54805HU 최소 높이의 가운데 정렬 셀 안에52416HU 중첩 표와 위/아래283HU 바깥 여백을 담는다. 셀 안 여백은141HU씩이다. 내용52982HU를 담고도1541HU의 정렬 공간이 남는다. PDF 안쪽 표의 위 괘선307.823px와 아래1005.94px가 독립 기준이며, 현재297.9px 상단은 약10px 위다.
- `height_measurer::measure_table_impl`은 빈1×1셀의 첫 표를 바로 반환하고, `layout::table_layout`도 외곽 셀을 제거해 자식에 안 여백만 더한다. 두 경로 모두 외곽 셀의 최소 높이와 가운데/아래 정렬을 잃는다. 글자처럼 표라는 속성은 이 공간을 제거할 근거가 아니다.
- 선언 내용과 실효 안 여백보다 큰 최소 높이에 가운데/아래 정렬이 지정된 상자는 투명 래퍼가 아니다. 측정·배치의 동일 판별에서 외곽 셀을 보존하고 기존 일반 셀의 물리 높이·정렬 결과를 소비하도록 한다. Top 정렬 및 정렬 공간 없는 기존 빠른 경로는 대조한다. 문서 번호·좌표 상수·출력 clamp를 추가하지 않는다.
- 신규 정식 검사는 원본 최종 tree에서 외곽 셀 높이, 안쪽 표 유일성·PDF 상단·점유 끝과3쪽 소유를 검사한다. 실제 전후 실행과 #6621/#6648/#7063 정상 대조 및3쪽 직접 Visual Sweep 뒤 결과를 연결한다. #6312는 이번 코드 수정과 분리해 다음 단계에서 다룬다.

### 보정 결과

- 공유 판별 `single_table_wrapper_has_vertical_alignment_space`에서 실효 안 여백과 안쪽 표의 선언 높이·바깥 여백을 포함해 외곽 셀의 남는 정렬 공간을 확인한다. 측정과 paint가 이 판별을 함께 소비해 이 셀을 제거하지 않는다. 이후 기존 일반 셀 경로의 최소 물리 행 높이와 실제 내용 높이가 가운데/아래 정렬의 원점·점유 끝을 결정한다. 실제 최종 tree는 외곽292.3/높이730.7px, 안쪽308.2/높이698.9px다(진단 JSON의0.1px 표기).
- 수정 전 정식 원본 검사는 외곽 표 소실로0PASS/1FAIL,exit100,0.028s였다. 수정 후 최초 집중·정상 대조33/33PASS,exit0,0.273s였다. Top/Center/Bottom 수동 IR 정렬 불변식을 포함한 최종 집중은34PASS/0FAIL,exit0,0.303s다. 수동 IR을 한컴 생성 출력 증거로 승격하지 않는다.
- #6950 전체3쪽은98.66178/99.13460/99.78442%,exit0,gate passed다. 3쪽의 목표/투자·화살표·지원방향 표 전체와 아래 끝, 앞2쪽의 제목·본문/표·로고를 새 review/standalone overlay6개에서 직접 확인했다. 기존3쪽79.76857%의 위치 차이를 해결했다. 글꼴 폭·얇은 선/색 표현의 잔여를 완전 픽셀 일치로 보고하지 않는다.
- #7382 원본HWPX11–14쪽도 새로 비교·직접 판독했다.68.96816/99.76685/94.95851/81.51575%,exit1이다.12/13쪽의 앞 보정은 유지됐고11/14쪽의 큰 위치 차이는 보류다. 원본215/PDF215쪽은 유지됐다. 이전 전수 snapshot을 현재 전수 통과로 재사용하지 않는다.
- fmt·manifest 정책(6199 static attrs)·source-unit 정책(4205검사/298모듈)은 고정base eb9142dd7에서 통과했다. 불변 새 CLI는release-test 빌드exit0,1m43s다. [source·명령·파일/CLI 해시와 최종 결과](../assets/pr7382_20260926/stage27_validation.json), [독립 PDF/HU 근거](../assets/pr7382_20260926/stage27_independent_geometry.json).
- 이번 변경 주석과 손댄 기존 영어 설명을 한글로 교정했다. baseline·golden·시각 허용치·글꼴 예외는 변경하지 않았다. 모든 로그는output에 남기며 커밋하지 않는다. #6312의 두 기존 실패·전체 최종 검증은 다음 단계에서 다루며 아직 통합 PR을 만들지 않는다.

![#6950 3쪽 정렬 공간 복원](../assets/pr7382_20260926/stage27_native_6950_review_003.png)
![#6950 3쪽 overlay](../assets/pr7382_20260926/stage27_native_6950_overlay_003.png)

보존된 외곽 셀이 실제로 분할·이월되는 물리 소유 경계는 이번 단계에서 실행하지 않았다. 해당 경로는 미검증이며 제출 전에 실제 컷·점유 끝·자식 내용의 누락/중복을 대조한다. 온전한 셀과 세 정렬의 통과를 이 경로의 증거로 대신하지 않는다.

## 메인터너 보정28 — 다른 셀의 확장이 빈 앵커 줄의 소유를 바꾸지 않도록 보정

### 사전 분석과 독립 기준

- #6312 머리 표의 세 셀은 최소2994HU를 공유한다. 왼쪽 로고의 유효 그림 흐름2704HU와 빈 줄1000HU는 각각 이 최소 안에 들어간다. 오른쪽 글자처럼 그림의 저장 줄3194HU+실효 안 여백282HU는 실제 행을3476HU로 확장한다. 다른 셀의 확장은 로고 셀의 빈 앵커 줄을 그림 위에 다시 쌓아야 한다는 증거가 아니다.
- `height_measurer::row_declared_covers_stored_content`는 모든 셀의 저장 줄이 초기 최소 안에 들어가야만 빈 줄 중복 계상을 제거했다. 오른쪽3194>2994 때문에 왼쪽에서도 억제가 풀려 머리 표가3985HU(53.1px)로 부풀었다. 측정 결과의 실제 row_heights를 typeset과 최종 table_layout이 소비하므로 뒤 형제 표·본문까지 약6.8px 내려간다. 셀 하나의 줄/개체 소유를 다른 셀의 줄 크기로 판정하는 가정을 제거한다.
- 입력 저장 정보는 한컴오피스2020이다. 동일 공개 슬라이스를2020 엔진으로 새 변환한 기준은 정상 세로4쪽이다. 기존2-up PDF의 원시 좌표와 직접 혼합하지 않는다. 새 PDF 머리 괘선98.132..144.3227px(높이46.1907px), 마지막 제목 표196.7454..318.6920px, 첫 본문 가시 글자358.2463px가 독립 근거다. 저장 실제 머리 높이3476HU=46.3467px와 본문 논리 시작(7085+19829)/75=358.8533px가 이 출력과 대응한다.
- 기존 검사의346.8px/표 하단307.0px는 과거 rhwp 출력 상수이며 현재 원본 수용 기준이 아니다. 실제 최종 tree의 마지막 표 하단과 호스트 줄2700HU+아래 여백283HU 간격을 검사한다. 이 간격 검사는 보정 전에도 통과할 수 있는 정상 대조이며 결함 검출로 세지 않는다. 머리 표 높이와 독립 본문 논리 시작 검사는 수정 전후를 실행한다. 시각 허용치·baseline을 늘리지 않는다.


### 보정 결과

- 초기 공유 행 최소를 각 셀의 자기 저장 줄·비인라인 개체 점유와 비교하도록 바꿨다. 오른쪽 셀이 행을 확장해도 왼쪽 로고의 빈 앵커를 새 높이로 더하지 않는다. 실제 확장된 row_heights는 기존 typeset·최종 paint가 소비한다. oversized 개체와 rowspan의 적용 제외, 기존 단일 빈 문단 조건은 유지했다.
- 독립 기준으로 교정한 정식3개는 수정 전1PASS/2FAIL(exit100,0.030s), 최초 수정 후3PASS(exit0,0.027s)다. 표 사이 간격 검사는 수정 전에도 통과한 정상 대조다. 셀 직렬화 순서를 뒤집은 수동 IR과 #7382 그림5의 독립 프레임 대조를 포함해 최종44PASS/0FAIL(exit0,0.692s,threads8)을 확인했다. 그림5 신규 검사는 수정 후에만 실행했으며 수정 전 결함 검출 증거로 세지 않는다.
- [새 한컴2020 기준 PDF](../../../pdf/issue6312/fiscal-trend-float-table-anchor-2020.pdf)와 #6312 전체4쪽을 비교했다. 첫 쪽65.74324→96.76471%, 나머지92.79928/71.83139/99.88721%다. 새 review·standalone overlay8개를 직접 확인했다. 머리 로고/표·제목과 첫 본문 위치가 복원됐으나3쪽의 표와 뒤 본문 차이는 수정 전 불변 CLI에서도 동일하게 남는다. 전체 gate는 exit1/re_review_required이며 다음 개별 보정 대상이다.
- #7382 HWPX11–14쪽도 새로 캡처해 review/overlay8개를 직접 읽었다.66.82800/99.76685/94.95824/81.51575%,exit1이다. 그림5 첫 행은179.6→178.9px로 독립 원본13417HU(178.893333px)에 맞아졌고 그림/캡션도 한컴 좌표에 가까워졌다. 자동 점수 감소를 그 자체로 배치 회귀로 판정하지 않으며 뒤 그림6·본문의 남은 위치 차이를 해결해야 한다.12/13쪽의 앞 보정과215쪽은 유지됐다.14쪽 표/그래프/뒤 본문 위치 차이도 계속 보류다.
- fmt·manifest(6202 static attrs)·source-unit(4205검사/298모듈)은 고정base eb9142dd7에서 exit0이다. 새 불변 CLI는 release-test build exit0,1m35s다. [source·diff·CLI 해시/정확한 명령/판정](../assets/pr7382_20260926/stage28_validation.json), [독립 입력·HU·PDF 좌표](../assets/pr7382_20260926/stage28_independent_geometry.json)를 연결했다. 중간 source+diff의 진단 증적이며 최종 head의 전수/fresh WASM/전체 회귀·lint·Skia로 대체하지 않는다.
- 이번 통합에 추가한 영어 설명 주석은0개다. [주석 재검사](../assets/pr7382_20260926/stage28_english_comment_scan.json). 모든 설명을 한글로 유지하고 로그·output·generated는 커밋하지 않는다. 골든·baseline·시각 허용치·글꼴 예외를 변경하지 않았다.

![#6312 첫쪽 복원](../assets/pr7382_20260926/stage28_native_6312_review_001.png)
![#6312 첫쪽 overlay](../assets/pr7382_20260926/stage28_native_6312_overlay_001.png)
![#6312 3쪽 다음 보정 대상](../assets/pr7382_20260926/stage28_native_6312_review_003.png)
![#7382 11쪽 프레임 대조와 남은 차이](../assets/pr7382_20260926/stage28_native_hwpx_review_011.png)

## 메인터너 보정29 — 빈 줄이 이미 소비한 간격의 지연 기준 재가산

### 사전 분석

- #6312 원본3쪽 첫 표 하단은 rhwp419.6px/PDF419.0614px로 대응하며 다음 빈 문단39도 저장 시작25155HU에 맞게429.8667px에 놓인다. 그 줄은1400HU 높이와772HU 간격을 소비해 다음 본문40의 논리 시작(7085+27327)/75=458.8267px로 전진한다. PDF 가시 글자457.9773px도 이 시작에 대응한다.
- typeset은 확정 page 기준70099HU에서364.36px를 소비해 올바르게 판정한다. layout은 TAC 표 뒤 기준을 초기화한 뒤 빈 문단의 간격을 lazy 역산에 다시 더해 base69327HU를 선택하고469.12px로 밀어낸다. 다음 표·뒤 본문도 같은772HU=10.2933px만큼 내려간다. 빈 텍스트라는 상태로 실제 간격 소비 여부를 대신한 가정이 원인이다.
- `layout::last_item_content_bottom → HeightCursor::prev_item_content_bottom_y → vpos_adjust의 lazy 역산 → 실제 다음 문단/표 원점`을 확인한다. 직전 실제 점유 끝과 순차 cursor 사이가 유효 저장 trailing 간격과 같고 다음 저장 줄이 그 끝을 이어받으면 이 간격은 이미 소비됐다. 미소비/비연속/합성/편집 경로에는 이 증거를 적용하지 않는다. 기존 그림 전용 계약과 누락 간격 bridge는 대조한다. 저장 좌표 상수·paint clamp·허용치 완화를 추가하지 않는다.
- 정식 원본 검사로40·46 문단 최종 원점과 빈 줄의 실제 점유 끝·전진량을 확인해 수정 전후를 연결한다. 원본4쪽과 정상 빈 줄/그림 대조군·전체 Native4쪽을 먼저 검증한다.

최초 후보는 가시 콘텐츠 하단을 소비해 빈 줄의 실제 줄 상자 끝을 얻지 못했다(6개중5PASS/1FAIL). 기존 배치의 `last_line_box_bottom` 결과를 별도로 전달한 후보는 원본6개를 통과했지만 확대105개중1개가 실패했다.175쪽의 뒤 제목은 독립PDF641.061px/보정 전641.1px에서627.8px로 올라갔다. 분할 표로 시작한 단의 빈 줄은 저장 원점이 아직 확정되지 않았으며 실제 간격 소비만으로 그 원점을 입증할 수 없었다. 처음 확정한 저장 단 원점을 유지해 원점·실제 줄 배치를 함께 대조하도록 가정을 교정했다. 원점이 없는 단의 기존 누락 간격 bridge와 독립 제목 좌표는 유지하며 최종 재검증한다.


### 보정 결과

- 실제 줄 배치에서 이미 계산한 `last_line_box_bottom`을 `last_item_flow_line_bottom → HeightCursor::prev_item_flow_line_bottom_y`로 전달했다. 셀 내부 줄은 본문 값을 덮어쓰지 않으며 항목마다 초기화한다. 가시 글자 하단은 기존 값으로 유지해 빈 줄의 공간 점유와 혼동하지 않는다. 빈 composed 문단의 실제 기본 줄 결과도 같은 소비자에 전달한다.
- TAC 이후 활성 기준을 초기화해도 처음 확정한 저장 단 원점을 독립 대조용으로 보존한다. 유효 저장 연속성, 실제 줄 상자 끝과 순차 전진, 확정 원점의 다음 줄 위치가 함께 일치할 때만 trailing 간격 재가산을 제거한다. 원점 없는 분할 표 시작·다른 원점·미소비·합성·편집 상태는 기존 bridge를 유지한다. typeset은 이미 확정된 원점에서 올바르게 판정하므로 실제 배치의 추측을 측정에 복제하지 않았다.
- 신규 실물 배치 회귀는 수정 전0PASS/1FAIL(exit100,0.030s)이었고 최종 원본6개와 확대105개가 모두 PASS다. 최종 집중105PASS/0FAIL(exit0,4.000s,threads8), 커서와 초기화 대조56PASS(exit0,0.072s)를 확인했다. 소스 단위 검사 총량을 늘리는 후보는 정책에 걸려 기준 상향 없이 기존 lazy 기준 검사에8개 경계를 보완했다. 이는 수동 커서 계약이며 실물 한컴 출력 증거와 구분한다.
- 최초 단위 빌드에서는 보정24의 `empty_opening_row_frame`이 기존 issue2424 fixture 초기화에 빠진 오류(exit101)를 발견했다. 이 정상 대조에서는 새 시작 프레임을 사용하지 않으므로 None을 명시하고 재검증했다. 빌드 실패를 결함 검출 증거로 세지 않는다. [중간 확대 실패와 가정 교정](../assets/pr7382_20260926/stage29_expanded_initial_result.json)도 보존했다.
- #6312의 전체4쪽은96.76471/92.79928/99.68480/99.88721%,exit0,gate passed다. 새 review·standalone overlay8개에서 머리 표/제목,3쪽 두 표·채무/국고채 본문·마지막 줄,4쪽 표·본문 소유를 직접 확인했다. 3쪽의71.83139% 위치 차이를 해결했다.2쪽의 기존 작은 표/글자 간격 차이는 전후 점수가 같으며 완전 픽셀 일치로 보고하지 않는다.
- #6950 전체3쪽도98.66178/99.13460/99.78442%,exit0으로 유지됐다. 새6개 PNG에서 제목·표 여백·마지막 본문·로고와 가운데 정렬 도표의 전체 끝을 확인했다. #7382의11/12/13/14/174/175쪽은66.82800/99.76685/94.95824/81.51575/99.85942/92.46177%,exit1이다. 새12개 PNG에서 기존12/13쪽 그림·표/캡션·각주와174/175쪽 표 앞뒤 내용·외곽·뒤 제목/각주 비충돌을 확인했다.11/14쪽의 위치 차이는 계속 보류다. 세 입력 모두 기준 PDF와 같은4/3/215쪽을 유지했다.
- fmt·manifest(6203 static attrs)·source-unit(4205검사/298모듈)은 고정base eb9142dd7에서 exit0이다. 새 불변 CLI는 release-test 빌드exit0,1m36s다. [source·파일/diff/CLI 해시·정확한 명령·최종 판정](../assets/pr7382_20260926/stage29_validation.json), [독립 HU/PDF와175쪽 대조](../assets/pr7382_20260926/stage29_independent_geometry.json)를 연결했다. 최종 전수·전체 Rust/lint/Skia·fresh WASM은 아직 미검증이다.
- 추가 영어 설명 주석은0개다. [주석 재검사](../assets/pr7382_20260926/stage29_english_comment_scan.json). baseline·golden·시각 허용치·글꼴 예외를 바꾸지 않았고 로그·output·generated는 커밋하지 않는다. 다음 개별 보정은 #7382의11쪽 그림6/뒤 본문 위치를 독립 원본과 대조한다.

![#6312 3쪽 표와 뒤 본문 복원](../assets/pr7382_20260926/stage29_native_6312_review_003.png)
![#6312 3쪽 overlay](../assets/pr7382_20260926/stage29_native_6312_overlay_003.png)
![분할 표 시작175쪽 정상 제목 보존](../assets/pr7382_20260926/stage29_native_hwpx_review_175.png)
![175쪽 overlay](../assets/pr7382_20260926/stage29_native_hwpx_overlay_175.png)

## 보정30 사전 분석 — 빈 개체 앵커의 오프셋·바깥 상자 소유

- 기준은 원본 HWPX 11쪽과 한컴2024 PDF다. 그림6 앞 본문239는313.5px/PDF313.381px로 맞지만 그림 상단373.0px/PDF376.229329px, 뒤 제목692.17px/PDF703.781006px가 어긋난다. 글꼴로 분류하지 않는다.
- 원본240의 호스트21265HU, 오프셋319HU, 위283HU, 전체 표22400HU, 아래283HU는 다음 빈 문단241의44550HU를 정확히 닫는다:21265+319+283+22400+283=44550. 그림/캡션 셀의 실측 전체 높이도21118+1282=22400HU다. 본문 원점6239HU를 더한 뒤 제목242의 논리 상단은703.853333px다.
- `empty_float::prepare`는 세로 오프셋이 있는 단일 T&B 표를 거절해 block whole-fit으로 보낸다. 기존 내부 줄 프레임 helper는 가시 다행 호스트/RowBreak만 수용하므로 이 빈 CellBreak 앵커는 공통 결과가 없다. paint의 빈 lane은 위여백을 누락하고 흐름 끝은 실제 오프셋을 제외한다. 그 뒤 lazy base874HU가 뒤 본문을 끌어올린다.
- 적용 조건은 원본 단일 단·미편집·표 재조판 아님, 유효 폭0 단일 빈 호스트와 다음 저장 줄, 선언 높이와 실측 전체 내용의 일치 및 위 등식이다. 합성/편집/잘못된 간격·높이, 여러 호스트 개체, 분할 저장 높이는 수용하지 않는다. `저장 닫힌 상자 → whole-fit 점유 하단/기록 → layout 원점/점유 하단`으로 한 결과를 소비하게 보정한다. 실제 정식 tree와 정상 대조군을 먼저 확인하고 새 시각 증적 뒤 결과를 보고·커밋한다.

### 보정 결과

- `stored_empty_control_table_frame`이 유효 빈 호스트와 다음 저장 줄 사이에서 오프셋·위/아래 여백·실측 전체 높이가 정확히 닫히는 원본 프레임을 만든다. whole-fit이 이를 예약·fit·배치 기록에 사용하고 기존 layout은 같은 표 상단과 점유 하단을 소비한다. 그림6 표 논리 상단374.746667px와 뒤 제목703.853333px를 복원했다. 합성/편집·높이/간격 불일치 및 분할 높이를 같은 근거로 수용하지 않는다.
- 원본 HWP/HWPX 정식2개는 수정 전0PASS/2FAIL(exit100,0.197s),수정 후2PASS(exit0,0.214s)다. 합성 호스트/후속 줄, 닫힘 간격 불일치, 선언/실측 높이 불일치의 수동 IR 대조까지 최종 집중108PASS/0FAIL(exit0,4.199s,threads8)이다. 이전 suite 번호로 정상3개를 놓친105PASS 시도는 최종 검증으로 세지 않고 현재 manifest의 실제 번호에서 재실행했다.
- Native HWPX의11쪽66.82800→94.81767%,14쪽81.51575→90.37937%,210쪽72.49487→99.91680%다. HWP의 같은 쪽은94.81767/91.00235/99.91680%다. 그림6·캡션과 뒤 제목/본문,14쪽 첫 표/그림10 프레임,210쪽 표 외곽/셀 글자를 직접 확인했다.2쪽 표 상단도142.0→143.9px로 PDF143.84269px에 맞아졌다. HWPX2쪽 점수95.34141→94.69572% 감소만으로 배치 회귀라고 판정하지 않는다.
- 양쪽 입력의2/11/12/13/14/23/68/210쪽 새 review·standalone overlay32개를 직접 읽었다.12/13쪽 앞 보정·68쪽 표/각주와215쪽 수는 유지됐다.23쪽은74.73376→75.76624%로 여전히 gate 미충족이다. 점수가 통과한11쪽 그림5 오른쪽 지도와14쪽 아래 그림11도 위로 치우친 차이가 남는다. 글꼴 예외나 점수 통과로 이 의미 차이를 해소하지 않으며 PR 보류를 유지한다.
- fmt·manifest(6206 static attrs)·source-unit(4205검사/298모듈)은 고정base eb9142dd7에서 exit0이다. 새 불변 CLI는 release-test 빌드exit0,1m28s다. [source/diff/CLI 해시·실제 명령·판정](../assets/pr7382_20260926/stage30_validation.json), [독립 HU/PDF](../assets/pr7382_20260926/stage30_independent_geometry.json), [영어 추가 설명 주석0개](../assets/pr7382_20260926/stage30_english_comment_scan.json)를 연결했다. 전체 최종 Rust/lint/Skia/fresh WASM과 최신 전수는 미검증이다. 로그·output·generated는 커밋하지 않는다.

![11쪽 그림6과 뒤 본문 복원](../assets/pr7382_20260926/stage30_native_hwpx_review_011.png)
![11쪽 overlay](../assets/pr7382_20260926/stage30_native_hwpx_overlay_011.png)
![210쪽 표 프레임 복원](../assets/pr7382_20260926/stage30_native_hwpx_review_210.png)
![23쪽 남은 그림/캡션 차이](../assets/pr7382_20260926/stage30_native_hwpx_review_023.png)

## 보정31 사전 분석 — 가운데 정렬 셀 그림의 흐름 프레임과 음수 오프셋

- 원본11쪽 그림5 오른쪽 상단은 PDF95.096029px/현재87.1px,23쪽 그림22는 PDF166.378662px/현재155.3px다. 두 그림은 단일 빈 문단·자리차지·문단 기준·흐름 제한 켜짐·가운데 셀 정렬이며 음수 오프셋−2179/−1696HU를 가진다. 왼쪽 그림은 각각0/+187HU이며 PDF88.704/151.994670px에 대응한다.
- 측정과 cell flow는 현재 `max(offset,0)+개체·여백`으로 점유 높이를 계산한다. `table_layout`의 셀 정렬은 이후 signed offset을 다시 더하며11쪽은 마지막 cell clamp에 걸려 셀 상단으로 올라간다. 공통 점유 프레임과 정렬에 쓰인 오프셋이 다른지 확인한다. clamp로 위치를 맞추지 않는다.
- 두 오른쪽 그림의 `pos.vertOffset`만0 및+1000HU로 바꾼 독립 한컴2024 대조군을 준비했다. 나머지 ZIP 내용·section XML 문자열은 그대로다. 저장 음수/정렬 계약은 변환 결과에서 확인한 뒤 구현하며, 먼저 원본 HWP/HWPX 정식 최종 그림 좌표 검사의 수정 전 실패를 기록한다. 양수 오프셋·다문단 저장 흐름·나란히 무리·제한 해제 경로는 적용 경계로 대조한다.

0 대조군은 원본과215쪽 전체 텍스트/그림 좌표가 동일했다. +1000HU 대조군의23쪽 오른쪽 그림은166.378662→173.090678px로 이동했고 캡션도 함께 내려갔다.11쪽은 같은 수동 속성 변화에서 PDF 좌표가 변하지 않았다. 이 문서는 HWPX 재저장/LineSeg 재생성 대조군이 아니므로 그11쪽 수용 계약은 미검증으로 남기고23쪽 양수 결과를 일반화하지 않는다. 원본 정식2개는 수정 전0PASS/2FAIL(exit100,0.218s), 최초 수정 후2PASS(exit0,0.261s)다.

확대125개는123PASS/2FAIL(exit100,4.631s)이었다. 새 양수/음수 정렬 대조의 캡션 단일 소유 assertion은 기존 캡션 중복을 함께 검출했다. 정렬과 캡션 소유를 별도 정식 검사로 분리해 두 의미를 모두 보존한다. #6782 일본 마크는 보정 전 불변30 CLI에서도320.2/320.5px로 동일하게 어긋나며, 이번 Center 변경의 회귀로 분류하지 않는다. 이 두 미충족을 허용치나 기대값 변경으로 숨기지 않고 다음 개별 보정에서 해결한다.

### 보정 결과

- `topbottom_flow_vertical_offset_hu`로 측정과 셀 흐름의 기존 앞 공간 계산을 공유했다. 온전한 셀과 이어받는 셀의 가운데 정렬은 같은 앞 공간을 소비하며 음수 저장값을 별도 이동으로 다시 더하지 않는다. 다문단 저장 vpos·나란히 무리·제한 해제, Top/Bottom의 다른 앵커 계약은 실제 기존 경로로 남는다. 부분 셀의 화면 이탈 판정을 가운데 정렬의 근거로 사용한 가정을 제거했다. 측정의 수치·컷 선택은 기존 max(offset,0)와 동일하며 paint 원점 소비를 교정한 변경이다.
- 최종 확대126개는124PASS/2FAIL(exit100,4.696s,threads8)이다. 원본 HWP/HWPX11·23쪽 그림 위치와23쪽−1696/0/+1000HU 대조는 통과했다. 다문단/자기 변위·작은 Top 음수 오프셋·실제 부분 셀 정상 대조도 통과했다. 캡션 소유 검사는 독립 검사로 그대로 남아 FAIL이며 일본 마크 기존 차이도 FAIL이다. [전후 동일한 마크 좌표](../assets/pr7382_20260926/stage31_prior_6782_difference.json)를 보존했다.
- 추가 이어받기 대조1개는PASS(exit0,0.448s)다. 원본103쪽 문서의 구역4/문단118은77쪽에서3..14행을 이어받는다. 그 실제 부분 셀의 그림 오프셋−187/0/+187HU를 수동 IR로 바꿔 물리 셀의 중앙 정렬 불변식·그림12개 보존·103쪽을 확인했다. 수정 후에만 실행한 경계 대조이며 수정 전 결함 검출이나 정상 한컴 재저장 문서의 증거로 세지 않는다.
- 새 Native HWPX11/12/13/14/23/68/210쪽은98.39093/99.76685/94.51340/90.37937/99.61877/90.85432/99.91680%,HWP는98.39093/99.76685/96.29076/91.00235/99.61877/90.85432/99.91680%다. 둘 다exit0,선택 gate passed이며215쪽을 유지했다. 새 review·standalone overlay28개를 직접 읽어 지도와 오른쪽 그래프 정렬, 그림/표·뒤 본문·각주 보존을 확인했다.13쪽 왼쪽 표 그림은626.1→628.0px로 PDF627.313px에 가까워졌고 점수 감소만으로 회귀라고 판정하지 않았다.
- 점수가 통과해도23쪽 캡션 중복과14쪽 그림11의 위치 차이는 미충족이다. 새 통합 PR을 만들거나 승인하지 않는다. 다음 개별 보정은 캡션 단일 소유를 해결한다. [독립 입력 변형·한컴 PDF 좌표](../assets/pr7382_20260926/stage31_independent_geometry.json), [재현 스크립트](../assets/pr7382_20260926/stage31_control_recipe.py), [0 대조 PDF](../../../pdf/issue7379/liver7379-cell-picture-offset-zero-2024.pdf), [+1000HU 대조 PDF](../../../pdf/issue7379/liver7379-cell-picture-offset-positive-2024.pdf)를 보존했다.
- 최종 fmt·manifest(6211 static attrs)·source-unit(4205검사/298모듈)은 고정base eb9142dd7에서exit0이다. 불변 CLI는release-test 빌드exit0,1m21s다. [명령·source/runtime/test diff·CLI 해시·남은 판정](../assets/pr7382_20260926/stage31_validation.json). 캡처 뒤 추가한 것은 이어받기 반례 검사뿐이며 runtime4파일/diff 해시는 동일하다. 이 중간 증거를 최종 exact head 전수/전체 Rust/lint/Skia/fresh WASM의 대용으로 사용하지 않는다. [추가 영어 설명 주석0개](../assets/pr7382_20260926/stage31_english_comment_scan.json)이며 로그·output·generated를 커밋하지 않는다.

![11쪽 지도 정렬 복원](../assets/pr7382_20260926/stage31_native_hwpx_review_011.png)
![11쪽 overlay](../assets/pr7382_20260926/stage31_native_hwpx_overlay_011.png)
![23쪽 그래프 정렬과 남은 캡션 중복](../assets/pr7382_20260926/stage31_native_hwpx_review_023.png)
![23쪽 overlay](../assets/pr7382_20260926/stage31_native_hwpx_overlay_023.png)

## 보정32 사전 분석 — 셀 그림 캡션의 중복 출력

- 원본23쪽 표339의 두 그림 캡션은 각각 원문5줄인데 최종 셀에 같은5줄이 두 번 출력된다. 보정31에 보존한 정식 `cell_picture_bottom_caption_has_one_owner`는 왼쪽 캡션2개를 검출해 실패했다. 독립 한컴2024 PDF에는 각 캡션이 한 번만 있으며 위치는 유지된 상태에서 글자가 겹쳐 굵게 보인다.
- `table_layout::layout_picture` 호출 → `picture_footnote::layout_picture`의 공통 방향별 캡션 좌표/소유/출력 → 호출자 아래의 별도 Bottom `layout_caption` 재호출을 추적했다. 부분 셀 경로는 공통 호출만 사용한다. 별도 Bottom 블록은 기존 메인터너 커밋9bed077a1a(2026-08-02), 소유 표식은4198c6df47(2026-09-11)에서 온 코드이며 원 기여자의 #7382 변경으로 분류하지 않는다.
- 그림 공통 경로가 예약한 캡션 띠와 실제 셀 문맥·원본 개체 소유를 한 번 출력하게 하고 호출자의 중복 블록을 제거한다. 최종 글자를 숨기거나 tree를 사후 중복 제거하지 않는다. 온전한 원본 HWP/HWPX의5줄·좌표·소유·뒤 본문 보존과 본문/상단/옆 캡션 정상 대조를 실행하고 직접 Visual Sweep으로 확인한다. 기준값·허용치·페이지 수를 바꾸지 않는다.

### 보정 결과

- 온전한 셀의 별도 Bottom 캡션 호출을 제거해 그림 공통 경로가 좌표·셀 문맥·개체 소유와 출력까지 한 번 처리한다. 측정·예약·분할 컷은 변경하지 않았다. 부분 셀과 글자처럼 그림은 이미 같은 공통 호출만 사용하며 본문/상단/옆 캡션 정상 대조를 함께 실행했다.
- 기존 정식 HWPX 검사는 수정 전0PASS/1FAIL(exit100,0.238s)로 캡션2개를 검출했다. 수정 후 두 형식의 정식2개는PASS(exit0,0.213s)다. 각 캡션의 원문5줄·전체 내용/순서·원본 개체 소유·독립 PDF 상단/원본 줄 간격·뒤 본문·215쪽을 확인했다. 신규 HWP 검사는 수정 후만 실행했으며 변경 전 HWP의 중복은 불변31 CLI의 실제 tree로 확인했다. 그 진단을 신규 HWP 검사의 수정 전 실행으로 보고하지 않는다.
- 확대142개중141PASS/1FAIL(exit100,4.959s,threads8)이다. 본문/상단/좌우 캡션·각주/그림·중첩 표·이전 저장 프레임 정상 대조는 통과했다. 남은 실패는 기존 #6782 일본 마크320.2/320.5px와 독립318.2/319.0px 차이이며 다음 개별 보정으로 다룬다. 기대값·허용치 상향으로 숨기지 않았다.
- 새 Native HWPX11/13/23/68쪽은98.39093/94.51340/99.62374/90.85432%, HWP는98.39093/96.29076/99.62374/90.85432%다. 두 입력 모두215/PDF215쪽, exit0,선택 gate passed다. 새 review·standalone overlay16개를 직접 읽어23쪽 두 그림과 캡션5줄/뒤 본문,11쪽 그림5/6·뒤 본문/각주,13쪽 표2/그림8/9·각주,68쪽 그림49·본문/각주를 확인했다. 변경 전20개 캡션 줄은10개로 줄었고 래스터 차이는23쪽 캡션 영역에 있다. 작은 실루엣 점수 개선을 완전 픽셀 일치나 다른 보류 사유의 해소로 보고하지 않는다.
- fmt·manifest(6212 static attrs)·source-unit(4205검사/298모듈)은 고정base eb9142dd7에서exit0이다. 불변 CLI는release-test 빌드exit0,1m27s다. [source/diff/CLI 해시·정확한 명령·검사/시각 판정](../assets/pr7382_20260926/stage32_validation.json), [원본/PDF·변경 전후 캡션 줄](../assets/pr7382_20260926/stage32_independent_geometry.json), [추가 영어 설명 주석0개](../assets/pr7382_20260926/stage32_english_comment_scan.json)를 보존했다. 모든 로그는output/logs에 남기며 커밋하지 않는다.
- 일본 마크 실패·14쪽 그림11의 기존 위치 차이·실제 TABLE 편집/보존 래퍼 분할 경계·최신 전수/전체 Rust/lint/Skia/fresh WASM은 남아 있다. 이번 중간 보정은 통합 PR 제출/승인의 증거를 대신하지 않는다.

![23쪽 캡션 단일 출력](../assets/pr7382_20260926/stage32_native_hwpx_review_023.png)
![23쪽 overlay](../assets/pr7382_20260926/stage32_native_hwpx_overlay_023.png)

## 보정33 사전 분석 — 빈 마지막 글줄과 부동 그림 묶음의 원점

- #6782 물리77쪽/인쇄55쪽 일본 셀의 두 그림은 독립 PDF318.051982/319.010701px에 대해 현재320.2/320.5px다. 행 괘선305.745/385.817px는 현재306.0/386.1px와 대응하며 전체 표 이동으로 분류하지 않는다. 원본103쪽과 정식 위치 검사의 실패를 그대로 보존한다.
- 원본 HWP의 압축된 `BodyText/Section4` 개체 공통 레코드를 직접 읽었다. 두 그림은 문단/Top 기준·흐름 제한 켜짐·비TAC이며 세로 오프셋780/862HU, 높이3806/3866HU, 바깥 여백0이다. 두 번째 그림 끝862+3866=4728HU는 빈 마지막 저장 줄의 vpos4728과 같다. 저장 셀1282HU는 그 줄1000HU와 양쪽 안 여백141HU씩을 담고, vpos4728+셀1282=형제 셀의 실제 선언 행6010HU를 닫는다.
- 기존 보정은 그림이 셀 밖으로 나갈 때마다 `실제 셀 하단−아래 여백−빈 줄 높이−각 그림 높이+상대 오프셋`으로 되돌린다. 이는 같은 문단의 두 그림에 각기 다른 원점을 만들어 그림 높이 차이60HU가 상대 위치에 다시 섞인다. 빈 줄의 실제 배치도321.2px로 묶음 시작에 남고, 줄의 높이/오프셋/그림 위치를 함께 소유한 저장 프레임을 소비하지 않는다.
- 원본 공통 레코드의 두 세로 오프셋만 바꾼 한컴 대조군으로 원점 계약을 추가 확인한다. 그 뒤 `저장 닫힌 셀 프레임 → 측정/실제 정렬 높이 → 빈 줄과 그림의 공통 원점`의 호출 경로를 연결해 기존 사후 이탈 보정의 가정을 제거한다. 수동 속성 변형을 정상 재저장 문서로 보고하지 않으며, 편집/합성/닫힘 불일치·실제 컷이 다른 경로는 적용 경계로 구분한다.


0 오프셋 대조군은 높이가 다른 두 그림의 상단이319.012/319.011px로 같았다. 따라서 그림마다 높이를 빼서 서로 다른 원점을 만드는 기존 가정을 제거한다. 저장 마지막 줄 앞의 공통 그림 띠 원점은 `마지막 줄 위치−묶음의 최대 개체 끝`이다. +750HU 대조군은 개체 끝5478HU가 저장 줄4728HU를 넘어 이 원본 닫힘 조건을 깨므로 정상 저장/편집 재조판의 수용 증거로 쓰지 않는다.

기존 PDF의 변환 엔진 차이를 구분하기 위해 원본도 같은engine2020으로 다시 출력했다. 원본/0/+750 대조군은 모두103쪽이다. 같은 엔진 원본과 비교하면 두 변형의 전체 텍스트 좌표는 같고 그림 좌표는77쪽에서만 바뀐다. 기존2022 PDF와의6–21/23쪽 텍스트·65쪽 그림 차이는 속성 변경의 영향으로 보고하지 않는다. [독립 PDF/CFB 바이트 동일성/전수 좌표](../assets/pr7382_20260926/stage33_independent_geometry.json), [레코드 변경 재현 절차](../assets/pr7382_20260926/stage33_control_recipe.py)를 보존했다.


### 보정 결과

- `stored_empty_picture_cell_frame`이 원본의 마지막 빈 줄·그림 띠·행 닫힘을 함께 결정한다. 측정의5728HU와 `cell_units`의 같은 높이를 사용하며 두 그림과 마지막 줄을 하나의 원본 유닛/개체 범위로 소유한다. 실제 정렬과 빈 줄·그림 배치도 같은 프레임을 소비한다. 개별 그림 높이로 셀 하단에 되돌리던 기존 가정을 제거했다. 온전한 셀의 기존 일반 clamp는 이 프레임에서 위치를 바꾸지 않음을 최종 좌표 검사로 확인했다.
- 기존 정식 위치 검사는 수정 전0PASS/1FAIL(exit100,0.130s), 첫 수정 후 기존7개는PASS(exit0,0.540s)다. 최종 확대145개는145PASS/0FAIL(exit0,4.890s,threads8)이다. 추가3개는 수정 후 경계 검사로 원본/0 오프셋의 빈 줄·공통 원점, 온전한 셀, 실제 부분 셀/이월에서 그림2장과 마지막 줄의 단일 소유·뒤 문단·본문 하단을 검사했다. 신규 검사의 수정 전 실행으로 보고하지 않는다. 원본/축소103쪽과77쪽 그림12개를 유지했다.
- 같은 원본의77쪽 Native 시각은99.16488→99.16303%, 0 대조군은99.15771%다. 점수는 근소히 내려갔으며2px 관용 점수만으로 이 위치 결함을 검출하지 못한다. 독립 최종 좌표·빈 줄/공통 원점 검사와 직접 판독으로 두 그림의 위치·상대 오프셋과 행 괘선, 나머지 마크/본문/쪽번호를 직접 확인했다. 실제 그림이 제거된 축소 fixture의 그림 없는 래스터를 마크 일치 증거로 사용하지 않고 전체 원본으로 다시 실행했다. 전체103쪽의 변경 전후 render-tree에서 달라진 것은77쪽뿐이며, 주 검토 HWP215쪽의 tree는 모두 동일했다. 이 수치 비교를 직접 판독의 대용으로 쓰지 않는다.
- 원본76/77/78쪽과0 대조77쪽, 주 검토 HWP11/14/23쪽의 새 review·standalone overlay14개를 직접 읽었다. 주 검토11/14/23쪽은98.39093/91.00235/99.62374%로 앞 보정의 지도/그림6·캡션·뒤 본문을 보존했다. 그러나 원본76쪽60.83855%는 표/캡션이 본문과 겹치고,78쪽49.22912%는 표 위치와 아래 그림 누락이 남는다. 두 쪽은 이전 CLI의 tree와 같지만 기존 차이라는 이유로 보류를 해소하지 않는다. 이 원본 선택 gate는 `re_review_required`이며 새 PR 생성/승인은 계속 보류한다.
- 자체 구현의 bool 타입·대조군 필드 빌드 오류, 이전 suite의0-test 실행, 잘못 구성한 본문 예산과 문단 문자 메타데이터는 같은 범위에서 수정했다. 실패 로그/원인을 [전후 명령·source/test/CLI 해시·최종 판정](../assets/pr7382_20260926/stage33_validation.json)에 구분해 보존하며 성공 증거로 세지 않는다. fmt·manifest(6215 static attrs)·source-unit(4205검사/298모듈)은 고정base eb9142dd7에서exit0이다. 불변 CLI는release-test 빌드exit0,79.103s다.
- [전체 원본 전후 tree 범위](../assets/pr7382_20260926/stage33_original_tree_difference.json), [영어 추가 설명 주석0개](../assets/pr7382_20260926/stage33_english_comment_scan.json), [같은 엔진 원본 PDF](../../../pdf/issue6782/japan-cell-original-2020.pdf), [0 대조 PDF](../../../pdf/issue6782/japan-cell-zero-2020.pdf), [+750HU 무효 저장 경계 대조 PDF](../../../pdf/issue6782/japan-cell-forward-750-2020.pdf)를 연결했다. 마지막 대조군을 정상 재저장/편집 출력 통과로 보고하지 않는다. 로그·output·generated는 커밋하지 않는다. 다음 개별 보정은14쪽 그림11의 위치 차이다.

![77쪽 일본 마크 원점 복원](../assets/pr7382_20260926/stage33_native_original_review_077.png)
![77쪽 overlay](../assets/pr7382_20260926/stage33_native_original_overlay_077.png)
![0 오프셋 독립 대조](../assets/pr7382_20260926/stage33_native_zero_review_077.png)
![78쪽 남은 표/그림 보류](../assets/pr7382_20260926/stage33_native_original_review_078.png)


## 보정34 사전 분석 — 그림11의 저장 바깥 여백과 최종 원점

- 원본14쪽 문단273의 빈 호스트는vpos45803HU/폭0이며 그림11은2×1 RowBreak 표 안의 글자처럼 그림이다. 문단 기준 오프셋948HU·위/아래 바깥 여백283HU·전체 표17819HU는 다음 저장 빈 줄65136HU를 정확히 닫는다:45803+948+283+17819+283=65136. 독립 원본/PDF와 이 저장 관계로 원점을 정하며 점수91% 통과를 위치 차이의 해소로 쓰지 않는다.
- 현재 최종 표 상단706.5px는 바깥 위 여백을 제외한 원점과 대응한다. 독립 저장 본문 원점6239HU를 포함한 표 상단은710.306667px, 원본 첫 행16537HU 뒤 캡션 줄은932.68px다. HWPX PDF의 그림 가시 상단711.382650px/캡션932.581055px와 대조한다. HWP PDF의 원시 이미지 bbox는 crop clip 전의 전체 이미지이므로 HWPX 가시 bbox와 혼동하지 않는다.
- 보정30의 닫힌 빈 호스트 프레임을 whole-fit에서 조회하지만 이 RowBreak 표의 실제 호출 경로에서 수용 조건·측정 높이·분할 선택·최종 원점 덮어쓰기를 추적해야 한다. 먼저 원본 두 형식의 최종 표/그림/캡션 정식 실패를 기록한 뒤 같은 원본 프레임을 실제 예약·배치에 연결한다. 분할 높이/편집 캐시·합성 줄·닫힘 불일치 경로를 같은 근거로 수용하지 않는다.


원본 정식2개는 수정 전0PASS/2FAIL(exit100,0.204s)로 표 상단706.506667px/독립710.306667px 차이를 검출했다. 진단에서 실측/유효 전체 높이는 모두237.6px로 선언17819HU와 같았다. 그런데 일반 각주 안전 여유40px를 뺀857.0px에 저장 바깥 끝868.48px가 들어가지 않아 유효 프레임 자체를 버리고 위여백 없는 폴백에 들어갔다. 실제 각주 경계는약897px로 원본 프레임이 들어간다. `원본 프레임 검증 → 실제 각주 경계로 fit → 같은 프레임 기록/배치`, 예산 실패 시에는 원점을 보존한 분할 준비/유닛 컷으로 이어지도록 수정한다.


### 보정 결과

- 그림11의 원본 호스트/다음 저장 줄 닫힘을 예산과 분리해 조회한다. 실제 각주 경계 안에 들어가는 전체 프레임은 안전 여유40px 때문에 원점을 잃지 않으며, 통째 수용과 실제 배치가 같은 하단을 소비한다.
- 예산 실패 시 `SplitTableEntry`의 프레임과 물리 쪽/단/영역 키를 분할 준비에 전달한다. 폭0 빈 앵커를 별도 글줄로 전진시키지 않는다. `FragmentBudget`은 소유 프레임에서만 원래 원점을 쓰고 이월/이어받기에서는 이미 소비한 앵커 거리를 반복하지 않는다.
- 좁은 본문19500HU에서 추가 검사는 처음에7.3px 표 넘침을 검출했다. 마지막 한 줄 주석 행이 강제 전진한 컷을 완전 소비라는 이유로 수용해6px밴드에17.1px행을 담는 경로였다. 실제 표시 높이를 같은 패딩 포함 컷으로 확인한다. 물리 끝행 높이를 바꾼 경우에는 `PartialTable`로 확정하여 통째 표의 원래 높이로 다시 덮어쓰지 않는다.
- 원본2개는 수정 전0PASS/2FAIL(exit100,0.204s), 최종 원본/수동 IR 경계3개는3PASS(exit0,0.959s)다. 본문24000/19500/18000HU의 두 형식에서 그림·캡션·뒤 문단 단일 소유, 원래 첫 표 원점, 행0/행1 실제 다른 쪽 소유, 첫 행 이월, 본문 점유 하단과 불필요한 빈 쪽을 확인했다. 추가 경계 검사를 원본 한컴 재저장 증거 또는 수정 전e7985dc9a 실행으로 보고하지 않는다.
- 자체 타입/누락 import 빌드 오류와 검사 코드의 부분 셀 표식/형식별 이미지ID 가정은 성공이나 결함 재현으로 세지 않는다. 온전한 행의 `page_fragment=false`와 형식별 BinData 번호를 구분해 실제 행/그림 소유를 검사했다.


실제 소비 경로는 `whole_fit::query_original_control_table_frame → entry의 occupied_bottom fit/기록 → place_table_with_text의 확정 원점`, `SplitTableEntry의 프레임/소유 키 → prepare의 첫 조각 예산 → fragment/budget의 같은 원점과 하단 → fragment/emit의 기록 → table_partial`, `row_entry의 실제 컷 표시 높이 → row_step의 수용 → emit의 end_row_height_override → 조각 행 높이`다. helper 반환 뒤 통째 표로 복원되던 덮어쓰기도 제거했다. 이월 후에는 이전 물리 프레임의 앵커 거리를 다시 적용하지 않음을 좁은 실제 쪽 소유 검사로 확인한다.

- 최종 확대 검사는25모듈/18개 실제suite에서155PASS/0FAIL(exit0,5.742s,threads8)이다. 추가154쪽 정식 검사는 한컴PDF 마지막 두 줄의942.181071/968.741048px 위치·원본 표1682 소유·다음 쪽 중복 없음을 확인했다. 이 신규 검사는 수정 후 경계 보강이며 수정 전 정식 실행이라고 보고하지 않는다. fmt·manifest(6219 static attrs)·source-unit(4205검사/298모듈)은 고정base eb9142dd7에서exit0이다. 불변 CLI는release-test 빌드exit0,82.817s다. [정확한 명령·파일/CLI 해시·실패 시도와 판정](../assets/pr7382_20260926/stage34_validation.json)을 연결한다.
- 새 원본 HWPX/HWP11/14/23/68/210쪽의review·standalone overlay20개를 직접 읽었다.14쪽은90.37937→97.52614%/91.00235→97.54155%로 그림11 가시 원점·캡션이 복원됐고 각주11/12와 뒤 쪽을 보존했다.11/23/210쪽은98.39093/99.62374/99.91680%로 앞 보정을 유지했다.68쪽은두형식모두90.85432→95.61071%이며 끝행 높이 조각 소비로 그림49 캡션 위치가 복원됐다. 남은 글자 메트릭·일부 글머리표 차이를 전수 일치로 보고하지 않는다.
- 전체215쪽 tree의 변경은 HWP14/68쪽, HWPX14/68/154/155쪽뿐이다. [전후 전체 tree 범위](../assets/pr7382_20260926/stage34_original_tree_difference.json). 추가154/155쪽은 수정 전후review·standalone overlay8개를 직접 확인했다.154쪽의 마지막 두 줄이 원래 쪽에 들어가94.61010→99.83033%이며155쪽 뒤 제목/본문 원점도 복원돼53.07895→75.49194%다. 하지만155쪽 그림64의표/본문 겹침과 기준PDF의그림 출력 차이가 남는다. 이 선택 gate는 여전히 `re_review_required`이며 PR 생성/승인 보류를 해소하지 않는다. 앞의14쪽 변경 전PNG2개까지 새PNG30개를 보존했다. tree 변화 범위를 직접 시각 판독의 대용으로 쓰지 않는다.
- [영어 추가 설명 주석0개](../assets/pr7382_20260926/stage34_english_comment_scan.json). Rust/Python/JS/Shell의 이번 통합 추가 설명 주석을 재검색했다. 기준값·골든·시각 허용치·글꼴 예외를 바꾸지 않았고 로그·output·generated는커밋하지 않는다. 전체시각/freshWASM·최종전체회귀/lint/Skia가 아직필요하며 다음 개별 보정은155쪽 그림64의 소유 원점·본문 겹침이다.

![14쪽 그림11 원점 복원](../assets/pr7382_20260926/stage34_native_hwpx_review_014.png)
![14쪽 overlay](../assets/pr7382_20260926/stage34_native_hwpx_overlay_014.png)
![154쪽 마지막 두 줄 소유 복원](../assets/pr7382_20260926/stage34_native_terminal_hwpx_review_154.png)
![155쪽 남은 그림 겹침](../assets/pr7382_20260926/stage34_native_terminal_hwpx_review_155.png)


## 보정35 사전 분석 — 다음 쪽 어울림 그림의 저장 소유와 원점

- 두 독립 원본 한컴2024 PDF는 그림64를155쪽이 아닌156쪽의가시상단89.981363px/캡션404.101033px에출력한다. 원본 문단1692의Square/문단Top/Column그림은오프셋518HU·높이22713HU·상하여백510HU와아래캡션을갖는다. 다음문단1693은앞쪽의전폭2줄뒤vpos0/폭25139HU로재개하며1694–1697의좁은어울림띠가이어진다. 단순빈공간추정이아니라이저장줄소유와PDF로쪽소유를확인한다.
- HWPX는Native전용next_page_owner가같은저장계약을제외해155쪽표/본문위에그림을남겼다. Native는그림을156쪽으로이월하지만배치helper가후속문단의첫LineSeg만조회해이미앞쪽에서소비한전폭줄을읽고518HU를다시더한다(상단96.9px). 유효저장계약의출처조건과실제로이쪽이소유한PartialParagraph시작줄을함께대조해야한다. 저장리셋/어울림폭이없는일반Square그림,편집/합성정보는같은근거로이월하지않는다.
- 소비경로는저장후속밴드조회→DeferredSquarePictureControl→push_new_page의정확한밴드문단/WrapAnchorRef→flush의첫Shape와실제후속Full/PartialParagraph→layout의그림원점→공통그림/캡션배치다. 원본두형식의155쪽없음/156쪽단일그림·캡션·독립좌표정식실패를먼저기록하고,측정·그림띠·후속본문소유와같은프레임을소비하도록보정한다.


### 보정35 결과

- 원본 정식 두 검사는 수정 전0PASS/2FAIL(exit100,0.208s), 수정 후2PASS였다. HWPX는155쪽에 그림을 남겼고 Native HWP는156쪽에서518HU를 다시 더했다. 저장 그림의 다음 쪽 조회를 두 형식에 공통으로 적용하고, 배치는 현재 쪽의 실제 Full/PartialParagraph 시작 컷과 같은 WrapAnchorRef를 확인한다. 앞쪽에서 소비한 문단 오프셋을 새 쪽에 반복하지 않는다. 일반 그림의 좌표를 clamp하거나 숨기지 않는다.
- 원본215쪽·155쪽 그림/캡션 없음·156쪽 단일 그림/캡션과 독립89.981363/404.101033px 좌표·157쪽 중복 없음·후속 본문을 검사했다. 저장 어울림 폭을1000HU 바꾼 수동 IR 대조군에서는 그림을 임의로 다음 쪽에 넘기지 않는다. 이 대조군을 한컴 재저장 출력 일치로 보고하지 않는다. 편집/합성 줄을 제외하는 소스 조건과 기존 객체 편집 검사를 확인했지만 이 그림의 실제 편집 후 한컴 대조는 미검증이다.
- 기존 #3738 검사의96.9px 기대값은 다른 문서의 문단 기준 그림으로 이 이월 경로를 추정한 값이었다. 동일 원본 한컴 PDF156쪽의67.486008pt 가시 상단과 새 원본 검사를 근거로89.981344px로 갱신했다. 허용치를 완화하지 않았다. 최종 실제 suite 배정을 재발견한 확대 검사는192개 중191PASS/1FAIL(exit100,8.350s,threads8)이며 실패는 기존120쪽 표1283의 중복 위여백이다. 보정 전34의 전체 tree에도 상단90.7px이며 PDF 괘선86.945312px와 다르다. 이 실패를 승인 가능으로 바꾸지 않는다.
- 검사 보강 중 지역 탐색 함수 참조 빌드 오류와 prepare 뒤 suite 이동으로 빠진 범위는 최종 검증으로 세지 않는다. 최종 정확한 명령·source/test/CLI 해시·실패 시도·판정은 [검증 기록](../assets/pr7382_20260926/stage35_validation.json)에 연결했다. fmt·고정base manifest(6222 attrs)·source-unit(4205/298)은exit0, 불변 CLI 빌드는exit0/94.476s다. 추가 영어 설명 주석은 [0개](../assets/pr7382_20260926/stage35_english_comment_scan.json)이며 변경한 경로의 기존 영어 설명도 한글로 바꿨다.
- HWPX155/156쪽은75.49194→99.15740%/66.51470→93.52007%, HWP156쪽은86.04285→94.81768%다. 각주211–215·표36·뒤 제목을 보존하고 그림64/캡션 원점을 복원했다. 전후 review와 standalone overlay16개를 직접 읽었다. 남은 글자 메트릭/일부 표 내부 차이를 완전 일치로 보고하지 않는다.
- 전체215쪽 tree 변화는 HWPX126/127/155/156쪽, HWP156쪽뿐이다. [전후 범위](../assets/pr7382_20260926/stage35_original_tree_difference.json). 추가 HWPX126/127쪽의 전후 review/overlay8개도 직접 읽었다. 그림56이126쪽 표/각주를 덮지 않고127쪽 좁은 띠에 출력되어67.14618→84.96470%/65.32413→97.52687%다.126쪽의 표 원점·캡션과 각주 차이는 남으며 이 선택 gate는 `re_review_required`다. 총 새PNG24개를 보존했다.
- 판정: 그림64 쪽 소유/원점은 충족.120쪽 표 회귀와126쪽 시각 gate는 미충족. 최신 전체215쪽 시각/fresh WASM·전체 nextest/lint/Skia·TABLE 편집 및 wrapper 분할 반례는 미검증이다. 통합 PR 생성/승인은 계속 보류하고, 이 단계 커밋 뒤120쪽 표 위여백을 다음 개별 보정으로 처리한다. 로그·output·generated는 커밋하지 않는다.

![155쪽 그림 겹침 해소](../assets/pr7382_20260926/stage35_after_hwpx_review_155.png)
![156쪽 그림64 원점](../assets/pr7382_20260926/stage35_after_hwpx_review_156.png)
![156쪽 독립 overlay](../assets/pr7382_20260926/stage35_after_hwpx_overlay_156.png)
![127쪽 그림56 소유 복원](../assets/pr7382_20260926/stage35_after_normal_hwpx_review_127.png)


## 보정36 사전 분석 — 확정 표 원점 뒤 바깥 위여백 중복 적용

- 원본120쪽 빈 호스트 문단1283은vpos0/폭0이며6×1 RowBreak 표는선언23790HU·위/아래여백283HU·오프셋0이다. 다음 저장 줄은24356HU로전체바깥프레임을닫는다. 독립 한컴 HWP PDF120쪽 괘선 상단65.208984pt=86.945312px이며본문83.16px+283HU=86.933333px와대응한다. 기존90.706667px는위여백을두번더한위치다.
- 실제 소비 경로는`stored_empty_control_table_frame → query_original_control_table_frame/whole-fit → paragraph_float_placements.table_top → layout.rs의확정원점선택 → table_layout의physical_outer_box_paint_inset`이다. 앞 결과가이미여백을포함하지만마지막legacy paint inset이다시283HU를더한다. 같은분기의흐름끝은이inset을빼므로뒤문단은맞고표원점만어긋난다. 최종원점을clamp하지않고확정계획과legacy inset중한계약만소비해야한다.
- 기존원본정식회귀는보정35의최종확대에서위여백중복으로FAIL했다. 먼저새독립PDF좌표검사와HWPX정상대조군을수정전에실행하고,위치·크기·뒤문단·215쪽보존을검증한다. 확정계획이없는legacy표의위여백은유지하고분할/편집을전체프레임증거로새로승격하지않는다.


수정 전 새정식2개는1PASS/1FAIL(exit100,0.252s)이며 HWPX정상대조군은PASS, Native원본은90.706667px/독립86.945312px 차이로FAIL했다. 확정계획이있는호스트는legacy paint inset과그흐름끝차감을함께비활성화하여표원점만한번소비하고후속흐름을유지한다.


### 보정36 결과

- 확정 `paragraph_float_placements.table_top`이 있는 호스트는 이미 바깥 위여백을 소비했으므로 legacy paint inset을 반복하지 않는다. 같은 조건으로 흐름 끝의 inset 차감도 끄며, 확정 계획이 없는 기존 경로는 유지한다. 측정 원점을 배치 뒤 clamp하거나 표 크기를 바꾸지 않았다.
- 과거 불변 CLI를 대조하니 보정29의120쪽 표 상단은86.9px였고 보정30에서90.7px로 밀렸다. 이번 결함은 기여자 원 변경이 아니라 메인터너 보정30의 회귀다. [발생 단계와 CLI 해시](../assets/pr7382_20260926/stage36_maintainer_regression_origin.json)를 보존했다.
- 원본 정식2개는 수정 전1PASS/1FAIL(exit100,0.252s), 수정 후 모두PASS다. 표 원점/크기·뒤 본문·각주와215쪽, 다음 쪽 중복 없음까지 검사했다. 확대208개는207PASS/1FAIL(exit100,6.676s,threads8)이다. 남은 #2097 실패는 마지막 행이 실제2쪽으로 넘어가며, 불변33/34/35 CLI 대조에서 보정34부터 생겼다. 단순 `PartialTable` 표기 변화로 분류하지 않으며 [행 소유 증거](../assets/pr7382_20260926/stage36_2097_owner_origin.json)를 남겼다.
- Native HWP120쪽은81.72702→99.67776%, HWPX120쪽99.67776%와 양쪽121쪽98.96031%는 유지됐다. 전체215쪽 tree 변화는 HWP2/119/120쪽뿐이고 HWPX는 없다. 추가 HWP2쪽91.23975→94.69572%,119쪽89.59948→99.75059%다. 새 전후 review·standalone overlay24개를 직접 읽어 표 원점·뒤 제목/그림55·각주158–160·후속 본문 보존을 확인했다. 점선 화살표·글자 메트릭의 작은 차이는 남으며 완전 픽셀 일치로 보고하지 않는다.
- fmt·고정base manifest(6224 attrs)·source-unit(4205/298)은exit0, 불변 release-test CLI 빌드는exit0/79.866s다. [정확한 명령·source/test/CLI 해시·전후 검사](../assets/pr7382_20260926/stage36_validation.json), [전수 tree 변경 범위](../assets/pr7382_20260926/stage36_original_tree_difference.json), [추가 영어 설명 주석0개](../assets/pr7382_20260926/stage36_english_comment_scan.json)를 연결했다. 모든 로그는output/logs에만 남기고 커밋하지 않는다.
- 판정:120쪽 위여백 중복은 충족. #2097 마지막 행 분할과126쪽·#6782의76/78쪽 시각 차이는 미충족이다. 실제 TABLE 편집/래퍼 분할·최신 전수 시각/fresh WASM·전체 nextest/lint/Skia는 미검증이며 통합 PR 제출/승인은 보류한다. 이 단계 커밋 뒤 #2097 실제 행 소유 변경을 다음 개별 보정으로 분석한다.

![120쪽 표 원점 복원](../assets/pr7382_20260926/stage36_after_hwp_review_120.png)
![120쪽 독립 overlay](../assets/pr7382_20260926/stage36_after_hwp_overlay_120.png)
![119쪽 표와 뒤 그림 보존](../assets/pr7382_20260926/stage36_after_additional_hwp_review_119.png)


## 보정37 사전 분석 — 수동 저장 셀 높이와 마지막 행의 실제 점유

- #2097 합성 입력은66300/1200/500HU 셀을68000HU 표로 묶었지만 마지막 두 셀의 저장 줄은vpos141+줄높이1200HU이고 양쪽 안여백141HU다. 특히 마지막 셀500HU는 가시 줄과 여백을 담지 못한다. 기존 README도 한컴이 이 값을 재실측·확장하므로 합성 선언-fit 기대를 한컴 정답지로 쓰지 말라고 명시한다. 실측 초과16px를 모두 노이즈로 보는 기존 검사의 가정부터 재검토한다.
- 동일 합성 원본을 저장 제품 메타데이터(한컴2020)에 따라 engine2020으로 독립 변환했다. 새 PDF는2쪽이고 BIG/MID ROW는1쪽, TAIL ROW EXPANDING과 AFTER TABLE은2쪽이다. 보정33의 강제 통째 표는 실제 본문 하단1028.04px를 넘어1043.1px까지 출력한다. 보정34/36의 마지막 행 이월은 한컴의 행 소유와 대응하며, 실제 원본3080901은 전후 모두1쪽이다.
- 따라서 보정34의 실제 행 소유 변경 자체를 결함이라고 단정하지 않는다. 독립 PDF·원본 저장값·실제 후속 점유를 근거로 잘못된 합성 기대를 교정하되 원본 실패와 전후 증거를 보존한다. 정상 실문서의 통째 배치와 이월 반례의 가시 줄/테두리/뒤 문단·단일 소유를 정식 검사로 연결한다. 합성 PDF와 저장 IR의 글자/셀 높이 차이는 재조판 여부를 구분해 기록하며, 이 증거 보정만으로 전체 시각/PR 승인을 선언하지 않는다.


### 보정37 결과

- 수동 원본의 마지막 행 이월은 동일 입력 한컴2020 PDF의 소유와 대응한다. 이전 검사는 실측 초과11.3px를 노이즈라고 간주해500HU 셀 안의1200HU 줄을1쪽에 강제로 남기는 기대였다. 그 기대는 독립 PDF와 물리 예산에 맞지 않아 교정했다. 렌더러의 수용 조건·paint·허용치를 완화하지 않았고 런타임 코드는 바꾸지 않았다. 보정36에서 실제 행 소유 변경을 잠정 보류한 판단은 이 근거에 따라 정정한다.
- 교정 전 기존2개는1PASS/1FAIL(exit100,0.019s)이다. 교정 후 최종 확대91개는90PASS/1FAIL(exit100,5.984s,threads8)이며 #2097의 원본/재생성 쪽 소유·물리 하단·뒤 문단·정상 실물17개 행은 충족했다. 새 검사는 현재 코드에서 통과한 경계 증거이며 수정 전 구현 결함 검출로 보고하지 않는다. 남은 #2105 합성 선언-fit 실패는 별도 보류로 기록하고 다음 개별 검토에서 독립 입력/기준부터 확인한다.
- 새 물리 검사에서 뒤 문단 그룹 bbox를 실제 글줄 위치로 잘못 조회한 자체 오류는 `TextLine`의 실제 원점으로 수정·재실행했다. venv symlink를 resolve해 다른 interpreter로 실행한 PIL 실패도 올바른 venv 절대 경로로 재실행했다. 같은 형식 HWPX 저장은 client에서 지원하지 않아 한컴2020 HWP→HWPX의 실제 저장 경로를 사용했다. 이 실패를 결함 재현이나 시각 gate로 세지 않는다. [명령·전후 검사·실패 시도·source/CLI 해시](../assets/pr7382_20260926/stage37_validation.json).
- 원본 수동 HWPX와 그 PDF를 보존했고, 한컴 HWP를 거쳐 독립 재저장한 HWPX 및 해당 PDF를 별도로 커밋한다. 재생성은 셀 줄높이1200→1000HU/vpos141→0, 표 높이68000→67582HU, 실제 저장 어울림 폭/뒤 문단 줄 정보를 바꿨다. [입력/PDF 해시·독립 텍스트 좌표·원본 행 소유·재생성 명령](../assets/pr7382_20260926/stage37_independent_geometry.json). 정상 실문서3080901의 전후 전체 page-items는 동일하고1쪽이며2020/2022 PDF도 같은1쪽이다.
- Native 수동 원본의1/2쪽은73.45554→86.39337%/27.87648→64.42367%로 마지막 행/뒤 문단 소유를 복원하지만 저장 줄의 글자 높이와 셀 괘선 차이가 남는다. 정상 실문서도51.76291%로 행 괘선과 내부 줄 위치가 차이나므로 전체 시각 충족으로 보고하지 않는다. 재생성 대조군의1/2쪽은100.00000/100.00000%이고 표 경계·마지막 행·뒤 글줄을 직접 확인했다. 이는2px 관용 점수이며 완전 픽셀 일치가 아니다. 서로 다른 입력의 통과를 원본 점수 개선으로 섞지 않았다.
- 전후12개와 재생성4개의 새 review·standalone overlay16개를 직접 읽었다. fmt·고정base manifest(6227 attrs)·source-unit(4205/298)은exit0이다. [추가 영어 설명 주석0개](../assets/pr7382_20260926/stage37_english_comment_scan.json). 모든 로그/중간 HWP/MCP 원문은output에만 두고 커밋하지 않는다.
- 판정: 독립 출력에 따른 #2097 마지막 행 이월과 실제 소유는 충족. 원본/실문서 전체 시각 gate 및 #2105 기대 재검토는 미충족/보류다.126쪽·#6782의76/78쪽,실제 TABLE 편집/래퍼 분할·최신 전수 Native/fresh WASM·전체 nextest/lint/Skia도 남아 있어 통합 PR 생성/승인은 계속 보류한다.

![수동 원본의 마지막 행 소유](../assets/pr7382_20260926/stage37_stage36_synthetic_review_002.png)
![재생성 대조군 첫쪽](../assets/pr7382_20260926/stage37_resaved_review_001.png)
![재생성 대조군 다음쪽](../assets/pr7382_20260926/stage37_resaved_review_002.png)
![원본 실문서의 남은 시각 차이](../assets/pr7382_20260926/stage37_stage36_real_overlay_001.png)


## 보정38 사전 분석 — 쪽 시작 RowBreak 합성 표의 짧은 마지막 셀

- 남은 #2105 검사는 #2097과 같은 수동500HU 마지막 셀/1200HU 저장 줄을 쪽 시작에 둔다. 큰 첫 행68000HU와 두 짧은 셀1200/500HU를69700HU 표 선언으로 묶었으므로 선언 합이 본문에 들어간다는 이유만으로 가시 내용까지 수용한다고 볼 수 없다.
- 먼저 같은 입력의 한컴2020 독립 PDF와 기존 실패를 보존한다. 원본을 재저장 대조군으로 바꾸어 승인하지 않고, 쪽 시작/중간 쪽의 실제 행 소유와 표 끝/뒤 문단을 각각 검사한다. README에서 언급한19378753 원본은 저장소와 Mac 기준 자료 경로에서 아직 확인하지 못했으므로 그 실물의 정합은 미검증으로 둔다.


재생성 대조군의 새 정식 검사는 수정 전 마지막 행 조각 누락으로FAIL했고 실제 표 하단이 본문을7.3px 넘었다. `TABLE_DRIFT`의 실측940.9px/본문933.6px와 `DIAG_FIT`의plain=false/declared=false/overlay=true를 대조했다. 원인은 빈 `current_items`의 `all(Shape)`가 참이 되어 실제 배경 개체가 없는데도12px 여유 경로를 사용한 것이다. 저장 높이 수용이나 측정 행 높이를 바꾸는 원인이 아니므로 그 조건을 넓히지 않는다. `TypesetState`의 실제 배경 개체 조회를 block/inline 호출이 공유하게 한다. inline 호출은 뒤의 명시적 `!current_items.is_empty()`가 빈 단의 이월 자체를 막으므로 이번 빈 단 결과 변경으로 동작이 바뀌지 않는다. 실제 Shape만 있는 비빈 단의 기존 판정/12px 값은 동일하다.


### 보정38 결과

- 원본 수동 #2105의 통째 기대는 독립 한컴2020 PDF의 행 소유와 달라 교정했다. 원본 PDF는 BIG/MID ROW가1쪽, TAIL ROW/AFTER TABLE이2쪽이다. 수동 원본과 HWP→HWPX 독립 재저장 대조군 및 두 PDF를 각각 보존했다. 재저장은 표69700→69282HU, 줄높이1200→1000HU/vpos141→0 등을 바꿨으며 대조군 통과를 원본 전체 일치로 바꾸지 않는다. [입력·PDF 해시/독립 좌표/재생성 명령](../assets/pr7382_20260926/stage38_independent_geometry.json).
- 재저장 대조군의 정식 검사는 수정 전217PASS/1FAIL(exit100,6.805s)에서 마지막 행 조각 누락과 본문7.3px 초과를 검출했다. 빈 배열의 `all(Shape)`가 참이어서 빈 단을 배경 도형만 있는 단으로 판단한 것이 실제 원인이다. `current_column_has_only_overlay_shapes`가 실제 개체 존재·Shape만 소유·흐름 높이를 함께 확인하고 block/inline 두 소비 지점이 같은 결과를 사용한다. 기존12px 예산·정상 배경 도형 경로·표 프레임 높이는 바꾸지 않았다.
- 최종 확대219개는219PASS/0FAIL(exit0,6.907s,threads8)이다. 쪽 시작/중간 쪽의 원본·재저장 행 단일 소유와 물리 하단·뒤 TextLine, 정상 실물17행 및 배경 도형 대조군을 검사했다. 기존 원본 기대 실패와 새 실제 구현 결함 실패를 구분한다. fmt·고정base manifest(6229 attrs)·source-unit(4205/298)은exit0, 불변 release-test CLI 빌드는exit0/78.616s다. [정확한 명령·source/test/CLI 해시·로그 해시·판정](../assets/pr7382_20260926/stage38_validation.json).
- Native 재저장 대조군은 수정 전83.99649/28.92386%에서 수정 후100/100%로 두 쪽의 마지막 행·뒤 글줄 소유와 표 경계가 복원됐다. 이는2px 관용 점수이며 완전 픽셀 일치가 아니다. 원본 수동 입력은78.03985/64.42264%로 unchanged이고 저장 줄 높이/괘선 차이가 남는다. 원본 점수 실패를 재생성 입력으로 숨기지 않는다.
- 주 원본 HWPX/HWP의14/120/156쪽 새 review·standalone overlay12개를 직접 읽었다. 각각97.52614/99.67776/93.52007% 및97.54155/99.67776/94.81768%이며 그림·표·캡션·각주와 후속 본문을 보존했다. 전체215쪽 tree는 두 형식 모두 변경 없이 유지됐다. [전체 tree 범위](../assets/pr7382_20260926/stage38_original_tree_difference.json). 전후 원본/재저장까지 새PNG32개를 직접 확인하고 보존했으며 tree 동일성을 전체 래스터 검증으로 보고하지 않는다.
- 추가 설명 주석902줄을 재검색하여 [영어 설명 주석0개](../assets/pr7382_20260926/stage38_english_comment_scan.json)를 확인했다. 활성 수정 파일의 기존 영어 설명도 한글로 바꿨다. 제품/형식/API 식별자는 유지한다.19378753 실제 원본은 Mac 및 Windows 지정 자료 경로에서 찾지 못해 실물 정합을 미검증으로 남겼다. 로그·중간 자료·generated는output에만 두고 커밋하지 않는다.
- 판정: 빈 단의 배경 오인 및 원본 행 소유 기대는 충족. 원본 수동/실문서 전체 시각,126쪽·#6782의76/78쪽,실제 TABLE 편집/래퍼 분할·최신 전체 Native/fresh WASM·전체 nextest/lint/Skia는 남아 통합 PR 생성/승인은 계속 보류한다. 이 단계 커밋 뒤126쪽 표/캡션/각주를 다음 개별 보정으로 분석한다.

![재저장 첫쪽의 표 분할 복원](../assets/pr7382_20260926/stage38_after_resaved_review_001.png)
![재저장 다음쪽의 마지막 행과 뒤 글줄](../assets/pr7382_20260926/stage38_after_resaved_review_002.png)
![원본 수동 입력의 남은 차이](../assets/pr7382_20260926/stage38_after_original_overlay_002.png)


## 보정39 사전 분석 — 캡션이 닫는 빈 호스트의 전체 저장 프레임

- 현재 CLI의126쪽은84.96470%로 재검토 상태다. 표1350은 빈 폭0 호스트vpos28000HU, 문단 상대Top오프셋479HU, 위/아래여백283HU, 표17432HU, 아래캡션 gap850HU/줄1000HU이며 다음 저장 줄48327HU다. `28000+479+283+17432+850+1000+283=48327`로 실제 전체 개체 프레임이 닫힌다. 두 원본 한컴2024 PDF의 괘선/캡션과 대조한 PDF 괘선 상단은466.209351px, 캡션710.34px다. 저장 프레임의466.653333px와0.444px차이는 같은0.5px허용 범위다. 현재 표/캡션은462.9/706.6px로 위여백 하나가 빠졌고 뒤 본문782.2px는 유지된다.
- 실측은 본체232.426667px+캡션24.666667px=257.093333px다. 기존 전체 저장 프레임 조회는 캡션을 일괄 제외하고 수평기준Column만 허용해, 동일한 수직 프레임을 가진 Para수평 표도 배치 계획을 얻지 못한다. 수평 기준은 같은 문단/단 계열이며 현재 가로 원점은 이미 PDF와 대응한다. 가로 원점이나 본체 높이를 다시 보정하지 않고 전체 수직 개체 프레임·캡션 측정 결과·실제 다음 저장 줄을 대조해야 한다.
- 소비 경로는 공통 캡션 높이→표 실측의 effective_height→query_original_control_table_frame→whole/split 준비의 occupied_bottom/소유 키→paragraph_float_placements.table_top→layout 확정 원점/캡션 배치다. 먼저 원본 HWP/HWPX의 표·캡션·뒤 본문 위치 정식 FAIL을 기록한다. 위/아래 캡션과 편집/손상 저장 줄의 적용 여부를 실제 코드와 대조하며, 전체 프레임을 입증하지 못한 수동 변형은 원본 출력과의 일치로 보고하지 않는다. 각주 들여쓰기 차이는 이 원점 수정과 별도 사유로 남긴다.


### 보정39 결과

- 공통 캡션 실측과 본체 높이·바깥 여백이 실제 다음 저장 줄을 닫는 경우 전체 수직 프레임을 공유한다. 수평 Para/Column은 기존 가로 원점을 유지한다. 후속 첫 줄의 앞 간격까지 저장 프레임이 소유하므로 측정의 문단 커서와 실제 layout이 같은 `stored_frame_successor_shared_spacing_px`를 소비한다. 부분 조각의 실제 컷/쪽 소유로 바뀌면 전체 후속 원점을 제거한다. 본체 높이·clamp·기존 허용치는 바꾸지 않았다.
- 원본 HWP/HWPX 정식 검사는 수정 전2FAIL(exit100,0.230s)로 표/캡션 위여백 누락을 검출했다. 최종 확대 검사는223PASS/0FAIL(exit0,7.660s,threads8)다. 위 캡션 수동 계약과 본문 예산을 줄인 실제 분할에서 원본15개 셀 문단 전체 내용·행 소유·캡션 단일 소유·뒤 제목·본문 물리 끝을 확인했다. 수동 변형은 한컴 재저장 일치 증거로 승격하지 않는다. [정확한 명령·source/test/CLI 해시·최종 검사·자체 오류·로그 해시](../assets/pr7382_20260926/stage39_validation.json).
- 후속 앞 간격을 중복 계상한 후보 실패3개, 생성자 필드 누락 컴파일101, 행 번호 반복을 중복으로 본 잘못된 검사1개를 같은 범위에서 수정했다. 마지막 검사 목록에서5개가 빠진218PASS 실행은 최종 확대 결과로 사용하지 않고 현재 파생 suite를 다시 조회해223개를 재실행했다. CLI는exit0/81.533s이며 이후15문단 내용 보강은 테스트만 변경했다. [컴파일된 런타임 불변 증거](../assets/pr7382_20260926/stage39_cli_runtime_proof.json). fmt·고정base manifest(6233 attrs)·source-unit(4205/298)은exit0이다.
- Native 원본126쪽은 두 형식 모두84.96470→92.75732%, 추가 영향131쪽은84.22025→94.67057%다. 표31 괘선 상단466.653333px와 캡션710.34px가 복원되고 뒤 제목782.18px는 유지됐다.127쪽 HWPX97.52687%/HWP98.83521%로 그림56/뒤본문도 보존됐다. 전후 review·standalone overlay24개를 직접 읽었다. [독립 PDF 좌표](../assets/pr7382_20260926/stage39_independent_geometry.json), [전체215쪽 tree 전후 범위](../assets/pr7382_20260926/stage39_original_tree_difference.json). 두 형식 모두126/131쪽만 tree가 변경됐으며 나머지213쪽 tree 동일성을 전수 래스터 통과로 보고하지 않는다.
- 앞 단계 추가 설명902줄 검사와 이후17줄을 연결해 [추가 영어 설명 주석0개](../assets/pr7382_20260926/stage39_english_comment_scan.json)를 다시 확인했다. 제품/형식/API 식별자는 보존한다. 로그·output·generated는 커밋하지 않는다.
- 판정: 캡션 표 원점과 후속 줄 간격의 공통 소비는 충족.126쪽 각주171/172의 이어지는 줄이 PDF x112.0px 대비 rhwp x94.5px로 약17.5px 왼쪽인 문제는 미충족이며 점수 통과로 해소하지 않는다. #6782의76/78쪽·실제 TABLE 편집/래퍼 분할·최신 전수 Native/fresh WASM·전체 nextest/lint/Skia도 남아 통합 PR 생성/승인은 계속 보류한다. 이 단계 커밋 뒤 각주 들여쓰기를 다음 개별 보정으로 분석한다.

![표31과 뒤 본문의 원점 복원](../assets/pr7382_20260926/stage39_after_hwpx_review_126.png)
![동일 프레임을 쓰는131쪽의 개선](../assets/pr7382_20260926/stage39_extra_after_hwpx_review_131.png)
![남은 각주 들여쓰기 차이](../assets/pr7382_20260926/stage39_after_hwpx_overlay_126.png)


## 보정40 사전 분석 — 번호 있는 각주의 이어지는 줄 내어쓰기

- 원본 각주171/172는 문단속성3의 indent=-2620HU(해석 후-1310HU=-17.466667px), margin_left=0을 공유한다. 저장 첫 줄 플래그0x60000/나머지0x160000은 둘째 줄부터 내어쓰기를 적용한다. 두 독립 한컴2024 PDF126쪽은 번호를 포함한 첫 줄 x94.56px, 이어지는 줄 x112.0px이며 현재 번호 전용 경로는 모든 줄을 각주 영역x94.466667px에서 시작한다.
- 생산은 공통 ParaShape 해석→ResolvedParaStyle.margin_left/indent→compose_footnote_paragraph의 저장 글줄이고, 번호 없는 조각은 공통 문단 배치의 줄별 들여쓰기를 소비한다. 번호 있는 경로 `layout_footnote_paragraph_with_number`는 문단 여백/들여쓰기를 조회하지 않고 매 줄 area.x를 사용한다. 번호 너비를 상수로 복제하거나 좌표 clamp하지 않고 같은 문단 줄별 규칙을 실제 원본 줄 번호로 소비해야 한다. 가시 높이·페이지 소유·줄 내용/간격은 변경하지 않는다.
- 먼저 원본 두 형식의 번호/이어지는 줄 TextRun 좌표·전체 내용·쪽 소유 실패를 검사한다. zero/positive/negative 문단 계약과 번호 없는 실제 이어받기 정상 경로를 함께 대조한다. 수동 IR 변형은 독립 한컴 출력 일치 증거로 사용하지 않는다.


### 보정40 결과

- 기존 일반 문단의 줄별 들여쓰기/저장 플래그 판정을 공통 `paragraph_line_indent_for_source`로 옮기고 번호 있는 각주도 동일 결과를 소비한다. 실제 원본 줄 번호 `line_start+offset`을 사용하며 첫 줄 번호와 뒤 텍스트는 같은 원점에서 이어진다. 문단 좌/우 여백을 함께 반영한다. 셀/재조판의 기존 적용 조건은 유지하고 각주 가시 높이·쪽 소유·줄 내용/간격은 변경하지 않는다.
- 원본 HWP/HWPX 정식 두 검사는 수정 전0PASS/2FAIL(exit100,0.208s)로 이어지는 줄x94.493333px를 검출했다. 수정 후 두 형식과 0/양수/음수 들여쓰기·저장 적용 비트 없는 수동 변형의3검사는3PASS/0FAIL(1.029s)다. 원본171/172 전체 내용과1172HU 줄 전진, 첫 줄 번호와 이어지는 글줄·인접 쪽 중복 없음을 함께 확인했다. 수동 문단속성 변형은 한컴 재저장 일치 증거가 아니다.
- 최종 확대는 이전40모듈에 #6190의 저장 비트·셀 내어쓰기·HWP3·각주 폭 재조판/줄 높이 대조군을 추가해243PASS/0FAIL(exit0,8.465s,threads8)이다. fmt·고정base manifest(6236 attrs)·source-unit(4205/298)은exit0이며 release-test CLI 빌드는exit0/89.873s다. [정확한 명령·source/test/CLI 해시·로그 해시·판정](../assets/pr7382_20260926/stage40_validation.json), [원본 문단 속성/줄](../assets/pr7382_20260926/stage40_note_source.json), [독립 PDF 좌표](../assets/pr7382_20260926/stage40_independent_geometry.json).
- Native126쪽은 두 형식 모두92.75732→94.99385%이며 이어지는 글줄x111.96px가 독립 PDF x112.0px와 같은 허용 범위에 들어왔다. 번호 없는 이어받기도 포함한67/179쪽을 함께 직접 확인했다. HWPX67쪽93.58939→96.21756%,179쪽은양쪽97.53225→97.93535%다. 번호 없는 기존 꼬리·각주 번호 단일 소유·표/본문·후속 제목을 보존했다. 새 review·standalone overlay24개를 직접 읽었다.
- 전체215쪽 tree는 각주 영역 안에서만 HWPX78쪽/HWP79쪽이 바뀌었고, 두 형식 모두 각주 영역 밖의 tree는 모든215쪽에서 동일하다. [전수 tree 범위](../assets/pr7382_20260926/stage40_original_tree_difference.json). 이는 각주 전체 래스터 통과의 대용이 아니며 변경된 모든 쪽은 최종 전수 시각에서 재검토한다.
- 현재 통합 base c80a8370a 이후 줄 시작 위치의 추가 설명 주석922줄을 재검색해 [영어 설명 주석0개](../assets/pr7382_20260926/stage40_all_language_comment_scan.json)를 확인했다. 코드 울타리와 제품/형식/API 식별자는 유지한다. 로그·output·generated는 커밋하지 않는다.
- 판정: 번호 있는 각주 내어쓰기 누락은 충족. HWP67쪽은 수정 전72.73184%→수정 후74.64010%로 표/뒤 본문이 PDF보다 위에 놓인 차이가 남아 gate 재검토 상태다. 이를 기존 차이라는 이유로 승인하지 않는다. #6782의76/78쪽·실제 TABLE 편집/래퍼 분할·최신 전수 Native/fresh WASM·전체 nextest/lint/Skia도 남아 통합 PR 생성/승인은 계속 보류한다. 이 단계 커밋 뒤 최신 전수 시각과 HWP67쪽 원점부터 다음 개별 사유를 분석한다.

![126쪽 각주 내어쓰기 복원](../assets/pr7382_20260926/stage40_after_hwpx_review_126.png)
![번호 없는 이어받기와 뒤 각주 보존](../assets/pr7382_20260926/stage40_after_hwpx_review_179.png)
![HWP67쪽의 남은 표와 뒤 본문 위치 차이](../assets/pr7382_20260926/stage40_after_hwp_overlay_067.png)


## 보정41 사전 분석 — 원본 HWP의 캡션 표 이어받기 바깥 프레임

- 같은 원본의67쪽 PDF는 두 형식 모두 표 상단86.945312px, 캡션156.434347px, 뒤 제목200.261047px다. 현재 HWPX는86.9/156.5/200.3px, HWP는83.2/152.8/192.8px다. 이어받은 표의 위여백283HU와 종료 아래여백283HU가 HWP 경로에서 빠져 표/캡션은3.773333px, 뒤 본문은7.546667px 위에 놓였다. 66쪽 첫 조각의 원점에도 두 형식의283HU 차이가 있다.
- 생산/소비 경로는 `hwpx_column_rowbreak_fragment_opens_outer_top`의 포맷 조건→continuation/budget의 host_before/terminal_outer_bottom→확정 ParagraphFloatPlacement→table_partial의 가시 원점→layout의 occupied_bottom이다. 같은 빈 폭0 앵커·수직 캡션·문단 기준 자리차지 RowBreak 프레임을 HWPX에서만 소비하는 가정부터 대조한다. 제목행 없는 일반 HWP 이어받기와 중첩 프레임은 기존 계약을 보존한다. 단순히 모든 Native 표에 여백을 추가하지 않는다.
- 먼저 원본 HWP/HWPX의67쪽 괘선/캡션/뒤 제목 및 원본 두 조각의 행·각주·캡션 단일 소유를 확인하는 정식 실패를 남긴다.66쪽의 약1px 잔여도 독립 PDF와 직접 대조하며 새 기대 허용치를 넓혀 숨기지 않는다.


### 보정41 결과

- 포맷 이름으로 제한했던 바깥 프레임 판정을 공통 `column_rowbreak_fragment_opens_outer_top`으로 바꿨다. HWP는 실제 폭0 빈 개체 앵커와 수직 캡션을 확인한 경우에만 같은 프레임을 소비한다. 캡션 없는 일반 HWP 이어받기·중첩 프레임·단 중간 조각은 기존 계약을 유지한다. 문서 ID나 수치로 대상을 고르지 않았다.
- 실제 소비는 block/whole_fit의 전체 캡션 배치 계획→continuation/fragment/budget의 이어받기 위여백/종료 아래여백→확정 배치→layout/table_partial의 가시 원점/끝점→layout의 occupied_bottom이다. 위여백283HU를 예약·배치에 같이 소비하고 종료 아래여백283HU를 후속 흐름에 한 번 소비한다. 실제 원본67쪽 괘선·캡션·뒤 제목의 독립 좌표는 [PDF 기하](../assets/pr7382_20260926/stage41_independent_geometry.json)에 보존했다.
- 정식 원본 두 검사는 수정 전1PASS/1FAIL(exit100,0.253s), 수정 후2PASS/0FAIL(exit0,0.262s)이다. 표23의5행/2행 조각, 끝 캡션·뒤 제목 좌표, 인접 쪽 캡션 중복 없음, 각주77 번호/꼬리 소유와215쪽을 확인했다. 이전 보정·일반 표/중첩/각주 대조군을 포함한 확대245PASS/0FAIL(9.793s,threads8), fmt·고정base manifest6238 attrs·source-unit4205/298도exit0이다. release-test CLI 빌드는exit0/106.338s다. [명령·exit·로그 해시·CLI 해시·시각 판정](../assets/pr7382_20260926/stage41_validation.json), [source/test 증거](../assets/pr7382_20260926/stage41_source_proof.json).
- HWP66쪽88.72983→98.94767%,67쪽74.64010→96.21756%,77쪽43.20381→98.35157%다. 표 외곽·후속 본문·그림/캡션 위치를 직접 읽었다. HWPX66/67/68쪽98.94767/96.21756/96.59031%와 HWP68쪽96.59031%는 그대로다. 전후 review/standalone overlay28개 중20개 고유 이미지를 직접 읽었고, 나머지8개는 이미 판독한 이미지와 바이트까지 동일함을 대조했다.
- 두 형식 모두215쪽이다. 전수 tree에서 HWP는66/67/77쪽만 변경됐고212쪽은 동일하며 HWPX215쪽은 모두 동일하다. 변경된 모든 쪽은 이번 Native 비교에 포함했다. [전수 변경 범위](../assets/pr7382_20260926/stage41_original_tree_difference.json), [점수/PNG 동일성](../assets/pr7382_20260926/stage41_visual_comparison.json). 전수 tree 동일성을 전체 래스터 통과로 보고하지 않는다.66쪽 약0.8px 저장 원점/독립 괘선 차이도 남겨 두며67쪽의0.5px 기대 허용치를 넓히지 않았다.
- [이번 추가 설명 주석4줄](../assets/pr7382_20260926/stage41_comment_scan.json)은 모두 한글이다. 앞 주석 전용 커밋292ba2161은 인라인을 포함한 추가 주석930줄·영어 설명0개를 확인했다. 로그/output/generated는 커밋하지 않는다.
- 판정: 원본 HWP 캡션 표의 바깥여백 누락은 충족. #6782의76/78쪽, 실제 TABLE 편집/래퍼 분할, 현재 수정 후 전체 Native/fresh WASM·전체 nextest/lint/Skia는 남아 통합 PR 생성/승인은 계속 보류한다. 실행 중인 전수 시각은 수정 전 동작의 stage40 고정 CLI이며 이번 보정41의 전체 시각 통과로 재사용하지 않는다.

![HWP67쪽 표와 후속 본문 복원](../assets/pr7382_20260926/stage41_after_hwp_review_067.png)
![HWP77쪽 같은 프레임의 후속 그림 복원](../assets/pr7382_20260926/stage41_after_hwp_overlay_077.png)


## 보정42 사전 분석 — 실제 표 셀 편집과 저장 프레임 제외

- 보정16의 표1136은 원본 저장 되감김/행 높이를 소비하지만 실제 셀 편집 경로는 아직 실행 증거가 없었다. 이전 보정18은 본문512의 편집이므로 이 증거를 대신하지 않는다. `insert_text_in_cell_native`→셀 문단 삽입→`reflow_cell_paragraph_after_text_edit`/셀 vpos 재계산→`mark_table_text_reflowed_after_edit`→재페이지네이션을 직접 실행한다.
- `query_closed_source_frame_placement`와 `saved_multirow_opening_frame_height`는 편집 세션/표 재조판 메타데이터를 거절한다. 이 조건의 실행 효과를 원본 HWP/HWPX에서 한 글자 공백과 실제 너비 부족 줄바꿈을 만드는 긴 삽입으로 확인한다. 수동 LineSeg 플래그 변경으로 편집을 대신하지 않는다.
- 독립 기대는 편집 API에 전달한 문자열과 편집 후 원본 셀/문단 전체 내용, 원본 용지의 네 여백/실제 각주 영역 경계다. 모든 표 조각에서 셀/문단 내용의 누락·중복, 원래 캡션 한 번, 표 물리 하단/후속 내용 소유를 확인한다. 변경된 문서에 원본 PDF 좌표·215쪽을 강제하지 않으며 한컴 편집 후 출력 일치로 승격하지 않는다. 아직 결함이 검출되지 않은 검증 공백이므로 수정 전 FAIL을 미리 주장하지 않는다.


- 첫 실제 실행은1PASS/1FAIL(exit100,1.145s)이다. HWPX의 한 글자 삽입에서105번쪽 표1136은 y669.827+높이408.52=1078.347px로 독립 용지 본문끝1039.347px보다39px(진단 로그38.8px) 넘었다. HWP의1자/120자 삽입은 전체216쪽에서 전체 셀 내용·표 조각·캡션·뒤 본문 검사를 통과했다. 편집 세션은 다른 기존 저장 배치 경로도 제외하며 그 문서의 다른 표/본문 overflow 진단이 있다. 이번 표29 검사의 통과를 편집한 문서 전체의 시각 통과로 확대하지 않는다.
- 원인은 저장 프레임을 거절한 뒤 일반 컷 높이를 선택하는 예약과, 온전한 행에서는 여전히 MeasuredTable을 소비하는 실제 배치가 갈리는 점이다. 재조판 메타데이터가 있는 온전한 행은 실제 공유 행 소유 판정과 같은 측정 높이를 fit에 연결한다. 내용 컷/중첩 행과 원본 저장 프레임 조건을 대신하지 않으며 저장4px 여유나 원점을 편집 경로에 재사용하지 않는다. 전체 fit과 분할 fit의 실제 소비를 함께 대조한다.


### 보정42 결과

- 실제 `insert_text_in_cell_native`로 표1136의 첫 셀에 공백1자 및120자를 삽입했다. 재조판 메타데이터가 있는 온전한 행은 기존 공통 배치 소유 판정의 측정 높이를 전체 fit/분할 예약에서 함께 소비한다. 원본 저장 프레임·4px 안전 여유·원점은 편집 경로에 재사용하지 않는다. 내용 컷/중첩 행의 기존 계약은 유지한다.
- 원본 두 형식의 실제 편집 검사는 수정 전1PASS/1FAIL(exit100,1.145s)에서 수정 후2PASS다. HWPX의 한 글자 삽입은 본문끝1039.347px를39px 넘던 표의 물리 높이를 검출했다. 최종 확대247PASS/0FAIL(exit0,10.594s,threads8)이며 전체 셀/문단 내용의 누락·중복, 실제 TextLine의 셀 내부 좌표, 표/각주 경계, 캡션 단일 소유와 뒤 문단 위치를 검사했다. 뒤에 보강한 셀 내부/캡션 좌표 검사를 최초 FAIL의 원인으로 보고하지 않는다.
- HWPX 공백 편집은215쪽/긴 편집216쪽, HWP 두 편집은216쪽이다. 변경한 문서에 원본215쪽/PDF 좌표를 강제하지 않는다. 편집 세션의 다른 기존 본문 overflow 진단은 남으며 이번 표의 통과를 편집 문서 전체 또는 한컴 편집 후 출력 일치로 확대하지 않는다. 독립적인 편집 후 한컴 PDF는 아직 없다.
- 실제 편집 상태의 SVG를 Native와 같은 local-font Style 경로로 생성해 PNG8개를 직접 확인했다. 본문 글꼴·표 이어받기·캡션/뒤 내용을 확인했다. 첫 None 출력과 굴림/휴먼명조의 cmap이 없는 Subset 출력은 글꼴이 깨져 승인 증거에서 제외하고 output에 보존했다. 이는 진단 캡처 경로의 보완이며 일반 Subset 내보내기 결함을 해결했다고 보고하지 않는다.
- 불변 CLI 빌드는exit0/93.189s이며 이후 변경은 테스트의 증적 생성 방식뿐이다. 원본215쪽 전체 tree는 HWP/HWPX 모두 이전 CLI41과 동일하다. fmt·고정base manifest(6240 attrs)·source-unit(4205/298)은exit0이다. [정확한 명령·source/test/CLI/로그 해시·직접 판독·자체 오류·추가 한글 주석](../assets/pr7382_20260926/stage42_validation.json), [원본 전체 tree 대조](../assets/pr7382_20260926/stage42_original_tree_difference.json), [거절한 Subset 글꼴 증거](../assets/pr7382_20260926/stage42_rejected_subset_font_proof.json). tree 동일성을 전수 래스터 통과로 보고하지 않는다.
- 판정: 보정16의 실제 TABLE 편집 검증 공백과 검출된 예약/배치 높이 불일치는 충족. 래퍼 실제 분할·이월/#6782의76/78쪽/최종 전수 시각·전체 검증은 남아 통합 PR은 보류한다. 새 설명 주석은 모두 한글이며 로그·output·generated는 커밋하지 않는다.

![실제120자 셀 편집의 표 시작](../assets/pr7382_20260926/stage42_hwpx_growth_page_106.png)
![실제 셀 편집의 이어받기와 캡션](../assets/pr7382_20260926/stage42_hwpx_growth_page_107.png)

## 전체 검증과 기존 기대값 재검토 — 보정43 분석

- 제품 runtime head `96c4e47771ecf7f46016bffa1e09c467b2878cbb`, base `eb9142dd7c73297d555383d7d8434a470bdef26e`다. 이 단계는 지침·증거 분석이며 Rust·기대값·래칫 허용치를 바꾸지 않았다. [검증 체크포인트](../assets/pr7382_20260926/stage43_validation_checkpoint.json)와 [43건 원시 실패 목록](../assets/pr7382_20260926/stage43_nextest_full_failures.json)을 보존한다.
- 전체 integration은 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast`:10,327PASS/43FAIL/50SKIP, exit100이다. 실패 목록의 반복 출력을 중복 집계하지 않았다. 43건은 개별21·본문넘침6·용지밖2·쪽수3·텍스트겹침11이며 아직43개 실제 출력 결함이라고 단정하지 않는다.
- release lib4,055PASS, Skia lib4,112PASS, Skia placeholder2PASS/direct PDF4PASS, doc8PASS, Studio1,785PASS/2SKIP다. fmt·Native/WASM/workspace Clippy·workspace build·고정 base manifest/unit 정책·TypeScript도 exit0이다. fresh WASM은 root wrapper의 `--no-opt` 로컬 대체 빌드이며 root pkg/Studio/frozen pkg 해시가 일치한다. Docker 최적화 빌드 통과로 보고하지 않는다. [WASM provenance](../assets/pr7382_20260926/stage43_wasm_pkg_provenance.json).
- 개별 실패 입력14개에 Native 수정 전·후27건, 래칫 증가 입력과 추가 대조군20묶음에40건의 Visual Sweep을 생성했다. 전체 자동 비교와 직접 읽은 대표 페이지를 구분한다. 한글 형식1.3 원본의 한컴 변환은 실패하여 그 PDF 대조는 미검증이다. 입력을 비정상으로 판정하지 않는다. [입력·PDF·페이지 매핑](../assets/pr7382_20260926/stage43_regression_baseline_visual_fixtures.json), [변환 출처](../assets/pr7382_20260926/stage43_regression_reference_conversions.json), [추가 출처](../assets/pr7382_20260926/stage43_regression_reference_extra_conversions.json). 새 기준 PDF는 `pdf/issue7382-regression-review/`에 보존했다.

| 대상 | 독립 출력과 직접 판독 | 현재 판단 |
| --- | --- | --- |
| #3738 HWP23쪽 | 동일 원본은 한컴2024 저장본. 기준 PDF 그림21 y151.995px·캡션498.322px, 현재152.1/498.4px, review99.62374% | 기존148.3±3/495.2±3은 PDF 자체를 벗어난다. 독립 기대값 보정 대상이며 허용치는 유지한다 |
| #6535 저슬랙/페이지 앵커, #6102 | 기준 PDF1쪽의 결재선·발신명·주소가 현재1쪽에서 사라지고 다음 쪽으로 이동 | 기존1쪽 기대는 유효한 실제 회귀. 보정27부터 높이 증가를 추적하며 코드를 보정해야 한다 |
| #7336 | 기준6쪽의 동의서 대신 현재6쪽에는 앞 표의 꼬리와 동의서 제목만 표시 | 7쪽·동의서 소유 기대에 실제 배치 회귀 근거가 있다 |
| #7390 KoPub | 대상 줄 폭414.2/574.1px는 전후 동일; 대상 줄은94/108→95/109쪽 이동 | 폭 회귀로 보고하지 않는다. 페이지 선택 실패와 실제 쪽 배치 회귀를 구분한다 |
| #1133 | HWP/HWPX PDF 괘선 간격113.476px, 현재HWP111.6/HWPX113.4px. 두 형식2쪽 직접 판독 | HWPX를111.6으로 되돌리지 않는다. 상대 일치만으로 양쪽 오답을 잡지 못해 독립 원점·간격 보강과 HWP 잔여 차이 검토가 필요하다 |
| #7203 | 뒤 표 PDF윗변436.961px, 전435.5/후437.4px. 11쪽 전후 직접 판독 | 저장 사다리32.43px와 실제 원점·여백의 계약을 추가 추적하며 기대 수정은 미검증 |
| #5941 | PDF302쪽, 전304/후303. 같은 마지막 내용의 실제 꼬리 PNG는 쪽번호 외 동일하나 PDF의 마지막 두 행 소유는 다름 | 304는 한컴 정답이 아닌 잠정 핀. 현재303으로 갱신하지 않으며 분할·내용 보존 계약을 추가 검증한다 |

[사례별 좌표·판정](../assets/pr7382_20260926/stage43_regression_reassessment.json). #6797 수동 변형은 원본 PDF로 기대값을 입증하지 않는다. 래칫 증가는 실제 보이는 글자·괘선과 raw 상자 진단을 대조하며, 점수 개선만으로 넘침/겹침 증가를 수용하지 않는다.

![#3738 현재23쪽 직접 비교](../assets/pr7382_20260926/stage43_caption3738_review.png)
![#6535 수정 전](../assets/pr7382_20260926/stage43_low6535_base_review.png)
![#6535 현재: 하단 블록 이월](../assets/pr7382_20260926/stage43_low6535_head_review.png)

전체 Native/fresh WASM 시각 실행과 미판독 경계가 남았다. 현재 통합 PR·승인·merge는 계속 보류한다. 모든 로그와 임시 자료는 `output/pr-review/planet6897-7382-20260926/full-96c4e4777/`에 두고 커밋하지 않는다.


## 보정44 결과 — #3738 기존 기대값의 독립 좌표 교정

- 사전 분석은 보정43의 동일 입력·한컴2024 정본 대조다. 그림21 본체151.99467px·캡션498.32166px를 독립 기대값으로 사용하고 기존±3px 허용치를 유지했다. 제품 코드·baseline·golden은 바꾸지 않았다. 현재152.1/498.4px는 정본에 대응하며 과거 회귀198.4/544.7px는 계속 거절한다.
- 기존 검사는 runtime `96c4e47771ecf7f46016bffa1e09c467b2878cbb`에서 의도한 좌표 assertion으로 FAIL(exit100)이었다. 기대 교정 후 파생 `regression_suite_001`의 같은 검사1PASS/178SKIP(exit0,0.837s,threads8)다. base 출력148.3/494.7px는 새 독립 범위를 벗어나지만 새 검사를 base에서 실행한 것으로 보고하지 않는다.
- 파생 준비를 누락한 최초 manifest 검사 실패도 보존했다. `--prepare`부터 fmt·focused nextest·Native/WASM/workspace Clippy·workspace build·고정 base manifest·diff를 순차 재실행해 모두exit0을 확인했다. 준비 후 suite가010에서001로 바뀌었으므로 새 파생 소속에서 검사를 다시 실행했다. generated 파일·로그는 커밋하지 않는다.
- Native23쪽99.62374%, fresh WASM23쪽99.62947%의 review/standalone overlay를 직접 판독했다. 그림21/22·캡션·후속 본문 위치를 대조했으며 자동 점수를 직접 판독의 대용으로 쓰지 않았다. 제품 소스 불변이므로 runtime96의 고정 Native/WASM 증적을 연결한다. 테스트 변경 head의 전체 nextest 통과라고 보고하지 않는다.
- [독립 좌표·정확한 테스트 해시·입력/PDF 해시·명령/exit·WASM provenance](../assets/pr7382_20260926/stage44_caption3738_validation.json). 이번 기대값 교정은 충족이며 다른 실제 페이지 밀림/배치 결함과 미검증 경계 때문에 통합 PR·승인·merge는 보류한다.

![#3738 fresh WASM23쪽 직접 비교](../assets/pr7382_20260926/stage44_caption3738_wasm_review.png)
![#3738 fresh WASM23쪽 standalone overlay](../assets/pr7382_20260926/stage44_caption3738_wasm_overlay.png)


## 보정45 사전 분석 — #7203 저장 사다리와 가시 표 원점 구분

- 같은 원본 `56345_regulatory_impact_analysis.hwp`의11쪽에서 pi186/187 저장 vpos는24560/26992HU다. 앞 개체 상자1300+566+566HU는 뒤 앵커까지2432HU를 닫는다. 뒤 표 자체의 위여백141HU(1.88px)는 이 앵커 간격에 포함되지 않는다. 기존 검사는 두 Table bbox의 간격을 원시 앵커 간격32.42667px와 직접 같게 검사했다.
- `stored_empty_control_table_frame`은 뒤 표의 닫힌 프레임을 anchor+offset+outer-top으로 생산하고 `query_original_control_table_frame`→whole-fit/continuation 예약→확정 ParagraphFloatPlacement→`layout.rs`의 col+placement.table_top→`table_layout.rs`의 resolved origin으로 전달한다. 확정 원점 경로는 physical inset을 재적용하지 않는다. 뒤 표 위여백은 여기서 한 번 소비된다. 앞 표는1300HU 개체 프레임과 특수 셀 여백을 가진 별도 경로이므로 두 raw bbox가 동일 앵커 규약이라고 추정하지 않는다.
- 독립 PDF11쪽 뒤 표 가로 괘선 y436.961344px, 전435.5/후437.4px다. 전후 review를 직접 판독했으며 현재 뒤 표는 기준에 더 가깝다. 다음 검사는 저장 간격에 뒤 표 위여백을 반영해 기존0.2px 공차를 유지하고, 같은 파일의 PDF 괘선으로 절대 원점도 검사한다(이 검사군의 기존1.5px 공차). 제품 코드·기준값·래칫은 바꾸지 않는다. 기대 교정 전 실제 FAIL은 보정43의 전체 nextest 원시 로그에 보존돼 있다.


### 보정45 결과

- 기존 간격 기대32.42667px는 뒤 표 바깥 위여백141HU를 누락했다. 독립 저장 메타데이터로34.30667px를 기대하며0.2px 공차를 유지했다. PDF11쪽 가로 괘선436.961344px의 절대 원점 assertion을 추가해 상대 간격만 맞는 양쪽 오답도 거절한다. 현재437.4px는 PDF와0.439px 차이다. 제품 코드·baseline·래칫은 변경하지 않았다.
- 같은 runtime의 기존 검사는32.43px 대신34.31px를 관측하며 FAIL(exit100)이었다. 교정 뒤 이 검사군 전체5PASS/207SKIP(exit0,4.086s,threads8)다. 저장 앵커·오프셋 적용 제외·TAC 대조·뒤 표 분할/본문 소유 검사를 함께 통과했다. 준비·fmt·Native/WASM/workspace Clippy·workspace build·고정 base manifest·diff 모두exit0이다. [입력/PDF 해시·독립 괘선·소스 메타데이터·정확한 테스트 해시·명령/exit](../assets/pr7382_20260926/stage45_anchor7203_validation.json).
- 전후11쪽 review와 현재 standalone overlay를 직접 읽었다. 대상 뒤 표 원점은 PDF에 가까워졌으며 앞 제목/설명 텍스트의 잔여 위치 차이도 남았다. 전93.74164/후93.19785%의 자동 점수를 전체 정합 판정으로 사용하지 않았다. 이 테스트의 기대 교정은 충족이며 다른 실제 회귀·전체 시각 보류는 유지한다. 테스트 변경 head의 전체 nextest 통과 또는 새 검사의 base 실행은 주장하지 않는다.

![#7203 현재11쪽 독립 기준 비교](../assets/pr7382_20260926/stage45_anchor7203_review.png)
![#7203 현재11쪽 standalone overlay](../assets/pr7382_20260926/stage45_anchor7203_overlay.png)


## 보정46 결과 — 전수 시각 완료와 실패43건의 기대 적절성 판정 갱신

- 제품 runtime head는 `96c4e47771ecf7f46016bffa1e09c467b2878cbb`다. 뒤 보정43–45는 문서/증적/테스트만 바꿔 제품 소스·Cargo·scripts는 동일하다. Native와 fresh WASM 각각 HWP/HWPX1–215쪽의 compare/review/standalone overlay를 누락·중복 없이 생성했다. 합계860쪽 자동 비교이며860쪽 전부를 사람이 직접 읽은 것으로 보고하지 않는다. 입력/PDF215쪽 대응·고정 CLI/pkg 해시·명령·최종exit·페이지 coverage는 [전수 시각 결과](../assets/pr7382_20260926/stage46_full_visual_summary.json)에 보존했다.
- Native HWP/HWPX 및 fresh WASM HWP/HWPX는 **각각20쪽이90% 미만**이며 네 gate 모두 `re_review_required`다. 모든 캡처 작업은 완료됐지만 실제 Visual Sweep exit1은 시각 보류다. orchestration의exit0을 시각 통과로 바꾸지 않는다. HWP/HWPX206·207쪽의 표 높이/캡션/뒤 본문 위치와 HWP169쪽 그림65의 배치 차이도 review를 직접 읽어 보류를 확인했다. 폰트 예외는 쓰지 않았다.
- 추가 대조군7개는 Native18쪽/fresh WASM18쪽이다. #6950/#6312와 독립 재저장 #2097/#2105는 양 backend gate를 통과했다. 원본 수동 #2097/#2105와 #6782의76/78쪽은 보류다. fresh WASM #6950의3쪽과 재저장 두 꼬리2쪽도 직접 판독했다. 재저장 대조군100% 점수를 원본 입력 통과로 보고하지 않는다.
- [43개 실패별 기대 적절성/독립 PDF/직접 비교/미검증 매핑](../assets/pr7382_20260926/stage46_regression_test_appropriateness.json)을 갱신했다. 두 기대 오류는 보정44/45에서 개별 교정 완료다. #5941의304쪽 및 #6101의12쪽은 한컴 정답이 아닌 기존 잠정 핀이며 현재값303/13으로 자동 갱신하지 않는다. #1133의 상대 간격은 독립 괘선/원점 검사로 보강해야 하고 #6797의 수동 변형은 원본 PDF로 정답을 입증하지 않는다. 래칫43FAIL은 중복 입력·같은 원인의 여러 검사·진단 proxy를 포함하므로43개의 독립 가시 결함이라고 단정하지 않는다.
- 실제 회귀의 유효한 기존 기대도 보존한다. #6535/#6102/field1948의1쪽 하단 내용, #7336의7쪽/6쪽 동의서, #2243의3쪽, PrEP140쪽/92·93캡션/105연구비 표, #1733의242쪽 마지막 부속서는 독립 PDF에 대응한다. #1733의 전242쪽은 정본 마지막 부속서이나 현재242쪽은 앞 부속서 꼬리이며 실제243쪽으로 이월됐다. 기존242를243으로 바꾸지 않는다.
- raw 진단은 실제 글리프·ancestor clip과 대조한다. hwpctl70쪽의 설명 글자 하단 가림은96.19% 점수여도 보류다. #2097의75544 입력22쪽 표가 본문끝을3.253px 넘으며, neartop-reset39쪽의 제목/표 헤더와 CBTA121쪽의 문단/표 헤더 겹침도 전후 직접 확인했다. 반대로 issue1937의44쪽 각주41 겹침은 base에서도 보이므로132→133만으로 새 가시 결함이라 확정하지 않는다. 기준PDF를 만들지 못한 형식1.3 입력과 상세 페인트 경계 부족은 미검증으로 유지한다.
- 전체 검사 결과는10,327PASS/43FAIL/50SKIP인 runtime96 실행 기록이며 두 테스트 교정 뒤 전체 실패 수를 산술로 줄여 최종 통과로 보고하지 않는다. 실제 배치 보정 후 새 runtime에서 영향 경계/시각과 전체 필수 검증을 다시 해야 한다. 다음 개별 보정은 #6535 하단 래퍼의 배경 표/흐름 높이와 셀 정렬이다. 통합 PR·승인·merge는 계속 보류한다.

![전체 비교: HWPX206쪽 표 높이와 후속 배치 보류](../assets/pr7382_20260926/stage46_hwpx_review_206.png)
![전체 비교: HWPX207쪽 배치 보류](../assets/pr7382_20260926/stage46_hwpx_review_207.png)
![기존242쪽 기대가 유효한 #1733 실제 꼬리 이월](../assets/pr7382_20260926/stage46_footnote1733_review_242.png)

로컬 전수 판독 목록은 `output/pr-review/planet6897-7382-20260926/full-96c4e4777/index.html`, 기존 회귀 전후91개 대응 페이지 목록은 같은 폴더의 `regression-index.html`이다. 로그·전체 캡처·임시 파일은 output에 보존하고 커밋하지 않는다. PR merge 후 소유한 output만 후속 절차에 따라 정리한다.


## 보정47 사전 분석 — #6535 하단 래퍼의 배경 표와 흐름 점유

- 동일 입력3개의 기준PDF는1쪽이고 하단 결재선·주소·전화가 같은 쪽에 있다. 저슬랙 입력의 하단 바깥 표는26353HU(351.373px), 가운데 정렬 셀의 같은 빈 host 문단에 자리차지 내부 표22672HU와 글 뒤로 표10935HU를 둔다. 현재 측정464.093px는 호스트 줄16+자리차지302.293+배경145.8의 합과 같다. 본문 마진50px는 현재 실패 원인이 아니며 보정27의 외곽 셀 정렬 보존이 드러낸 측정/배치 계약 차이를 보정한다.
- 측정의 `unabsorbed_nested_tables_height`/저장 줄 그룹 높이→MeasuredCell/MeasuredTable→whole-fit의 하단 개체 fit과 예약→`resolve_row_heights`/`calc_nested_controls_bottom_height`→수직 정렬→중첩 표 실제 원점을 대조한다. 기존 배치의 overlay 판정은 HWPX BehindText+Para+Column만 인정해 이 입력의 Para 가로 앵커 배경도 흐름 표로 취급한다. 단순히 외곽 셀을 다시 제거하거나 배경을 숨기지 않는다.
- 배경 개체가 흐름을 밀지 않는 속성과 시각 정렬에 필요한 공간을 분리한다. 공통 역할 판정을 측정·배치에 연결하고 시각 끝점·호스트 줄·자리차지 표의 양수 문단 오프셋이 실제 정렬에서도 한 번 소비되는지 직접 확인한다. HWP5 legacy/TAC/자리차지 대조군을 유지한다. 먼저3개 원본의1쪽·하단 전화 좌표를 정식 검사로 고정하며 #6950 가운데 정렬과 #5593 전면 개체는 정상 대조군이다.
- 전화 글줄의 독립 PDF y는 저슬랙1030.514px·페이지앵커1029.386px·초과근무1029.733px다. 기존1쪽 FAIL에 더해 실제 후속 내용 위치를 검사한다. 한컴 PDF의 출처와 입력 해시는 보정43/46의 매핑을 사용하며 공차를 자동 확장하지 않는다. 이 분석은 구현 완료 주장이 아니다.

### 보정47 진단 중 검사 자체의 오류와 범위 구분

처음 추가한 전화 위치 검사는 PDF의 글자 잉크 bbox 상단과 RenderTree의 글줄 상단을 ±3px로 대조했습니다. 서로 다른 메트릭을 같은 좌표로 취급한 검사 오류입니다. PDF span의 `origin.y`를 96dpi로 변환한 기준선(1041.920/1040.783/1044.000)이 실제 전화 글줄 상자에 속하는지로 의미를 바로잡습니다. 이는 공차 확장이나 현재 출력값 재인용이 아니며, 원래 2쪽 이월 상태에서는 세 원본 모두 페이지 소유 assertion으로 실패합니다. 페이지·내용 소유 교정과 세로 위치의 완전한 일치는 구분합니다. 새 Native 직접 판독은 세 원본 모두 결재선·주소·전화를 1쪽에서 확인했으나 잔여 위치 차이와 page6535의89.96% 게이트 미달은 계속 보류합니다.

### 보정47 결과 — 하단 내용 소유 회복, 전체 시각 승인 보류

- 저장 HWPX 배경 표의 역할을 공통 판정으로 묶어 측정의 미흡수 흐름 합·저장 줄 그룹·배치의 전진/원점 선택이 함께 소비합니다. 흐름 표의 나란함 판단에서 배경을 제외하고 배경 시각 높이는 같은 앵커의 최댓값으로 보존합니다. HWP5 legacy/TAC/자리차지 경로는 유지하며 배경 출력 자체를 제거하지 않습니다. 양수 문단 오프셋의 전체 숫자 결과를 모든 측정 경로가 공유한다는 주장이나 잔여 정렬 완료 주장은 하지 않습니다.
- 새 정식 검사는 `tests/cases/maintainer_hwpx_overlay_table_extent.rs`에1쪽·전화1회·독립 PDF 기준선의 실제 글줄 소유를 추가했습니다. 수정 전에는 동일한 페이지 소유 assertion으로3FAIL, 현재3PASS/204SKIP입니다. 초기에 추가한 잉크/글줄 상단 대조 오류의 로그도 output에 보존했습니다. 기존 대조군42PASS/1288SKIP, 중첩 표 그룹9PASS/419SKIP입니다.
- fmt/Native Clippy/WASM Clippy/workspace build/전체 target Clippy/고정 base manifest·unit 정책/diff/fresh WASM 모두exit0입니다. WASM은 root wrapper의 `--no-opt` 로컬 대체 빌드이며 root pkg·Studio·보존 package 해시가 동일합니다. [명령·소스/입력/기준 해시·결과](../assets/pr7382_20260926/stage47_validation.json). 새 주석은 한국어입니다.
- Native/fresh WASM의 세 원본 하단1쪽 review와 Native 세 standalone overlay/WASM 페이지앵커 overlay를 직접 판독했습니다. 두 backend 모두 저슬랙96.73936%, 페이지앵커89.95899%, 초과근무91.75100%입니다. 세 원본의 결재선·주소·전화가1쪽에서 보이지만 페이지앵커 gate와 하단 위치 차이는 계속 보류합니다. 정상 가운데 정렬 #6950의3쪽99.78%, #6312의4쪽99.89%에서도 표/뒤 내용을 직접 확인했습니다.
- 기존 통과 검사도 직접 PDF로 재검토했습니다. #7066의 #2470/#2083은 두 표의 줄 소속·세로 정렬을 유지(95.27/92.19%)합니다. #6787은 카드 두 개가 같은 줄에 있고 내용도 보이지만83.51%이며 카드/투표버튼 원점 차이가 남습니다. 그룹·내용 소유 검사 통과를 전체 시각 정합으로 바꾸지 않으며 기존 공차를 바꾸지 않습니다.
- 원 검토 문서 HWP/HWPX의 Native 각215쪽 트리는 runtime96과 모두 동일합니다. 새 코드의 전체 nextest·전수 래스터 통과를 주장하지 않습니다. 초기43건 실패에서 단순히 통과 건수를 빼서 현재 잔여 건수를 추정하지 않습니다. 한글 형식1.3은 동일 바이트를 영문 파일명으로 재변환해도 실패하여 미검증입니다. [재시도 출처](../assets/pr7382_20260926/stage47_manual13_reference_retry.json).
- 기존 회귀 기대 재검토 지침에 PDF 잉크/글줄/기준선 메트릭 구분을 추가했고 검증 명령·Visual Sweep 보고 예시에 남아 있던 임시 `/tmp` 출력 경로도 `output/pr-review`로 교정했습니다. **판정: 페이지·내용 소유 보정은 충족, 정확한 하단 배치와 전체 통합 PR 준비는 보류입니다.** 로그·전체 캡처·파생 파일은 커밋하지 않습니다.

![하단 내용1쪽 회복](../assets/pr7382_20260926/stage47_native_low6535_review_001.png)
![페이지 앵커의 잔여 차이](../assets/pr7382_20260926/stage47_native_page6535_overlay_001.png)
![fresh WASM 페이지 앵커 보류](../assets/pr7382_20260926/stage47_wasm_page6535_review_001.png)
![기존 통과 그룹 검사의 시각 한계](../assets/pr7382_20260926/stage47_group6787_review_001.png)

## 보정48 사전 분석 — 보정47 커밋의 전체 회귀 재실행

제품/검사 head는 `c4ce9ff96436d94468557f09794ee0fa84b35ce6`입니다. 소스·정식 검사 blob은 보정47 검증 해시와 모두 일치합니다. 초기 runtime96의43건에서 기대 교정2건·하단 코드 보정의 통과 건수를 단순 차감하지 않고 새 head 전체 `cargo nextest ... --tests --test-threads 8 --no-fail-fast`로 실제 잔여 실패와 새 회귀를 다시 수집합니다. 검증 동안 소스/정식 검사/파생 suite는 고정하며 실패 후 실제 출력 결함·기대 오류·미검증을 개별 입력 PDF/Visual Sweep 근거로 다시 판단합니다. 이번 단계는 새 코드 보정의 완료 또는 PR 승인 주장이 아닙니다.

### 보정48 검토 중 발견한 검사 생성 경로의 우려

#6797 수동 변형 helper는 로드한 core의 `document_mut()`에서 LineSeg/표 오프셋을 직접 바꾸고 곧바로 기존 page 계획을 렌더합니다. `document_mut`는 셀 서식 vpos flush만 수행하며, `set_document`의 파생 상태 재구성을 호출하지 않습니다(`commands/document.rs:2047/2058`). 따라서 원문 캐시를 소비하면서 변형 좌표를 검사하는지 실행으로 확인해야 합니다. 코드 검토상 우려이며 아직 파생 상태 재구성 전후 비교를 실행하지 않았습니다. 현재 전체 nextest가 실행 중이므로 정식 검사·제품 코드는 바꾸지 않습니다. 원문 표 순서 FAIL은 별개로 한컴 괘선 독립 좌표로 유효함을 확인했습니다.


### 보정48 결과 — 전체 회귀 및 기존 기대 적절성 재판정

- 고정 head `c4ce9ff96`, base `eb9142dd7`에서 전체 nextest10,373건 실행을 완료했습니다. **10,335PASS/38FAIL/50SKIP**,8slow,검사666.862s/컴파일 포함913.944s,exit100입니다. 실행 중 소스/정식 검사/파생 suite를 바꾸지 않았습니다. [정확한 명령·head·실패 목록·이전/현재 대응](../assets/pr7382_20260926/stage48_full_nextest.json).
- 이전 실패에서 실제 PASS로 바뀐6개는 #3738/#7203 기대 교정2개, #6535저슬랙/#6535페이지앵커/#6102초과근무 페이지 소유3개, #6535계열이 포함된 oracle page count partition11의1개입니다. #1133의 상대 간격 검사는 여전히 실패하며 독립 절대 괘선 검토를 유지합니다. 새 하단 정식 검사3개도 전체 실행에서 PASS입니다.
- [#6797 재판정](../assets/pr7382_20260926/stage48_float6797_reassessment.json): 원본 표 순서 기대는 독립 PDF 괘선294.557/298.393px로 유효합니다. 현재294.9/292.0px는 실제 순서 역전입니다. 수동 IR 변형의 캐시/기대574.8px는 미검증이며 원본 PDF로 대신 입증하지 않습니다.
- [#5585 재판정](../assets/pr7382_20260926/stage48_indicators5585_reassessment.json): 현재85쪽 꼬리는 PDF86쪽 평가인증률 표에 대응합니다. 내용 이동 시작과 전체 보존 원인은 미검증이며 기대86을85로 바꾸지 않습니다. 기준의 MediaBox만 복구한 provenance는 #7269 기록과 대조했습니다.
- [#1853 재판정](../assets/pr7382_20260926/stage48_caption1853_reassessment.json): 넘친 글줄은 빈 줄이 아닌 유사입법례입니다. PDF14쪽 기준선1010.240px와 현재 글줄1009.76..1028.427px 및 이어받기 시작 내용을 직접 대조해 실제 하단 배치 차이를 확인했습니다. 반복 문구는 본문pi100과 표pi105의 셀 내부pi1에 각각 속하므로 같은 유닛 중복으로 판정하지 않습니다.44쪽 경계는 미검증입니다.
- 신규 FAIL #1658은 max TextLine bottom1108.4px가 body1103.6px를 넘습니다. 그 최하단 줄의 TextRun은 빈 문자열이며, 보이는 전화 줄은1075.1..1088.4px입니다. 전후 Native Visual Sweep을 실행하고 현재1쪽 review를 직접 읽었습니다(92.40%,gate passed). 보이는 하단 내용과 빈 줄의 물리 공간을 구분하지 않은 검사 의미를 재검토하되 빈 줄의 점유 자체를 무효로 단정하지 않습니다. 제품 코드를 바꾸거나 기존0.5px 공차를 넓히지 않았습니다.
- 기존 회귀 검토 지침에 수동 IR 변경의 파생 상태 전달 확인과 문단/표 셀 소유 대조를 추가했습니다. **38FAIL은38개 독립 가시 결함을 뜻하지 않습니다.** 실제 회귀는 코드에서, 기대 오류는 독립 근거로 개별 보정하며 미검증은 유지합니다. PR·승인·merge는 보류입니다. 로그·전체 산출물은 `output/pr-review/planet6897-7382-20260926/stage48-full-c4ce9ff96/`에 보존하고 커밋하지 않습니다.

![기존 검사 의미를 재검토하는 #1658 현재1쪽](../assets/pr7382_20260926/stage48_gwanak1658_review_001.png)


## 보정49 사전 분석 — #6797 수동 IR 변형의 파생 상태 재구성

전체 실행이 종료된 뒤 동일 release-test 라이브러리로 output 진단을 실행했습니다. 기존 helper처럼 mutable IR만 바꾸면 세 변형 모두 원문 후속표 y291.98667px를 출력합니다. 같은 IR을 `set_document`로 재설정하면 offset30000HU 변형은574.77333px, 사다리 누락/합성은 각각174.77333px로 달라집니다. 캐시 미갱신 우려가 실제로 확인됐으며 기존 offset 기대574.8±0.5는 재구성한 해당 변형과 일치합니다. 원본 표 순서 FAIL과 독립 PDF 정합은 별개입니다. 제품 코드/기대값/공차를 바꾸지 않고 helper의 변형 후 파생 상태 재구성만 수정해 정식 변형 검사군을 다시 실행합니다.


### 보정49 결과 — 기존 검사 helper의 원문 캐시 재사용 제거

- `maintainer_float_variant`가 IR 변경 후 `set_document`로 파생 상태를 재구성하도록 수정했습니다. 제품 코드·574.8px 기대값·0.5px 공차는 그대로입니다. 이전 전체 실행의 이미 회피된 offset 검사 FAIL은 원문 캐시의291.98667px를 잘못 소비한 결과였습니다. 재구성 후574.77333px로 해당 기존 검사 PASS입니다. 다른 두 변형은 이전에도 PASS였으나 원문 좌표를 함께 소비했으므로 실제 변형 증거가 아니었습니다. 현재 사다리 누락/합성174.77333px와 범위 밖 좌표를 실제 변형 상태에서 검사해 총3PASS를 확인했습니다.
- 정식 nextest(threads8)는3PASS/10420SKIP(exit0,0.148s), release-test 빌드3m05s입니다. 준비·fmt·Native/WASM/workspace Clippy·workspace build·고정 base manifest·diff를 순차 실행해 모두exit0입니다. 새 설명 주석은 한국어입니다. [진단 출력·정확한 검사 해시·명령·exit](../assets/pr7382_20260926/stage49_float6797_cache_validation.json). 로그와 진단 실행 파일/소스는 output에만 둡니다.
- 원본 #6797의 표 순서 역전은 제품 코드 불변으로 그대로 남습니다. 수동 변형의 PASS를 원본 PDF 정합으로 보고하지 않습니다. helper 수정 후 전체 nextest를 다시 실행한 것으로 보고하지 않으며 전체38FAIL은 수정 전 고정c4ce9ff96 실행 결과입니다. [기존43개 각각의 현재 전체 결과/기대 적절성 매핑](../assets/pr7382_20260926/stage48_regression_test_appropriateness.json)을 함께 갱신했습니다.
- **판정: 검사 생성 경로 보정은 충족, 원본 배치 회귀·전체 통합 PR 준비는 보류입니다.** 다음 제품 보정은 원본 #6797의 저장 앵커→배제 밴드→확정 표 원점의 소비 경로를 추적해 독립 괘선으로 해결합니다. 기존 기대 자체와 실제 제품 결함을 구분하는 검토를 계속 적용합니다.


## 보정50 사전 분석 — 빈 호스트 전체 저장 프레임의 좌표계

- 원본 #6797의7쪽 표71은 빈 호스트, vpos16306HU, 높이12430HU, 사방 여백141HU이며 후속 vpos29018HU가 높이+양쪽 여백12712HU를 정확히 닫습니다. 기존 `stored_float_anchor`의 물리 쪽 기준 원점은 body79.36+(16306+141)/75=298.65333px이며 독립 PDF 괘선298.39331px와 대응합니다. 현재291.98667px는 같은 값에서 첫 문단vpos500HU(6.66667px)를 뺀 값입니다.
- `stored_empty_control_table_frame` 생산→`query_original_control_table_frame`→whole-fit/예약→`paragraph_float_placements`→`layout_column_table_item`/`layout_table_control_block`→`resolved_table_origin`을 추적했습니다. 빈 호스트 전체 프레임은 기존 `stored_single_topbottom_top_px`와 달리 page base를 빼서 저장되고, 최종 resolved 원점은 #6797밴드가 옮긴 y_offset296.8px보다 우선합니다. 실행 진단도 cursor296.8/실제Table291.9867을 확인했습니다.
- 내부 글줄 호스트의 상대 프레임은 유지하고, 빈 호스트의 전체 물리 개체 프레임은 쪽 기준 저장 좌표를 사용하도록 맞춥니다. 단 영역 변환은 한 번만 수행합니다. 원본 순서 assertion과 기존공차는 유지하고 독립 PDF 표71상단298.39331±0.5px도 검사합니다. offset/합성/범위밖 저장 좌표 및 글자가 있는 호스트를 대조하며 정상 #7203/3738/6950/6312도 확인합니다. 첫 단계는 원본5검사와 Native7쪽 직접 판독이며 전체 gate가 완료되기 전 제출하지 않습니다.

### 사용자 지정 #1133 2쪽의 추가 보류

사용자가 지정한 `stage50-float6797/visual/native-unresolved/nested1133-hwp/review/review_002.png`와 같은 입력의 fresh WASM review를 직접 대조했습니다. 자동 일치율94.52%는 아래 실제 배치 차이를 해소하지 않습니다. 글꼴 예외로 처리하지 않습니다.

- 안내문 회색 표는 Native SVG x215.180px, 독립 한컴 PDF 채움 상자 x190.457px로 **24.723px 오른쪽**입니다. 제목은221.980/197.333px, 이어지는 `합격 또는 채용이 취소됨`은258.940/234.273px입니다. 전체 중첩 표의 가로 원점이 다르므로 내어쓰기만 바꾸지 않습니다. 현재 `compute_table_x_position`의 depth>0 좁은 비TAC 표 가운데 배치가 저장 `horzRelTo=PARA, LEFT, offset=0`보다 우선합니다. 기존 #3308 정상 직인 표는 `horzRelTo=COLUMN, offset=6226`이므로 두 입력의 앵커 기준과 독립 출력 계약을 대조합니다. 아직 수정하지 않았습니다.
- 자동 쪽 번호의 실제 SVG 기준선은1090.244px, 독립 PDF 기준선은1080.419px로 **9.825px 아래**입니다. `build_page_number`는 줄 상자를y-font_size에, TextRun 상자를y에 만들고 run.baseline에도font_size를 싣습니다. 최종 기준선 소비와 글꼴별 추가 보정을 함께 검토합니다. 상자y만 검사하는 기존 #7336 검사를 최종 glyph 기준선의 증거로 대신하지 않습니다. 아직 수정하지 않았습니다.
- 바깥 분할 표의 첫/이어받기 조각도 HWP에서 각각439.1/75.6px, 독립 PDF 괘선440.477/77.356px로 위여백141HU와 대응하는 차이가 남습니다. HWPX와 같게 만드는 상대 검사만으로 원본 PDF와의 일치를 입증하지 않습니다. 각 조각의 예약·소유·최종 배치를 추적한 뒤 별도 보정합니다.

좌표·입력/PDF 해시·실제 SVG 기준선은 `output/pr-review/planet6897-7382-20260926/stage50-float6797/nested1133-user-residual.json`에 보존했습니다. 실행 중인 전체 nextest의 소스·정식 검사·파생 suite는 고정합니다. 현재 관측은 해결 완료가 아닌 **추가 보류의 실행 증거**입니다.


### 보정50 결과 — #6797 원본 표 경계 회복, 전체 재검사와 추가 보류

- 빈 호스트의 닫힌 물리 프레임에서 page base를 다시 빼지 않도록 수정했습니다. 글자가 있는 호스트의 상대 프레임과 단 좌표 변환은 유지했습니다. 원본 Table71 상단은291.98667→298.65333px이며 독립 한컴 PDF298.39331px와0.26002px 차이입니다. 앞 표 하단을 넘지 못하던 순서 assertion을 유지하고 독립 절대 괘선 assertion을 추가했습니다. 수정 전 전체 정식 FAIL과 수정 후 집중5PASS/전체5PASS를 연결했습니다.
- fmt·Native/WASM/workspace/all-target Clippy·workspace build·고정base manifest·fresh WASM(no-opt,Mac 로컬 대체) 모두exit0입니다. WASM 루트pkg/Studio/frozen 패키지 해시를 대조했습니다. 전체 nextest는threads8/no-fail-fast로 **10,337PASS/36FAIL/50SKIP**,9slow, 검사777.990s/컴파일 포함1254.495s,exit100입니다. 소스·정식 검사·파생 suite는 실행 동안 고정했습니다. 이전c4 전체 실패에서 offset 수동 검사와 원본 순서 검사2개가 PASS이며 새 실패는 없습니다. offset 검사 생성 경로의 교정은 보정49에 속합니다. [전체 실행 및 실패 대응](../assets/pr7382_20260926/stage50_full_nextest.json), [기존43개 검사 재판정 갱신](../assets/pr7382_20260926/stage50_regression_test_appropriateness.json).
- Native/fresh WASM7쪽 review·standalone overlay를 직접 읽었습니다. 일치율89.15174→93.53093%이고 두 표 외곽·후속 제목/본문의 위치가 개선됐습니다. PDF의 해당 차트 셀은 비어 있어 차트 payload 충실도의 정답지로 사용하지 않습니다. 하단 쪽 번호의 실제 차이도 남습니다. #6950/#6312/#3738/#7203의 새 Native/fresh WASM 비교를 실행하고 대표 review를 직접 읽었습니다. #7203의 앞 설명 클리핑 차이는 통과 점수와 별개로 남습니다.
- 주 문서HWP/HWPX215쪽씩 **430개 Native tree가 보정47과 동일**합니다. 이 비교는 새430쪽 raster 판독이나860쪽 시각 승인 증거가 아닙니다. 사용자 지정 #1133의2쪽 Native/fresh WASM을 직접 읽고 표 원점/실제 쪽번호 기준선 차이를 [독립 좌표·해시 증거](../assets/pr7382_20260926/stage50_nested1133_user_residual.json)로 보존했습니다. 자동gate94.52% 통과를 실제 배치 해결로 보고하지 않습니다.
- **판정: #6797 원본 표 경계 보정은 충족, 전체36FAIL·#1133 및 기존 시각 보류는 유지합니다.** 이번 제품/검사·결과·증적을 커밋한 뒤 #1133 안내문 가로 앵커를 다음 개별 단계로 보정합니다. `.log`와 실행 파일·파생 suite·output은 커밋하지 않습니다.

![#6797 Native7쪽 경계 보정](../assets/pr7382_20260926/stage50_native_float6797_review_007.png)
![#6797 fresh WASM7쪽 경계 보정](../assets/pr7382_20260926/stage50_wasm_float6797_overlay_007.png)
![사용자 지정 #1133 2쪽의 미해결 배치](../assets/pr7382_20260926/stage50_nested1133_user_review_002.png)

## 보정51 사전 분석 — 안내문 중첩 표의 문단 기준 가로 앵커

- 기준은 보정50 커밋 `5a6bef2bfd2173fa8f45fbb4a08527a24da7d786`입니다. #1133의 두 저장 형식은 중첩 표에 `horzRelTo=PARA, horzAlign=LEFT, horzOffset=0`, 자리차지/비TAC를 저장합니다. 독립 한컴 PDF2쪽의 회색 표 x190.457px/폭469.508px와 이어지는 글줄 x234.273px를 기대값으로 사용합니다. 현재 x215.180/258.940px는 표 전체가 가운데 배치되며 생긴 차이입니다. 글자 폭·내어쓰기·표 선언 폭은 바꾸지 않습니다.
- 중첩 표 선언 폭→셀 안쪽 영역→`layout_table`→`compute_table_x_position(depth>0)`→Table/Cell/TextRun 절대x를 추적했습니다. 측정은 선언 폭을 유지하며 가로 원점은 공통 배치 helper에서 결정됩니다. 같은 helper 이후 독립 가로 덮어쓰기가 있는 나란한 float 무리는 `inline_x_override` 경로이므로 기존 원점을 보존합니다. #3308 직인 표는 COLUMN 기준이고 독립 PDF598.7px가 가운데 배치를 뒷받침하므로 그대로 대조합니다. #5787 어울림/SQUARE 표도 이번 자리차지/PARA 계약 밖입니다.
- 정식 원본HWP/HWPX 검사에서 수정 전FAIL을 먼저 확인합니다. 수정 뒤 표 원점·선언 폭·이어지는 글줄의 실제x를 독립 PDF로 확인하고 COLUMN 직인/SQUARE 오프셋/나란한 float 대조군을 실행합니다. 첫·이어받기 세로 위여백과 자동 쪽번호의9.825px 차이는 이번 가로 보정으로 해결했다고 보고하지 않습니다. 새 영향 Native/fresh WASM review·overlay와 필수 lint/policy를 완료한 뒤 결과·커밋을 남깁니다.

### 보정51 대조군 반례와 경로 재검토

- 첫 후보에서 #3308/#5787의 Native tree·SVG·raster는 수정 전과 동일합니다. #6787은 사진 칸 x199.6→196.0px로 바뀌었고 독립 PDF199.573px에 비해 새 차이가 생겼습니다. 일치율83.51487→83.16366%를 기존 차이로 덮지 않습니다. 버튼은300.4→312.7px로 독립 PDF312.472px에 가까워졌으므로 문단 기준 선언 앵커 자체를 제거하지 않습니다.
- 부모 카드의 나란한 무리 경로가 `inner_area.x + horzOffset`을 lane 원점으로 만들고, 배치에는 `inline_x_override=inner_area.x`를 전달합니다. 이 경로는 저장 바깥여백283HU를 빠뜨립니다. 카드 최종x119.2/415.1px와 독립 PDF122.813/418.655px를 확인했습니다. 사진만 선언 앵커로 되돌리면 이 잘못된 부모 원점을 따릅니다. 무리의 가로 원점도 `compute_table_x_position`의 같은 문단 앵커 결과를 소비하고, 확정 원점과 lane 끝을 실제 배치에 넘겨 오프셋·여백을 재가산하지 않도록 보정합니다. 세로 예약/같은 줄 소유와 선언 폭은 유지합니다.
- 기존 #6787 same-line 검사는 두 카드의 y 차와 x 차만 확인해 절대 가로 원점의 누락을 검출하지 못합니다. 같은 원본의 독립 PDF 카드·사진 칸·버튼 왼쪽 괘선을 새 정식 검사로 확인합니다. 첫 후보에서 FAIL을 재현하고 재보정 후 PASS 및 기존6개 무리 대조군을 연결합니다. 새 코드 뒤 이전 lint/WASM 결과를 최종 결과로 재사용하지 않습니다.

### 보정51 결과 — 문단 가로 앵커와 나란한 무리의 원점 공유

- #1133의 비TAC/자리차지/PARA 표는 선언한 가로 정렬·signed offset·바깥여백을 공통 가로 helper에서 해석합니다. 회색 표 x215.180→190.65333px로 독립 PDF190.45734px와0.196px 차이입니다. 선언 폭469.69333px는 유지됐고 독립 PDF469.508px 및 이어지는 글줄의 독립x도 정식 검사로 확인했습니다. 원본HWP/HWPX 새2검사는 수정 전2FAIL/수정 후2PASS입니다. 기존 COLUMN 직인/SQUARE 오프셋 경로는 유지했습니다.
- 첫 후보의 #6787 사진 칸 이동을 보존하고 원인을 다시 추적했습니다. 무리 lane 원점과 최종 배치는 같은 `compute_table_x_position` 결과를 소비하며, `resolved_table_origin`으로 최종x/y를 전달해 가로 오프셋·바깥여백을 다시 더하지 않습니다. 독립 PDF 카드122.813/418.655px·사진199.573/497.492px·버튼312.472px를 새 정식 검사로 확인했습니다. 첫 후보에서 카드119.18px로 FAIL, 재보정 후PASS입니다. 처음 잘못 선택된 suite의0tests/exit4는 결함 검출로 계산하지 않습니다.
- 최종 집중 **14PASS/1055SKIP**, 검사1.048s,exit0입니다. 신규3개와 #6787 기존6개, #7066 정렬3개, #3308/#5787 각1개를 포함합니다. fmt·세 Clippy·workspace build·고정base manifest·diff check·fresh WASM(no-opt,Mac 로컬 대체) 모두exit0입니다. 새 검사 포맷 후 source weight가 변해 최초 manifest 검사가 실패한 결과를 보존했고, 재prepare 후 전체 lint/policy를 통과했습니다. generator·공차·baseline은 바꾸지 않았습니다. fresh WASM의 루트pkg/Studio/고정 패키지 해시를 대조했습니다.
- 원본HWP/HWPX 각3쪽과 대조군3쪽을 **Native9쪽/fresh WASM9쪽** 실행했습니다. 두 형식의2쪽 review·standalone overlay, #6787의1쪽 review·overlay를 두 backend에서 직접 판독했습니다. #1133 HWP2쪽99.22385%/HWPX2쪽98.84583%, #6787 1쪽91.02058%이며 backend 간 값이 같습니다. #3308 p7=77.10310%/#5787 p1=59.04745%는 수정 전후 tree·SVG·raster가 동일한 기존 보류입니다. 기존 정식 검사의 통과를 전체 페이지 시각 승인으로 승격하지 않습니다. #6787의 작은 세로 위치·글자 형태 차이도 남습니다.
- 주 문서HWP/HWPX215쪽씩 **430개 Native tree가 보정50과 동일**합니다. 이는 새860쪽 전체 시각 승인이나 현재 head 전체 nextest의 증거가 아닙니다. Center/Right의 모든 조합과 무리 구성원별 바깥여백 차이 경계는 미검증입니다. 자동gate 통과와 별개로 #1133의 실제 쪽번호 세로 차이 및 HWP 분할 조각의 위여백을 다음 단계로 보정합니다.
- **판정: 검증한 가로 원점·선언 폭·후속 글줄·대조군 범위는 충족, PR 머지 보류 유지.** 결과·증적을 이 단계 커밋에 포함하고 `.log`·실행 파일·WASM package·파생 suite·output은 커밋하지 않습니다.

![#1133 Native2쪽 가로 보정](../assets/pr7382_20260926/stage51_native_nested1133-hwp_review_002.png)
![#1133 fresh WASM2쪽 가로 보정](../assets/pr7382_20260926/stage51_wasm_nested1133-hwp_overlay_002.png)
![#6787 원점 누락 재보정](../assets/pr7382_20260926/stage51_wasm_group6787_review_001.png)

## 보정52 사전 분석 — 자동 쪽번호의 아래쪽 여백 기준선

- 시작 head는 보정51 `0ab575a57`입니다. 원본 #1133 2쪽의 실제 쪽번호 기준선은 rhwp1090.244px/한컴1080.419px입니다. 한컴 출력 대조군은 꼬리말 여백2834→4252HU에서 기준선 이동0px, 아래쪽 여백2834→4252HU에서−18.859375px입니다. ZIP 한 속성만 바꾼 입력과 새 `engine=2020` PDF를 samples/issue1133·pdf/issue1133에 보존하며 원본 생성 방식과 구분합니다.
- `PageLayoutInfo`의 footer_area(본문 끝/높이=아래쪽 여백)→`footer_page_number_y`(종이 높이와 footer_area 끝에서 꼬리말 여백 복구)→`build_page_number`의 글꼴별 +font_size/2→TextLine(y-font_size)/TextRun(y,baseline=font_size)→실제 paint(y+baseline)를 추적했습니다. 지금은 꼬리말 여백 중앙을 기준으로 삼고 같은 y를 기준선과 글상자 상단으로 다르게 소비합니다. 실제 쪽번호 밴드의 아래끝은 `footer_area.y + margin_footer`, 즉 종이 높이−아래쪽 여백입니다. 기존 font_size/3의 기준선 여유를 그 아래끝에 적용하고 footer TextLine/Run이 같은 최종 기준선을 소비하게 합니다. 글꼴 이름별 임시 이동은 제거하고 문서의 원래 쪽번호 스타일은 유지합니다. 머리말의 실제 glyph 배치는 이번 변경 범위 밖으로 유지합니다.
- 원본HWP/HWPX 세 쪽과 독립 여백 대조군의 최종 기준선에서 수정 전FAIL/수정 후PASS를 확인합니다. #7336의 기존 글꼴/상자 공차를 그대로 유지하고 실제 첫 쪽 기준선도 독립 PDF로 대조합니다. 다른 글꼴/쪽테두리/각주 경로는 Native/fresh WASM 원본 캡처로 확인하며 큰 차이는 먼저 보정합니다. 분할 표 위여백과 기존 전체 보류는 이 쪽번호 단계로 해결했다고 보고하지 않습니다.

### 보정52 첫 후보 결과 — 꼬리말 밴드와 실제 쪽번호 기준선 공유

- 꼬리말 밴드의 아래끝을 `footer_area.y + margin_footer`로 구하고 `font_size/3`의 기준선 여유를 적용합니다. footer의 TextLine과 TextRun은 같은 최종 기준선을 소비합니다. HCRDotum 이름별 반 글자 크기 이동과 각주 유무별 임시 분기는 제거했습니다. 문서의 쪽번호 글꼴·크기·색·가로 정렬·감추기·테두리 판단은 유지하며 머리말 glyph 배치는 이번 범위 밖입니다. 보정51 무리 가로 원점에 관한 오래된 주석도 실제 resolved 원점 경로에 맞췄습니다.
- 원본 HWP/HWPX 두 검사와 여백 대조군은 수정 전 FAIL, 기존 #7336 첫 쪽 기준선 대조는 PASS였습니다(4개 실행:1PASS/3FAIL,exit100). 별도 각주 쪽 검사도 수정 전 FAIL(1055.264px/PDF1061.720px,exit100)입니다. 새5개를 포함한 첫 후보 집중 **20PASS/855SKIP**, 검사0.158s,exit0입니다. 처음 Clippy가 새 테스트의 숫자 자릿수 표기를 거부한 결과를 보존하고 값·공차 변경 없이 포맷을 고친 뒤 prepare·집중 검사·fmt·세 Clippy·workspace build·고정 base manifest·diff check를 모두 재실행해 통과했습니다.
- 같은 ZIP의 한 여백 속성만2834→4252HU로 바꾼 입력2개와 독립 한컴2020 PDF2개를 정식 증거로 포함합니다. 꼬리말 여백만 바꾸면 실제 번호 이동0px, 아래쪽 여백만 바꾸면−18.906667px(독립 PDF−18.859375px)입니다. 생성 대조군의 통과를 원본 생성 정보의 증거로 대신하지 않습니다. [입력과 생성 절차](../../../samples/issue1133/README.md), [해시와 출처](../../../samples/issue1133/MANIFEST.json)를 연결했습니다.
- 원본 #1133 두 형식의 실제 SVG 쪽번호 기준선은1090.244→1080.248889px로 바뀌었고 독립 PDF1080.419271px와−0.170382px 차이입니다. 아래쪽 여백 대조군은1061.342222px/PDF1061.559896px로−0.217674px입니다. 각주 쪽은1061.382222px/PDF1061.719727px로−0.337504px입니다. 다른 문서의 #7336 5쪽−1.338px, aift 1쪽−1.262px, #6797 7쪽−1.471px, sample16 5쪽−1.991px 차이는 남으며 글꼴 예외나 완전 픽셀 일치로 보고하지 않습니다.
- fresh WASM은 Mac 로컬 `--no-opt` 대체 빌드이며 Docker 최적화 검증이 아닙니다. 루트pkg·Studio·고정 WASM 패키지의 js/wasm SHA-256 일치를 확인했습니다. Native와 fresh WASM 각15쪽(10입력), 총30쪽을 캡처했습니다. 원본 두 형식2쪽 review/overlay와 여백 대조군, aift·#7336·#6797의 선택 review를 직접 확인했습니다. HWP3쪽 review도 읽었습니다. #1133 HWP2쪽99.27978%/HWPX2쪽98.90081%, #6797 7쪽93.59494%, #7336 5쪽92.84501%이며 backend 간 같은 값입니다. 자동 점수100%도 픽셀 일치 증거가 아닙니다.
- 주 문서430쪽과 대조군211쪽의 모든 Native tree 변경 경로를 검사했습니다. 변화는 Footer 숫자/대시 TextLine·TextRun의 bbox.y뿐이며 구조·내용·본문·그림·표 좌표에는 새 변화가 없습니다. 초기 진단은 compact tree의 `pi:0`을 원본 문단 번호로 해석해 쪽번호를 제외하지 못했으므로 폐기·보존하고 실제 전체 변경 경로로 재검사했습니다. 검사의 메타데이터 오판을 제품 회귀로 기록하지 않습니다. 이 좌표 비교는641쪽 전체 시각 판독의 대용이 아닙니다.
- #1937 24쪽40.88012%와 sample16 5쪽33.98424%는 보정 전부터 있던 본문 소유/배치 보류이며 새 Native review를 직접 확인했습니다. #1937의 PDF24쪽은 표 이어받기이고 실제24쪽은 표 뒤 본문·각주여서, 새 정식 검사는 번호 밴드 위치만 확인합니다. 각주/본문 소유의 일치까지 통과한 것으로 보고하지 않습니다. #1133 첫/이어받기 HWP 표 위여백과 기존 전수860쪽 보류도 남습니다.
- 정확한 명령·제품/검사/입력/PDF/패키지 해시·전후 기준선·선택 페이지 지표는 [보정52 증거](../assets/pr7382_20260926/stage52_page_number_validation.json)에 연결했습니다. 이 첫 후보의 전체 nextest는 뒤 절의44FAIL 결과로 종료됐습니다. 이20개 집중 통과와30쪽 캡처를 재보정된 최종 검증과 구분합니다. **첫 후보 집중 범위의 관측이며 전체 승인·PR 생성은 보류입니다.**

![#1133 Native2쪽 쪽번호 보정](../assets/pr7382_20260926/stage52_native_nested1133-hwp_review_002.png)
![#1133 fresh WASM2쪽 쪽번호 보정](../assets/pr7382_20260926/stage52_wasm_nested1133-hwp_overlay_002.png)
![독립 아래쪽 여백 대조군](../assets/pr7382_20260926/stage52_wasm_bottom15_review_001.png)

### 보정52 전체 실행의 추가 실패 재검토

- 고정된 첫 후보 전체 nextest는10,337PASS/44FAIL/50SKIP,731.749s(11slow),exit100입니다. compile7m40s를 포함하며 source/test/generated SHA는 실행 전후 동일합니다. 번호를 제거한 검사 이름의 비교에서는 Task634 표시4개, 겹침 partition3개, form-002 SVG1개가 추가 실패처럼 보였습니다. 새 샘플은 corpus partition 소속을 바꾸므로 겹침3개를 새 제품 회귀3건으로 세면 안 됩니다. 이후 같은 입력별 사건 수와 실제 좌표로 다시 비교했습니다.
- Task634의 표시 여부8검사는1083.6/1069.7067/1055.24px의 고정 y에 글자가 있는지 확인했습니다. 표시 검사4개가FAIL하고 숨김4개가PASS한 결과는 번호 부재의 증거가 아닙니다. 추가 Native/fresh WASM에서 aift1/4–7쪽과 HWP3원본1쪽을 같은 PDF로 비교해 실제 번호 표시/숨김을 확인했습니다. 자동 번호 런의 실제 최종 기준선에서 SVG 글자를 세어 표시 여부를 검사하고, 위치는 기존 신규 독립 PDF 검사로 분리합니다. 공차·golden·baseline은 바꾸지 않습니다. aift6쪽의 번호 값3/PDF4는 전후 동일한 별도 미충족이므로 표시 검사PASS를 번호 값의 일치로 승격하지 않습니다.
- 추가 겹침은 원본 본문과 자동 Footer 런 사이에 발생합니다. hwp3-sample5-hwp5는12→16건, issue6575는0→2건, rowbreak-problem-pages는1→2건이며 첫/끝 실제 bbox·paint와 독립 PDF를 대조 중입니다. issue1891의12건과 outline2건은 보정51에서도 이미 발생했습니다. 보정50 대비 증가와 보정52에서 새로 증가한 사건을 구분합니다. form-002의 SVG 변경도 자동 갱신 없이 독립 PDF와 전후 diff로 확인합니다. 이 경계를 해결하기 전 전체 완료·PR 승인을 보고하지 않습니다.

- 추가 독립 PDF에서 꼬리말 여백0인 HWP의 일반화 오류를 확인했습니다. sample5의 아래쪽 여백15mm/꼬리말0에서 번호 기준선은1091.519938px, #6575의 아래쪽20mm/꼬리말0에서는1082.079997px입니다. 첫 후보가1061.382/1042.476px로 올린 것은 제품 회귀입니다. 존재하는 꼬리말 밴드의 끝과, 밴드가 없는 경우 자동 번호가 점유하는 아래쪽 여백 중앙을 구분해야 합니다. 특정 문서 ID/글꼴 이름이 아닌 실제 밴드 유무로 원점 계약을 바로잡고 두 원본의 독립 좌표 반례를 정식 검사에 추가합니다. 여백0 경로의 PDF 잔여약1.8px는 명시하며 기존 공차는 변경하지 않습니다. 신규 기대값 공차2px는96dpi 직접 판독과 함께 쓰고 시각 승인이나 글꼴 예외로 대신하지 않습니다.

- 표시 검사 교정 후28개 중27PASS/1FAIL을 확인했습니다. 새 실패는 국립국어원3쪽의 기대 숨김0/실제3글자였습니다. 같은 입력의 한컴2020·2022 PDF(각35쪽)에서3쪽 `- 1 -`, 기준선1064px가 실제 보이고1쪽에는 번호가 없습니다. 과거 특정y에서0글자라는 결과를 숨김으로 해석한 기대값을 폐기하고3글자 표시로 보정하며, 정상1쪽 숨김 검사는0을 유지합니다. 과거 숨김 주석·기여자 설명을 독립 PDF보다 우선하지 않습니다. 번호 값 자체는 직접 출력으로 확인합니다.
- 꼬리말 여백0 반례2개는 첫 후보에서2FAIL/exit100을 재현했습니다. 실제 빈 밴드의 자동 번호 원점은 아래쪽 여백 중앙으로 구하고 줄/런의 공통 기준선 계약은 유지합니다. 양수 밴드가 있는 원본 #1133·여백 대조군은 기존 새 결과를 유지하며 전체 영향 Native/fresh WASM을 재생성합니다. 첫 후보의30px/40px 오류를 기존 본문 차이로 돌리지 않습니다.

- form-002 SVG는 자동 번호3글자의 y만1080.791111→1080.248889px로 바뀌고 나머지 바이트가 같습니다. 독립 한컴2022 PDF 기준선1080.579102px에 대해 이전+0.212010/새−0.330213px를 기록했습니다. 공통 line/run 기준선 계약의 의도된 변화만 골든3속성에 반영하며 전체 자동 재생성·공차 완화는 하지 않습니다. 본문 시각72.66%의 기존 큰 차이는 별도 보류이며 스냅샷PASS를 한컴 일치로 승격하지 않습니다.

### 보정52 최종 재보정 검증 — 여백0 반례와 기존 검사 재판정

- 여백0 반례2개는 수정 전2FAIL/exit100, 재보정 후PASS입니다. 양수 꼬리말 밴드는 아래끝, 밴드가 없을 때는 아래쪽 여백 중앙을 같은 실제 line/run 기준선으로 소비합니다. 특정 문서 ID·글꼴 이름 조건은 없습니다. sample5 5·6쪽 실제1089.715556px/PDF1091.519938px(−1.804383px), #6575 6·8쪽1080.268889px/PDF1082.079997px(−1.811108px)입니다. 첫 후보의30/40px 제품 회귀는 제거했고 잔여 오차는 명시합니다.
- 최종 집중31PASS/5389SKIP,0.329s,exit0입니다. fmt·Native/WASM/workspace-all-target Clippy·workspace build·고정base manifest·source-side unit tier·fresh WASM·diff check 모두exit0입니다. 실행 source 해시와 Native binary SHA는 증거 JSON의 최종 provenance로 교체했습니다. fresh WASM은 root pkg/Studio/고정 패키지 해시 일치의 Mac no-opt 대체 빌드입니다.
- 최종 Native29쪽/fresh WASM29쪽, 총58쪽 raster를 재생성했습니다. #1133 두 형식2쪽 review/overlay, 여백0 sample5 5쪽/#6575 6쪽 review, 국립국어원3쪽 review를 두 backend에서 직접 읽고 #6575 Native8쪽도 확인했습니다. 전58쪽 직접 판독을 주장하지 않습니다. #1133 HWP2쪽99.27978/HWPX2쪽98.90081%는 두 backend에서 같습니다. 국립국어원3쪽은 Native93.56947/WASM93.15120%로 같은 값이 아니며, 두 출력 모두번호1이 보입니다. 실제1061.368889px/독립 PDF1064px로−2.631111px 차이가 남습니다. 표시 검사 수정은 이 위치 오차를 통과 판정하지 않습니다.
- 같은 입력별 text overlap은 보정50/51/첫후보/최종 순서로 sample5 12/12/16/11, #6575 0/0/2/0, outline 2/2/2/2, issue1891 12/12/12/12, rowbreak 1/1/2/2입니다. partition 이름만으로 회귀 여부를 판정하지 않습니다. rowbreak16쪽의2런은 부모 Body/Cell clip의 아래끝1027.987/1023.64px 밖이므로 실제 SVG 표시 겹침과 raw bbox 겹침을 구분합니다. off-canvas/본문 누락/표 경계 보류를 숨기지 않으며 diagnostic의 유효 clip 소비는 다음 개별 보정에서 다룹니다.
- 주 문서430개 최종 Native tree는 첫 후보와 바이트 동일합니다. CLI export tree의 좌표는 요약 정밀도이므로 subpixel 무회귀나430쪽 직접 시각 확인의 대용이 아닙니다. 제품 수정 범위와 독립 SVG 실제 기준선도 함께 대조합니다. 기존 본문 보류는 Native/WASM 각각 sample16p5 약33.98%, #1937p24 40.88%, sample5p6 38.77%, rowbreakp16 72.45%, form002p1 72.66%에 남습니다. aift6쪽 번호3/PDF4도 유지합니다.
- 최종 전체 nextest를threads8/no-fail-fast로 완료했습니다. **10,346PASS/37FAIL/50SKIP**,9slow,검사690.388s/컴파일 포함1152.317s,exit100입니다. source/test/generated/golden/생성 대조군의 snapshot은 전후 동일합니다. 신규 쪽번호7개와 기존 표시8개·form002 스냅샷이 전체에서PASS입니다. 첫 후보44FAIL 중 표시4개·스냅샷1개·text partition4/7의2개가 해소됐으며 새 실패 함수는 없습니다. 보정50과 이름만 비교하면 partition6/15가추가FAIL,4가PASS로 보이지만 샘플 소속이 바뀌었으므로 동일 입력 사건 비교를 우선합니다. 기존43개 검사 실행 상태도 JSON에 연결하며 모든 독립 기대값을 새로 시각 검증했다는 뜻으로 바꾸지 않습니다. **집중 범위 충족, 전체 통합 PR 보류**입니다. [최종 제품·명령·해시·좌표·58쪽 지표·같은 입력별 사건](../assets/pr7382_20260926/stage52_page_number_validation.json).

![여백0 Native 번호 재보정](../assets/pr7382_20260926/stage52_native_overlap6575_review_006.png)
![여백0 fresh WASM 번호 재보정](../assets/pr7382_20260926/stage52_wasm_overlap6575_review_006.png)
![국립국어원 PDF의 실제 번호 표시](../assets/pr7382_20260926/stage52_wasm_gukrip-number_review_003.png)

- 기존 회귀 검사의 정상 대조군도 같은 PDF로 재검토했습니다. #76076 34쪽의 독립 괘선은 첫 표77.514648→463.493327px, 뒤 표511.760010→752.457357px입니다. 현재 CLI 요약 tree는75.6→461.8/509.4→768.1px이며 기존 #3128의 top77±2/bottom463±2/뒤top512±4만으로 글줄·뒤 표의 실제 높이까지 확인할 수 없습니다. 새 Native/fresh WASM1쪽씩을 직접 읽었으며 각각53.36721%입니다. 보정51과82쪽 Body tree는 동일하므로 이번 쪽번호 변경의 본문 회귀로 기록하지 않지만 정상 시각 대조군이라는 설명도 인정하지 않습니다. 바깥 위여백뿐 아니라 본문 줄/뒤 표 높이를 추가 보류합니다. 핵심58쪽과 별개인 추가2쪽 판독입니다.

![기존 정상 대조군의 실제 시각 보류](../assets/pr7382_20260926/stage52_wasm_normal76076_review_034.png)

- 보정52 제출 범위는 자동 번호 기준선·여백0 반례·표시 검사와 독립 PDF3쪽 숨김 오판·골든3속성뿐입니다. 새 주석은 한국어이며 로그/실행 파일/WASM package/파생 suite/output은 커밋에서 제외합니다. 입력2개와 한컴 기준 PDF2개, 최신 대표 PNG18개를 증거로 포함합니다. 이 단계 결과를 커밋한 뒤 #1133 바깥 분할 위여백과 clip 진단의 남은 보류를 각각 분석·수정·검증·보고·커밋합니다. 현 단계에서 push·PR 생성·승인·머지는 하지 않습니다.


## 보정53 사전 분석 — 전체 실패 분류와 #1133 분할 프레임 위여백

- 사용자 요청에 따라 보정52 최종37FAIL의 로그를 원인별로 다시 분석합니다. 본문 넘침6·용지 밖2·텍스트 겹침12는 corpus 묶음 검사20개이고, oracle 쪽수2 및 개별 배치/쪽수15개가 나머지17개입니다. 묶음 함수 수를 실제 결함 수로 환산하지 않으며 예상 쪽의 글줄 부재도 삭제인지 페이지 이동인지 추가 판별합니다. 기준은 `4ddd0ccfc`이고 기존 기대값/래칫을 통과 목적으로 완화하지 않습니다.
- 첫 보정은 #1133입니다. 실제 저장 HWP의 pi29는 텍스트 없는 단일 표 호스트·폭0 LineSeg·PARA 자리차지·COLUMN 가로 기준·RowBreak·바깥 위여백141HU이며 캡션이 없습니다. 현재 `float_placement::column_rowbreak_fragment_opens_outer_top`은 native 경로에서 수직 캡션을 요구해 위여백을 제외합니다. 캡션은 바깥 프레임 여백의 소유 조건이 아닙니다. 독립 PDF2쪽 표 상단440.478678px/3쪽 상단77.356038px·하단296.156006px와 현재439.066667/75.573333/294.613333px를 비교합니다. HWPX 상대 비교만으로 정답을 정하지 않습니다.
- 생산된141HU → `typeset/table/host_spacing.rs`의 첫 host-before → `continuation/fragment/budget.rs`의 새 쪽 예약·page_avail → scan/refit 컷 → emit 흐름 전진 → `layout/table_partial.rs`의 실제 최상위 조각 원점을 추적합니다. 첫 host-before는 이미 위여백을 포함하고 continuation 예약은 같은 술어를 소비합니다. paint에서 resolved 원점·저장 reset·중첩 셀의 별도 원점은 재가산하지 않습니다. whole-fit 캡션 경로는 이번 캡션 없는 split 계약에 비해당입니다.
- 정식 신규 검사에서 수정 전 실패를 먼저 재현하고 실제2·3쪽 표 원점/끝점·컷 소유를 확인합니다. 기존 #1133 간격 검사와 캡션 #6797·기존 RowBreak/중첩 대조군을 실행합니다. #76076은 기존 시각53.37%로 정상 시각 대조군이 아니므로 기계 검사 통과를 정상 출력으로 승격하지 않습니다. 첫 후보에서 적용 범위와 동일 입력의 컷·좌표 변화를 확인한 뒤 Native/fresh WASM 영향 페이지와 필수 lint/policy를 검증합니다. 단계 결과를 보고·커밋하고 남은 원인은 별도 단계로 진행합니다.

- 위여백 술어가 종료 컷과 동시에 참인 경계도 검사합니다. #86712 원본의 실제 빈 저장 앵커·마지막 행 시작 컷을 보존하고 가로 기준만 PARA→COLUMN으로 바꾼 제어 IR에서 세로 원점/후속 표/쪽수/실제 컷 소유를 검증합니다. 원본2024 PDF의 세로 경계를 독립 기준으로 쓰되, 이 변형 자체의 한컴 출력 일치는 주장하지 않습니다. 원본의 기존 #7358 대조군과 새 술어가 동시에 적용되는 변형을 구분하며 위여백을 두 번 예약하지 않아야 합니다. 첫 신규 검사에서 PDF 괘선을 bbox 끝과 비교한 측정 오류를 수정했고, 교정 검사로 수정 전 HWP FAIL/HWPX PASS를 다시 확인했습니다.

### 보정53 제품 보정과 영향 시각 확인

- native 빈 저장 앵커의 프레임 소유에서 캡션 조건을 제거했습니다. 첫 조각의 host-before와 새 쪽의 continuation 예산·실제 원점은 같은 바깥 위여백을 소비합니다. 종료 컷 술어와 함께 참이거나 엄격/단일 셀 경로가 이미 예약한 경우 한 번만 가산합니다. 중첩 셀·확정 원점·수직 캡션 whole-fit의 별도 계약을 보존하며 문서 ID/행 수의 예외는 추가하지 않았습니다.
- 새 정식 검사의 측정 오류를 괘선 중심으로 교정한 뒤 수정 전 **HWP FAIL/HWPX PASS**, 수정 후 **2PASS**를 확인했습니다. 실제 두 조각의 `[0,5)`/`[5,8)` 소유·시작/끝 컷·뒤 빈 문단도 검사합니다. #86712 원본의 가로 기준만 바꾼 제어 IR은 종료 컷과 프레임 조건이 동시에 참인 경계를 확인하며 세로 원점/후속 표/쪽수/가로 원점과 폭을 보존합니다. 변형의 한컴 출력 일치를 주장하지 않습니다.
- #1133 Native 첫 괘선은440.946667px/독립 PDF440.478678px, 이어받기 상단77.743333px/PDF77.356038px, 하단296.493333px/PDF296.156006px입니다. 첫 조각 하단은1043.68px/PDF1042.540039px로 약1.14px 차이가 남습니다. bbox 끝과 선 중심을 혼동한 첫 검사 결과는 정답 증거로 사용하지 않습니다.
- 선택17개 integration binary의 확대 검사는 **85PASS/11FAIL**, 총96개입니다. 기존 #1133 간격 실패는 해소됐고 나머지11개 실패 함수는 기존과 같습니다. 앞107개 실행에 들어간 library11개는 이96개에 포함되지 않으며 최종 전체에서 다시 확인합니다. fmt·세 Clippy·workspace build·고정 base manifest/unit-tier·Native CLI·fresh WASM·diff check 모두exit0입니다. fresh WASM은 Mac no-opt 대체 빌드이며 root pkg/Studio/고정 package SHA가 일치합니다.
- #1133 HWP/HWPX1~3쪽과 #86712 28쪽을 Native/fresh WASM으로 각각7쪽, 총14쪽 캡처했습니다. 두 형식2·3쪽 review와 대표 overlay, #86712 28쪽 review를 직접 확인했습니다. #1133 HWP2쪽99.27%/3쪽100%, HWPX2쪽98.90%/3쪽100%, #86712 28쪽94.35%는 관용 실루엣 보조값이며 엄격 픽셀 일치로 보고하지 않습니다. 여덟 개 after sweep gate는 글꼴 예외 없이passed입니다.
- 주 문서 HWP215개 tree에서는78·79쪽만 바뀌고 HWPX215개는 동일합니다. 실제 SVG의 HWP78쪽 표 상단523.853333→527.626667px/PDF526.666667px, 79쪽83.45→86.933333px/PDF86.666667px입니다. 첫 괘선의 약0.96px 차이를 명시하며 나머지 경계/내용/뒤 각주를 같은 페이지에서 직접 확인했습니다. Native 관용 점수는79.15979→97.36493%,80.58628→97.39699%로 개선됐고 fresh WASM도약97.37/97.43%입니다. 변경 전gate의exit1을 실행 오류로 간주하지 않고 실제 보류로 보존했습니다. 추가 before2쪽/after4쪽으로 총20쪽 raster이며 PDF217/현재215쪽 불일치는 여전히 보류입니다.
- 후보 전체 nextest는 **10,344PASS/41FAIL/50SKIP**,9slow, 검사708.330s/총1225.466s,exit100입니다. 실행 중 source/test/generated/golden/입력3201개 snapshot은 동일합니다. 기존 #1133 간격1개는PASS로 바뀌고 새 실패 함수5개(본문 넘침 partition4개와 #5885 행 선택)가 나타났습니다. 이번 후보의 전체 회귀 통과나 승인 가능을 주장하지 않습니다. 로그·실행 파일·WASM package·파생 suite·output은 커밋에 포함하지 않습니다. [실패37개 분류·명령·해시·독립 좌표·대표 PNG](../assets/pr7382_20260926/stage53_rowbreak_outer_top_validation.json).

![Native78쪽 분할 프레임](../assets/pr7382_20260926/stage53_native_primary_review_078.png)
![fresh WASM79쪽 이어받기 프레임](../assets/pr7382_20260926/stage53_wasm_primary_review_079.png)
![Native1133 이어받기](../assets/pr7382_20260926/stage53_native_nested1133-hwp_review_003.png)
![fresh WASM1133 이어받기 overlay](../assets/pr7382_20260926/stage53_wasm_nested1133-hwp_overlay_003.png)

### 보정53 전체 실패 후 재검토 — 픽스쳐 시각 검증 선행

- 사용자 추가 지시에 따라 실패한 렌더링·쪽수 검사의 적절성을 먼저 검증합니다. 관련 각 페이지90% 이상, 쪽수 검사는 전체 페이지를 비교한 뒤 검사 오류를 판정합니다. 90% 미만/미측정이면 테스트·기대값·baseline을 바꾸지 않고 원본과 실패를 보존하며 해당 픽스쳐의 실제 출력부터 개선합니다. 검사가 정확하면 기대값을 유지하고 메인터너 보정으로 해결합니다. canonical 증적 절차와 CONTRIBUTING/CLAUDE에도 같은 순서를 연결했습니다.
- 새 #5885 실패는 과거 y344.6±3px로 행을 선택하기 때문입니다. 독립 PDF2쪽의 행 시작347.94px/행 끝882.394667px에 대해 새 시작348.4px입니다. Native/fresh WASM1·2쪽은 각각90.38/96.07%로 확인했으나 검사에는7쪽 조건도 있어 전체7쪽 sweep을 선행합니다. 이 시점에는 테스트를 변경하지 않았으며, 아래 최종 전체 시각 결과 뒤에 행 선택을 교정했습니다. 첫 쪽 실제 상단189.56px/PDF189.233333px는 가깝지만 현재 표 하단1011.626667px/PDF1002.902667px의 약8.72px 차이는 별도 실제 높이 보류입니다. 기존 시작 위치 오류와 표 높이 오류를 한 판정으로 합치지 않습니다.
- `.hwpx`라는 파일명의 #3637 입력은 실제 HWP5 바이너리입니다. 컨테이너 확인 후 native 경로 영향을 기록하며 순수 HWPX 변경으로 분류하지 않습니다. 새 넘침 사건의 전후 같은 페이지/노드/크기를 비교하고 새 Native/fresh WASM review/overlay를 생성합니다. 래칫 공차와 baseline은 그대로입니다.
- #80250 원본은 metadata상 한컴2024 저장본입니다. Mac/Windows의 지정 보관 경로와 저장소에서 기존 기준 PDF를 찾지 못해 같은 원본으로 engine2024·timeout1800의 한컴 MCP 변환을 완료했습니다. 기준 PDF는17쪽/316,988bytes이고 `pdf/80250_regulatory_analysis-hwp-2024.pdf`에 보존합니다. 인증값과 endpoint는 기록하지 않습니다. 전후 전체17쪽 sweep으로 판단하며 새 PDF의 생성 자체를 시각 통과로 간주하지 않습니다.

- #5885 전체7쪽은 Native/fresh WASM 각각 모든 페이지90% 이상(최저90.38139%, 글꼴 예외 없음)이며 양쪽7쪽과 독립 PDF7쪽이 대응합니다. 1·2쪽의 표 외곽/중첩 행과 마지막7쪽의 내용 보존을 직접 확인했습니다. 이 선행 증거 뒤에만 기존 검사에서 과거y로 행을 찾는 오류를 원본row=5 소유로 교정하고 독립 PDF 시작347.94±0.6px 검사를 추가했습니다. 기존 행 바닥/중첩표 포함/뒤 행 인접의0.5px 공차는 유지했습니다. 90% 통과를 첫 쪽 약8.72px 높이 차이의 해소로 보고하지 않습니다.
- #80250 전후 전체17쪽 비교는 완료됐습니다. after Native/fresh WASM 모두10쪽47.42712%,11쪽74.22791%,13쪽69.30824%,15쪽80.78939%로 보류입니다. Native 네 review를 직접 확인했습니다. 10쪽 표 전체의 세로 이동, 11쪽 표 시작/라벨/경계 차이, 13쪽 안내문 표시와 줄바꿈 차이, 15쪽 글줄/문단 시작 차이가 남습니다. PDF 생성 성공이나 진단 수치만으로 정상 fixture로 판정하지 않으며 테스트/기대값/baseline을 그대로 보존합니다.

![#5885 전체 시각 검증 뒤 행 선택 교정](../assets/pr7382_20260926/stage53_native_nested5885-full_review_002.png)
![#5885 첫 조각 높이의 남은 차이](../assets/pr7382_20260926/stage53_native_nested5885-full_overlay_001.png)
![#80250 실제 표 위치 보류](../assets/pr7382_20260926/stage53_native_regulatory80250-full_review_010.png)

- #5885 교정 검사를 보정52 런타임에 실행해 독립 PDF 원점347.94px에 대해344.626667px로FAIL을 확인했고, 세 제품 파일을 해시 검증으로 복원한 보정53 런타임에서 **1PASS/234SKIP**를 확인했습니다. 검사 내용은 양쪽 실행에서 동일합니다. 그 뒤 fmt·Native/WASM/all-targets 세 Clippy·workspace build·고정base manifest/unit-tier·diff check를 다시 모두 통과했습니다. 테스트만 바뀐 뒤 제품/source와 고정 Native/fresh WASM 해시가 같음을 확인했으며 이미 같은 런타임으로 완료한 전수 시각 결과의 출처를 유지했습니다. 교정 후 전체 nextest는 아직 재실행하지 않았습니다.
- 사용자는 #80250의 현재 검사 자체를 제거하고 전체 피델리티 개선은 별도 이슈로 나중에 해결하도록 지시했습니다. 테스트를 통과시키는 예외나 공차 완화 대신 해당 문서의 검사 항목을 제거하고 입력/기준 PDF/실패 증거를 보존하는 다음 개별 단계로 처리합니다. 이번 단계의37개 기존 실패 분석, 최종 전체41FAIL 및 #80250 비교는 제거 전 관측 그대로 유지합니다. **보정53 검증 범위 충족, 전체 통합 PR 보류**입니다.

## 보정54 사전 분석 — #80250 검사 제거와 별도 피델리티 이슈

- 사용자 지시는 단순 ignore/skip이 아니라 현재 의미 없는 #80250 검사 항목 자체의 제거입니다. [전체 피델리티 개선 #7445](https://github.com/edwardkim/rhwp/issues/7445)를 등록했으며 #6643의 부분 증상과 같은 원인이라고 단정하지 않습니다. 독립 한컴2024 PDF17쪽과 Native/fresh WASM 전수 비교의 네쪽90%미만을 근거로 현재 문서 검사를 정상 회귀 증거로 사용하지 않습니다.
- `tests/issue_1891.rs`의 HWP와 그 파생 HWPX17쪽/왕복 행2개를 제거합니다. 두 입력은 사용자가 승인한 `mydocs/pr/assets/issue7445/`에 바이트 동일하게 이동·보존하며 samples 재귀 수집에서 빠집니다. 해당 문서의 IR baseline2행과 text-overlap baseline1행만 삭제합니다. 별도 corpus 제외 로직이나 ignore 속성은 추가하지 않습니다. 다른 문서의 baseline 값·공차·검사 본문은 유지하며 제거를 피델리티 해결로 보고하지 않습니다.
- 제품 런타임은 바꾸지 않습니다. 제거 전후 입력 해시·정확한 제거 행·수집 범위·나머지 문서의 검사 결과를 확인하고 필요한 corpus/정책 검증을 수행한 뒤 결과보고·별도 커밋합니다. 다음 렌더링 보정은 #5885 첫 조각 높이이며 #80250은 이번 구현 대상에서 제외합니다.

- 최종 사용자 보충 지시에서 `mydocs/pr/assets/issue7445/80250_regulatory_analysis.hwp`로 이동해 원본을 보존하는 방식은 괜찮다고 확인했습니다. 그 승인에 따라 원본HWP·파생HWPX를 해당 이슈 자산 폴더로 정리했습니다. 임시로 추가했던 렌더 corpus 제외 목록/helper는 모두 제거하고 원래 검사 코드를 복원했습니다. 입력 이동으로 자동 수집에서 빠지며, 명시적 검사2행과 baseline3행만 삭제합니다. 중간 두 상태의 검증은 완료 증거로 사용하지 않습니다.

### 보정54 결과 — 검사 항목 제거와 원본 보존

- 원본 HWP58,368bytes와 파생 HWPX90,624bytes는 `mydocs/pr/assets/issue7445/`에 바이트 동일하게 보존했습니다. Git의 실제 변경은 두 입력의100% 동일 rename, #1891 문서별 행2개와 해당 baseline3행의 제거입니다. corpus 제외 helper/ignore 속성·런타임/공차 변경은 없습니다. 나머지 baseline 행과 네 렌더 corpus 검사 소스는 부모 head와 동일합니다. [입력 해시와 승인 범위](../assets/issue7445/MANIFEST.json).
- threads8/no-fail-fast로 다섯 corpus 게이트와 #1891을 실행해 **48PASS/22FAIL/1087SKIP**,70개,4slow,검사174.205s/전체201.836s,exit100을 확인했습니다. IR·셀줄 넘침 및 #1891 다섯 검사 모두PASS이고 나머지 본문 넘침/용지 밖/텍스트 겹침 실패22개는 보류합니다. 실패 입력/증가값33종은 보정53 전체 로그에 모두 존재하며 새로운 입력별 증가값은 없습니다. partition 소속 재배치로 실패 함수 수가 바뀔 수 있어 이 숫자를 고유 결함 수로 환산하지 않습니다. #80250 제거 전 전체41FAIL을 새 전체 결과로 다시 표시하지 않습니다.
- fmt·세Clippy·workspace build·고정base manifest/unit-tier·diff check는 모두exit0입니다. 실행 전후 검사/입력/baseline 해시가 같습니다. 중간 두 상태의 중단 실행은 최종 검사에서 제외했습니다. 렌더 제품은 보정53과 같아 같은 Native/fresh WASM 전수 증거를 유지하며 테스트 제거를 새로운 시각 통과로 보고하지 않습니다. [명령·70개 결과·실패입력·lint·source 해시](../assets/issue7445/test_removal_validation.json).
- **#80250 검사 제거와 원본 보존은 완료, 전체 피델리티는 #7445로 이관·보류, 통합 PR 보류 유지**입니다. 다음 개별 보정은 #5885의 첫 조각에서 원점0의 저장 줄 뒤 간격을 물리 컷 높이에 그대로 예약한 경로입니다. 독립 PDF 첫 조각 하단1002.902667px/현재1011.626667px와 실제 블록 컷의 네셀1유닛·25.6px·뒤 source0 재시작을 연결하며 기대값·래칫 완화 없이 해결합니다.

## 보정55 사전 분석 — 첫 줄의 원점0 재시작 컷과 줄 뒤 간격

- 시작 head는 `f677301d4`입니다. #5885 원본의 Native/fresh WASM 전체7쪽은 예외 없이 모두90%이상으로 검증됐으므로 그 조건 뒤에 독립 PDF 첫 조각 괘선189.233333→1002.902667px를 검사합니다. 현재 상단189.56px는 맞지만 하단1011.626667px가 약8.72px 큽니다. 실제 블록 컷은 r4 네셀 각각첫1유닛이고 각25.6px를 예약하며, 원본은 첫 줄vpos0 뒤에 새 프레임의0→1920HU 사다리를 저장했습니다. 줄높이1200HU=16px와 뒤간격720HU=9.6px 중 뒤간격은 첫 물리 조각의 표 하단을 늘리지 않습니다.
- 기존 저장 reset 트림은 양수vpos→0을 처리하며 이 첫 줄0→0 경계는 빠집니다. 끝 컷의 실제 source unit 범위와 같은 행의 authentic/control-free 원점 재시작·후속 줄 전진을 확인합니다. 단일 셀 로컬 reset·모든줄0·합성tag·control 문단·온전한 행/컷없음·비native는 적용하지 않습니다. 전체 CellUnit 합은 유지하고 선택된 physical cut의 끝 간격만 공통 결과로 제외합니다.
- `cell_units` 생산 → 블록 컷 scalar/row-offset 워크의 누적 높이 → `SelectedBlockCut::occupied_height`의 예산·끝행 → `row_block_content_height`/`row_cut_content_height` → `table_partial` 행 재구성·`cell_cut_visible_height`의 rowspan 보정으로 이어집니다. 원점0 경계의 같은 helper를 누적 예약과 두 paint 소비 지점이 사용하고 별도 clamp/덮어쓰기로 숨기지 않습니다. 기존 양수reset 계약은 이번 보정으로 재정의하지 않습니다.
- 정식 첫 조각 경계 검사를 수정 전FAIL로 확인한 뒤 구현합니다. 실제 컷·첫 조각 표 하단/점유·다음쪽 시작과 마지막 내용·기존 중첩 행 바닥을 확인하며 Native/fresh WASM 전체7쪽과 #1133/#6797 정상 페이지를 직접 비교합니다. 현재 주 문서215/PDF217 및 다른 전체 실패는 보류입니다.

- 첫 후보는 실제 컷 높이19.76px를 계산했지만 `BlockCutQuery::allows_split`의 일반 최소25px 조건이 유효한 저장 첫 프레임도 거절했습니다. 그 결과 첫 쪽은 해당 행을 전부 이월해 하단983.906667px, 다음 쪽 행 원점374px로 두 검사가FAIL입니다. 기대값은 유지합니다. 원본 여러 셀의 동일 첫 프레임을 모든 소비 컷으로 확인한 경우에만 짧은 완결 조각을 수용하며, 일반 최소 높이는 바꾸지 않습니다. 이 소비 분기와 실제 paint를 함께 재검증합니다.

- 최종 코드 검토에서 편집 세션/실제 표 텍스트 재조판 provenance를 판정 앞에서 제외합니다. authentic tag만으로 편집 뒤 저장 캐시의 유효성을 대신하지 않습니다. Native 원본 첫 쪽은 예약 끝과 실제 괘선이1001.786667px로 일치했으며 그 계약도 정식 검사에 추가합니다. 첫 후보 Native7쪽 최저92.04292%, 정상 대조군 gatePASS/주 문서430tree불변은 후보 증거입니다. 이후 source/test가 바뀌므로 최종 검증·Native/fresh WASM 캡처를 새로 수행합니다.

- 일반 per-row 분기도 실제 호출 경로를 확인했습니다. `row_step.rs`는 raw 워크 높이를 곧바로 예약하지 않고 `row_cut_content_height`의 `split_total`을 실제 예산/예약으로 쓰며, 기존 `stored_zero_origin_rewind_keep`는 확인된 저장 첫 줄 경계를 최소25px 기각에서 제외합니다. 따라서 블록 경로의 누락된 수용 분기만 보완합니다. 기존 #6761 원본/사다리 전진을 깨뜨린 정식 반례도 실행하고39·40쪽 독립 PDF를 추가 비교합니다.

### 사용자 지시 반영 — 기존 전수 회귀 완료 우선

- 추가 중이던 미커밋 첫 조각 검사 `issue_5885_first_fragment_closes_at_independent_pdf_border`를 제거하고 #5885 검사 원본을시작head로 복원했습니다. 해당 파일의diff는0이며 새 회귀 검사는 이번 보정에 포함하지 않습니다. 앞의2PASS/1FAIL 자료는 후보 진단 이력이며 최종 제출 검사 목록으로 보고하지 않습니다.
- 게이트 순서 제어를 중단하고 Cargo/Rust 프로세스가 남지 않았음을 확인한 뒤 파생 목록을 다시 준비합니다. 이 중단 실행은 필수 lint 완료 증거로 세지 않습니다. 먼저 기존 전체 nextest를threads8·no-fail-fast로 완료하고 기존 Native Skia3종을 순차 실행합니다. 추가 검사·불필요한 새 픽스쳐를 만들지 않으며 실패 수와 실제 로그를 근거로 다음 보정 범위를 정합니다.

- 직후 사용자께서 “현재까지 만들어 둔것은 유지”라고 명확히 하셨으므로, 방금 제거한 검사와 예약 assertion을 그대로 복원했습니다. 앞 제거 설명은 중간 이력이며 최종 상태는 **기존 작성 검사 유지/이후 추가 검사 없음**입니다. 잘못 해석해 시작한 전수 실행은 빌드 단계에서 작업 소유 프로세스만 중단했고 결과로 세지 않습니다. 복원된 목록에서 전체 nextest를 다시 시작합니다.

### 보정55 결과 — 저장 첫 프레임의 물리 높이 공유

- 원점0→0→전진 사다리가 같은 시작 행의 여러 원본 셀에서 확인되는 첫 줄 컷은 끝 줄간격을 물리 조각에서 제외합니다. 셀 유닛 전체 합은 유지하고 블록 워크 두 경로의 예약 높이와 paint의 `cell_cut_visible_height`를 공유합니다. 편집/실제 재조판 캐시와 control·합성 줄은 제외합니다. 일반 최소25px는 유지하며 확인된 완결 source 첫 프레임만 짧은 조각을 수용합니다.
- 이미 작성한 #5885 두 검사는 유지했습니다. 첫 조각 검사는 수정 전 실제 괘선1011.386667px로FAIL, 수정 후1001.786667px로PASS이며 예약 끝도 같은 값입니다. PDF1002.902667px와의−1.116px 차이는 남습니다. 기존2쪽 재정상태 행 원점·중첩표 포함·뒤 행 인접 검사도PASS이며 공차/baseline을 완화하지 않았습니다. 첫 후보의 두FAIL과 중단 실행은 완료 검증에 포함하지 않습니다.
- 최종 전체 nextest는 `--locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast`로 **10,348PASS/38FAIL/50SKIP**,10,386개 실행,검사589.749s/전체921.304s,exit100입니다. 실행 전후3196개 source/test/generated/입력 해시가 같고 최종 lint 준비 뒤에도 불변입니다. 이전 전체는4ddd0ccfc+보정53 workingtree의 기록이며 그 뒤 검사 교정을 포함한ef09f1b19 최종head 전체로 재표시하지 않습니다. 이전41실패 중3함수가이번실패목록에서사라졌고새실패함수0입니다. 그중 #80250 corpus 메시지 제거는 검사 이관이고 #5885/#6126 같은 입력의 넘침 메시지 해소와 구분합니다.
- Native Skia library는4112PASS/13ignored,그림 placeholder2PASS,직접PDF4PASS입니다. fmt·Native/WASM/workspace all-targets 세Clippy·workspace build·고정base manifest/unit-tier·fresh WASM wrapper·diff check 모두exit0입니다. fresh WASM은 Mac 로컬대체빌드이며 Docker최적화빌드 통과로보고하지않습니다. 루트pkg/Studio/frozen JS와WASM해시도각각같습니다.
- 최종 Native/fresh WASM 각17쪽을새로캡처했습니다. #5885전체7쪽은두backend모두최저92.04292%,#6797/#1133두형식/#86712대조군도gatePASS입니다. 첫쪽review/standaloneoverlay와2쪽/마지막7쪽을직접판독했습니다. p2마지막행전체하단의기존약14px차이는남으며전문서완전일치를주장하지않습니다. 주문서HWP215+HWPX215=430tree는보정53과바이트동일합니다.
- #6761기존원본/음성대조계약4검사는PASS지만PDF39·40쪽은Native/fresh WASM모두50.26618/71.40357%로gate보류입니다. 본문과표경계의실제위치차이를직접확인했으며원본계약PASS를독립PDF90%이상정상대조군으로승격하지않습니다. 기대값과입력은보존합니다. 주문서215/PDF217쪽과다른38실패도남아통합PR은보류입니다. [최종검증·명령·해시·전후결과](../assets/pr7382_20260926/stage55_zero_origin_cut_validation.json).

![보정55 첫 조각의 독립 PDF 비교](../assets/pr7382_20260926/stage55_native_nested5885-full_review_001.png)
![보정55 첫 조각의 standalone overlay](../assets/pr7382_20260926/stage55_wasm_nested5885-full_overlay_001.png)

### 고정38개 실패의 순차 해결

- 사용자의 최신 지시에 따라 현재38개실패함수를고정했습니다. 각함수를사전분석→원인수정→개별실행→결과보고→커밋순서로해결하고,모두해결한뒤그38함수를각각다시실행한후전체nextest를수행합니다. 이미작성한검사는유지하고이후새회귀검사는추가하지않습니다. [38개목록과진행상태](../assets/pr7382_20260926/remaining38_regression_plan.json).
- 첫개별대상은 #1658하단고정틀입니다. 과거PDF90%이상증거가있으므로현재head에서재확인한뒤검사적절성을판정합니다. 빈줄의실제공간과표/본문소유를구분하며단순가시문자필터·clamp·공차완화로통과시키지않습니다. 다음단계구현전분석과전후실행을이기록에연결합니다.

## 보정56 사전 분석 — 고정38개 중 #1658 하단 고정 틀

- 시작head `1a91646d3`에서 실패함수 `issue_1658_page_bottom_fixed_exclusion::gwanak_bottom_fixed_frame_renders_at_page_bottom`를 단독실행해 **0PASS/1FAIL/206SKIP**,실제끝1108.4px/본문끝1103.6px로 원인을 재현했습니다. 같은코드의 Native/fresh WASM 한쪽전체 시각일치율은둘다92.39813%,PDF1/현재1쪽으로gatePASS입니다. Native review를직접판독한뒤기존검사와0.5px공차는유효한본문공간계약으로판정하고그대로유지합니다.
- 한컴저장 원본 HWPX의 마지막셀/빈줄높이는각1000HU=13.333px입니다. 실제tree에서 고정외곽표 pi6 하단1103.6px,중첩12행표 마지막셀은y1095.1/h8.5px인데빈줄은h13.3px로남습니다. 머리말·꼬리말이아니라pi6고정표의실제빈줄공간입니다. 가시문자필터로숨기거나기대값을넓히지않습니다.
- `HeightMeasurer::cell_nested_controls_bottom`은재귀측정한중첩표높이를소비하지만양수문단기준vertOffset을누락합니다. 배치의 `calc_nested_controls_bottom_height`와nested_y는마지막호스트문단에 `para_relative_float_table_lead`를더합니다. 외곽 `MeasuredTable.total_height`→하단앵커 `compute_table_y_position`→부모col_area→`inferred_viewport_split`의끝셀cap으로이어져큰줄상자를작은부모viewport에남깁니다. 원본nested offset3401HU=45.347px와저장높이23392HU=311.893px,실제내용높이약316.693px를함께보면실제끝점이기존부모높이를4.8px넘습니다. 전화번호도PDF보다약4.8px아래이므로부모의정당한점유끝을측정에반영하는방향입니다.
- 측정에배치와같은기존helper를소비하고같은마지막문단조건을적용합니다. TAC·overlay·음수offset·뒤형제문단은helper/소유조건으로비적용입니다. 수정후기존실패함수와#1658전체/#1611/#1858/#6697기존대조군,필수lint와Native/freshWASM독립PDF를확인해보고·커밋합니다. 새로운회귀검사는추가하지않습니다.
- fetch에서upstream/devel이 `013bc846f`(#7443중첩표hitTest/Studio)로진행한것을확인했습니다. 이번단계정책base는이SHA로고정합니다. 보정55의eb9142dd7기준검증은당시고정base증거로보존하며최신base동등검증으로재표시하지않습니다. 38개수정중임의rebase로검증대상을바꾸지않고제출전최신base통합과필수검증을별도확인합니다.

- 첫후보는같은FAIL입니다. 진단에서자식측정행높이합311.893px/실제배치316.693px를확인했습니다. 차이는저장선언보다작은r3줄과사실상0인r7빈행의여백성장1.066667+3.733333px입니다. 기존배치는이여백을계상하지만측정의relaxed-pad갈래는항상제외했습니다. 그기존성장계산을실제반환높이helper로공유하며두paint소비자가공통결과를직접사용합니다. 기존배치의거대표보호조건은새허용치를추가하지않고그대로이동합니다.
- 추가로원본의한개빈앵커줄을압축-단조사다리로오판해`16px+중첩표합`갈래로보내는실제호출을확인했습니다. 한개앵커는압축사다리의근거가아니므로저장줄의실제전진이확인된경우만그갈래를사용합니다. 그외는기존저장앵커/그룹의실제끝점과같은리드를소비합니다. 임시측정진단출력은원인을확인한뒤제거했으며기존검사를재실행합니다.

- 두번째후보의단독1PASS와대조군9PASS,시각92.42847%는완료판정이아닙니다. 직접review/좌표대조에서제목잉크가수정전/PDFy808px→후보y803px로잘못이동했습니다. 전화잉크는1075→1071px(PDF1071px)로개선됐지만상단정합을훼손하므로이후보를기각하고제품두파일을시작head로복원했습니다. 측정에잘못된paint행성장을복사해일관성을맞추는방향이었습니다. 후보증거는output에보존하고최종source로보고하지않습니다.
- 원본자식표는`noAdjust=1`이고진정한저장셀줄프레임을갖습니다. r3/r7에서paint가안여백을다시더해행합을4.8px키운반면한컴PDF제목은원래원점과맞고전화만그증가분만큼밀립니다. 비편집·미재조판HWPX의noAdjust·컨트롤없는가로셀에서local0시작/단조전진원본줄의점유끝을공통결과로계산해측정과배치가직접소비하도록보정합니다. 선언행의최소값은유지하고실제저장빈줄도끝점에포함하며clamp나가시성필터는쓰지않습니다. 합성/캐시편집/되감김·별도개체는기존경로를유지합니다.

- 저장프레임끝점후보는단독1PASS와기존대조군9PASS입니다. 제출전source검토에서되감김뿐아니라명시적page/column첫줄표시로새물리프레임이시작되는경우도제외합니다. 수정범위의완전한셀프레임만인정하며고정첫유닛/각쪽의줄을최댓값으로합쳐내용을줄이지않습니다. 그최종source로검사를다시실행한뒤lint와시각증거를만듭니다.

### 보정56 결과 — 고정38개 중 #1658 해결

- 제품 보정은 `0c841eb4b`입니다. 첫 리드 가설과 두번째 행성장 공유 후보는 기각했습니다. 최종 원인은 원본 HWPX `noAdjust` 셀의 저장 점유 끝에 paint가 안여백을 다시 더해 자식 행합만4.8px 늘린 것입니다. 비편집·미재조판·가로·control-free·authentic local0/단조 저장 프레임의 끝을 측정의 단일행/rowspan과 배치 fallback의 두 경로가 공유합니다. 명시적 새 page/column, 되감김·문단 로컬 reset·합성 입력은 기존 경로를 유지합니다. 선언 최소행높이와 빈줄 점유를 보존하며 clamp/숨김으로 해결하지 않습니다.
- 기존 실패함수는 수정 전0PASS/1FAIL, 최종 단독1PASS입니다. #1658 전체3개와 #1611/#1858/#6697 기존 대조군을 포함10회, 실제 delta가 있는 #1133 기존3개까지 **13회PASS(12개 고유 함수)**입니다. 검사 원본/공차/baseline 변경과 새 검사 추가는 없습니다. 최종 검사 source 해시와 필수 게이트 source 해시가 같습니다.
- 최종 Native/fresh WASM #1658은 각각 **92.39813→97.46031%**, PDF1/현재1쪽입니다. 두 backend review/standalone overlay를 직접 확인했습니다. 고정틀 원점746.4px/높이357.2px와 제목잉크y808px는 그대로이며 전화 label 잉크는1075→1071px=PDF1071px입니다. 빈줄 높이13.3px를 유지한 채 끝1108.4→1103.6px=본문끝으로 맞췄습니다. 단독PASS지만 제목을5px 옮겼던 후보2를 완료 증거로 재사용하지 않습니다.
- Native/fresh WASM 각각17쪽을 새로 비교했습니다. #5885 전체7쪽 최저92.04292%, #6797/#1133 두형식 정상 대조군 gatePASS입니다. #1133 HWPX2쪽 상단표 일부 행 높이 재배분을 직접 PDF와 대조했으며99.25401%와 기존3검사PASS를 확인했습니다. 주 문서 HWP215+HWPX215=430tree는 보정55와 바이트동일합니다. #6697 30/31쪽은 기존44.73837/70.87030%와 tree 동일하며 피델리티 보류를 유지합니다.
- fmt·세Clippy·workspace build·정책검사·fresh WASM·diff check 모두exit0입니다. 게이트 최초base013bc846f는 당시 고정 증거이며 fetch로확인한 최신base443844b59에 대해 manifest/unit-tier 정책 검사를 추가 실행해둘다exit0입니다. 아직 그base를제품에통합한검증은아닙니다. Mac fresh WASM 로컬대체빌드이며 루트pkg/Studio/frozen JS·WASM해시가각각같습니다. [명령·source/runtime해시·기각후보·전후좌표·시각증거](../assets/pr7382_20260926/stage56_fixed_frame_validation.json).
- **고정38개 중1개 개별 해결/37개 미완료**입니다. 최종38함수 각각재실행과전체nextest는모든개별해결뒤에실행합니다. 현재주문서215/PDF217과다른기존보류를유지하며통합PR은만들지않습니다. 다음개별대상은 #2243 `issue_2243_gyeoljae_sliver_page_pins`의4/3쪽차이입니다.

![보정56 하단고정틀의 독립 PDF 비교](../assets/pr7382_20260926/stage56_after_native_gwanak1658_review_001.png)
![보정56 fresh WASM standalone overlay](../assets/pr7382_20260926/stage56_after_wasm_gwanak1658_overlay_001.png)

## 보정57 사전 분석 — 고정38개 중 #2243 결재문서 sliver 쪽

- 시작head668dd9bc4에서 `issue_2243::issue_2243_gyeoljae_sliver_page_pins`를단독실행해 현재원인을재현합니다. 이전전수의실패는`36382819_gyeoljae_pm_traffic.hwpx`4/한컴3쪽입니다. 원본4문서핀은유지하고새검사를추가하지않습니다. 현재소스의독립PDF전쪽Native/freshWASM을먼저확인하며90%미만을정상회귀픽스쳐로보고기대값만바꾸지않습니다.
- XML의본문은표앵커와저장사다리이며첫쪽결재표뒤에저장vpos가0으로되돌아옵니다. 실패입력의8개표는모두noAdjust=0이므로보정56의noAdjust분기는비적용입니다. 중간비공개마스킹문단일부에는줄캐시가없습니다. 저장본의정상/합성출처를확인하고재조판이필요한줄과저장앵커의실제흐름을대조합니다. 문서ID예외·빈줄숨김·수치공차완화로통과시키지않습니다.

- 단독실패는再現0PASS/1FAIL입니다. Native/freshWASM1/2/3쪽은91.73884/97.90609/39.32301%,현재4/PDF3쪽으로보류이며3쪽review에서지도표와뒤일정이4쪽으로이월된것을직접확인했습니다. 기대핀은그대로두고3쪽의제품출력을90%이상으로개선한뒤회귀적절성을다시판정합니다. 첫진단의PDF4쪽범위오류(exit99)는결과로세지않고보존한뒤1–3쪽전체범위로정정해재실행했습니다.
- 원인추적은지연원점누락가설보다더앞의좌표생산단계입니다. `DocumentCore::reflow_zero_height_paragraphs`의구역재계산은원본캐시vpos21480HU를렌더누적160276HU로덮고원본을`source_line_seg_vertical_pos`에보존합니다. 기존`stored_empty_control_table_frame`은물리쪽원점0을쓰면서누적캐시를읽어상단2138.89px/점유2341.96px를만들었습니다. `query_original_control_table_frame`→`WholeFit.closed_source_frame_placement`→`entry.resolved_host_placement.occupied_bottom`에서원래fit하는표를거절하고`SplitTableEntry`→`prepare`의첫행예산0으로이월합니다. 실제앞본문커서는286.4px이고원본저장21480HU=286.4px+여백141HU로3쪽PDF괴선과대응합니다.
- 완전한빈호스트/후속닫힘계약에서원본쪽상대vpos스냅샷을기존`ladder_vpos`로읽고,스냅샷없는원본은기존캐시를사용합니다. helper의같은확정원점/끝을fit·예약·paint에전달하며후속줄닫힘도같은좌표축으로검사합니다. 문서재계산cursor의지연base를원본물리프레임에서무조건빼는보정은하지않습니다. 정상#6797의물리500HU원점과새/편집/합성계약을기존검사·PDF로대조합니다.

- 사용자 추가 지시에 따라 네 픽스쳐 전체 페이지의 90% 검증을 완료한 뒤 기존 함수를 독립 PDF 좌표/점유 검증으로 보완합니다. 교통 후보1은 3쪽 96.80630%로 개선했으나 컨설팅5쪽 85.39289%, 세운2–5쪽 85.36335/40.80688/74.59056/60.68470%가 남아 #2243은 미완료입니다. 후보1의 코드·런타임·검증은 output의 candidate1에 고정했습니다.
- 컨설팅5쪽의 재조판 개체 줄은 호스트15pt 대신 고정12px로 간격을 만들어992HU를 기록합니다. 독립 PDF의 표/붙임 경계는 글자 크기1500HU×110%를 4HU 단위로 반올림한1652HU와 대응합니다. 합성 줄의 문단 앞 간격 사전 차감과 TAC cap도 fit/paint 원점을 갈라 표−6.17px, 붙임−22.01px가 됩니다. 합성 단일 TAC 줄의 측정 점유·여백·줄간격을 확정 원점/끝으로 공유하고, 저장 줄과 다중/편집/분할 경로의 정상 대조를 수행합니다.

- 세운 3쪽은 합성 줄에서 TAC 표의 높이를 첫 제목 줄에 일괄 부여하고, composer가 한글 개행의 UTF-8 바이트 위치를 문자 위치로 소비해 표/제목 순서를 바꿨습니다. 재조판의 실제 control 소속 줄 → `TacFlowQuery::tac_table_line_index` → pre-table 텍스트 끝 → 최종 배치가 같은 줄 소속을 소비하도록 보정합니다. 다음 줄의 가로 공백 뒤 개체도 명시적 개행 경계에 포함하되 개행 자체를 공백으로 합치지 않습니다.
- 합성 텍스트 사이의 원본 줄 사다리는 원본 끝/재계산 끝의 대응을 유지해 저장 빈 공간을 보존합니다. 개체 문단에서는 이 연결을 끊습니다. 개체 공간까지 원본 틈으로 다시 더한 후보12는 정상 컨설팅/교통도 악화시켜 기각했으며 output에 실패 증거를 남겼습니다. 원본 텍스트 간격과 개체가 소비한 물리 공간을 구분합니다.
- TAC 표 x는 해당 개체의 실제 composed 줄에 있는 머리 공백과 `paragraph_line_indent_for_source`의 저장 `TAG_INDENTATION`을 소비합니다. 세운 4쪽 표의 내어쓰기를 첫 제목 줄에서 추정해 약20px 왼쪽으로 옮기는 가정을 제거합니다. 합성 개행 줄과 정상 저장 줄을 구분하고 일반 표/편집/분할 경로는 기존 소유를 유지합니다.
- 단 맨 위의 완전한 원본 단일 TAC 줄은 원본 양수 vpos가 해당 문단 앞 간격 범위 안에 있을 때 그 간격을 공통 원점/끝으로 보존합니다. 구역 누적 좌표는 이 조건에 해당하지 않습니다. 각주·미주·캡션은 일반 예약/배치 경로가 소유하므로 통배치 단축에서 제외합니다.
- 기존 함수 하나의 쪽수 5/3/5/1은 한컴 PDF와 같아 유지합니다. 기존 쪽수만의 검사가 놓친 표 위치·크기와 제목/뒤 문단 기준선·내용 소유·본문 점유를 PDF 독립 좌표로 검사합니다. 새 `#[test]`는 추가하지 않습니다. 새 위치/크기 검사는 각각1px 이내이고, 세운3쪽 절대 하단의1.119px 잔여 차이(상단0.680+높이0.439)는 숨기지 않고 시각 증적에 남깁니다. 보정 전 코드에서 컨설팅5쪽 표 상단384.280/PDF390.453px로 의도한 FAIL(exit100), 보정본에서1PASS(exit0)를 확인했습니다. 최종 필수 검증·fresh WASM과 전체 영향 확인은 진행 중이므로 아직 고정38개 해결 건수를 늘리지 않습니다.

- 폭 부족에 따른 자동 줄바꿈을 원문의 명시적 개행과 혼동한 중간 후보는 #7160 표 x91.0→83.2px 회귀를 만들었습니다. 앞 composed 줄의 표시뿐 아니라 원문의 실제 개행을 확인하도록 원인 조건을 정정했고 기존 6함수를 다시 통과했습니다. 처음의 파생 suite 불일치 실행(exit4)은 검증 실패 증거로 남기며 검사 통과로 세지 않았습니다. 현행 `--prepare` 뒤 동적으로 배정된 target에서 실제 summary를 확인합니다.
- 후보16은 관련29함수와 Rust 필수 게이트를 통과했어도 #1658 Native가97.46031→75.53978%로 떨어졌습니다. 본문 합성 TAC 표가411.52→400.12px로 올라갔고 `끝.`도601.5px로 이동했습니다. 구조 검사 통과가 이 배치를 검증하지 못하므로 후보를 기각하고 후속 캡처를 중단했습니다. Native/freshWASM 완료 또는 수용 가능으로 쓰지 않습니다.
- 원인은 앞 문단의 저장 줄 점유와 압축된 수용 커서가 다른데 새 단일 TAC 계획이 후자만 원점으로 쓴 것입니다. 단 첫 완전한 저장 표의 줄 base는 기존 layout 초기 `HeightCursor`와 같은 입력이며, 이후의 lazy 역산과 다릅니다. `State::stored_tac_page(paragraphs)`가 이 첫 표의 저장 base를 읽고 → `prepare_computed`가 원점/간격을 포함한 끝을 계산 → fit 예산과 `commit_stored_tac_control`이 같은 값을 소비 → inline placement의 최종 paint가 이를 소비합니다. 첫 항목이 부분 표이거나 합성 줄이면 이 저장 표 축을 사용하지 않고, 새 단에는 이전 구역 누적 좌표를 적용하지 않습니다. 더 자란 실제 흐름을 저장 좌표로 되감지 않습니다.
- 독립 PDF에서 #1658 본문 표 상단411.071/하단599.344px, `끝.` 기준선626.400px를 확인했습니다. 후보17은 표411.52/600.04px, 기준선626.413px로 맞추고 Native97.60662%로 회복했습니다. 기존 #1658 함수의 본문 점유 검사에 이 표와 뒤 문단 위치를 보완했으며 새 함수는 추가하지 않았습니다. 기존 code head e82a946d8에서 보완된 함수는 `끝.`618.413/PDF626.400px로 의도한 FAIL(exit100)입니다. #2243도표상단384.280/PDF390.453px로 FAIL입니다. 보정 소스 복원과 각 테스트 해시를 기록했습니다.
- 후보17의 #2243 전체14쪽 Native 최저는컨설팅95.71558%,교통91.73884%,세운96.43610%,택시94.79597%입니다. 보완한 기존 두함수를포함한29함수가다시PASS했습니다. 최종freshWASM·필수게이트·대조군·주문서확인은진행중이며그결과전에는완료건수를올리지않습니다. #7160의기존2쪽89.36485%는전후3tree불변을확인했으며기존미해결차이로보존합니다. 새회귀나golden으로고정하지않습니다.
- 사용자 추가 지시의 신규 렌더링 회귀 최저90% 조건은 공통/기여/검증/review/PR템플릿/기여스킬12곳에 반영해 `e82a946d8`로 별도 커밋했습니다. 관련 모든페이지·fixture·Native/freshWASM의 최저값을 사용하고, 쪽수검사는전체페이지를비교하며,미달/측정불가이면새회귀·fixture/golden추가를보류합니다. 평균·글꼴예외·CI성공으로대신하지않고기존검사는자동삭제하지않습니다. 변경12문서링크와기여스킬형식검사는통과했습니다.

### 보정57 최종 결과 — #2243 개별 해결

- 코드 `1c82df86f`에서 기존 #2243 쪽수 5/3/5/1을 유지하고, 표 위치·크기·뒤 문단 기준선과 내용 소유를 독립 PDF로 보완했습니다. 새 테스트 함수·skip·baseline·공차 변경은 없습니다. 보완한 #2243/#1658은 보정 전 코드에서 의도한 좌표 차이로 각각 FAIL, 보정 후 PASS입니다.
- 관련 10개 case의 29개 함수 PASS, 필수 12개 게이트 exit0입니다. Native/fresh WASM 각각 26쪽을 새로 비교했으며 #2243 전체 14쪽의 문서별 최저는 컨설팅95.71558%, 교통91.73884%, 세운96.43610%, 택시94.79597%입니다. #1658은97.60662%, PC 셧다운 정상 대조군96.05407%입니다. Native 교통3쪽·컨설팅5쪽과 fresh WASM 세운3쪽·#1658 overlay를 직접 읽어 표 외곽·뒤 문단·쪽 소유를 확인했습니다. 세운3쪽 표 절대 하단1.119px 차이는 유지한 잔여 차이이며 픽셀 완전 일치를 주장하지 않습니다.
- #7160 2쪽은 Native/fresh WASM 모두89.36485%로 재검토 보류입니다. 해당 원본의 전후 tree는 불변이며 기존 검사는 유지합니다. 주 문서 HWP/HWPX 각215쪽, 합계430개 tree도 보정56과 바이트 동일하지만 PDF217쪽 차이는 미해결입니다.
- 고정38개 중 **2개 개별 해결/36개 미완료**입니다. 전체38개 개별 재실행과 전체 nextest는 아직 실행하지 않았으며 통합 PR을 생성하지 않습니다. 다음 개별 대상은 #1733 HWPX 243/PDF242쪽 차이입니다. [명령·소스/입력 해시·최저 페이지·수정 전후·잔여 차이](../assets/pr7382_20260926/stage57_sliver_validation.json).

![보정57 교통3쪽 Native 비교](../assets/pr7382_20260926/stage57_after_native_traffic2243_review_003.png)
![보정57 세운4쪽 fresh WASM overlay](../assets/pr7382_20260926/stage57_after_wasm_sewoon2243_overlay_004.png)

## 보정58 사전 분석 — 고정38개 중 #1733 HWPX 마지막 부속서 이월

- 시작 head `80f7e778a`, 코드 `1c82df86f`입니다. 기존 `issue_1733_hwpx_matches_hancom_pdf_page_count`의243/PDF242쪽 실패를 재현하고 정상 HWP 대조군도 실행합니다. 기대값242와 기존 줄 소유 검사를 유지하고 새 함수는 추가하지 않습니다.
- 입력은 `samples/task1725/text_footnote_tail_overpagination.hwpx`와 HWP 원본이며, 독립 기준은 커밋된 `pdf/issue1733/text_footnote_tail_overpagination-hwpx-2020-20260814.pdf` 및 HWP PDF 각242쪽입니다. 마지막 기준쪽은 ‘부속서12 고속선 운항한계 지정시 고려사항’으로 확인했습니다. 과거 마지막쪽 비교만으로 전체 회귀 적절성을 확정하지 않고, 현재 출력의 첫 차이 경계·본문/각주·실제 쪽 소유를 추적합니다. 쪽수 검사의 픽스쳐 근거는 최종 전체쪽 시각 비교로 판단합니다.
- 원인 후보는 저장 줄 reset의 물리 쪽 판단과 각주 예약/본문 점유입니다. 입력의 실제 저장 줄·독립 PDF 내용 경계 → paragraph scan/section tail의 컷 → 예약 높이·이월 → 최종 배치를 대조한 뒤 구현합니다. 현 단계에서는 원인 미확정이며 검사 기대값을 갱신하지 않습니다.

### 보정58 원인 추적 갱신 — 최초 추가 쪽은45/46쪽 수식 줄

- 기존 두 검사 실행은 HWP1PASS/HWPX1FAIL(243/242), exit100입니다. 원본 본문과 PDF242쪽의 전체 텍스트 n-gram 대응에서 최초 추가 쪽은46쪽, pi995의 ‘D는 용골의 하면으로부터…’ 한 줄입니다. 220쪽 reset은 최초 경계가 아니며 PDF220쪽에 실제 다음 줄이 있습니다. 과거242쪽 출력은 그 reset을 버려 본문을 y1485px까지 그렸으므로 쪽수 PASS만으로 정답이라고 판단하지 않습니다.
- pi974의 실제 저장3줄은 (lh32+ls720), (lh32+ls720), (lh2808+ls720)HU입니다. 첫 두 줄은 수식 개체 마커 U+FFFC만 가진 줄로 가시 글자 런이 없지만 `composed_line_max_font_size`와 배치의 런 font fold가 이를 일반16px 글자로 확대했습니다. 원본 줄 점유67.093px 대신98.240px로31.147px가 더해져 pi995 현재커서1009.1px에서 fit을 거절합니다. 원본 pi995 vpos73344HU=977.92px, 글상자끝993.92px는 본문1009.147px 안에 있습니다.
- 생산 경로는 composed marker-only runs → typeset `paragraph/format.rs` max_fs와 line pair → `place_fitted_paragraph` flow advance → pi995 whole-fit/이월입니다. 배치는 `layout/paragraph_layout.rs` 별도 font fold → corrected metrics → 마지막 텍스트 y를 소비합니다. 두 소비자가 공통 ‘가시 글자가 아닌 개체 마커는 글꼴 높이를 예약하지 않음’ 조건을 사용하게 수정합니다. 정상 빈 문단·공백 글자·가시 텍스트와 수식이 섞인 줄은 기존 글꼴 계약을 유지하며 저장 줄의 양수 높이는 보존합니다.
- 첫 두 수식 common.height32HU(0.427px)와 너비96868HU는 원본에 저장된 압축 개체 프레임입니다. 현재 SVG/Skia/Canvas는 가로만 스케일하고 세로/기준선은 intrinsic 값을 사용해 커다란 식을 그립니다. 원본 압축 프레임의 실제 높이·기준선과 backend별 변환을 함께 확인한 뒤, 텍스트 흐름의 해결을 출력 완료로 승격하지 않습니다. before45쪽 최저56.64324%이므로 기존 테스트/기대값 변경은 보류합니다.

- 첫 후보는45쪽56.64324→94.65215%,46쪽99.20749%로 개선했지만219쪽17.92892%와 HWP241/PDF242쪽 반례로 기각했습니다. 전체 Native 실행도 이 실제 반례로 중단(exit143)했고 완료 증거로 사용하지 않습니다. 기존 두 함수는0PASS/2FAIL이며 기대값은 변경하지 않았습니다. 기각 사유와 부분 산출물은 `output/pr-review/planet6897-7382-20260926/stage58-tail1733/candidate1/rejection.json`에 보존했습니다.
- 219쪽 그림 앞의 실제 저장 경계는 앞 문단 마지막 줄 끝1920HU + 그림 오프셋872HU + 높이16376HU = 호스트 ‘그럼 3’ 줄 원점19168HU입니다. 호스트 줄 점유1920HU 뒤의 후속 원점21088HU도 정확히 닫힙니다. 독립 PDF 그림 상단113.333px과 대응하지만 기존 배치는 호스트 뒤368.4px에서 그림을 다시 그려 후속 본문을 잘랐습니다.
- `stored_picture_before_host_placement`가 실제 앞 흐름과 저장 앞 줄 끝, 그림 하단/호스트 원점, 호스트 끝/후속 줄의 동일성을 1HU 반올림 범위에서 함께 확인합니다. 편집·합성 줄·복수 개체·음수 오프셋·닫히지 않는 저장 경계에는 적용하지 않습니다. 생산한 공통 앵커/호스트/후속 원점 → controls의 예약 끝 → layout의 호스트 생성 원점 → shape의 실제 그림 앵커 → 문단 최종 끝으로 연결합니다. 그림 높이와 8px gap 휴리스틱을 새 경로에서 재계산하지 않으며 paint 뒤 좌표 이동/출력 숨김을 사용하지 않습니다.
- 두 번째 후보의 HWP/HWPX는 각242쪽이고 pi1217은 실제56/57쪽으로, pi4726은219/220쪽의 두 조각으로 보존됩니다. 기존 검사의57/58쪽 및 ‘pi4726/4731 모두220쪽 FullParagraph’ 기대는 이전의 잘못된46쪽과 누락된220쪽 경계를 전제로 했습니다. 아직 전체 시각 근거가 없으므로 기존 검사는 수정하지 않고 경계 캡처를 먼저 수행합니다.

- 두 번째 후보의 경계7쪽은 모두 gatePASS(최저219쪽92.50112%)이며45/219쪽 review와219쪽 standalone overlay를 직접 확인했습니다. 기존 그림/수식/정상 TAC의8개 case는 모두exit0입니다. 전체242쪽은4개의 독립 페이지 구간으로 동일 소스·실행파일을 검증하고 있으며 완료로 보고하지 않습니다. 최초 직렬 실행의 부분 산출물은 속도 개선을 위한 재시작(exit143)으로 보존하고 통과 증거에 포함하지 않습니다.
- 전체 비교에서64쪽87.12033%,65쪽17.92506% 반례를 발견했습니다. 두 그림의 빈 호스트는 앞 줄 끝=호스트 원점, 그림 하단=뒤 설명 줄 원점으로 닫히지만 기존 배치는 빈 호스트 줄을 그림 앞에 또 더했습니다. 세 번째 그림은 원본67284HU + offset1231HU + height10672HU가 본문을 넘고, 후속 본문의 저장 리셋 원점은10672HU로 그림 전체 높이와 같습니다. PDF64쪽에 없는 그림이 실제65쪽 xref484에 존재하므로 누락/숨김으로 처리하지 않습니다.
- 보정 범위는 완전한 저장 빈 그림 호스트의 원점/끝 공유와 물리 쪽을 넘는 그림의 독립 이월입니다. 호스트와 설명은 앞쪽에 남기고, 다음쪽 그림 상단 프레임과 본문 원점을 같은 예약 결과로 소비합니다. 편집·복수 개체·Square 흐름·불일치 저장 경계는 기존 계약을 유지합니다. 193/195/200/201쪽의 추가 미달도 전체 후보 증거에 보존하며 새 후보로 원인 경계를 다시 확인합니다. 아직 기존 #1733 테스트를 수정하거나 해결 건수를 늘리지 않습니다.

- 세 번째 후보는 HWP/HWPX 각242쪽을 유지하고64쪽99.98957%,65쪽99.97744%로 개선했습니다. 아직155쪽46.34230% 및 표 이어받기193/195쪽 등의 차이는 남습니다. 상태 관측면에 직접 큐를 변경한 최초 빌드 오류exit101은 별도 보존했고 기존 Command 계층의 `defer_stored_picture`로 정정한 재빌드는exit0입니다.
- 다음 원인은155쪽입니다. pi3248의 실제 직접 저장6줄은65280/67200/69120/71040/0/1920HU이며, XML 각주 내부 줄은 호스트 줄과 구분해서 읽었습니다. 현재870.4px 흐름이 첫 저장 앵커65280HU에 정확히 맞지만 빈 본문의 각주46/47을 개체 흐름으로 분류해 reset 전4줄 대신5줄을 수용했습니다. 두 각주는 텍스트/개체가 없는 원본 빈 문단 하나이며 PDF154/155쪽에도 각주46/47 본문·번호·구분선은 없습니다. 본문 내 참조 번호는 보존합니다.
- 저장본의 내용 없는 각주 본문과 비어 있거나 합성된 메타데이터는 구분합니다. 유효 저장 빈 문단으로 확인된 본문 없는 참조만 예약/등록에서 제외하고, 그 인라인 참조가 일반 텍스트의 유효 reset 앵커 판단을 막지 않게 합니다. 내용·공백·개체가 있는 각주와 무효/편집 입력의 기존 안내 줄 계약은 유지합니다. 원본/source-anchored 줄 컷 → 본문 소유 → 실제 각주 등록/예약 → 최종 첫 줄 위치를 검증하며 155쪽 점수를 이유 없이 공차로 넘기지 않습니다.

- 후보2의 전체 HWPX242쪽 Native 비교는4개 구간 모두 완료했고13쪽이90% 미만입니다. HWP/fresh WASM 전체 실행은 아직 하지 않았습니다. 후보4에서154쪽99.46776%,155쪽95.47009%, HWP/HWPX 각242쪽을 확인했습니다. pi3248은154쪽0..4/155쪽4..6으로 PDF의 두 후속 줄을 보존하고 본문 내 각주 참조46/47도 남깁니다. 내용 없는 원본 각주 본문을 등록하지 않아 인쇄 기준에 없는 번호/구분선을 생성하지 않습니다. 최초 import 오류exit101과 정정 재빌드exit0도 output에 보존했습니다.
- 다음 표 경계 사전 분석: 4175/4186은 원본 XML `pageBreak=CELL`이며 내부 모델에서는 `RowBreak`로 처리됩니다(실제 TABLE_DRIFT 경로 확인). 4175의row24 마지막 점선 줄이vpos0으로 다음쪽을 소유하지만 현재193쪽은row25부터 시작해 row24 tail 띠를 버립니다. 4186의row20 점선 마지막줄도vpos0인데 현재195쪽은row20 전체를 이월해 앞쪽 소유의COSPAS-SARSAT까지 다시 그립니다. 단순row index/전체셀 높이만으로 저장된 내용 컷을 대신한 문제입니다. 실제 셀별 저장 줄과 PDF192–195쪽의 끝/시작 내용, 요구 높이·예약 높이·컷·재배치를 대조한 뒤 수정합니다. 내부 `CellBreak`의 원자성 정책을 바꾸는 사례가 아니며 실제 RowBreak 컷 선택 경로를 보정합니다.

- 후보5 구현 전 경로 확인: 저장 줄→`row_stored_terminal_zero_origin_cut`의 셀별 유닛 컷→`SourceFrameQuery`의 초과 통째 수용 제외→`scan_ordinary_row_step`의 동일 컷 고아 판단→기존 실제 높이 예산 검사→`PartialTable` 컷→paint의 같은 컷 높이입니다. 한 셀의 마지막 0→0 줄과 완결된 한 줄 파트너를 구분하며, 합성 줄/다문단/개체/rowspan/모든 셀 0 좌표는 제외합니다. 행 번호·문서 ID·새 허용치는 추가하지 않습니다. 192–195쪽 직접 비교 후 적용 범위를 판단합니다.

- 161–162쪽 추가 사전 분석: note51은 한 문단 안의 reset이 아니라 25개 문단의 각주 공통 좌표를 사용합니다. 문단0–12는0→15840HU로1320HU씩 전진하고 문단13이0으로 다시 시작해 이후도1320HU씩 전진합니다. PDF161쪽은 .9까지,162쪽은「선박 지구국」부터 보여줍니다. 기존 reset 조회는 각 문단 내부만 검사해25개 문단을 전부161쪽에 그립니다. 후보6은 전체 저장 사다리의 정확한 전진·단일 문단 경계 reset·원본 줄과 composed 줄 일대일 대응을 확인해 기존 FootnoteFragment 예약/배치 경로로 넘깁니다. 문단마다0을 쓰거나 전진이 맞지 않는 입력의 계약은 바꾸지 않습니다.

- 193쪽 바깥 프레임 사전 분석: 이전 표의 PDF 종료 괘선288px와 후속 빈 줄 원점은 저장16032HU(본문 기준213.76px)에 대응합니다. 기존 종료 조각은 바깥 아래141HU를 생략했고, 이어지는4178 표는 원본19872HU 앵커+1512HU 오프셋+바깥 위141HU의 원점362.59px 대신358.8px에 그렸습니다. 후보7은 종료 실측 하단+아래여백과 후속 저장 원점의 등식을 확인해 공통 배치 계획을 예약·paint에 전달합니다. 다음 표는 앞 글줄 전진 끝=앵커와 실측 본체=선언 높이, 다음 본문 원점 재시작을 확인해 같은 공통 프레임을 사용합니다. 허용치를 늘리거나 페이지별 좌표를 지정하지 않습니다.

- 200–201쪽 사전 분석: 4342 표는 원본 `pageBreak=NONE`이고 저장 내부 reset도 없습니다. PDF200쪽에는 표가 없으며201쪽은 영향/기준/비고 머리부터 LEVEL1–4 전체가 놓입니다. 현재는200쪽에 머리와LEVEL1을 남겨201쪽이LEVEL2부터 시작합니다. `none_table_is_atomic_here`가 주석에 명시한「양수 오프셋의 가시 호스트」예외를 호스트 글자 여부 없이 모든 양수 오프셋 표에 적용한 것이 원인입니다. 후보8은 실제 visible host 상태를 이월 게이트와 행 컷에 동일하게 전달하고 빈 호스트 표의 원자 배치를 복원합니다. 가시 호스트 예외는 유지합니다.

- 210–211쪽 사전 분석: 4558 그림은 원본 `BEHIND_TEXT`,겹침허용,flowWithText,호스트71040HU의 배경입니다. 다음 저장 원점0에는 그림 범위 안의 겹침허용 레이블 표들이 있고 PDF도 배경/레이블을211쪽에 같이 둡니다. 현재 그림은210쪽으로 높이를 되돌려 그려 본문을 덮고 레이블만211쪽에 남습니다. 후보9는 원본 호스트의 현재 실제 흐름 일치·프레임 초과·엄격한 빈 줄 사다리·다음 원점의 레이블 프레임을 확인해 기존 그림 이월 계획을 사용합니다. 배경은 가시 높이를 보존하되 흐름 예약은0으로 분리합니다. 원본·합성/편집·일반 자리차지 그림 계약을 섞지 않습니다.

- 후보8 기각: 표 머리/LEVEL1–4는 통째로201쪽에 복원됐지만4343–4348 앞 두 글줄까지 강제 쪽 이동에 따라가며 HWPX243/PDF242쪽이 됐습니다. 원본은 후속4348 문단의 세 번째 줄35328HU부터201쪽을 소유하고 앞 글줄은200쪽에 남아야 합니다. 빈 호스트 원자 수용만 바꿔서는 표와 호스트 후속 줄의 소유 분리가 해결되지 않으므로 후보8 원자 정책 수정만 되돌렸습니다. 원본 기대값을 변경하지 않았으며 후보8/9의243쪽 산출물을 최종 증거로 사용하지 않습니다. 후보10은 유효한 후보7까지의 수정과 배경 그림 이월만 따로 재검증합니다.

- 200–201쪽 소유 분리 근거: 4342 표의 선언/실측 본체35048HU와 위/아래140HU의 합은35328HU이며, 뒤4348 문단 세 번째 저장 줄의 새 원점35328HU와 정확히 같습니다. 앞 호스트61440HU부터 해당 reset 전까지1920HU(빈 작은 줄은700HU) 사다리는200쪽의 본문입니다. 후보11은 표 원자 정책을 바꾸는 대신 원본 프레임 초과·실측/선언 동일·엄격한 후속 사다리·프레임 하단=본문 재시작 등식을 확인해 표 개체만 다음 쪽으로 미루고, 기존 그림 이월과 같은 개체 배치 계획으로 예약/실제 표를 전달합니다.

- 후보12 사전 분석: 후보11은200쪽 앞 본문과201쪽 표를 분리했으나4348 문단 전체를201쪽에 넘겨 앞 두 줄이 중복 소유됐습니다. 기존 anchored body reset은0만 받고 표 컨트롤 문단을 모두 제외합니다. 검증된 다음 쪽 개체 프레임 하단35328HU를 읽기 전용 줄 스캔 관측값으로 전달하고, 동일 원점에 재개하는 겹침허용 비TAC float 표 문단에만 기존 prefix/suffix 컷을 적용합니다. 원점이나 프레임을 새로 추측하지 않으며 앞 글줄의 실제 현재 흐름 앵커 일치도 계속 요구합니다.


- 후보5–7 경계 결과: Native192쪽97.35340%,193쪽99.94669%,194쪽99.66887%,195쪽99.94977%로 저장 행 컷/종료 바깥여백/후속 표 원점이 개선됐습니다. note51 분할 후161쪽96.65225%,162쪽95.25674%이며161쪽 review를 직접 판독해 본문과 각주 겹침이 해소됐음을 확인했습니다. 정상 대조군77개는 모두PASS입니다. 이 결과는 전체242쪽/fresh WASM 완료 증거가 아닙니다.
- 후보11은 표 전체와 뒤 본문을 분리했지만 HWPX243쪽,200쪽92.96561%/201쪽67.33730%입니다. 추가 정상 대조군 포함81개PASS를 확인했으며 기존 #1733 기대값은 유지합니다. 후보12의 읽기 전용 reset 전달만으로는 점수/쪽수 변화가 없었습니다. 실제 호출 경로 확인에서 표 컨트롤 문단은 `section/flow → controls/paragraph_flow → block/entry → place_table_with_text`로 가므로 일반 `paragraph/flow`의 강제 글줄 경계를 사용하지 않았습니다.
- 후보13 사전 분석: 검증된 이월 프레임과 동일한 양수 reset을 가진 단일 겹침허용 Square 표 호스트에 한해 `prepare_forced_page_boundary → typeset_paragraph`의 기존 prefix/suffix 결과로 본문을 먼저 분할합니다. 표는 suffix 원점+저장 오프셋+바깥위여백을 공통 `ParagraphFloatPlacement`로 기록하고, 실제 출력도 이를 소비합니다. 표 기하와 본문 흐름을 구분해 Square 띠만 예약하며 앞 글줄을 표의 통째 fit에 동반 이월하지 않습니다. 재조판/합성/각주/복수 개체/원점 불일치 경로에는 적용하지 않습니다.
- 201쪽 화살표 사전 분석: 원본4352 묶음 자식은 실제5/5/6개 직선이 있으며 변환 HWP 재파싱에도 보존됩니다. Group 자식 소실 가설은 이 증거로 배제했습니다. 원본 `lineShape headStyle=ARROW/tailStyle=ARROW`를 HWPX `parse_line_shape_attr`가 읽지 않아 attr에 화살표 모양 비트가 없고, SVG에 marker가 출력되지 않습니다. 기존 serializer는 bit10–15/16–21 역매핑을 제공하므로 parser가 같은 독립 XML 속성을 보존하도록 보완하고 Native/fresh WASM 같은 쪽에서 화살표 표시/위치를 확인합니다.

- 후보13은 실제 PageItem200쪽4348 `0..2`,201쪽 `2..3`으로 소유 컷을 보존했습니다. 그러나 layout의 Square 호스트 조기 반환이 이 명시적 부분 문단도 건너뛰어200쪽 글줄은 아직 누락됩니다.201쪽은68.95362%, 전체243쪽입니다. 새 Square 밴드 등록은 혼합 폭 문단4350의 prefix를 처리하기 전에 `close_square_band`로640.8px까지 흐름을 밀어 다음 별표를202쪽에 배치했습니다. 저장 사다리로 이미 폭/원점이 확정된 이 경로의 추가 밴드를 제거하고, 실제 suffix 원점과 확정 배치를 후처리에서 보존합니다. 기존 일반 어울림 분기를 바꾸기 전에 해당 경계를 직접 재캡처합니다.

- 후보14 결과: HWPX242쪽으로 복원되고200쪽99.94907%,202쪽99.31836%,210쪽99.76592%입니다. 기존10case/81함수 nextest는 모두PASS(exit0)입니다.201쪽74.59831%,211쪽52.96277%,215쪽89.52023%,224쪽85.19583%로 아직 미충족이며 전체/fresh WASM 완료로 쓰지 않습니다.
- 후보15 사전 분석(201쪽): 원본4350의 처음3줄은 cs44560/sw3628, 뒤 줄은 cs800/sw47388로 저장된 표 옆/아래 혼합 폭입니다. 첫 띠는4348 Square 표의 실제 폭+여백과 정확히 같습니다. whole/prefix 흡수 실패 후 일반 배치로 넘기는 분기가 WrapAnchorRef를 기록하지 않아 서식화/최종 배치에서 첫3줄을 왼쪽으로 바꿉니다. 띠 매칭 결과를 일반 fit에도 전달하고 기존 저장 줄 폭을 유지합니다. 또한 명시적 부분 호스트와 확정 Exclusion 계획을 소비한 Table 이후 layout의 HeightCursor 초기화가 typeset에서 보존한 원점을 지워 후속 표를 아래로 밀었습니다. 같은 실제 부분 컷/확정 원점 조건에서 최종 소비자의 초기화도 막습니다. 일반 통째 호스트·다른 표/Shape는 기존 계약을 유지합니다. 비교 기준은 커밋된 PDF201쪽의 오른쪽3줄과 그룹 상단798.667px이며 코드의 현재 좌표를 기대값으로 재사용하지 않습니다.

- 후보16 사전 분석(211쪽): 검증된 다음 쪽 배경 계획이 새 원점을0으로 기록했으나 최종 그림 배치는 다시 원본 호스트의1320HU 오프셋을 더했습니다. PDF 이미지 상단76.000px 대비 현재93.200px로17.6px 차이가 있고 레이블의 저장 원점은 새 쪽0입니다. 이월이 검증된 배경 프레임의 상단은 새 쪽0이므로 공통 계획에서 anchor=-offset/table_top=0/occupied_bottom=height를 함께 확정합니다. 원래 호스트의 프레임 초과/실제 흐름 일치/후속 레이블 reset 조건을 유지하고 일반 그림 원점을 clamp하지 않습니다.

- 후보15 결과:201쪽은 오른쪽3줄의 x가 복원됐지만74.42363%로 여전히 미충족입니다. 직접 review와 LAYOUT_Y에서4348 부분 본문 종료572.2px 뒤 Exclusion 표가 흐름을724.1px로 잘못 전진시킨 것을 확인했습니다. 이 확정 계획의 계약은 개체 기하와 본문 흐름을 분리하는 것이므로 Table 소비자는 이미 배치된 본문 흐름을 유지해야 합니다. 또한 부분 문단 직후 별도 reset 분기가 원점을 지워 Table 뒤 보존만으로는 충분하지 않았습니다. 후보17은 원본 부분 컷과 같은 확정 Exclusion 원점을 가진 실제 페이지에 한해 부분 줄의 저장/실제 좌표로 기준을 세우고 Table은 입력 흐름을 그대로 반환합니다. 표 외곽은 확정 계획의 top/bottom 그대로 출력하며 숨기거나 잘라 맞추지 않습니다.

- 후보16 결과:211쪽52.96277→99.11028%,210쪽99.76592%,212쪽99.87081%입니다. 새 review를 직접 열어 배경/레이블/화살표 상단과 외곽 정렬을 확인했습니다. 기존 쪽수는242이며 fresh WASM은 아직 미실행입니다.
- 후보18 사전 분석(215쪽 흑백 그림): 원본 image12.bmp는613×396 RGB이며 PDF 내장 xref1543은 같은 크기의1bitGray입니다. 원본 BT601 정수 명도<=128을 검정으로 먼저 변환하면242,748픽셀 중 차이0(검정14,370/흰색228,378)입니다. 현재 SVG/Skia/Canvas는 크기 변경 뒤 흑백 필터를 적용해 획이 달라집니다. shared image_resolver에서 effect=BlackWhite/밝기·대비0인 지원 raster만 원본 해상도에서 변환하고, 모든 소비자는 suppress_effects를 사용합니다. 바이트 조회 key도 원본/흑백 variant를 구분하며 compact DOM/paint JSON에는 같은 확정 효과를 전달합니다. 디코드 상한·알파를 유지하고 파손/미지원/다른 밝기·대비는 기존 필터로 돌아갑니다. 독립 픽셀 근거는 output의 blackwhite-independent-pixels.json에 기록했습니다.

- 후보17 결과:201쪽92.31167%로 개선됐으며199쪽99.13071%,200쪽99.94907%,202쪽99.31836%입니다. 새 review를 직접 열어 표 오른쪽3줄, 뒤 전폭 줄, 별표2 외곽과 뒤 그룹의 쪽 내부 표시를 확인했습니다. 화살표 획/작은 아래첨자의 잔여 차이는 보존하며 픽셀 완전 일치를 주장하지 않습니다. 전체/fresh WASM 및 정상 대조군은 아직 이 후보로 완료하지 않았습니다.
- 후보19 사전 분석(224쪽): 실제 원본4825는 첫 런 탭4000HU + HL, 다음 아래첨자 런2 + 탭1856HU입니다. parser는 두 확장을 보존하지만 layout의 run별 TextStyle이 전체 tab_extended를 복사하고 측정 함수는 런마다 인덱스0을 사용해 두 번째 탭도4000HU로 잘못 늘립니다. 출력=의x211.4px 대비 PDF137pt=182.667px입니다. 서식화 전 측정과 실제 run emission 둘 다 문단에서 소비한 탭 순번으로 같은 확장 slice를 전달하고, 줄 재시작에도 앞 줄 탭 순번을 보존합니다. RIGHT/CENTER 탭 기존 분류의 전역 순번과 측정의 로컬 slice를 구분해 같은 원본 확장을 사용하게 합니다. 원본 저장 값이 없는 자리표는 기존 TabDef 폴백을 유지합니다.

- 후보18/19 결과:215쪽89.52023→91.41441%,224쪽85.19583→98.17651%입니다. 새215쪽 review에서 흑백 획과 프레임,224쪽은 같은 후보의 탭 폭 보존으로 판정합니다.201쪽92.31167%,211쪽99.11028%를 유지합니다.209쪽89.21484%는 남아 있어 #1733의 전체쪽 시각 기준/기대값 갱신은 여전히 보류입니다.
- 후보20 사전 분석(209쪽 각주): 원본82번 각주 본문에는 AutoNumber82가6개 연속 저장되고 PDF도82)를6번 표시합니다. PDF 결함으로 가정한 앞선 추측은 원본 XML 증거로 철회합니다.83번은 자동 번호 없는 별표 수동 표기인데 현재는 합성83)를 붙입니다. 기존 각주 전용 경로는 원본 번호 슬롯을 공백으로 두고 전역 번호 하나를 별도 추가하며, run의 저장 탭도 전달하지 않습니다. 원본 저장 문단의 실제 선두 AutoNumber 슬롯/형식/횟수만 사용하고 합성·편집 경로는 기존 fallback을 유지합니다.
- 같은 호스트의 마지막84번은 번호/텍스트 없는800HU 저장 줄입니다. 앞서 모든 bodyless 참조를 무조건 제외한 후보는 기존 각주 영역 안의 이 물리 빈 줄까지 제거했습니다. PDF78번 첫 기준선969.333px 대비 후보983.5px의 약14.17px 차이는800HU 빈 줄+각주 사이 여백14.44px와 일치합니다. 같은 실제 쪽/호스트에 이미 등록된 유효 본문 각주 바로 뒤의 빈 각주는 번호/구분선 없이 저장 물리 높이만 예약하고 그립니다. 단독 빈 참조는 여전히 새 각주 영역을 만들지 않습니다. 측정·최종 배치가 같은 저장 빈 줄을 소비하게 하며 쪽수와154–155/161–162쪽 정상 경계를 재검증합니다.

- 후보20의 최초 빌드는 새 helper의 Option flatten 오류로 exit101이었습니다. 실패 로그를 보존하고 실제 note_number를 호출 경로로 전달하도록 수정한 뒤 build-corrected exit0을 확인했습니다. 번호 문자열을 역파싱해 원본 숫자를 추측하는 후보는 사용하지 않습니다.
- 후보21은 저장 탭이 전혀 없는 문단에서 앞 줄 탐색을 즉시 생략해 기존 일반 문단 비용을 유지합니다. 새 각주 번호 경로에서 별도 위첨자·미지원 사용자 번호 형식은 기존 경로로 보내며 그 범위를 검증했다고 주장하지 않습니다.

- 후보20 결과:209쪽89.21484→98.50712%,154쪽99.46776%,155쪽96.42697%,161쪽98.33620%,162쪽94.47524%,224쪽98.17651%입니다.209쪽 review를 직접 열어 원본82)6회/83번 수동 별표, 각주 상단/마지막 줄 위치를 확인했습니다. 표본 통과를 전체242쪽 완료로 바꾸지 않으며 후보21의 최종 Native/fresh WASM 전체쪽·기존 정상 대조군을 진행합니다.

- 후보22 사전 분석(HWP 대조군): 후보21 HWPX는242쪽이나 HWP는243쪽이며 원본/PDF 텍스트 대응으로 첫 추가 경계를200–201쪽으로 좁혔습니다. HWP4348의 저장 줄71020/72940/35328HU와 이월 표4342의 프레임35048+140+140=35328HU는 HWPX와 같습니다. 이월 표 계획 생산은 양 형식을 받지만 `stored_body_reset_fragment_matches_current_flow`와 anchored reset 후보 선택이 HWPX만 받아 HWP 호스트 앞2줄을 보존하지 못합니다. 검증된 다음 쪽 프레임 하단과 양수 reset의1HU 이내 일치, 실제 앞줄 원점/현재 흐름 일치, 원본 비편집 단일 단 조건을 그대로 유지해 HWP에도 같은 계약을 적용합니다. 일반 HWP vpos=0 본문 reset은 확대하지 않습니다. Native 양 형식200–202쪽을 먼저 재검증하고 전체/fresh WASM은 새 소스에서 갱신합니다. 기존 회귀 기대값/해결 건수는 변경하지 않습니다.

- 후보21 검증: fresh WASM wrapper exit0이며 흑백 이미지 key/inline 바이트 일치와 원본 variant 분리를 확인했습니다. 변환 PNG는 독립 PDF xref1543과613×396/242,748픽셀 전부 같았습니다. 진단 스크립트의 최초 base64 spread는 브라우저 인자 상한으로 exit1이었고 chunk 단위로 수정한 뒤 exit0입니다. 실제 배경 그림은 본문 Flow query에서 제외돼 compact 목록은0건이며 이 범위의 검증으로 세지 않습니다. Native HWPX 전체 비교는66쪽/최저94.65215%/90미만0쪽까지 기록한 뒤 새 소스 검증을 위해 소유 프로세스를 중지(exit143)했습니다. 부분 결과를 전체 통과로 세지 않습니다. HWP 대조군은243/PDF242에서 gate 실패(exit1)였습니다.
- 후보22 최초 빌드는 TypesetState의 존재하지 않는 필드를 사용해 exit101이었으며, 실제 `paragraph_line_scan_page()` snapshot의 프레임 원점을 소비하도록 수정해 Native build exit0입니다. HWP242쪽으로 복원됐고199/200/201/202쪽98.36961/99.92076/92.35483/99.86509%입니다.201쪽 review를 직접 열어 표 프레임, 표 옆3줄/뒤 본문 및 별표2의 같은 쪽 표시를 확인했습니다. 얇은 화살표/아래첨자 잔여 차이를 완전 일치로 표현하지 않습니다. 기존15case를 nextest로 실행 중이며 새 회귀 함수나 테스트 기대값은 추가·수정하지 않았습니다.

- 후보22 정상 대조: 기존15case 중14case는 exit0, `issue_3738_rowbreak_table_footnote_fragment`는31PASS/2FAIL(exit100)입니다. 실패는 각주26/60/62의 쪽 소유가 없어져서가 아니라 추출 문자열에 번호 뒤 정확히3공백을 요구한 assertion이며 현재1공백입니다. 저장 번호/슬롯 보존 보정 전 후보14의81PASS와 구분합니다. 실패를 승인 근거에서 제외하고 독립 PDF27/52–54쪽 Native/fresh WASM 직접 비교 후에만 공간 표기와 검사 적절성을 판정합니다. 선행 검사 실패 시 WASM을 시작하지 않도록 한 driver는 exit1로 중단됐으며, 새 WASM 실행은 실패 원인 진단용으로 진행합니다. 전체 최종 게이트 통과로 쓰지 않습니다.

- 후보22 영향쪽 결과: 정책연구 HWP26/27/52/53/54쪽99.97251/87.56570/99.12801/98.59349/94.72291%로27쪽이90미만입니다. PDF 원문은26)·60)·62) 뒤1공백이지만 아직 테스트를 고치지 않습니다.27쪽 review 직접 판독에서 묶음/그림30 상단721.2px 대비 PDF714.099px를 확인했습니다. 저장381은 원본vpos47375HU/lh19816HU/bl16844HU, 묶음높이16366HU이며 차이3450HU는 아래 캡션+간격입니다. `layout_empty_runs_line`/run Shape 등록은 잉크 높이만 baseline에서 빼며, 그림 분기의 기존 `tac_object_box_height_px`는 같은 캡션을 포함합니다. 후보23은 기존 shape caption accessor를 공유해 두 Shape 등록 분기도 같은 전체상자 높이를 사용하게 합니다. 저장 줄 점유·다음 문단 원점은 유지하고 무캡션/좌우캡션·선·바깥여백 정상 대조를 기존 검사로 실행합니다. 일반 선을 줄 상단으로 옮기는 임시 가정은 사용하지 않습니다.
- 후보22 전체 Native 진단은 현재47쪽씩 진행했고 HWP33쪽45.81889%, HWPX최저94.65215%입니다. HWP33쪽의 그림/본문 위치 차이는 별도 반례로 직접 review를 열었으며 #1733 전체 완료를 계속 보류합니다. fresh WASM 진단 빌드는 exit0(4분57초)이며 package를 candidate22/pkg에 고정했습니다.

- 후보23 Native build exit0입니다. 정책연구27쪽87.56570→99.29352%로 개선됐고 review를 직접 열어 그림30 묶음/캡션 상단과 각주26 배치를 확인했습니다. 나머지 선택쪽/fresh WASM과 정상 대조는 아직 완료하지 않아 기대값은 유지합니다.
- 후보24 사전 분석(HWP33쪽): 원본698은 빈 호스트인데 저장 sw47684HU를 가진 정상 줄이며 앞697끝1352HU=호스트원점입니다. 그림상단1495HU+높이12210HU=후속699원점15057HU와 호스트1352HU 차13705HU에 정확히 맞습니다.701 그림도1212+11717HU=702원점30690−호스트17761HU입니다. 현재 `stored_picture_empty_host_placement`가 sw!=0이라는 잘못된 빈 문단 대용 조건으로 완전한 계획을 거절합니다. 원본 빈 텍스트·단일그림·앞끝/현재flow/뒤원점·프레임의 정확한 일치는 유지하고 저장 줄폭을 빈 문단 판정으로 사용하지 않습니다. 생산 계획이 받아들여져도 `layout_shape_item`의#683 후처리가 다시 호스트 줄을 더하므로, 검증된 successor 원점이 있는 계획에는 그 일반 폴백을 적용하지 않습니다. before-host/next-page 중 successor가 없는 경로는 그대로 둡니다. PDF33 그림B y329.333px 대비 Native348.6px의 약20px(호스트698한줄), 뒤본문 약40px의 중복을 같은 계획/실제 소비 지점에서 제거합니다. 독립 이미지 좌표는 candidate22/hwp-p33-independent-geometry.json에 고정했습니다.

- 후보24 Native build exit0입니다. HWP32/33/34쪽99.94301/99.93670/99.92813%, HWPX99.94140/99.94329/99.94023%이며 HWP33쪽은45.81889→99.93670%입니다. 새 review를 직접 열어 그림A/B/C 레이블, 그림B, 뒤본문 및 마지막 그림/제목의 정렬을 확인했습니다. 후보22 전체 진단은 양 형식89쪽씩 기록 뒤 최종 후보24 재검증을 위해 중지(exit143)했습니다. HWP는33쪽 하나가90미만, HWPX최저94.65215%였으며 partial은 전체 통과가 아닙니다. 후보24 Native 전체242쪽 두 입력과 fresh WASM 빌드를 실행합니다. 기존 기대값은 아직 변경하지 않았고 해결 건수도2/38로 유지합니다.

- 후보24 정책연구 Native26/27/52/53/54쪽99.97251/99.29352/99.12801/98.59349/94.72291%, fresh WASM99.97597/99.29412/99.05413/98.55875/94.68525%입니다. WASM27/52쪽 및 Native53쪽 review를 직접 열어 원본 번호/각주 본문 소유·기준선·묶음/캡션 위치를 확인했습니다. 영향5쪽·두 출력의 최저94.68525%로 기존 공간 표기 검사 재검토 조건을 충족합니다. PDF 원문은26)/60)/62) 뒤1공백이고 현재실제추출도 같으므로 기존3738의2함수에 있는3공백 문자열3개만1공백으로 정정합니다. 번호/본문/쪽소유/FootnoteArea/후속쪽 부재 assertion와 쪽수 상한은 유지합니다. 렌더 코드는 바꾸지 않고 새 함수/fixture/baseline은 추가하지 않습니다. 기존15case 및 캡션/선·바깥여백·흐름의 기존 대조군3case를 재실행합니다. #1733 전체쪽 기대값 갱신은 별도 전체 gate를 기다립니다.

- 후보24 기존18case/109함수 모두PASS(exit0)입니다.3738도33/33PASS로 정상 쪽 소유·각주 경계를 유지합니다. 앞선 채팅의 후보22 총계100PASS/2FAIL은 단수 `1 test` 요약4개를 빠뜨린 집계이며 정확히104PASS/2FAIL입니다. 수정 전후 실제 exit/개별 summary를 output에 보존했습니다. 세 Clippy 단계/workspace build/base 고정 manifest 등 필수 게이트를 순차 실행하며 전체242쪽 Native/fresh WASM은 아직 진행 중입니다.

- 후보24 필수 Rust 게이트9개 모두exit0입니다: 파생 suite 준비, fmt/fmt check, 기본/WASM/workspace all-targets 세 Clippy, workspace build, base443844b593c62a722cf9cc3d9d0256e94ab88cb8 고정 manifest check, diff check. 실제 명령·종료·시간은 candidate24/gates-results.json입니다. 소스의#[cfg(test)] 변경은 없으며 source unit-tier 비교의 추가 조건은 비해당입니다. 전체 시각 검증과 #1733 기존 기대값 보완은 아직 완료하지 않아 PR 준비 완료로 쓰지 않습니다.


- 후보25 사전 분석: 후보24 전체/분할 비교에서 HWP110쪽48.33396%의 반례를 직접 확인했습니다. HWPX는 동일 본문2226을109쪽0..3/110쪽3..4로 소유하나 HWP는0..1/1..4로 두 줄을 일찍 이월합니다. 양 원본의 저장 본문67520/69440/71360/0HU와 현재flow900.3px는 같으며, PDF110쪽은 마지막 ‘시해야 한다.’ 한 줄로 시작합니다. 원인은 본문2202의 각주25가108/109쪽으로 나뉜 뒤 `native_hwp5_existing_body_footnote_area_height`가 fragment를 모두 거절해 본문 reset 경계와 각주 위의 저장 tail 수용이 꺼지는 것입니다. 실제109쪽 각주 높이35.4px/시작973.7px(본문 상대) 앞의 본문 끝967.5px는 들어가고, 다음 줄 끝993.1px는 침범합니다.
- 보완 경로는 기존 `native_hwp5_footnote_reset_fragments` 또는 기존 두 줄 분할 계약이 생산한 **동일한** fragment/높이 → 기존 Body 각주 영역의 exact 높이 → reset 앞/뒤 본문 충돌 검사 → 기존 줄 스캔/소유 → 실제 배치입니다. 일반 HWP vpos=0 reset의 수용 조건을 넓히지 않으며 알 수 없는 fragment·셀/글상자 출처는 재측정 완료로 판단하지 않습니다. 후보24의8개 시각 프로세스와 분할 driver는 이 실행 반례로 종료하고 부분 증거를 보존합니다. 첫 분할 시도의 절대 실행파일 경로는 resume 출처 불일치로 거절됐으며 원 경로로 수정한 재실행과 구분합니다. 테스트 기대값/해결 건수는 변경하지 않습니다.


- 후보25 Native build exit0(1분33초)입니다.108/109/110/111쪽98.54181/99.98557/99.97870/97.84527%이고 영향4쪽 gatePASS/exit0입니다.110쪽48.33396→99.97870%이며109/110 review를 직접 열어 현재 쪽의 끝3줄, 이어받은 각주25의 구분선/본문, 다음 쪽 마지막1줄/표제와 뒤 문단 위치를 확인했습니다. 양 형식은242쪽이고2226의0..3/3..4 소유가 PDF와 같습니다. 필수 정상 대조/fresh WASM 및 최종 전체 비교는 새 후보에서 진행하며 이전 후보의 부분 통과를 재사용하지 않습니다.


- 후보25 기존18case/109함수 모두PASS/exit0입니다. #1733 기존2함수는 아직 수정하지 않았습니다. 전체 검증이 통과하면 두 형식의 count242뿐 아니라 독립 PDF56/57·109/110·200/201·219/220쪽의 실제 본문 컷/후속 표·문단 소유를 기존 함수 안에서 확인하도록 보완합니다. count만 통과했던 HWP의 추가45/46쪽과 잘린220쪽이 상쇄하는 거짓 양성을 막기 위한 검사이며 새#[test]를 추가하는 작업이 아닙니다. 독립 PDF 텍스트/해시와 원본 줄은 candidate25/independent-owner-oracle.json에 모았습니다. 보정 전 소스는 head80f7e778a의src/Cargo 파일을 output의before-source에 고정해 같은 보완 검사의 수정 전 실패도 실제 실행할 준비를 했습니다.


- 후보25 fresh WASM wrapper는exit0(6분04초)이고 고정package에서 흑백 key/inline 바이트 동일·원본과 변형 분리·중복 효과 제거를 다시 확인했습니다. 독립 PDF215쪽의1bit Gray 이미지(xref1543)와613×396/242,748픽셀 전부 같습니다. 실제 배경 그림은 Flow compact 조회에 포함되지 않으므로 compact 출력 일치의 검증으로 쓰지 않습니다. 정책연구 영향5쪽 Native/fresh WASM은 다시 모두gatePASS(최저94.72291/94.68525%)이며 새WASM27쪽을 직접 판독했습니다. 일반/WASM/workspace Clippy와 workspace build/base 고정 manifest 등9개 lint 게이트도exit0입니다. #1733 전체242쪽 및 기존 함수 보완/수정 전후실행·최종 전체 회귀는 여전히 미완료입니다.


- 보정 전 실행 준비의 첫 library build는 snapshot에 `saved/blank2010.hwp` 연결이 없어exit101이었습니다. 원본saved의tracked 상태가 unchanged임을 확인하고 출력용snapshot에만 연결한 뒤 재실행했습니다. 이 빌드 오류는 의도한 회귀 재현으로 세지 않으며 before-library-first-attempt 기록과 실제 테스트 실패를 구분합니다.


- 후보26 사전 분석: 후보25 전체 새캡처에서 HWP192쪽83.13857%를 직접 확인했습니다(HWPX97.35340%). 원본4175는 명시적 새 쪽에 있는 한 줄 표제 ‘2 구명설비의 상세’(vpos0/lh1200HU)와 단 상대 자리차지 RowBreak 표이며 표의문단오프셋2833HU/바깥위여백141HU입니다. Native HWP 상단113.4px, HWPX115.2px, 독립 PDF 괘선87pt=116px이며 약1.9px 누락은 첫/이어지는조각의 같은 프레임 여백입니다.193쪽도Native75.6/PDF77.33px이고 HWPX77.5px입니다.194/195쪽은 각각99.52959/99.93955%로 다른 호스트 경로입니다.
- 공통 `column_rowbreak_fragment_opens_outer_top`은 HWPX 또는 HWP의빈개체호스트만 받아 가시 표제의 명시적 page-top 프레임을 버립니다. 새 수용 근거는 원본·단일실제저장줄·vpos0·명시적쪽나누기·단일표·표제잉크높이 이하가 아닌 양수문단오프셋입니다. 표제는 글줄을 소유하고 표의바깥여백은 별개 개체 프레임에 속합니다. 생산한 동일 조건을 fragment budget의 top overhead와 layout_partial_table의 실제 top이 소비하게 합니다. 일반본문/local reset·편집·합성·다줄/복수개체호스트는 그 page-top 근거가 없으므로 확대하지 않습니다. 기존 분할행/내용컷을 그대로 두며 다른 좌표를 임의로 clamp하지 않습니다. 후보25 부분비교와driver를 실행 반례로 종료하고 기존 기대값/해결건수는 유지합니다.

- 후보26 호출 경로 보완 기록: 첫 조각의 바깥위여백141HU는 기존 `table/host_spacing.rs`의 before 간격과 `partial_rowbreak_fragment_spacing_px`에서 이미 예약합니다. 첫 조각은 실제 paint의 동일 여백 수용 누락을 고치고, 이어받기는 공통 helper를 통해 fragment budget 예약과 paint가 함께 같은 여백을 소비합니다. 첫 조각의 예산을 중복 추가하지 않습니다. Native 빌드 exit0을 확인했으며191–196쪽 양 형식과 기존18case를 새 소스에서 재실행합니다.

- 후보26 영향191–196쪽 Native 양 형식 gatePASS/exit0입니다. HWP192쪽83.13857→97.23828%로 개선됐고 새192/193 review를 직접 판독해 첫/이어받은 표의 상단, 행의 보존, 표제와 하단 괘선을 확인했습니다. 쪽수·내용컷과 다른194/195쪽 경로를 유지하며 최종전체/fresh WASM/기존 정상 대조를 진행합니다.

- 후보26 기존18case/109함수 모두PASS/exit0이며 새 테스트 함수는 추가하지 않았습니다. 정책연구 Native 영향5쪽은 최저94.72291%/gatePASS이고 HWP/HWPX 모두242쪽이며 독립 PDF 경계의 본문 소유가 일치합니다. fresh WASM과 전체 비교를 계속 진행합니다. 보정 전 library snapshot의 누락된saved 연결을 보완한 두 번째 빌드도exit0(203.64초)로 완료됐으며 실제 수정 전 회귀 검사와 구분합니다.

- 후보26 fresh WASM wrapper exit0(345.3초), 정책연구 영향5쪽 양 backend 최저94.72291/94.68525%/gatePASS입니다. 새WASM27 review를 직접 판독해 묶음/캡션 및 각주26의 위치를 확인했습니다. 흑백 key 바이트는 두 기준 PDF215쪽 xref1543과 각613×396/242,748픽셀 전부 일치합니다.9개 Rust lint 게이트도exit0이며 fixed base443844b593c62a722cf9cc3d9d0256e94ab88cb8를 유지합니다. fmt 뒤에도 Native/fresh WASM 고정 후보의 모든 변경 소스 해시가 현재 파일과 같습니다. 기존 #1733 두 함수는 전체 시각 검증 전이므로 변경하지 않았습니다.

- 후보27 사전 분석: 후보26 전체의196쪽 HWP91.42464/HWPX91.95959%를 직접 판독한 뒤 점수만으로 완료하지 않습니다. FootnoteArea y1026.4/h58.4의 실제 마지막 줄 y1085.4/h10.7이 예약 끝1084.8을 약11.3px 넘습니다. 원본65번 각주는 두 문단이고 둘째 문단의 앞 간격852HU, 저장 원점1892HU=첫 줄800+간격240+앞간격852입니다. 독립 PDF도 첫/둘째 줄 약25.3px 차를 유지합니다. 현재 실제 generic paragraph paint는 앞 간격을 적용하지만 `estimate_footnote_area_height_with_metrics`는 줄 높이·줄간격만 더합니다. 결과적으로 각주 전체를11.36px 낮게 bottom-anchor합니다.
- 보완 경로는 선택 문단의 번호/이어받기 경로·앞/뒤 간격을 공통 계획으로 생산 → table 각주 budget/FootnoteArea 측정 → generic paint의 명시적 간격 소비입니다. 기존 번호 전용 첫 문단의 간격0, 줄 중간 이어받기의 앞 간격0, 구분선 없는 영역 상단의 기존 앞 간격 생략은 유지하고 원본 문단 ID/852 상수로 분기하지 않습니다. 일반 문단 엔진에는 계산한 명시적 간격을 전달해 재판정·중복 가산을 피합니다. 후보26의 전체8worker/driver를 실행 반례로 중지하고 부분 증거와9lint/109PASS·fresh WASM 결과를 보존합니다. #1733 두 함수/해결 건수는 변경하지 않습니다.

- 후보26 종료 때 부분 비교는 Native HWP174쪽/HWPX175쪽, fresh WASM102/105쪽입니다. HWP206쪽63.41295%가 추가로 확인됐고 review 직접 판독에서 PDF의 첫 줄 ‘질 수 있다.’가 현재 쪽에 없으며 아래 표/뒤 문단이 약한 줄 위로 밀립니다. 원본4447의 양 형식 저장 줄61440..71040/0HU는 같지만 HWP는205쪽 Full, HWPX는205쪽0..6/206쪽6..7을 소유합니다. 각주 앞뒤 간격 보완의196쪽/정상 대조 확인 뒤 이 별도 본문 reset 경계를 해결합니다. 현재 어떤 부분 통과도 #1733의 전체 완료로 세지 않습니다.

- 후보27 Native build exit0(86초),195–197쪽 양 형식 gatePASS/exit0입니다.196쪽 HWP91.42464→99.40261%, HWPX91.95959→99.52606%이며 새HWP196 review를 직접 판독했습니다. 기존3738도33/33PASS(exit0)입니다. 첫 좌표 진단은 소수1자리 JSON의 네 값을 합해0.1px 반올림 차이를 실제 overflow로 오인해 실패했으므로 허용치를 바꾸어 완료하지 않았습니다. 동일 현재 library의 공개 Native tree API를 진단용으로 직접 호출해 실제f64 좌표를 비교했고 양 형식 각주 끝/영역 끝 모두1084.733333333333px(차0)에 맞음을 확인했습니다. 진단은output에만 저장했으며 새#[test]는 추가하지 않았습니다.
- 후보28 사전 분석:206쪽 원본4447의7줄은61440..71040/0HU이며 현재 실제flow819.2px=첫 원본61440HU입니다. HWP/HWPX 저장 줄·본문·한컴 PDF의205/206쪽 소유는 같으나 anchored reset 생산자와 scan 소비자가 HWP의 일반 본문을 형식만으로 거절합니다. ‘HWP는 bodyless각주 또는 양수 이월 프레임일 때만 원본zero-reset을 쓸 수 있다’는 가정을 제거하고 같은 미편집 원본 줄/실제flow 일치·control 제한·양수prefix/원본reset 계약을 두 형식에 적용합니다. 실제flow가 원본 앵커와 안 맞는 local cursor, 편집/합성·표/그림·다단 조건은 그대로 거절합니다. 새 문서ID/글꼴크기/tag 조합으로 예외를 추가하지 않습니다.196쪽 공통 간격 계획의 저장 빈 줄 경로도 측정이 실제 소비하는 raw 높이/무후행간격을 그대로 사용하도록 정리합니다.

- 후보28 Native build exit0(81초)입니다. 양 형식242쪽을 유지하고4447을205쪽0..6/206쪽6..7로 소유합니다.191–197/205–207쪽 Native 양 형식 gatePASS/exit0, 최저HWP97.23828/HWPX97.35340%입니다.206쪽 HWP63.41295→99.97910%, HWPX100.00000%이고 새HWP206 review를 직접 판독해 첫 본문 tail, 캡션/표 상단/하단과 뒤 문단 위치가 보존됨을 확인했습니다.196쪽99.40261/99.52606%도 유지합니다. 일반 HWP 원본zero-reset의 형식별 거절을 제거한 뒤 기존18case/109함수 및 final Native/fresh WASM 전체 비교를 다시 진행합니다. 기존1733두 함수는 아직 변경하지 않았고 해결 건수도2/38입니다.

- 후보28 기존18case/109함수는 전부PASS/exit0이며 fresh WASM wrapper도exit0(314.8초)입니다. 정책연구 영향5쪽 Native/fresh WASM gatePASS이고 새WASM27쪽을 직접 판독했습니다. 흑백 이미지 key/inline 바이트 검사는exit0이며 두 독립 PDF215쪽 xref1543과 각각613×396/242,748픽셀이 모두 같습니다. compact 배경 조회의 검증으로 확대하지 않습니다. Native/WASM/workspace Clippy를 포함한9개 lint 게이트가 모두exit0이며 source/test 기대값과 해결 건수는 전체242쪽 비교 종료 전까지 유지합니다. 현재 immutable 후보28의4개 전체 비교는 진행 중입니다.

- 후보28 전체 페이지 비교는242쪽×4조합=968개 모두 점수를 산출했고90미만/측정불가0쪽입니다.8개 Native/WASM 앞·뒤 worker는 모두exit0입니다. 결과를 합치던 출력 전용 driver는 원 SVG/render_tree를 hardlink한 파일을 같은 inode로 copyfile해 SameFileError(exit1)였습니다. 렌더링/게이트 실패로 세지 않고 최초 오류를 보존하며, 동일 inode는 복사를 생략하는 별도 finalizer에서 같은 SHA/출처의 체크포인트를 합친 후4개 전체 `--resume` 게이트를 재실행합니다. 이 최종4개 종료 전에는 전체 시각 완료로 판단하지 않습니다.

- 출력 전용 finalizer의 최초 보완도 복사 if의 들여쓰기가 바깥으로 나가 마지막 analysis 파일만 합쳤습니다. `--resume`가 tail PNG 누락을 정상 검출한 로그를 보존했고, 불필요한 재raster를 피하려고 정확한 소유4검사+driver만 종료(exit143)했습니다. 원본 shard의8개exit0/유효캡처는 그대로이며 source/실행파일/pkg/임계값은 바꾸지 않았습니다. 복사를 각 artifact 루프 안으로 옮겨4조합×242쪽의8종 산출물 누락0을 확인한 뒤 최종 `--resume`를 새 로그(full-final2)로 재실행합니다. 최초 오류를 성공으로 바꾸어 기록하지 않습니다.

- 후보28 최종4개 전체 `--resume`는 모두exit0이며각 run_state=complete/pr_review_gate=passed입니다. 원본HWP/HWPX와PDF는각242쪽,Native/freshWASM 전체968개 시각 점수의 최저는HWP91.19751%(215쪽),HWPX91.41441%(215쪽)이며90미만/측정불가0쪽입니다. 글꼴 예외는 쓰지 않았습니다. 영향/대표/최저점17개 review를 직접 판독했으며968개를 사람이 모두 한 장씩 읽었다고 주장하지 않습니다.45쪽 수식 자간/획,162쪽 일부 줄바꿈/각주 번호,201쪽 얇은 화살표/작은 글자,215쪽 지도 위 별도 글상자의 위치/줄바꿈과 프레임 잔여 차이는 보존합니다. 이미지215쪽의242,748픽셀 일치는 별도 원본 그림 변환의 증거이며 전체 배치 일치로 확대하지 않습니다.
- 전체 시각 통과 뒤 `tests/issue_1733.rs`의 기존2함수만 보완했습니다. 잘못된57/58·220쪽 소유를 독립 PDF의56/57·109/110·200/201·205/206·219/220쪽 대응으로 바꾸고, 분할 소유/후속 표·문단이 정확히1회 있는지 확인합니다.242쪽만 우연히 통과하던HWP의 거짓 양성도 검출하며196쪽 실제f64 각주 끝이예약끝을 넘지 않는지 검사합니다.1e-8은f64산술의표현차만 다루며baseline/시각임계값을 완화하지 않습니다. 새#[test]는0개이고 기대값은현재 구현 출력으로 재정의하지 않았습니다. 보정 전80f7e778a snapshot과현재 소스로 같은2함수를실행한뒤 최종9lint를재실행합니다.

- 기존2함수 보완 뒤 파생 suite의 배치가바뀌어 고정했던regression_suite_001의 보정전 첫 실행은0tests/185skipped,exit4였습니다. 빌드는exit0이나 의도한 회귀 재현이 아니므로 성공/실패증거에 넣지 않습니다. 최초로그/results를output에별도보존하고 `--prepare` 뒤 `resolveCasePlan(issue_1733)`의 실제target을기록해 같은두함수를80f7e778a/현재소스에서 다시실행합니다. 함수수2와 원본 fixture/시각임계값은 그대로입니다.

- 동적 target(regression_suite_026)으로 실제 보정 전2함수는0PASS/2FAIL,exit100입니다. HWPX243/242와여러본문컷/쪽소유의 오류를모두수집했고 HWP도count242의거짓양성을검출합니다. 뒤이어현재소스 실행은cargo가0.12초fresh로끝나면서같은보정전geometry를재사용해0PASS/2FAIL였습니다. 공유 `target/pr-review/release-test/deps/librhwp.rlib`가before-source 빌드에서덮어써진상태인데현재fingerprint가fresh여서그결과를현재코드의회귀로세지 않습니다. 해당실패/라이브러리SHA를보존하고 build.rs의내용은그대로/mtime만갱신해현재library를강제재빌드한뒤같은2함수와9lint를다시실행합니다. 공유target삭제나새target생성,소스/기준값변경으로해결하지 않습니다.

- 현재library 강제 재빌드는exit0(64초)이며라이브러리SHA변경/build.rs내용일치를확인했습니다. 같은2함수의현재소스 실행은2PASS/exit0입니다. 최종fmt가후보검사 파일의byteweight를바꿔suite가026→016으로이동해첫lint시도의manifest--check는exit1이었습니다(세Clippy/workspacebuild는exit0). 이정책실패를보존하고 formatted상태에서--prepare를다시실행한뒤9단계와동적target의기존2함수를재검증합니다. baseline/golden/임계값은바꾸지 않았습니다.


#### 보정58 최종 결과 — #1733 기존 검사와 본문·각주 경계

- 기존 두 회귀 함수를 보정 전 `80f7e778a`에서 실행해 **0 PASS / 2 FAIL(exit100)**, 현재 포맷팅된 소스에서 **2 PASS(exit0)**를 확인했습니다. 기존 18case/109함수와 최종 Rust lint 9단계도 통과했습니다. 원본 문서·기준 PDF·242쪽 기준은 유지했고 새 `#[test]`는 추가하지 않았습니다.
- #3738의 기존 두 함수에 있는 3공백 조건은 독립 PDF의 1공백 및 영향 5쪽의 Native/fresh WASM 비교 근거로만 수정했습니다. 쪽 소유와 각주 영역 검사는 유지했습니다.
- 전체 시각 검증은 각 242쪽 × HWP/HWPX × Native/fresh WASM = **968개**입니다. 90% 미만·측정불가·글꼴 예외는 모두 0쪽이며, 네 최종 게이트는 `passed`/exit0입니다. 대표·영향·최저점 review 17개와 standalone overlay 2개를 직접 판독했습니다. 전체 페이지를 사람이 한 장씩 읽었다고 주장하지 않습니다.

| 형식 | Native 최저 | fresh WASM 최저 | 쪽수/검증쪽 | gate |
| --- | --- | --- | --- | --- |
| HWP | 91.19751% (215쪽) | 91.19751% (215쪽) | 242/242 | passed |
| HWPX | 91.41441% (215쪽) | 91.41441% (215쪽) | 242/242 | passed |

- 구현 근거와 실제 호출 경로는 위 후보별 사전 분석·결과에 연결했습니다. 저장 reset/이월 개체 프레임과 실제 흐름의 일치로 본문 줄 소유를 확정하고, 같은 각주 간격 계획을 예약·배치에서 소비합니다. RowBreak의 실제 컷·물리 높이·공유 프레임 여백도 보존했습니다. 문서 ID/수치 조합의 예외나 좌표 clamp로 잘림을 숨기지 않았습니다.
- 영구 증적: [전체 결과·소스/입력/PDF/binary/pkg SHA·명령·페이지별 점수](../assets/pr7382_20260926/stage58_tail1733_validation.json), [독립 PDF/원본 저장 줄 기대값](../assets/pr7382_20260926/stage58_independent_owner_oracle.json), [직접 판독 기록](../assets/pr7382_20260926/stage58_direct_review.json), [각주 간격 보정 전 트리](../assets/pr7382_20260926/stage58_before_spacing196.json), [흑백 그림 픽셀 근거](../assets/pr7382_20260926/stage58_blackwhite_pixel_proof.json). 대표 Native/fresh WASM review·overlay는 같은 assets 폴더의 `stage58_` PNG이며 역할과 SHA는 전체 결과 JSON에 기록했습니다.
- 전체 compare/review/overlay와 로그는 `output/pr-review/planet6897-7382-20260926/stage58-tail1733/candidate28/visual-full-{native,wasm}-{hwp,hwpx}/tail1733-{hwp,hwpx}/`에 있습니다. `.log`·output·pkg·파생 suite는 커밋하지 않습니다.
- 잔여 차이: 수식 자간/획(45쪽), 일부 본문 줄바꿈/각주 번호(162쪽), 얇은 화살표/작은 글자(201쪽), 지도 위 별도 글상자/프레임(215쪽)을 기록했습니다. 원본 흑백 그림의 242,748픽셀 일치는 그림 변환 범위의 증거이며 전체 배치·compact 출력 일치로 확대하지 않습니다.
- 고정 38개 계획은 **3개 해결/35개 대기**입니다. HWP 함수는 원래 38개 밖의 거짓 양성 보완이므로 해결 건수를 두 개 늘리지 않습니다. 보정58 커밋 뒤 다음 실패 함수를 분석하고, 모든 개별 보정·고정 38개 재실행 뒤 전체 nextest·Native Skia3·최종 PR head 필수 검증을 수행합니다. 다른 문서의 쪽수·시각 보류는 유지하며 현재 PR 준비 완료로 판정하지 않습니다.

- 보정58 코드·증적 커밋은 `d6992abefb46744defce93b9333aff82d699cb97`입니다. 고정 Native/fresh WASM의 변경 소스 SHA와 최종 검사 SHA를 해당 커밋의 파일과 대조해 모두 같음을 확인했습니다. 캡처 당시 git HEAD(`80f7e778a`)+작업트리 출처와 코드 커밋의 연결을 결과 JSON에 기록했으며, 이후 최종 통합 PR head 검증을 대신하지 않습니다.


### 보정59 사전 분석 — #2006 PrEP 페이지 수 검사

- 시작 head `6a7278334`/코드 `d6992abef`에서 기존 `prep_1790387_page_count_pin`은 실제141/기준140쪽으로 FAIL(exit100)입니다. 입력 마지막 저장 제품은 한컴2024이며 동일 원본의 한컴2024 PDF(SHA256 `04b95a6e41420fb45934ce2ee5abd8cf6dac4ce12fd47977dacbe7fca28018a8`)도140쪽입니다. 2020 PDF140쪽과2022 글꼴 대조군146쪽을 혼동하지 않고2024출력을 이번 독립 기준으로 사용합니다. 원본 문서/140쪽 기대값/검사 함수 수는 유지합니다.
- `fidelity_compare --text-only --export-all-svg --layout-ledger` 전수 진단은exit0이고 기준140/실제SVG·tree141쪽을 분리 기록했습니다. 이exit0은 시각 통과가 아닙니다. 첫 지속적 후행 소유 후보는92쪽 그림14캡션→93쪽이며 이후 내용이1쪽 늦습니다.69/70쪽 표는 별도의 조기 배치 후보입니다.
- Native 영향7쪽(69/70/90–94) Visual Sweep은exit1입니다.69/70쪽62.45907/35.57223%,93/94쪽1.64872/27.83830%이고 새69/92 review를 직접 판독했습니다.69쪽 표 행이 압축되어 마지막 행들이 앞쪽에 당겨지고,92쪽 그림 본체는 같은 위치지만 캡션이 빠져 단독93쪽을 만듭니다. 원인 파악 전 테스트 기준을 변경하지 않습니다.
- 그림214의 원본 저장 호스트46384HU에는 앞 간격500HU가 포함되며 offset504+높이20145HU 뒤 빈 줄은66533HU입니다: `(46384-500)+504+20145=66533`. 빈 줄1050HU와 간격−420HU는 다음 캡션67163HU의630HU 전진을 독립적으로 입증합니다. 실제 그림 바닥과 빈 줄의 원점은 일치합니다. 현재 일반 그림 pushdown은 이미 소비한 호스트 줄 높이를 다시 더하고, 저장 successor 계약은 양수 앞 간격만 허용해 이0앞간격 빈 줄을 거절합니다. 예약 생산자(확정 그림 프레임)→controls의흐름종료→paragraph fit→layout의같은placement 종료를 추적해 보완합니다. 단순 기하 접촉만으로 흐름을 소유한다고 확장하지 않고 후속 원본 줄의 전진까지 대조합니다. 편집/합성·TAC·내부캡션·줄소유 불일치는 기존 경로에 남깁니다.
- 관련 기존17함수를 보정 전 실행 중입니다. 먼저92/93경계의 보정 방향과 정상 대조를 확인한 뒤69/70행 높이 소비를 분석합니다. 전체140쪽 Native/freshWASM 최저90이상 전에는 픽스쳐 검증 완료나 #2006해결/PR준비로 판정하지 않습니다. 증적과 모든 로그는 `output/pr-review/planet6897-7382-20260926/stage59-prep2006/`에 보존합니다.

- 후보1 build exit0이며140쪽으로 복원됐습니다. Native90–94쪽 gatePASS/exit0,92쪽94.95742% review를 직접 판독해 그림 본체 원점과 캡션 소유를 확인했습니다. 저장 successor의 양수 앞 간격만이 독점 흐름을 증명한다는 가정을 수정하고,0앞간격의 빈 줄은 후속 원본 줄까지의 전진과 실제 앞 커서가 일치할 때만 같은 프레임 끝을 소비합니다. 아직69/70쪽 결함과 전체 fixture 최저90 검증이 남아 #2006을해결로 세지 않습니다.
- 표 추가 사전 분석:69쪽38행 표의 일반 행은 저장cellSz1280HU, 실제 적용 표 안여백425+425HU이고 글줄1000HU입니다. 보정56 helper가 noAdjust만으로 점유끝1000HU를 반환해 본문 표의 물리 안여백850HU를 제외합니다. 이후 최소 선언/표 전체 높이 확대로 그 잘못된행합이809.3px에 맞춰져도 PDF의행경계·후속70쪽 소유는 복원되지 않습니다. noAdjust는 안여백이 줄좌표에 포함된다는 증거가 아닙니다. 기존중첩/TAC의relaxed-pad 계약과 본문 표의패딩 계약을 구분해, 저장 줄 프레임 조회가 여백을 이미 계상한 경로에서만 그 끝을 반환하도록 생산/측정/배치 네 소비점을 함께 고칩니다. #1658중첩하단틀은 원래공간 계약을 유지하며 기존대조검사를 다시실행합니다. 테스트 기대값/공차를바꾸지않습니다.

- 후보2의 표69/70쪽은99.94315/100%로 복원됐으며 기존 #2006·#7390·#1658 정상 대조가 통과했습니다. 다만 제가 저장 successor helper의 public 인자를6→8개로 바꿔 기존 #7048 검사 호출이 컴파일되지 않았습니다(exit101). 이는 결함 재현이 아닌 보정 작업의 API 오류이며 최초 로그를 보존했습니다. 기존6인자 API를 유지하고 다음 문단/실제 커서 검증을 사용하는 private helper로 분리한 후보3에서 기존10case/95함수 모두PASS(exit0)를 확인했습니다. 테스트 기대값/함수 수는 아직 변경하지 않았습니다.
- 글꼴 공급 분석: Windows 실제 H2HDRM/Dotum 파일도 Chromium에서 잘못된 format4 cmap 범위로 거절됐습니다. output의 검증용 사본만 Unicode→glyph 매핑을 디컴파일/재기록하고 KoPub 실제 face 이름으로 공급했습니다. H2의12,480 매핑 및 glyf/hmtx/vmtx/name/GSUB 바이트는 원본과 같고 실제 Chrome의 HYHeadLine-Medium/H2hdrM 선택·Dotum loaded·console error0을 확인했습니다. 글꼴 파일은 커밋하지 않으며 renderer의 일반 글꼴 처리 개선 완료로 확대하지 않습니다. 후보2 공급 재검증 표지91.56068%,69/70쪽99.94299/100%는 선행 방향 확인이며 최종 후보3의 전체 Native/fresh WASM을 대체하지 않습니다.
- Visual Sweep 빈 쪽 사전 분석: 전체 Native의2/4/10쪽에서 양쪽 이미지의 모든 채널이255로 완전히 비어 있고 독립 PDF에도 같은 빈 쪽이 있습니다. PNG 높이1123/1122의 기존 DPI 반올림 차이는 기록했으며 바이트 동일이라고 주장하지 않습니다. 내용 union=0을None으로 반환해 정상 빈 쪽이 측정 불가로 보류되는 도구 오류를 확인했습니다. 최초 partial과 로그를 보존하고 정확한 소유 Native worker/driver만 종료했습니다. 양쪽 빈 실루엣은100%,한쪽만 비면0%로 계산하고 캡처/지표 누락 보류는 유지합니다. 기존 Python 검사 함수 안에서 양쪽 빈 쪽/한쪽만 빈 쪽을 검증했으며 새 테스트 함수는 추가하지 않았습니다. 새 도구 SHA에서 whole140쪽 Native/fresh WASM을 다시 시작했습니다.

- 후보3 Native 전체140쪽은run_state=complete/gate=passed/exit0이며 최저91.82221%(1쪽),90미만/측정불가/글꼴예외0쪽입니다. 정상 #1658 Native/fresh WASM도97.60662%/exit0이며 새 하단 표 review를 직접 판독했습니다. Python 도구의 같은 기존 함수는 수정 전None !=100으로FAIL/수정 후PASS이고 기존77함수 전체도PASS입니다. Rust9lint도base443844b593c62a722cf9cc3d9d0256e94ab88cb8에서exit0입니다. fresh WASM 전체는진행중이며 아직 #2006완료·기준설명갱신·해결건수증가·커밋은하지않았습니다.


#### 보정59 최종 결과 — #2006 140쪽 검사

- 기존 `prep_1790387_page_count_pin`의 **140쪽 기대값은 정확**했습니다. 보정 전 141쪽/FAIL(exit100)의 원인은 그림 호스트·빈 후속 줄의 중복 전진과 본문 표의 안 여백을 누락한 메인터너 보정입니다. 저장 프레임 끝과 실제 앞 커서/다음 원본 줄 전진이 일치할 때 같은 그림 계획을 예약·배치에서 소비하고, noAdjust만으로 본문 표 안 여백을 제외하던 가정을 제거했습니다. 문서 ID/수치 예외나 좌표 clamp를 추가하지 않았습니다.
- 같은 원본의 한컴2024 PDF **140쪽**을 기준으로 Native/fresh WASM 각 140쪽, **280개 전체 비교**가 `complete`/`passed`/exit0입니다. 두 출력의 최저는 **91.82221%(1쪽)**이며 90% 미만·측정 불가·글꼴 예외는 모두 0쪽입니다. 대표/영향/최저점 review 14개와 standalone overlay 2개를 직접 판독했으며 280개를 사람이 모두 한 장씩 읽었다고 주장하지 않습니다.
- 기존 관련 10case/95함수 PASS, 최종 #2006 개별 1함수 PASS, 기준 설명 갱신 뒤 Rust lint 9단계 PASS입니다. 정책 base는 `443844b593c62a722cf9cc3d9d0256e94ab88cb8`이고 파생 suite를 다시 준비해 실제 `regression_suite_007`에서 개별 실행했습니다. 기존 140쪽 값/원본/검사 함수 수는 유지하고, 독립 2024 PDF 출처와 nextest 검증 명령만 설명에 반영했습니다.
- 새 테스트 함수는 0개입니다. Visual Sweep의 양쪽 실제 빈 쪽은 100%, 한쪽만 비면 0%로 계산하도록 도구 오류를 고쳤고, 같은 기존 Python 함수의 수정 전 FAIL/후 PASS와 기존 77함수 PASS를 확인했습니다. 누락된 캡처/지표의 보류와 90% 기준은 유지합니다. 빈 쪽 2/4/10의 RGB255 근거와 DPI 반올림 높이 1123/1122 차이를 기록했으며 PNG 바이트 동일이라고 주장하지 않습니다.
- 글꼴 검증용 사본은 output에만 둡니다. H2의 12,480 Unicode 매핑과 glyf/hmtx/vmtx/name/GSUB, TTC 각 face의 Unicode 매핑/윤곽/폭을 유지해 잘못된 cmap을 재기록했고, 실제 Chrome의 H2hdrM·KoPubDotumLight 및 Dotum loaded를 확인했습니다. 이 환경 보완을 renderer의 일반 글꼴 처리 개선 완료로 확대하지 않으며 글꼴 바이너리를 커밋하지 않습니다.
- 남은 차이는 표지의 일부 영문/번호 자간과 약1–2px 기준선, 작은 표·그림 문자의 획, 괘선 색/두께, 105쪽 Σ/원문자 인접 자간입니다. 그림14/15 캡션 소유, 69→70쪽 마지막 행/머리행/뒤 문단, 105쪽 병합 셀/총액 위치는 새 양 출력에서 확인했습니다. 정상 #1658의 Native/fresh WASM 97.60662%와 하단 글줄/틀도 보존됐습니다.
- [소스/입력/PDF/binary/pkg/글꼴 SHA·전체 점수·명령·종료·직접 판독·실제 호출 경로](../assets/pr7382_20260926/stage59_prep2006_validation.json)에 연결했습니다. 대표 PNG는 같은 assets 폴더의 `stage59_after_`이며 전체 이미지·로그는 `output/pr-review/planet6897-7382-20260926/stage59-prep2006/candidate3/visual-final-{native,wasm}/prep2006/`에 있습니다. `.log`·output·pkg·글꼴·파생 suite는 커밋하지 않습니다.
- 고정 38개는 **4개 해결/34개 대기**입니다. 이번 코드·결과를 커밋한 뒤 #7390의 105쪽 기존 함수를 개별 분석·실행·기록합니다. 관련 검사 묶음에서 이미 통과했다는 이유로 대기 중인 다른 함수를 해결로 세지 않습니다. 고정 38개 개별 완료 뒤 각각 재실행하고 전체 nextest/Native Skia3/최종 PR head 검증을 수행하며, 현재 전체 PR 준비 완료로 판정하지 않습니다.

- 보정59 코드·결과 커밋은 `fd10e7fd9d9fda41648b0d8ba2f57d219ae8d197`입니다. 고정 소스/최종 검사/Visual Sweep 도구/대표 asset 1064개 SHA256이 해당 커밋 blob과 모두 같음을 확인했습니다. 캡처 당시head+작업트리 출처는 보존하고 코드 커밋과의 연결을 결과 JSON에 남겼습니다. 이후 최종 통합 PR head 검증을 대신하지 않습니다.


### 보정60 사전 분석 — #7390 105쪽 병합 연구비 표

- 대상은 고정38의21번 `prep_page_105_merged_budget_table_keeps_saved_row_boundaries`입니다. 보정 전 함수는105쪽에서 표를 찾지 못해FAIL(exit100)이며, 동일 before tree에서20×6/문단336 표는106쪽에 있습니다. before106/after105의 표 bbox와 행1/10/19 좌표는같습니다. 이번 실패를 병합 행 높이 오류로 해석하지 않고 앞92쪽 그림14 캡션의 추가 단독 쪽으로 인한1쪽 지연으로 구분합니다.
- 독립2024 PDF105쪽의96dpi 괘선은243.2533/279.0547/634.5053/975.4133/1019.2053px입니다. 기존 기대279/634/975/1019px와2px 공차는 적절하며, 현재 행279.5/635.3/976.5px·표끝1020.4px도 범위 안입니다. 표끝 약1.2px 차이와Σ/원문자 자간은 잔여 차이로 기록합니다. 기대값·공차·함수 수를 변경하지 않습니다.
- 보정59의공통 그림 successor 흐름수정이이미 적용됐으며 추가 코드수정은필요없습니다. 현재head `d4d16ca1f`의src SHA가보정59 snapshot/코드 `fd10e7fd9`와일치함을확인한뒤 같은원본/PDF의전체280쪽90이상증적과양105쪽직접review를재사용합니다. 새로운부분캡처만으로전체근거를대체하지않습니다. 현재함수1개를동적suite에서nextest로실행하고결과를기록·커밋하기전다른대기함수를해결로세지않습니다.

- 보정60 최종: 기존105쪽 함수 **1PASS/exit0**입니다. 수정 전과현재 검사 소스SHA가같고 기대좌표/2px 공차/함수 수는변경하지않았습니다. 생산 코드도보정59와같아추가renderer수정·중복전체sweep은하지않았습니다. [독립PDF 괘선·before105/106/after105 소유·개별 명령/종료·재사용 소스 SHA](../assets/pr7382_20260926/stage60_budget105_validation.json)에기록했습니다. 전체280쪽최저91.82221%,105쪽양출력99.89982%와직접판독이미지는보정59증적을재사용합니다. 표끝약1.2px/원문자자간잔여차이를보존하며골든/공차를완화하지않습니다. 고정38은 **5해결/33대기**입니다. 결과를커밋한뒤92쪽함수를따로진행합니다.


### 보정61 사전 분석 — #7390 92쪽 음수 간격과 그림14 캡션

- 고정38의22번 기존 함수가 대상입니다. 동일 독립2024 PDF92쪽 캡션 yMin742.872009pt는96dpi990.496012px로 기존990.496px/1.5px 공차를 입증합니다. 원본section3의215/216 저장원점66533/67163HU와215의줄상자1050HU·간격−420HU는630HU 전진으로 닫힙니다. 빈 문단을0높이로 처리하거나 음수간격을 양수로 바꾸지 않습니다.
- 수정 전92쪽에서캡션을찾지못한FAIL과93쪽단독캡션은같은원인입니다. 보정59의동일그림계획/앞커서 검증으로92쪽 소유가복원돼추가코드수정은필요없습니다. 기대990.496/공차1.5/기존함수수는유지합니다. 동일srcSHA의전체280쪽/양92쪽직접판독근거를재사용하고해당함수1개만개별실행한뒤결과를기록·커밋합니다.


#### 보정61 결과 — #7390 92쪽 캡션

- 기존 함수 개별 실행은 **1 PASS/exit0**입니다. 독립 한컴2024 PDF 캡션742.872pt→990.496px와 원본 빈 줄1050−420=630HU가 기존 기대값을 입증합니다. 기대990.496/공차1.5/함수 수는 변경하지 않았습니다. 보정 전92쪽 캡션 누락·93쪽 단독 캡션과 보정 후92쪽 y990.0px를 같은 원본에서 대조했습니다.
- 추가 구현 없이 보정59(`fd10e7fd9`)의 그림 프레임/앞커서/원본 후속 줄 계약으로 해결됐습니다. 전 src SHA가 같음을 확인해 전체140쪽×Native/fresh WASM=280개 증적을 재사용했습니다. 양쪽 최저91.82221%,92쪽96.95925%,직접 판독 Native/WASM review·Native overlay92 근거는 [보정59](../assets/pr7382_20260926/stage59_prep2006_validation.json)에 있습니다. 모든280쪽을 수동 판독했다고 주장하지 않습니다.
- [개별 분석·명령·전후 좌표·SHA 결과](../assets/pr7382_20260926/stage61_caption92_validation.json). 로그는 `output/pr-review/planet6897-7382-20260926/stage61-caption92/individual.log`에만 보존합니다. 캡션 약0.5px와 작은 글자/괘선 차이를 남겼습니다. 고정38개는 **6개 해결/32개 대기**이며 다음은93쪽 함수입니다. 전체 회귀/최종 PR head 검증 전까지 PR 준비 보류를 유지합니다.


### 보정62 사전 분석 — #7390 93쪽 그림15 및 후속 문단

- 한컴2024 PDF93쪽의 캡션429.071991pt→572.095988px,첫 문단459.767975pt→613.023966px,CDC 문단514.368042pt→685.824056px는 기존572.1/613.0/685.8 기대값과1.5px 공차의 독립 근거입니다. 원본 저장 줄은 캡션35775HU,첫 문단38849HU,CDC44309HU이며 수정 전94쪽의571.5/612.5/685.3px와 수정 후93쪽의같은좌표를 대조했습니다.
- 수정 전93쪽은 그림14 단독 캡션 때문에 그림15/뒤 문단이없어 기존함수가FAIL입니다. 보정59로앞쪽 소유가복원돼좌표/공차/검사수 변경과 추가구현은 필요없습니다. 동일srcSHA의전체280쪽/양93쪽직접판독 근거를 재사용하고 해당함수만 개별 실행한 뒤 결과보고·커밋합니다.


#### 보정62 결과 — #7390 93쪽 캡션과 후속 문단

- 기존 함수 개별 실행은 **1 PASS/exit0**입니다. 독립 PDF/실제 캡션572.096/571.5,첫 문단613.024/612.5,CDC685.824/685.3px를 대조했고 세 요소 모두93쪽에 돌아왔습니다. 기존 기대값572.1/613.0/685.8과1.5px 공차 및 검사 함수 수를 유지했습니다.
- 추가 구현 없이 보정59의 공통 그림 흐름 보정으로 해결됐으며 전 src SHA가 같음을 확인했습니다. 전체140쪽×Native/fresh WASM=280개 증적을 재사용하고 양쪽 최저91.82221%,93쪽99.39962% 및 이미 직접 판독한 양출력 review를 연결했습니다. 약0.5~0.6px와 작은 글자/괘선 차이는 남았습니다.
- [개별 분석·독립 좌표·명령·전후 소유·SHA 결과](../assets/pr7382_20260926/stage62_caption93_validation.json). 로그는 `output/pr-review/planet6897-7382-20260926/stage62-caption93/individual.log`에만 있습니다. 고정38개는 **7개 해결/31개 대기**입니다. 결과 커밋 뒤 KoPub 영문 줄 폭 함수를 분석하며 전체 검증/PR 준비 보류는 유지합니다.


### 보정63 사전 분석 — #7390 KoPub 영문 줄 폭

- 고정38의19번 함수는 원래 전체 실행에서94쪽 대상 줄 조회가None여서FAIL이며 폭 assertion에 도달하지 않았습니다. 보정 전94/108쪽에 대상이 없고95/109쪽에 동일 줄이 있음을 실제 트리로 확인했습니다. 보정59의 쪽 소유 수정 후94쪽 점유폭414.2px,108쪽574.1px가 복원됐습니다.
- 독립2020 PDF 잉크폭412.5467/570.6667px는 기존412.55/570.67 기대값의 근거이며2024 PDF도412.2458/571.0213px로 대조됩니다. 잉크폭과 마지막 글자 전진을 포함하는 점유폭을 구분하는 기존10px 공차 및 쪽너비 비례식을 유지합니다. 원문·baseline·검사 함수 수는 바꾸지 않습니다.
- 동일srcSHA의전체280쪽 최저91.82221% 증적을 재사용합니다. 새로 직접 판독한94/108쪽 Native/fresh WASM review4개와WASM standalone overlay108에서 줄 끝·줄바꿈·후속내용 소유를 확인했습니다. 양94쪽99.57016%,양108쪽99.83575%이며 글자 획/약1px 높이·제목 위 얇은선 차이는 남습니다. 해당 함수만 개별 실행한 뒤 결과보고·커밋합니다.


#### 보정63 결과 — #7390 KoPub 영문 점유폭

- 기존 함수 개별 실행 **1 PASS/exit0**입니다. 원래 실패는94쪽 줄 소유이며 글꼴 폭 실패로 확대하지 않았습니다. 같은 줄의 점유폭414.2/574.1px와2020 PDF 잉크폭412.5467/570.6667px,2024 PDF412.2458/571.0213px를 대조해 기존 기대폭/10px 공차/쪽너비 비례식의 유효성을 확인했습니다. 코드·검사·baseline·함수 수는 바꾸지 않았습니다.
- [독립 폭·전후 쪽 소유·개별 명령/출처/SHA·직접 판독 PNG](../assets/pr7382_20260926/stage63_kopubwidth_validation.json). 새로 판독한94/108쪽 양출력 review4개와WASM overlay108을 영구 assets에 보존했습니다. 전 src SHA가 보정59와 같아 전체280개 비교를 재사용하며 양쪽 최저91.82221%,94쪽99.57016%,108쪽99.83575%입니다. 글자 획/약1px 높이·얇은선 잔여 차이를 기록했습니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage63-kopubwidth/individual.log`에만 있습니다.
- 고정38개는 **8개 해결/30개 대기**입니다. 개별 결과를 커밋한 뒤 기존 공백 함수로 진행합니다. 전체38개 재실행과 전체회귀/최종head검증 전 PR 준비 보류를 유지합니다.


### 보정64 사전 분석 — #7390 KoPub 공백 폭 하한

- 고정38의20번 기존 함수도 원래 실패는108쪽 대상 줄 조회None이며555px 하한 assertion에 도달하지 않았습니다. 보정59 후 동일 줄의 점유폭은574.1px로 복원됐습니다. 동일srcSHA 전체280쪽 최저91.82221%,108쪽 양출력99.83575%와 보정63에서 직접 판독한 양review/standalone overlay를 재사용합니다.
- 원본charPr28은1100HU/장평98/자간−5/useFontSpace0이며 대상 줄의 실제 공백은 **11개**입니다. 기존 주석의9개/약25px는 잘못되어11개/약33px로 바로잡습니다. 기대하한555px/함수 수/렌더러는 그대로입니다. PDF 공백glyph상자와 다음 글자origin 전진은 다르므로 잉크폭을 자연공백 폭으로 대체하지 않습니다. 해당 하한은 전체 줄 축의 감소 검출이며 개별 공백 전진의 정확한 등식을 입증한다고 확대하지 않습니다.
- 같은 독립 PDF의11개 공백origin 전진과 원본 텍스트/charPr를 기록하고 주석만 수정한 후 해당 함수1개를 실행합니다. Rust test source 변경이므로 파생suite 준비·fmt·세Clippy·workspacebuild·base고정 정책 검사도 순차 완료한 뒤 결과보고·커밋합니다. 새 테스트는 추가하지 않습니다.


#### 보정64 결과 — #7390 공백 하한

- 기존 함수 개별 실행 **1 PASS/exit0**,필수 Rust 검증9단계 **모두PASS**입니다. 정책 base는 `443844b593c62a722cf9cc3d9d0256e94ab88cb8`이며 실제 파생target은 `regression_suite_012`입니다. 원본/PDF 공백11개와useFontSpace0을 근거로 주석9→11/예상폭감소약25→33px만 수정했습니다. 검사555px 하한/생산코드/검사 함수 수와baseline은 유지했습니다. source-side `#[cfg(test)]` 변경은 없습니다.
- [독립 공백origin/원본charPr·개별 명령·lint/SHA·재사용 증거](../assets/pr7382_20260926/stage64_kopubspace_validation.json). 동일srcSHA의전체280개/최저91.82221%,양108쪽99.83575%와 이미 직접 판독한양review/overlay를 재사용합니다. 전체 줄의 감소검출 범위이며 개별공백의정확한 등식검사로 확대하지 않습니다. 약1px 글자높이/작은 획/제목선 차이를 보존합니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage64-kopubspace/`에만 있습니다.
- 고정38개는 **9개 해결/29개 대기**입니다. 이번 결과·주석보정을 커밋한 뒤 #5585의85/86쪽 차이를 분석합니다. 전체38개 개별 재실행/전체회귀/최종PRhead검증 전 PR 준비 보류를 유지합니다.


### 보정65 사전 분석 — #5585 지표 문서 쪽수

- 고정38의14번 기존 함수는 실제85/기대86쪽으로 실패했습니다. 현재 보정59와동일 생산소스의 고정CLI info도85쪽입니다. 독립2020 PDF86쪽의 출처는 #7269이며 MediaBox만 복구한 기준과 드라이버 원본의86개 내용스트림SHA가 전부 같음을 다시 확인했습니다. 원문과86쪽 기대값은 유지합니다.
- 기존보정48에서 마지막 실제85쪽은 PDF86쪽 평가인증률에 대응했지만 이동시작/전체보존 원인은 미검증이었습니다. 현재 같은파일로 개별 실행과 전수 text/SVG/layout 원장을 만들고 가장앞선 쪽 소유 차이와 표 분할 경계를 찾습니다. 쪽수만 맞추거나 기준을85로 바꾸지 않습니다. 이전 낮은22/23쪽 점수를 정상 회귀의 시각근거로 재사용하지 않고 실제 배치를 개선한 뒤 전체 Native/fresh WASM 최저90%를 확인합니다.

- 추가 원인 분석: 자동 sequence 후보의 최초55쪽보다 이른 **53쪽**이 실제 첫 오류입니다. 원본pi72의16개8-unit 표 제어 문자와 저장줄ts0/8/120을 직접 읽었습니다. ci0은 첫 줄TAC,ci1~14는 가운데 줄float,ci15는마지막리셋줄TAC입니다. 현재ci0은67쪽에 있으므로 내용누락이 아니라 잘못된소유입니다. `order::for_paragraph`는 중간vpos리셋에서offset키만끄고float/TAC보조키는남겨 첫줄TAC를후행float뒤로정렬합니다.
- 기존 생산자 `layout::control_line_seg_index`는 이 빈원본carrier의8-unit 위치를0/1/2줄로 해석하며paint도이를소비합니다. 이 결과를 `order::for_paragraph`의1차키→`paragraph_flow`의control순회→각표예약/PageItem→`layout_table`의같은소유줄 경로에 연결합니다. 원본빈carrier/유효저장줄/미편집 범위에서만 줄간순서를보존하고 같은줄의기존float정렬과편집·합성 경로는유지합니다. 일반TAC이기때문에모든표가같은줄이라고가정하지 않습니다. 추가로ci1/2의독립PDF순서와TAC끝줄의분할을후속검증합니다. 기대86/공차를바꾸거나결과를clamp하지않습니다.

- 후보1은53쪽 첫TAC 소유를복원했으나85쪽이므로 미완료입니다. 별도 원인은77쪽의pi81/82 두 독립TopAndBottom 표가같은쪽을소유하는 것입니다. pi82의원본빈호스트0+847+바깥여백140+높이52308+하단140=다음pi83저장53435HU는유효한닫힌프레임입니다. 기존query_original_control_table_frame→whole_fit→entry가현재751.1px에서이프레임712.5px를수용해−38.6px전진으로앞표영역을되돌렸습니다. 다른호스트의flowWithText/자리차지/겹침비허용표가이미있는쪽에서는이후독립프레임의원점이점유흐름이전이면쪽을이월한뒤같은원본프레임을다시조회합니다. 기하값clamp·임의높이·문서ID예외는추가하지않습니다. PDF77/78의별도소유와수정전겹침을영향쪽에서먼저대조합니다.

- 후보2 중간 결과:77/78의독립표소유를분리해 **85→86쪽** 복원했습니다. 하지만아직54/55및79~81순서가다르고9개Native선행비교는최저31.02805%로gate FAIL이므로14번은해결로세지않습니다. 직접53/77 review를읽어기준PDF에작성일/오른쪽셀/중첩표연도누락이있음을확인했습니다. 동일바이트MCP2020재생성86쪽에서도누락과portrait clip이재현됐습니다. Windows별도변환저장소의`HwpPrintPaper.cs`가문서방향을읽지않고Math.Min/Max로용지를강제세로정렬하는것을읽기전용으로확인했습니다. 기존MediaBox복원은86개내용stream을보존하지만인쇄전누락된셀을복원하지못합니다. 이결함에렌더러를맞추거나90%규칙을완화하지않습니다. output의`reference-recheck/paper-orientation-proposed.patch`를준비했고rhwp밖서비스수정/배포는별도승인대기입니다. 원문/기존PDF/재현출력을보존하고그동안현재86쪽보정의기존개별/정상대조군을실행합니다. Windows rhwp에는기존untracked `samples/issue7333/`가있어그경로는수정하지않았습니다.

- 후보2 선행 검사:고정14번함수 **1 PASS/exit0**(수정전85/86 FAIL),기존sliver검사 **1 PASS**,기존#5807 **1 PASS**,기존#6879 **4 PASS**로총7개PASS입니다. 파생suite 준비/fmt/check도PASS입니다. 현재문서쪽수86 복원은실행으로확인했지만손상된기준PDF의원본소유순서/전체양출력90%검증이완료되지않아보정65완료·커밋·PR준비로보고하지않습니다. 세Clippy/freshWASM/전체38개및전체회귀는최종보정뒤수행합니다. output의`candidate2/partial-validation.json`에명령·소스SHA·부분gate·미검증범위를기록했습니다.


- 변환 서비스 보정 결과: 이후 승인된 별도 작업으로 `win10-ted`의 변환 저장소에 가로 방향 보정 `a5857a1`을 커밋했습니다. 기존 검사 안의 잘못된 회전 기대값만 수정했으며 수정 전 의도한 용지 검사 FAIL, 수정 후 전체 68개 중 **66 PASS / 2 SKIP / 0 FAIL**, x86 worker 빌드 PASS입니다. 검증 서비스에는 검사한 worker를 배포했고 구성 해시 불변·배포 전 실행 작업 0·배포 후 서비스/인증 health 정상임을 확인했습니다. 기본 endpoint에 같은 보정이 배포됐다는 의미는 아닙니다.
- 보정된 검증 서비스의 동일 원문 출력은 job `b98b57a2-a9c4-4832-915f-5cb2c6a30be4`, PDF SHA `bd6e6a2c43bf69d5ae3175314690333900a50e64133e1195d0ff513edc52a3f4`, **86쪽 / 실제 MediaBox 841×595pt**입니다. 후처리 없이 작성일·오른쪽 셀·중첩 연도 내용이 복원됐습니다. 그러나 이 정상 출력과 후보2의 영향 9쪽 Native 비교도 최저 **34.63821% / gate FAIL**입니다. 54/55·79~81쪽 소유 순서뿐 아니라 같은 내용을 소유하는 77쪽 행 경계도 다릅니다(68.30656%). 해당 `review_077.png`를 직접 읽었습니다. 변환 결함 해결을 렌더러 완료로 바꾸어 보고하지 않습니다.
- 사용자가 지정하신 `/Users/tsjang/Cloud/Devel/hwp-convert-2024/.env.local` 그대로 재변환했습니다. job `af064381-4a7d-4656-bc2f-b750c5a851e2`는 성공했지만 결과 SHA `9549ff158e7525e1e1e9fc9a4e2a494f4e44f0b38495defdee03517375ad6f6c`의 **86쪽 / MediaBox 595×841pt**로 강제 세로 용지와 오른쪽 내용 누락이 재현됐습니다. 이 PDF는 실패 증거로 보존하며 정상 정답지로 채택하지 않습니다. `.env.local`의 endpoint·인증 값은 바꾸거나 기록하지 않았습니다. output의 `canonical-reference-after-investigation.json`·`reference-paper-fixed/canonical-pdf-proof.json`에 원문 SHA·변환 결과·두 출력의 차이를 남겼습니다.
- 한컴 HWP→HWPX 저장 결과도 원본 표 배열과 72/81/82/83 문단의 저장 줄 좌표를 보존했습니다. 따라서 PDF의 54/55쪽 순서 차이를 입력 배열의 임의 교환으로 해결할 근거가 없습니다. `hancom-model-order-inspection.json`에 **문단 직접 소유 표만** 추출해 중첩 표와 구분했습니다. 현재 고정38은 여전히 **9개 해결 / 29개 미완료**이며 #5585는 완료·커밋·PR 제출 대상으로 세지 않습니다.


- 사용자 제공 정상 PDF 재확인: `tests/fixtures/stored_float_anchor_control/1351000_policy_indicators-2020.pdf`를 실제 기준으로 채택합니다. SHA-256 `4873b182ad42ffe1cea6ebd1a585691b703f32cf6cda769a5124d715bd850b96`, SHA-1 `55e796fc2c44aca3c6442fad0002bfb3cac813cd`, PDF 1.4 / Creator `Hwp 2020 11.0.0.9136` / Producer `Hancom PDF 1.3.0.550`입니다. 전체 **86쪽 / 가로 MediaBox 841×595pt**이며 53·77쪽 작성일/오른쪽 셀/중첩 연도를 직접 확인했습니다. 보정 검증 서비스 출력과 모든 86쪽의 렌더 픽셀·추출 텍스트가 같지만 PDF 파일 해시는 다르므로 각 출처를 구분해 유지합니다. 사용자가 가져온 원본을 덮어쓰거나 중복 이동하지 않았습니다.
- 해당 제공 PDF로 후보2 소스·CLI 해시가 이전 검사와 같음을 확인한 뒤 영향 9쪽 Native Sweep을 새 경로 `stage65-indicators5585/visual-user-reference-native/indicators5585/`에 다시 실행했습니다. **exit1 / re_review_required**이며 53쪽98.00101%, 78쪽99.75692%, 77쪽68.30656%, 최저80쪽34.63821%입니다. 53·77·80쪽 review PNG를 직접 읽었습니다. 80쪽은 한컴의 노인일자리 표 대신 장기요양보험 표가 배치되어 같은 쪽 내용 소유가 다르고, 77쪽은 같은 치매조기검진 표의 행 경계가 다릅니다. 이제 이 차이를 정상 PDF에 대한 렌더러 결함으로 추적합니다. 새 PDF 정상 판정은 후보2의 전체 피델리티 통과가 아니며, fresh WASM/전체86쪽 양출력 검증은 보정 이후 남아 있습니다. 고정38 상태는 **9개 해결 / 29개 미완료**로 유지합니다.


- 보정65 후보3 사전 분석(77/86쪽 행 경계): #5923의 비-TAC 내용 합계는 셀 마지막 줄간격을 이미 제외합니다. 그러나 #1763의 선언 높이 수용 분기가 같은 마지막 간격을 다시 빼고 있었습니다. 원본 pi81 r1은 `content=36.8 + pad=3.7 > decl=38.8px`, r3은 `41.6 + 3.7 > 38.8px`인데 두 행 모두 선언 38.8px로 수용됐습니다. 한컴 p77의 대응 행 높이는 약40.46/45.25px이며, 저장 줄 끝과 안 여백으로 독립적으로 설명됩니다. 이후 `fit_measured_table_to_declared_height`가 부족한 총높이를 모든 행에 비례 배분해 다른 경계도 이동했습니다.
- 수정할 공통 측정 결과: 텍스트 합계 생산 시 실제 포함한 **셀 마지막 trailing**을 함께 기록하고, 선언 수용 분기는 그 값만 소비합니다. 비-TAC·완전히 빈 마지막 줄·개체 마지막 줄·세로쓰기·제외된 마지막 간격에는0입니다. 실제 포함되는 TAC 다문단의 기존 #874/#1086 계약은 유지합니다. `HeightMeasurer::measure_table`의행 결과 → `fit_measured_for_host` → whole fit 예약 → layout의`measured_table.row_heights` 소비에서 독립 기대40.46/45.25px와 최종 원점/행 경계를 대조합니다. 새 테스트 함수/기준 완화 없이 기존 정상 대조군을 먼저 실행하고 영향 쪽 Native를 재캡처합니다.


- 후보3 Native 결과: 고정 소스/CLI와 사용자 정상 PDF로 53·77·78·86쪽을 새 캡처했습니다. 점수는98.00101/95.41459/99.91907/94.50935%이며4쪽 gate는 PASS입니다. 77·86쪽 review를 직접 읽었고, 77쪽 실제 r1/r3은40.5/45.3px로 독립 PDF의 약40.46/45.25px와 일치합니다. 원점에는약1.8px 차이가 남아 완전한 픽셀 동일성은 주장하지 않습니다. 문서는86쪽을 유지합니다. 기존TAC/빈마지막줄/개체마지막줄 및 #5585 검사 결과는 실행 완료 후 연결합니다. 이4쪽 통과를 아직 다른 쪽의 소유 순서나 전체 WASM 통과로 승격하지 않습니다.
- 79~81쪽 순서 사전 분석: pi83은 원본 한컴 저장 HWPX에서도 배열0(소외감)→1(일자리)→2(장기요양)이며 정상 PDF도그순서입니다. `order::for_paragraph`가문단 상대양수오프셋847/11500을같은쪽좌표로정렬해1→2→0으로뒤집습니다. 그러나ci0은 offset+자체높이+바깥여백이66481HU로현재단전체55843HU를넘어한쪽좌표로비교할수없는자리차지·겹침비허용·나누지않는표입니다. 이물리경계에서소유/흐름이월을먼저존중해야합니다. 문서ID·표개수·특정높이로정렬을교환하지않고,유효원본의분할불가흐름개체가한단의오프셋프레임밖으로가는경우원본순서를보존하는방향을검토합니다. 같은쪽에들어가는#986/#1639양수대조군의offset정렬은유지해야합니다. 54/55쪽은별도fit/지연경계문제이므로이조건으로같이해결됐다고추정하지않습니다.

- 후보4는 기존 정상 대조군 **26 PASS/0 FAIL**이며 86쪽을 유지했습니다. Native 영향9쪽은 완료했으나 54/55쪽72.00776/35.92641%, 79~81쪽40.21137/38.32459/39.56450%로 gate FAIL입니다. 79/80쪽 review를 직접 읽어 원문 표 소유 순서는 복원됐지만 표 전체 원점이 약9~13px 내려감을 확인했습니다.
- 후보5 사전 분석: 앞쪽 끝의 원본 문단 앵커/양수 오프셋이 소진됐다는 기존 `para_offset_consumed_by_page_break` 계약은 조각 예산/partial paint에만 적용되어 통째 표는 legacy offset/clamp를 다시 소비합니다. 원본 미편집·빈 호스트·자리차지/흐름 참여/겹침 비허용·나누지 않음·새 단의 동일 경계에 통째 배치 계획을 마련합니다. 계획의 top은 현재 원점+바깥 위여백, bottom은 top+실측 표 높이+아래여백이며 whole-fit 예약과 layout이 같은 결과를 사용합니다. 편집·합성 줄·절대 배치·캡션·RowBreak에는 적용하지 않습니다. 54/55쪽은 이 경계의 해결 대상으로 승격하지 않으며 후보5 출력과 기존 대조군을 재확인합니다.

- 후보6 사전 분석(54/55쪽): 같은 원본 저장 줄의 ci1 초기 상자(847+54726+140+140=55853HU)는 전체 단55843HU를 넘지만 ci2 상자54721HU는 들어갑니다. PDF54쪽은 ci2의상단31.9827px,55쪽은 ci1의상단20.7893px이며 뒤 표는 초기 오프셋을 다시 더하지 않습니다. 저장 줄의 첫 수용 가능한 형제를 먼저 놓고 거절된 원래 형제를 그 뒤에 유지하는 한 번의 수용 선택을 적용합니다. 일반 높이 정렬·문서ID·개수 조건을 추가하지 않습니다. 앞쪽 말미에서 이미 소비한 앵커는 원래 순서를 보존합니다(pi83). 원본 줄 소속을 공동 조회하고, 실제 앞쪽 PageItem에 배치된 같은 저장 줄의 흐름 표만 소비 증거로 사용해 whole-fit/출력에 동일 top/bottom 계획을 전달합니다. 같은 줄의 아직 미배치 형제·다른 줄 TAC는 소비 증거가 아닙니다. 기존 #986/#1639의 한 단 안 정렬 및 음수·텍스트 호스트 대조군을 재검증합니다.

- 후보5는 빌드PASS이지만 Native9쪽이 후보4와 같아 원점 개선이 없었습니다. 새 계획이 통째 entry에서만 조회됐고, 실제 입력은 whole-fit 실패→`prepare_block_table_continuation`의 이월→스캐너 전체 행 수용→전체 표 출력으로 지나갔습니다. 이를 통과로 보고하지 않습니다. 이월 뒤 준비된 `host_placement`에 동일 조회를 연결하고 스캐너 예산→`emit_table_fragment` 예약→layout의최종 원점까지 실행으로 다시 대조합니다. 후보5의무효 보정 실행은 output에 보존합니다.

- 후보7 실제 이월 경로 연결 후54쪽은 원본 ci2/초기 원점으로 복원돼 **99.92009%**이며 review를 직접 확인했습니다(최종 수치는page JSON에고정). 그러나91쪽으로5꼬리쪽이재발했습니다. `push_new_page()`가빈현재페이지를`pages.last()`로추가하므로 앞쪽실제소유조회가항상빈현재쪽을보았습니다. 이오류는기존#5585의86쪽/꼬리쪽검사가검출하는유효한회귀이며테스트기준을바꾸지않습니다. 실제내용이있는직전PageContent의같은저장줄흐름표를조회하고완료된PartialTable소유도함께대조합니다. Native9쪽/기존검사실행은후보7의실패증거로보존하고수정한후보에서재실행합니다.

- 후보7 기존검사는 **25 PASS/1 FAIL(exit100)**입니다. 고정38의#5585쪽수함수가91/86으로의도한회귀를검출했습니다. `no_sliver_tail_pages_remain`은꼬리쪽이실제로5장인데도PASS이며, 전체트리의Footer TextRun이`deepest`에포함되는검사범위문제가있습니다. 전체정상PDF/90이상근거가확보되면기존함수의본문검사로수정하며새함수를추가하거나86쪽기준을완화하지않습니다. 후보8은빈현재쪽을건너뛰어같은구역의직전소유쪽을조회하고같은저장줄Table/PartialTable의실제소유를확인하도록수정했습니다.

- 후보8 빌드PASS/86쪽이며 실제 소유는53(pi72ci0),54(pi72ci2),55(pi72ci1),77(pi81ci0),78(pi82ci0),79~81(pi83ci0/1/2),86(pi88ci0)로 정상 PDF와 대응됩니다. 단지쪽수복원을완료로세지않습니다. 같은후보소스/고정CLI로9쪽Native와기존26검사를재실행중이며이후전체86쪽Native/freshWASM90이상·최종검증이남아있습니다.

- 후보8 영향9쪽Native완료:53/54/55/77/78쪽은98.00101/99.92009/99.90920/95.41459/99.91907%입니다. 55쪽review를직접확인했습니다. 79~81쪽은40.21137/38.32459/39.56450%로보류입니다. 원본pi83은공백두칸·cc27·3표·단일원본LS(vpos53435/lh2400)라`para_has_visible_text`가true이며새저장계획을우회했습니다. 이공백줄의앞쪽앵커소비와표의다음쪽원점은구별해야합니다. 모든공백을0높이로바꾸지않고,독립저장앵커가앞쪽에서소진됐고확정표배치계획이있는경로의중복pre-text만제외합니다. #5871공백호스트대조군및#7330post-text의기존함수를확인하며새함수를추가하지않습니다.

- 후보8의기존26검사는26PASS/0FAIL입니다. 후보9공백보정은빌드PASS이며실행전코드검토에서중복pre-text조건을확정프레임유무만으로정하면현재쪽에속한정상공백줄까지제외할수있다고판단했습니다. 후보10은유효확정프레임+미편집+앞쪽앵커소비가함께확인된경로에만적용합니다. 일반공백줄의기존높이계약을바꾸지않으며후보9를시각통과로보고하지않습니다.

- 후보10 영향9쪽Native는 **exit0/게이트passed**,53/54/55/77/78/79/80/81/86쪽98.00101/99.92009/99.90920/95.41459/99.91907/99.54002/99.73334/99.67156/94.50935%입니다. 79쪽review와standaloneoverlay를직접읽어표원점/괘선/줄바꿈/내용을확인했습니다. 실제86쪽및9쪽소유도정상PDF와대응합니다. 9쪽통과를전체문서완료로세지않으며같은소스/입력/PDF/CLI provenance의checkpoint를90%완화없이resume해전체86쪽을비교중입니다. 기존28검사및freshWASM/최종lint가남아있고고정38은아직9해결/29대기입니다.

#### 보정65 후보11 사전 분석 — 완전 셀의 마지막 줄간격

- 사용자 기준 PDF와 후보10 전체 비교에서 19쪽 65.69735%, 20쪽 55.98696%, 37쪽 71.00386%를 확인했습니다. 후보2의 봉인된 바이너리에서도 19·20쪽 점수가 동일하여 이번 앵커 보정 이전부터 있던 행 높이 불일치입니다.
- 원인 경로: `HeightMeasurer::measure_table`의 단일행·병합 셀 높이 계산은 TAC 다문단 셀 마지막 줄간격을 추가하지만, `MeasuredCell::line_heights`와 `LayoutEngine::calc_para_lines_height` 및 실제 셀 배치는 제외합니다. 19쪽 수식 행은 66.1px로 측정되지만 독립 PDF는 약61.3px이며, 20쪽 두 행도 각각4.8px의 마지막 줄간격이 과대 계상됩니다. 37쪽에서는 병합 라벨 칸의 과대 하한으로 TAC 축소가 다른 행에 전파됩니다.
- 완전 셀(None/CellBreak)은 마지막 줄간격을 제외해 실제 배치와 동일한 점유량을 측정합니다. RowBreak TAC의 분할 회계는 별도 계약으로 이번 변경에서 유지하며, KTX 목차 RowBreak 대조군 출력과 기존 #6030/#6681/#7097을 재검증합니다. 문서 ID나 특정 높이를 구현 조건으로 사용하지 않습니다.
- 독립 좌표 및 변경 전 증거: `output/pr-review/planet6897-7382-20260926/stage65-indicators5585/candidate10/tac-row-independent-proof.json`, `diag20.log`, `visual-before-tac1920-native/indicators5585/manifest.json`. 전체86쪽 Native/fresh WASM 최저90% 충족 전에는 기존 #5585 검사 수정·완료 커밋을 보류합니다.

- 후보11 부분 결과: 19쪽 65.69735→99.93810%, 20쪽 55.98696→99.79913%, 37쪽 71.00386→99.84914%, 44쪽 47.30234→99.82000%. 19·20쪽 review와 37쪽 standalone overlay를 직접 판독해 표 경계·본문 시작 위치를 확인했습니다. KTX 목차 SVG SHA-256은 변경 전·후 `7e3d9945e59218396e2ed4498be63d8cc4f81eac78c61b2cb8f6cef3a1036fce`로 동일합니다. `candidate11/tac-four-partial-proof.json`에 소스·바이너리와 독립 기준을 고정했으며 전체86쪽 Native/fresh WASM 및 기존 회귀 재실행은 아직 진행 중입니다.

#### 보정65 후보12 사전 분석 — 병합 셀의 동일 물리 하한

- 후보11에서 #6660 기존6개 중4개가 실패했습니다. 봉인 후보2/10과 후보11의 실제 출력 대조로 새 회귀를 확인했습니다: exam_science 1쪽 문단23 표 높이131.6→133.5px, 뒤 그림y1084.3→1086.2px; 4쪽 그림은1010.9px로 불변입니다. `candidate11/exam6660-before-after.json` 및 기존 검사 로그를 보존했습니다.
- 마지막 줄간격 제거로 TAC 전체 높이가 축소 임계 아래가 되면서, 원래 TAC 축소 뒤 복원에서만 적용하던 `merged_cell_restore_floor_hu`의 비활성 하단 여백 규칙을 일반 병합 셀 측정이 소비하지 않는 문제가 드러났습니다. `<보기>` 제목은 저장1150HU+활성 상단141HU로 선언1289HU를 행 반올림2HU 내에서 닫지만, 일반 측정은 비활성 하단141HU를 추가해 뒤 내용을 약1.88px 밉니다.
- 기존 독립 출력으로 검증한 같은 하한을 일반 병합 셀 측정에도 연결합니다. 저장 텍스트 끝과 실제 합성이 일치하고 원본 저장본이며 재조판되지 않은 경로만 소비하고, 명시적 여백·내용 초과 반례는 기존 helper로 유지합니다. 복원 단계만의 예외를 더 좁히는 대신 그 단계에 숨었던 물리 하한을 생산 단계에서 공유합니다. 원문 #6660 PDF와 현행 MCP 재변환을 비교하고 실제 회귀를 수정한 뒤 기존 검사 기대값의 적절성을 판정합니다.


- 후보12 부분 검증: 정책 지표 영향14쪽 Native는 모두90% 이상(최저94.50935%)이고, #6660 원문 전체4쪽 Native는 최저90.98817%로 gate `passed`입니다. 원문 용지272×394mm를 보존한 MCP engine2020 기준 PDF SHA-256은 `6d362eeba4ceae575b7b8279b461ecd661355ae4aca3d2281fe605444533fdbf`입니다. 1쪽 review와4쪽 standalone overlay를 직접 확인했습니다. 그림·표 위치 보정 범위와 별개로 머리말 글꼴 및 꼬리말 쪽번호 차이는 남아 있으며 전체 피델리티 완료를 주장하지 않습니다.
- 기존 검사 갱신 준비: #5585의86쪽 기대값은 유지하고 기존 함수 안에 PDF 표 소유·상단·행 높이 검사와 본문 전용 꼬리쪽 탐색을 준비했습니다. #6660의 두 합성 높이 기대값은 원문 저장 행·글줄·명시적 여백으로 계산한9870HU에 연결하고, 그림 기대 좌표는 정상 용지 PDF의 직접 pt→96dpi 값으로 바꿉니다(1px 공차 유지). 새 테스트 함수는 추가하지 않습니다. 각 원문 전체 Native/fresh WASM 최저90%와 Cargo 종료를 확인한 뒤 실제 테스트 소스에 적용·재실행하며, 준비 파일만으로 수정 완료로 세지 않습니다.


- #6660 기존 검사 갱신: 후보12 원문 전체4쪽 Native 최저90.98817%, fresh WASM 최저90.68713%로 모두 gate `passed`입니다. 실제 그림y1084.3/1010.9px는 정상 용지 PDF 직접 좌표1083.50399/1010.11865px와 각각0.79601/0.78135px 차이로 기존1px 기준 안입니다. 정상 기준은 `pdf/exam_science-2020.pdf`에 보존했습니다. 이전 두 PDF와 원문은 그대로 남겼습니다.
- 기존6함수를 수정 없이 후보12에서 실행하면4 PASS/2 FAIL이고, 실패는 common.height만10200/4000HU로 변경한 합성 대조군의 이전 기대값입니다. 셀·글줄·행 선언을 바꾸지 않았으므로 독립 필요량9870HU를 검사하도록 기존 두 assertion을 갱신했습니다. 두 그림의 간접 A3 좌표도 원문 용지 PDF 직접 좌표로 갱신하되1px 공차는 유지했습니다. 함수6개는 동일하며 추가 테스트가 없습니다. 수정한 기존 검사 재실행 결과는 별도로 확인합니다.

- #6660 갱신 후 실행은 기존6개 전부 PASS/exit0입니다(nextest run `4a64d730-9440-464a-88f1-0b9a3edafa28`, runtime0.804s). 명시적 여백·실제 내용 초과의 기존6변형을 포함한 검사도 통과했습니다. 파생 suite 미갱신으로0개가 선택된 첫 시도는 검증에서 제외했고 `--prepare` 뒤 올바른 두 suite로 다시 실행했습니다. 파생 파일은 커밋하지 않습니다. #5585 전체86쪽 검증이 끝나기 전에는 보정65 완료·고정38의해결수 증가로 세지 않습니다.


#### 보정65 전체 시각 검증 및 기존 검사 갱신

- 사용자 제공 정상 PDF와 후보12 정책 지표 전체86쪽은 **Native 최저92.57413% / fresh WASM 최저92.57413%**, 양쪽 `pr_review_gate=passed`입니다. #6660 원문 전체4쪽도 **Native90.98817% / fresh WASM90.68713%**로 통과했습니다. 지표4쪽 Native review·54쪽 WASM review·79쪽 WASM standalone overlay, 시험1쪽 양쪽 review·4쪽 Native overlay를 직접 확인했습니다. 2px 이웃 내용 실루엣 지표이며 완전한 픽셀 일치가 아닙니다. 지표77·86쪽 표 상단 약1.8px 차이와 시험 머리말 글꼴·꼬리말 쪽번호 차이는 잔여 사항으로 남깁니다.
- 기존 #5585 함수2개를 갱신했습니다. 86쪽 기대값을 유지하고, PDF의53/54/55·79/80/81쪽 표 소유·상단 및19/20/37/44/75쪽 행 높이를 검사합니다. 다른 호스트의77/78쪽 두 표도 별도 쪽 소유를 확인합니다. 꼬리쪽 함수는 CLI 최종 트리의 **Body TextRun만** 검사하고 트리 읽기 실패를 성공처럼 건너뛰지 않습니다. 출력은 `output/pr-review/regression-temp/`에 두며 종료 시 해당 실행의 임시 트리만 정리합니다.
- 새 테스트 함수 없이 기존8함수(#5585의2개 + #6660의6개)를 유지했습니다. 파생 suite 준비 결과도1433 sources /6256 static test attrs로 증가하지 않았습니다. 형제 표 함수 개별 실행은 **1 PASS/exit0**(run `1f98c3f0-8e0d-433e-8756-b7313a055645`)입니다.
- 꼬리쪽 검출 입증: 이전 후보7 봉인 CLI를 `RHWP_5585_RENDER_BIN`으로 지정해 **갱신한 기존 함수 자체**를 실행하면 **1 FAIL/exit100**이며60/62/65/67/71쪽 본문 끝72.2px 다섯 곳을 정확히 검출합니다. 정상 실행은 현재 Cargo 출력기를 사용합니다. 이전의 전체 트리 탐색은 꼬리말774.1px 때문에 같은 결함을 PASS로 놓쳤으므로 검사 범위를 수정했습니다. 이 결과는 별도 JSON 진단만으로 주장한 것이 아니라 실제 Rust 회귀 실행입니다. 현재 출력의 기존 대조군34개·최종 lint/정책 검증은 이어서 확인합니다.


- 최종 기존 대조군은 **34 PASS/0 FAIL/exit0**입니다(nextest run `8da5f721-ccb1-4bec-9426-fa612f5eeadc`, runtime1.233s). 기존8함수 갱신 후 source hash는 봉인 후보12와 동일하며, fmt check·Native/WASM/전체target Clippy·workspace build·고정base `443844b593c62a722cf9cc3d9d0256e94ab88cb8`의 manifest 정책 검사가 모두PASS입니다. source-side test 변경이 없어 unit-tier 증가는 비해당입니다. fresh WASM은 Mac의 `--no-opt` 대체 빌드이며 Docker 최적화 빌드 통과로 보고하지 않습니다.
- 코드·기존 검사·정상 PDF2개와 대표PNG8개 및 [보정65 검증 원장](../assets/pr7382_20260926/stage65_validation.json)을 같은 결과 커밋에 보존합니다. 대표 증적은 [Native19쪽 review](../assets/pr7382_20260926/stage65_native_review_019.png), [WASM54쪽 review](../assets/pr7382_20260926/stage65_wasm_review_054.png), [Native79쪽 overlay](../assets/pr7382_20260926/stage65_native_overlay_079.png), [시험4쪽 WASM overlay](../assets/pr7382_20260926/stage65_exam_wasm_overlay_004.png)입니다. 정상 PDF는 [지표86쪽](../../../pdf/pr7382/1351000_policy_indicators-2020.pdf), [시험4쪽](../../../pdf/exam_science-2020.pdf)에 있습니다. 원문·이전PDF·사용자제공fixture경로의PDF는 삭제하지 않습니다. 로그·임시트리·파생suite·pkg·font는 커밋 대상이 아닙니다.
- 이번 결과는 고정38의14번을 해결한 보정65 범위입니다. 고정38 각각의 최종 재실행과 전체 nextest·Native Skia·최종head 시각 검증이 아직 남으므로 전체 PR 준비 완료로 판정하지 않습니다.

- 보정65 결과 커밋은 `3f2951b6b0ff522b4f702ed8aa895f8195a43a4a`입니다. 고정38 상태를 **10 해결 /28 대기**로 갱신했고, 다음 단계는15번 `issue_5941_tail_overflow_drift_gate::issue_5941_drift_past_saved_tail_still_grants_overflow`의 개별 원인·입력 유효성 분석입니다. 사용자 제공 fixture PDF는 원래 경로에 유지한 untracked 파일이며 증적 커밋에는 byte-identical `pdf/pr7382/` 사본만 포함했습니다.

## 보정66 사전 분석 — 로드맵 #5941 회귀 제외와 #7445 이관

- 사용자 지시로 `1490000-201600081_roadmap_research.hwp`를 현재 회귀 대상에서 제외하고 전체 피델리티 문제는 #7445에서 별도 추적합니다. 기존304쪽은 정상 한컴 PDF302쪽과 다른 잠정 핀이며 현재299쪽으로 바꾸어 통과시키지 않습니다.
- 보정65 제품의 Native 비교1·61·145쪽에서1쪽84.58019%,145쪽42.91058%로 gate가 `re_review_required`입니다.145쪽 review에서는 정상 PDF의 온전한 표를 앞쪽/현재쪽으로 나눈 차이를 직접 확인했습니다. render-tree/dump-pages에서142·146·157쪽의 그림 겹침도 관측했으나 원인 계층은 아직 확정하지 않았습니다. fresh WASM·전302쪽 비교는 이번 단계에서 수행하지 않았습니다.
- 해당 단독 검사1개와 이 문서의 baseline4행만 제거하고 원본은 `mydocs/pr/assets/issue7445/`에 바이트 동일하게 이동해 samples 자동 수집에서 제외합니다. 정상 기준 PDF와 기존 실패/시각 증거를 보존합니다. 다른 #5941 문서·검사 및 제품 코드는 유지합니다. 신규 skip/helper나 허용치 완화는 추가하지 않습니다.
- 제거 후 파생 suite 준비·정책 base 검사·필수 Rust lint와 회귀 목록에서 단독 검사 미수집을 확인하고 결과보고 후 커밋합니다. 고정38은10해결/1이관/27대기로 구분하며 제거를 결함 해결 또는 PASS로 세지 않습니다.

## 보정66 결과 — #5941 검사 제외·원본 보존·#7445 추가 등록

- 원본 HWP3,923,456bytes를 이슈 증적 폴더에 바이트 동일하게 보존했고 정상 한컴 PDF302쪽도 유지했습니다. 단독 검사1개와 해당 baseline4행만 제거했습니다. 나머지 baseline 행은 부모 head와 동일하며 파생 suite와 samples 수집에 제거 문서/검사가 남지 않습니다. 제품 코드·허용치 변경 및 신규 검사/skip/helper는 없습니다. [이동·해시·제거 행·검증 명령](../assets/issue7445/roadmap5941_test_removal_validation.json).
- fmt, Native/lib WASM/workspace all-targets 세 Clippy, workspace build, 고정base `443844b593c62a722cf9cc3d9d0256e94ab88cb8` manifest 검사는 모두exit0입니다. 유지한 다른 #5941 문서의 기존 검사4개는 nextest release-test/threads8/no-fail-fast로4PASS/0FAIL입니다. 전체 nextest·유지된37함수 재실행은 아직 미완료이며 이번 focused 결과로 대체하지 않습니다.
- [#7445 추가 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5865123120)을 완료하고 게시 본문이 UTF-8 원문과 동일함을 API로 확인했습니다. 제목은 규제영향분석서와 비정규직 로드맵을 함께 추적하도록 갱신했습니다.145쪽 Native review/overlay PNG를 커밋 증적에 보존합니다. 아직 전302쪽/fresh WASM 비교는 실행하지 않았고 페이지 수299/302 및90%미만 문제는 이슈 보류입니다.
- 고정38은 **10해결/1이관/27대기**입니다. 이관은 PASS 또는 렌더링 해결이 아닙니다. 다음 개별 보정은 #6101이며 통합 PR 준비는 계속 보류합니다.

![로드맵145쪽 표 분할 차이](../assets/issue7445/roadmap5941_native_review_145.png)

## 보정67 사전 분석 — #6101 두부문자와 미검증 쪽수 핀

- 현재 기존 #6101 두 함수는2PASS이나 소방교육 원본은12쪽/독립 한컴11쪽으로 출력됩니다. 기존12쪽 기대는 정상값이 아닌 잠정 핀이므로 유지한 채 먼저 실제 출력을 개선합니다. 새 검사는 추가하지 않습니다.
- 기존 local2020 PDF는 Linux cairo1.18 출력이며 현재 Windows 한컴 engine2020으로 동일 원문을 다시 변환했습니다(job `2ee76728-acd9-48ba-8cea-3fb0d943b6c2`,11쪽). 원문 hash와 정상 대응을 확인했으며 기존 PDF는 보존합니다. 옛 출력의 글꼴 대체/굵기 차이를 Windows 글꼴 계약으로 그대로 사용하지 않습니다.
- 두부문자는 원문 일반 한글/숫자가 사라진 문제가 아닙니다. 실제 `휴먼명조` HMKMM의 cmap와 glyf 윤곽선은 존재하지만 EBDT/EBLC bitmap strike가 함께 들어 있습니다. 임시 진단에서 이 두 테이블만 제거한 같은 글꼴의 전체 embed로3쪽 연락처/하단 문장이 정상적으로 표시됩니다.7쪽의 같은 글꼴에도 적용되며 현재 일치율80.93690/78.28182%여서 시각 통과는 아닙니다. 진단용 글꼴 사본은output에만 둡니다.
- 수정 경로는 원문 글꼴 → SVG CSS 생성용 폰트 bytes → data URI → 브라우저 glyph입니다. 글리프 ID·cmap·윤곽선·advance·조판 좌표를 그대로 보존하는 단일sfnt의 outline 가능 문자에만 bitmap strike 제거를 적용하고 bitmap-only/collection/잘못된 입력은 변경하지 않습니다. 입력 파일 자체나 본문 문자열은 고치지 않습니다. WASM의 문서 내장 font CSS와 Native 전체 embed의 실제 호출 경로를 함께 확인합니다.
- 작은 동일 페이지 전후 비교로 두부 제거를 먼저 확인하고 남은 표 높이·쪽수 차이를 추적합니다. 모든11쪽 Native/fresh WASM90% 이상 전에는 기존 쪽수 기대나 geometry oracle을 변경하지 않습니다.

- 후보1 재검증에서 초기 원인 분류를 정정했습니다. 실제 첫 sweep의 `휴먼명조` embed는HMKMM이 아니라 선행 검색 디렉터리의Batang.ttc(sha `84b0ba79a5d1c6012bbccf8d364ddc04a0c8b135d227a159196c558225cf1d89`)였습니다. 이 collection의첫 face에도EBDT/EBLC가 있어단일sfnt만보정한후보1로7쪽의일부두부가남습니다. 임시진단의HMKMM은글리프윤곽선보존방향만입증하며실제font선택의근거를대신하지않습니다. 실제선택face의cmap/glyf/advance를그대로단일sfnt사본에보존하는collection지원과byte동일원글꼴공급을함께재검증합니다. 이전후보결과는완료근거로쓰지않습니다.

## 보정67 결과 — 두부문자 임베드 보정, #6101 전체 검토 보류

- 정상 Windows 한컴 PDF가 사용하는 원 휴먼명조를 byte 동일하게 공급하고 `HMKMM.TTF`를 대체 글꼴보다 먼저 찾도록 수정했습니다. 원 글꼴은 `output`에만 보관하며 커밋하지 않습니다. 기존 Linux PDF도 그대로 보존합니다. 새 정상 기준은 [소방교육 11쪽](../../../pdf/pr7382/36361137_firefighter_training_plan-2020.pdf)과 [결재문서 2쪽](../../../pdf/pr7382/36501883_approval_doc_body-2020.pdf)입니다.
- `svg_outline_font_data`는 실제 사용 문자에 윤곽선이 있는 TrueType 단일 글꼴 또는 기존 기본 TTC face 0의 임베딩 사본에서 EBDT/EBLC만 제거합니다. cmap·glyph ID·glyf·advance와 나머지 테이블 bytes는 보존하고 sfnt 오프셋·checksum만 재작성합니다. 비트맵 전용 사용 글리프, 비혼합 글꼴, 잘못된 입력은 원본을 유지합니다. Native 전체/혼합 서브셋 임베드와 공통 문서 내장 글꼴 CSS 경로가 같은 결과를 사용합니다. 사전 분석의 collection 전체 미변경 제한은 후보1의 불완전한 조건이므로 최종 구현에는 적용하지 않습니다.
- 최종 Native 및 fresh WASM의 실제 SVG에서 휴먼명조가 원본의 EBDT/EBLC만 제거한 사본임을 확인했습니다. 다른 모든 테이블은 head checksum 필드를 제외하고 byte 동일하며 전체 sfnt checksum은 `0xb1b0afba`입니다. 글꼴/문서 원본이나 본문 문자열을 수정하거나 글자를 숨기지 않았고 후보2의 12쪽 render-tree는 변경 전과 동일했습니다. [원인·입력·실제 글꼴 보존·빌드·실행 검증 원장](../assets/pr7382_20260926/stage67_validation.json).
- 최종 Native 7쪽 review·3쪽 standalone overlay와 fresh WASM 3쪽 review·7쪽 standalone overlay를 직접 읽어 연락처·지원금·일정의 두부문자 제거를 확인했습니다. WASM은 새 패키지의 실제 SVG/render-tree를 사용하며, 문서에 없는 설치 글꼴의 CSS 공급만 같은 Native 전체 임베드 정책으로 보충합니다. Native 트리를 WASM 트리로 대신하지 않습니다. **두 backend 모두 3쪽80.65124% /7쪽78.42703%, gate `re_review_required`**입니다. 표 높이·지원금 줄바꿈·일부 글꼴 굵기·12/11쪽 불일치를 두부문자 보정 완료와 구분합니다. 전체11쪽 시각 통과나 #6101 해결을 주장하지 않습니다.
- fmt check·Native/lib WASM/workspace all-targets 세 Clippy·workspace build·고정 base `443844b593c62a722cf9cc3d9d0256e94ab88cb8`의 manifest/unit-tier 정책 검사는 모두exit0입니다. 기존 #6101 두 함수2PASS(run `92772039-d892-4d97-9d41-d1f3492ca0c4`), 기존 Bold 함수1PASS(`f96904f8-869b-484e-a7de-699bfeade490`), 수정한 기존 별칭 함수1PASS(`7e83a09b-8529-40f3-99d9-668b768cd757`)입니다. nextest는 release-test/threads8/no-fail-fast이며 새 회귀 함수는 없습니다. Mac fresh WASM은 `--no-opt` 대체 빌드 통과이고 Docker 최적화 빌드 통과가 아닙니다.
- 추가 대조군은 후보2의 정책 지표4·53·79·86쪽 Native 최저92.57413%/gate `passed`이며4쪽 review를 직접 확인했습니다. 최종 source와 주석만 다르지만 CLI hash가 달라 이 결과를 최종 head 전체 검증으로 재사용하지 않습니다. 최종 source 두 파일 hash 및 CLI/JS/WASM hash는 검증 원장에 고정했습니다.
- 로그·임시 SVG/글꼴·pkg·파생 suite는 커밋하지 않습니다. 사용자 제공 fixture PDF는 원래 경로의 untracked 파일로 유지합니다. 기존 #6101의12쪽 잠정 핀과 geometry oracle은 아직 갱신하지 않습니다. 고정38은 **10해결/1이관/27대기**이며 #6101은 계속 대기입니다. 다음 단계에서 표 높이·줄바꿈·추가쪽의 실제 소비 경로를 분석하고, 전11쪽 Native/fresh WASM 최저90% 이상 및 정상 쪽수가 확인된 뒤 기존 검사 기대값을 갱신합니다. 통합 PR 준비는 보류합니다.

![Native 7쪽 두부문자 보정과 남은 줄바꿈 차이](../assets/pr7382_20260926/stage67_native_review_007.png)

![fresh WASM 3쪽 두부문자 보정과 남은 표 높이 차이](../assets/pr7382_20260926/stage67_wasm_review_003.png)

## 보정68 사전 분석 — #6101 표 여백과 미저장 줄 재조판

- 작업 기준: 현재 브랜치 `review/planet6897-7382-20260926`, 코드 기준 `27713d82f`. 별도 PR로 분리하지 않습니다. 기존 두 검사만 개선하며 새 검사 함수를 추가하지 않습니다.
- 독립 기준: `pdf/pr7382/36361137_firefighter_training_plan-2020.pdf`(11쪽)의 3쪽 표는 y=700.03–850.27px, 높이 150.24px입니다. 현재 출력은 y=700.8px, 높이 176.2px이고 문서 전체는 12쪽입니다. 7쪽 지원내용은 기준에서 한 줄인데 현재 마지막 ‘원’이 다음 줄로 넘어갑니다.
- 원문 3쪽 표는 `hasMargin=0`, 표 `inMargin=(0,0,0,0)`, 셀 보존값은 141HU입니다. `Cell::effective_padding`은 이때 수직 셀 여백을 되살리지만 `paragraph_frame_padding`은 0을 사용합니다. `effective_padding → height_measurer`의 내용 요구 높이/행 높이와 `table_layout`의 실제 원점 소비가 같은 여백을 사용하도록 이 불일치부터 제거합니다. 선언 높이로 출력을 잘라 맞추지 않습니다.
- 기존 86712 근거 재검토: 대응 HWP/HWPX 정상 PDF는 모두 64쪽이며 해당 구분 문단은 현재 기준 PDF의 25쪽입니다. 원문 para159/160은 표 선언 1300HU, 셀 선언 282HU, 저장 줄 높이 1300HU이고 표 여백 0/hasMargin=0입니다. 구분 표에는 측정할 외곽선이 없습니다. 과거 21.1px 주장은 직접 괘선 측정이 아니라 빈 문단·바깥 여백과 섞인 간격 폐합 추정이므로 수직 셀 여백 복원의 독립 근거로 사용할 수 없습니다. 영향 출력은 별도로 비교합니다.
- 7쪽 para110은 원문에 LineSeg가 없고 크기·자간·굵기가 다른 여섯 run으로 구성됩니다. 합성된 저장 좌표를 정답으로 쓰지 않고 실제 프레임 재조판의 스타일별 폭 소비를 추적합니다. 글꼴 선택도 정상 PDF의 실제 face와 대조합니다.
- 선행 검증: 3/7쪽 직접 Visual Sweep, 전체 쪽수와 후속 내용 보존, 기존 여백 대조군. 기대값 변경은 정상 PDF와 Native/fresh WASM 전체 검증 뒤 판단합니다. 90% 미만은 미해결로 유지합니다.
- 후보1 Native 직접 비교: 3쪽은80.65124→86.70332%로 개선했고 7쪽78.42703%와12쪽은 유지됩니다. 3쪽 review에서 표 하단과 뒤 문단의 남은 차이를 확인했습니다. 7쪽 정상 PDF의 휴먼명조 한글 전진은 약15.2px인데 현재 측정은 약16.0px입니다. 원문 상대 크기95%를 파서는 보존하지만 `resolve_single_char_style`이 소비하지 않았습니다. `relative_sizes → ResolvedCharStyle.font_sizes → resolved_to_text_style → 재조판 폭/ComposedTextRun.text_style → 최종 glyph`를 공유하도록 보정하며 기본 줄 크기와 자간 기준 크기는 별도로 보존합니다. 상대 크기100% 대조군은 불변이어야 합니다.

- 후보1의 3쪽 표 실제 높이는156.8px로 줄었지만 정상150.4px보다6.4px 큽니다. 원문 두 문단 셀의 마지막 줄 끝은2520HU(33.6px)이고 마지막 줄간격120HU는 후속 줄이 없으므로 내용 높이가 아닙니다. 다문단 TAC/RowBreak 예외가 이120HU를 네 행에 반복해 추가했습니다. 마지막 가시 줄의 후행 줄간격을 제외하고, 명시적 공백 문단이 만드는 빈 줄 점유는 유지합니다. 행 축소 하한과 별도 배치 높이 경로에서 원시 셀 여백을 다시 더하는 소비 지점도 유효 여백 합으로 통일합니다. 기존 #6660의 비활성 여백 예외는 필요 없는 우회가 되므로 공통 물리 하한으로 대체합니다.

- 상대 크기 적용 후보2는 전체11쪽으로 줄었지만 전체 비교에서 큰 흐름 차이를 확인했습니다. 원문 단일 스타일·상대 크기95%인 저장 문단68건에서 LineSeg의 `vertsize/textheight`는95% 글리프 크기가 아니라 기준 크기를 보존합니다. 후보2는 폭에 쓰는 크기를 토큰 줄 높이에도 사용해 미저장 줄 간격까지 줄였습니다. 토큰의 기본 줄 상자를 기준 크기와 분리해 보존하고 글리프 폭/표시는95%로 적용합니다. 11쪽이라는 개수만으로 시각 통과 또는 기존12쪽 핀 갱신을 선언하지 않습니다.

- 후보4 Native 3쪽은90% gate를 통과했으나4·7쪽은 미달입니다. `RHWP_DIAG_ADV`는24번 제목 문단 끝을198.0px로 계상했고 `RHWP_DEBUG_TAC_CURSOR`도 실제 끝198.0px를 반환하지만25번 본문 입력이216.1px로 재상승합니다. `layout_column_item`의 두 인라인 표를 배치하는 FullParagraph 경로가 TAC 줄 소비를 반환하지 않아 HeightCursor의 lazy 기준 재산출이 이미 소비한1360HU(18.13px)를 bridge로 추가했습니다. 해당 실제 소비 경로의 상태를 반환해 일반 Table 항목과 동일하게 다음 문단이 이미 소비한 간격을 인식하도록 보정합니다. 좌표 clamp나 특정 문단 번호 분기는 추가하지 않습니다.

- 후보6 사전 분석: 7쪽 문단102의 표 뒤 본문을 배치한 뒤 TAC 상한이 누적 높이 440.5px를 415.4px로 되감았습니다. 문단113은 typeset 755.6px 원점을 사용하지만 paint 흐름은 791.5px여서 표가 앞 `선발절차` 줄과 겹칩니다. 한컴 PDF의 절차 표 위/아래 테두리는 893.58/998.91px입니다. 원본 LineSeg가 없는 단일 TAC 표의 확정된 첫 개체 줄과 뒤 본문 줄은 기존 fmt의 줄 전진량을 함께 소비하며, 개체 줄 끝점을 inline placement로 전달하고 본문을 소비한 끝점을 유지하도록 보정합니다. 실제 저장 줄·다중 표·개체 높이와 줄 높이가 불일치하는 경로는 기존 계약을 유지합니다.

- 후보7은7쪽94.36649%로 개선했고 절차 표는 y=896.4px, 다음 III 제목은 기준처럼8쪽으로 이월됩니다. Native 전체에서2쪽74.07941%,6쪽56.38273%,10쪽89.99636%가 남아 아직 미해결입니다. 2쪽 선방출 PartialParagraph는 y=96→116.8px이나 원점을 등록하지 않아 뒤 para-relative 표의 앵커가116.8px로 바뀝니다(정상 테두리118.59px, 현재137.7px). 첫 부분 글줄의 원점을 실제 배치 시 보존합니다. 6쪽은 원문 LineSeg가 없는 문단72→73,79→80에서 현재 흐름을6.7px씩 되감습니다. 공통 HeightCursor가 계산한 vpos에 문단 앞 간격이 이미 저장되었다고 가정해 이를 차감하므로, 앞·현재 줄이 모두 재구성된 경우에는 이 저장 간격 사전 차감을 적용하지 않습니다. 실제 저장 줄과 혼합 줄의 기존 재앵커 경로는 유지합니다.

- 후보8 Native2쪽99.77495%로 개선했으나6쪽57.15101%와10쪽89.99636%는 남습니다. 문단73은 원래 앞 간격을 복원했지만 다음74에서 같은6.7px를 다시 되감았습니다. 재구성한 일반 본문에는 절대 저장 원점이 없으므로 실제 전진량 뒤에 합성 vpos를 적용하는 가정부터 제거합니다. 또한 결재문서18→19에서 미저장 본문 다음 저장된 빈 문단의 옛 vpos가51.2px를 추가합니다. 빈 문단의 줄 높이·간격은 보존하되, 재조판한 앞 본문 이후의 절대 vpos를 별도 빈 밴드로 소비하지 않습니다. 개체를 포함한 문단의 재앵커와 실제 저장 본문의 상대 줄 정보는 별도 기존 계약이며 이번 일반 본문 경로의 근거로 사용하지 않습니다.


- 후보9 Native6쪽은98.63320%로 개선됐고 직접 review에서 본문과 절차 상자 원점을 확인했습니다. 결재문서21번 빈 줄은 실제 흐름436.1px에서 정상 소비되지만, 다음 저장 빈 줄22번이 옛 절대 vpos로538.5px에 재앵커되어76.8px의 빈 밴드를 추가합니다. 재조판 본문 다음 저장 빈 줄의 좌표 기준을 실제 현재 흐름으로 다시 잡고, 이후 연속 저장 빈 줄은 같은 기준에서 상대 간격을 소비하도록 HeightCursor를 보정합니다. typeset의 상태 회수도 lazy/page 기준 모두 반영해 측정과 배치의 좌표계를 일치시킵니다. 빈 줄 높이와 간격을 없애지 않습니다.
- 결재문서1쪽 표의 정상 왼쪽 괘선은76.76px(중심 약77.4px)인데 현재 표 x=81.2px입니다. 원문은 첫 표 뒤에 두 공백과53개 별표가 있는 문단입니다. 기존 첫 글자 공백 조건이 실제 후속 본문이 있는 경우에도 셀 padding과 바깥여백3.76px를 별도 inset으로 추가했습니다. 공백만 있는 호스트의 inset과 실제 본문을 분리하며, #5585의 정상 호스트 대조군도 다시 비교합니다.


- 후보10 결재문서2쪽은95.79660%이며 직접 review에서 빈 줄·아래 결재란 보존을 확인했습니다.1쪽은78.78416%로 아직 미해결입니다. 소방교육10쪽의 마지막 ‘우’는 정상 셀 폭407.1px 안에 들어갈 측정 폭인데도 별도 줄로 출력됩니다. `DocumentCore::fit_hwpx_rowbreak_synthetic_cell_lines`가 저장 빈 anchor 없이 TAC라는 이유만으로 셀 선언 높이의 빈 공간을 내용 부족으로 해석하고 `append_synthetic_cell_line`로 가짜 경계를 만들었습니다. 선언 높이는 물리 공간이며 실제 글줄 개수가 아닙니다. 저장 anchor가 없는 경우의 높이 추정으로 내용 줄을 추가하는 허용을 제거하고, 실제 저장 anchor 근거가 있는 기존 경로는 대조군으로 검증합니다. 원문 문자열·셀 높이·정상 줄 전진량은 유지합니다.


- 후보11 소방교육10쪽은89.99636→96.94338%이며 직접 review에서 여분 ‘우’ 줄 제거와 셀 중앙 배치를 확인했습니다. #5585의4·53·79·86쪽 Native 대조군은 최저92.57665%로90% 이상입니다. 두 입력에 같은 페이지 목록1·10을 준 첫 호출은 결재문서가2쪽이라 exit99였으며 그 호출을 전체 통과로 세지 않고 결재문서 전체를 별도 재실행했습니다.
- 후보12 진단에서 결재문서4번 TAC 개체 줄의 확정 끝은687.5733px이고 최초 높이309.2133/간격12.16px로 정상입니다. 그러나 앞 Page 기준 헤더 표는 typeset이246.4px를 전진하고 paint는248.2px를 점유합니다. `typeset/table::format`은 Para 기준 표만 선언 fit를 적용하지만 `table_layout::resolve_row_heights_with_common_fit`는 Page 기준 표에도 선언 높이를 소비합니다. 한컴 PDF와 현재 header 줄 위치 차이는약1.3px이고, 이1.84px 측정/배치 불일치가 뒤 TAC 표의 확정 원점을 다시위로 옮깁니다. Page/Paper 기준 자리차지 표의 선언 fit를 같은 측정 결과에 반영해 원점·예산을 일치시킵니다. 저장 글줄이나 좌표에 임의 delta를 더하지 않습니다.

- 후보13 소방교육 전체11쪽 Native는 최저92.26525%입니다. 결재문서1쪽은78.78416%로 남았으며 선언 fit 확장은 행 합이 이미236.8px여서 실제 영향을 주지 않았습니다. 이 후보를 해결로 세지 않습니다. 헤더의 쪽 기준 상단 배치가 표 바깥 위 여백138HU를 누락하고, 별도 공간 예약도 같은 여백을 버립니다. 정상 PDF 하단 괘선313.737px와 현재312.4px, 원문 위 여백1.84px를 대조했습니다. 또한 typeset은 표236.8+host gap9.6=246.4px를 소비하지만 paint는 표236.8+아래여백11.37=248.16px를 예약합니다. 쪽·종이 기준 상단 표의 원점/예약에 같은 위 여백을 반영하고, 뒤 본문이 따로 줄간격을 소비하는 경우에는 표 흐름의 여백을 host 글줄간격으로 대신하지 않도록 바로잡습니다. 영향 없는 선언 fit 확장은 제거합니다.
- 규제영향분석서25쪽 Native는 이전84.68649%/현재84.45249%이며 표 외곽의 직접 overlay는 같은 위치입니다. 기존 글꼴/셀 본문 차이는 아직90% 미만이라 통과로 세지 않습니다. KTX 전체27쪽은 요소·문자열을 보존하나 상대크기 반영 및 셀 여백 변경으로 일부 x/y가 변하므로 바이트 동일을 주장하지 않습니다.2·13·17·18쪽 직접 비교를 진행합니다.

- 표 흐름의 host 간격 변경은 실제 post-text 방출 경로에만 적용합니다. 비양수 offset의 단일 절대 표는 `place_table_with_text`가 `pre_table_end_line=0`으로 본문을 뒤에 따로 방출합니다. 양수 offset은 같은 함수가 본문 전부를 앞에 방출하고, 다중 표는 마지막 표가 본문을 소유하므로 그 두 경로의 host 간격을 이번 단일 post-text 근거로 바꾸지 않습니다. 가장자리 원점 helper는 쪽/종이 표의 paint와 두 예약 소비 지점에 공유하며, 문단 기준/글자처럼취급 표는 기존 원점을 유지합니다.

- KTX13쪽 반례: 첫 표의 실제380px는 정상 괘선96.2147–475.8px와 맞지만 이전384px 과측정이 뒤 표의 여백 누락을 가리고 있었습니다. 뒤 표의 정상 상단은495.1387px, 원문 common12492HU+위566HU+아래0HU=저장밴드13058HU입니다. 기존 `tac_stored_band_is_outer_box`는 정확한 이 등식을 확인하면서도 위·아래가 모두 양수여야 한다고 가정하여 위 여백7.55px를 버렸습니다. 한쪽0은 저장 외곽 밴드의 무효 근거가 아니므로 이 잘못된 가정을 제거하고, 단일 실제 저장 밴드·정확한 높이 등식과 음수 여백 배제는 유지합니다. 정상 첫 표 높이를 다시384px로 키우어 보상하지 않습니다.

- 후보16 KTX13쪽은98.70732%/gate passed입니다. 정상 첫 표 높이를 유지하며 뒤 표의 위 여백을 보존했습니다. KTX2쪽은 이전/현재49.70934%로 동일한 미해결이며2쪽 끝 공백 문단 점유 높이를 되살린 기존 계약의 불변 확인과 전 문서 피델리티를 구분합니다.13쪽의 발생 회귀를 기존 차이라고 분류하지 않고 실제 배치를 수정했습니다.

- fresh WASM 비교의 탐색 시간초과 원인도 확인했습니다. `export_wasm_target`이 모든 Native 페이지의 font-face 규칙17개를 각 WASM 페이지에 삽입해1쪽 SVG가305.71MiB(실제 Native6규칙37.54MiB)가 됩니다. WASM의 페이지 소유는 Native와 다를 수 있으므로 Native 동일 쪽의 규칙만 대입하지 않습니다. 전 문서 별칭 공급 목록에서 실제 WASM 페이지의 font-family 참조만 선택하고 같은 font bytes를 보존합니다. 원래 WASM SVG/text/tree/좌표와 변환 후 비CSS 노드를 대조한 뒤 전체 재캡처합니다. 90% 문턱·시간 제한을 완화하거나 Native 트리로 WASM 트리를 대체하지 않습니다.

- WASM 보충 규칙은 페이지의 실제 font-family 속성·inline style·stylesheet 참조만 선택합니다. 원래 fallback 목록도 보존하므로 Native 같은 쪽의 규칙으로 페이지 소유를 강제하지 않습니다. 기존 Python 검사77개가 통과했고, 별칭 목록/인라인 CSS/중복 규칙/사용하지 않는 face 반례는 기존 검사 함수 안에서 보강했습니다(함수 추가 없음). 소방교육1쪽은17→8규칙,320,555,913→159,795,601바이트로 줄었습니다. 선택 규칙과 font bytes는 원본 정책과 같고, 삽입 CSS를 제거하면 원본 WASM SVG 바이트가 그대로 복원됩니다. 실제 글리프·도형 노드도 같습니다. 초기 XML 비교의 root 서식 개행 이동을 출력 변화로 오인한 assertion은 비교 대상을 바로잡아 재확인했습니다. 중단된 첫 WASM 비교와 부분 산출물은 전체 통과로 세지 않습니다.

- 기존 HeightCursor 단위 검사 재판정:161개 중160PASS/1FAIL입니다. 실패는`lazy_path_applied_and_base_set`안의 조건3(직전만 합성LineSeg)에서 이전 기대200HU와 현재800HU가 다릅니다. 조건4(현재도 합성)도 이전 기대200HU에서 기준 없음으로 바뀝니다. 합성vpos는 절대 저장 원점이 아닙니다. 저장 빈 줄2600HU를 실제124px로 연결할 기준은2600−(124−100)×75=800HU이며, 현재도 재조판이면124px를 보존하고 저장 기준을 만들지 않습니다. 정상 결재문서2쪽95.79660%와 소방교육 전체11쪽92.26525% 이상으로 뒷받침했습니다. 기존1함수 안의 두 조건만 보강하고 실제 저장 줄 대조6개 조건은 유지했습니다. 새 함수는 없습니다. 변경은`#[cfg(test)]`뒤에만 있으며 후보16과 비test 코드 바이트가 같음을 확인해 시각 증적 재사용 범위를 고정했습니다.

- 기존 #6101 3쪽 검사의 수집 구간도 재검토했습니다. 이전 표 하단877.0px까지 수집하도록690–900px로 넓히고 쪽수 assertion보다 외곽 검사를 먼저 실행합니다. 하단은 병합 셀로 여러 선분이므로 전폭 한 선분만 선택하면 안 됩니다. 정상 PDF 하단850.269px/현재851.213px/이전877.0px를 직접 추출했습니다. 기대값이나2px 허용치는 완화하지 않습니다.

- 기존 #6660 합성 변형은39개 대조 검사 중1FAIL을 재검토했습니다. 원문 제목 병합 셀은hasMargin=false/표 기본0/보존141HU씩이고 글줄1150HU, 다른 단일 행 셀의 높이는646HU씩입니다. 높이−10/Center/textheight−1은 여백 활성화 근거가 아닙니다. 두 행의 원문 물리 하한1292HU를 요구하고, 활성 셀 또는 명시적 표 여백 조건은1432HU를 유지합니다. 추가 둘째 글줄 조건은실제 끝2300HU를 요구해 기존1432HU보다 강하게 검사합니다. 본문 끝6878HU+활성 여백1700HU도 보존합니다. 정상 원문 전체4쪽 Native/fresh WASM 최저91.87033%를 확인한 뒤 기존 함수 안의 기대값만 수정했습니다. 수동 변형은 정상 생성본의 한컴 출력 증거로 보고하지 않습니다.

- 시험 문서 직접 판독에서1쪽 하단 왼쪽 번호rhwp2/PDF1 차이가 남습니다. 보정65의 보존 review PNG에도 같은 차이가 있어 이번 발생 회귀는 아니지만, 점수91.87033%만으로 완전 시각 승인으로 바꾸지 않습니다. 원문4쪽과 이번 표·그림 원점 계약 확인,90% gate 통과와 번호 미해결을 구분하며 전체 PR 최종 판정 전 후속 보정합니다.


### 보정68 결과 — 현재 브랜치 #6101 개별 완료

- 코드 commit: `1005da62eecb20b7bfd7972f0797d01f1b0aa0fe`. 별도 브랜치·PR을 만들지 않았습니다. 기존 회귀 함수만 수정했고 새 함수는 없습니다. [전체 검증 기록](../assets/pr7382_20260926/stage68_validation.json)에 원문/PDF·생산 코드/CLI/fresh WASM·글꼴·각 페이지 수치·명령·증적 해시를 연결했습니다. 후보16 이후 변경은 기존 검사이며 비test 생산 코드는 동일합니다.

| 대상 | 정상/현재 쪽수 | Native / fresh WASM 전체 최저 | 직접 확인한 결과 |
|---|---:|---:|---|
| 소방교육 `36361137` | 11 / 11 | 92.26525% / 92.26525% | 3쪽 표 하단877.0→851.213px(정상850.269px), 7쪽 표 뒤 본문·지원내용 줄바꿈·절차 표 및8쪽 이월 보존 |
| 결재문서 `36501883` | 2 / 2 | 95.79660% / 95.79660% | 1쪽 헤더 예약/본문 표의 원점,2쪽 빈 줄과 하단 결재란 보존 |
| 시험 문서 대조군 | 4 / 4 | 91.87033% / 91.87033% | 보고된 두 그림이 정상 PDF 원점에서0.80/0.78px 차이. 기존 하단 번호 차이는 아래 제한 참조 |

- `cargo nextest`는 `--locked --cargo-profile release-test --target-dir target/pr-review --test-threads 8 --no-fail-fast`로 실행했습니다. 보강한 #6101 기존2함수는 봉인 이전 출력에서 **0PASS/2FAIL**(표 하단877px/결재 표 좌단81.2px), 현재는 **2PASS**입니다. 관련 기존 대조 검사 **39PASS**, 기존 단위 검사 **161PASS**, Visual Sweep Python 기존 검사 **77PASS**입니다. 처음의 helper 타입 오류와 독립 target 지정 오류는 수정 후 재실행했으며 결함 재현이나 통과로 세지 않았습니다. HeightCursor 단위 검사와 #6660 합성 변형에서 각각1FAIL을 확인했으며, 정상 입력·90% 증거를 대조해 기존 기대값을 바로잡았습니다.
- 파생 suite 준비, fmt/check, Native·WASM lib·workspace all-targets 세 Clippy, workspace build, manifest/unit-tier 정책 검사를 모두 통과했습니다. 정책 base는 최신 fetch의 `d6cf1605327ced1c276f17a529192a743a766209`입니다. 최종 PR 전에 최신 devel 통합과 전체 검증이 필요합니다. Mac fresh WASM은 `--no-opt` 대체 빌드이며 Docker 최적화 빌드 통과가 아닙니다.
- #5585의4·53·79·86쪽 Native 대조 최저92.57665%, KTX13쪽 발생 회귀는98.70732%로 보정했습니다. KTX2쪽49.70934%와 규제영향분석25쪽84.45249%는 후보13의 진단이며 전 문서/최종 후보16의 시각 통과로 보고하지 않습니다. 시험1쪽 왼쪽 하단rhwp2/PDF1 번호는 보정65에도 있던 차이지만 남은 문제로 유지합니다. 소방교육7쪽 절차 표에는약2.8px 차이가 남습니다. 100% 픽셀 일치를 주장하지 않습니다.
- 사용자가 승인한 이전 후보 SVG436개39.95GiB를 정리했습니다. 최종 Native/fresh WASM과 모든 PNG·JSON·원문·기준 PDF·봉인 실행기는 보존했습니다. `.log`, 임시 SVG/글꼴/pkg, 파생 suite는 커밋하지 않으며 사용자 제공 fixture PDF도 원래 untracked 경로에 유지합니다.
- 고정38은 **11해결/1이관/26대기**입니다. 다음은 #7336의17번 기존 함수입니다. 유지37함수 각각의 재실행·전체 nextest·Native Skia3·최종 PR head 검증 전에는 통합 PR 준비 보류입니다.

![Native 소방교육3쪽의 정상 표 높이](../assets/pr7382_20260926/stage68_firefighter6101_native_review_003.png)

![fresh WASM 소방교육7쪽의 표·본문 흐름](../assets/pr7382_20260926/stage68_firefighter6101_wasm_overlay_007.png)

![fresh WASM 결재문서1쪽의 표와 후속 본문](../assets/pr7382_20260926/stage68_approval6101_wasm_review_001.png)

## 보정69 사전 분석 — #7336 기존17번 저장 인라인 표 간격

- 기준 head `8b5f09c566204ab7041c12a075c0b07dee98361f`에서 기존 `hwpx_stored_inline_table_does_not_double_charge_last_line_spacing`만 먼저 실행합니다. 원문은 `samples/issue7336/stored_frame_page_larger_rowbreak.hwpx`이고 기존 기대는6쪽 동의서의 첫 행3040HU(40.5333px)입니다. 저장 행 높이·정상 PDF의 해당 괘선을 독립 대조하며 정상 쪽수 및 Native/fresh WASM 전체 최저90% 검증 전 기존 기대값을 바꾸지 않습니다. 새 회귀 함수는 추가하지 않습니다.

- 개별17번은 exit100: 현재6쪽에 para4/control1이 없어 실패했습니다. 한컴2020·2024 모두 수행계획서는5쪽에서 완결되고 동의서는6쪽입니다. 현재는8쪽이고5쪽의 마지막 행15가6쪽으로 이월됩니다. `row_cut_content_height → block/prepare.cut_row_heights → row_step.whole_row_height/required_height → table_partial의 같은 행 높이`를 추적했습니다. 진단은 행15의 전체 유닛13개가 내용350.9px로 전부 수용되지만 선언377.8667px가 잔여 약377.51px를 약0.35px 초과해 `res.fully_consumed`의 중간행 분기가 행 전체를 이월함을 보여줍니다. 기존0.5px 경계 공차 안에서만 전체 높이를 그대로 수용하고, 모든 내용·안 여백이 실제 예산에 들어간다는 확인을 추가합니다. 선언 축소·새 공차·새 테스트 함수는 추가하지 않습니다. 초과가0.5px보다 큰 행, 내용 미완결, 원본 컷이 있는 행은 종전 이월을 유지합니다. 전체 출력7쪽/Native·fresh WASM90% 및 직접 괘선·후속 본문 확인 전 해결로 판정하지 않습니다.

## 보정69 결과 — #7336 기존17번의 페이지 소유 복구

- 기존 검사는 유효합니다. 한컴2020·2024 모두7쪽이며6쪽 동의서 첫 행은 저장3040HU와 정상 PDF 약40.44px로 대응합니다. 기존 기대40.5333px/공차0.3px를 유지했고 테스트 파일·함수는 수정하지 않았습니다. 수정 전6쪽에 표가 없어 exit100, 최종 코드에서 개별1 PASS(exit0)입니다.
- 마지막 행의13개 유닛은350.9px 내용으로 모두 들어갔지만 선언377.8667px가 잔여를 약0.35px 초과해 행 전체가6쪽으로 밀렸습니다. `row_step`에서 전체 내용·안 여백의 실제 수용과 기존0.5px 경계를 함께 확인한 후 원래 높이·모든 유닛을 예약합니다. clamp·행 축소·새 공차·기준값 갱신은 없습니다. 실제 컷, 미완결 내용, 큰 초과는 종전 경로를 유지합니다. 수행계획서5쪽 마지막 행 → 동의서6쪽 → 확인서7쪽으로 원본 소유를 복구했습니다.
- 원문 본문 아래 여백만0/8/20HU 늘린 진단 대조군은 수정 전8/8/8쪽, 수정 후7/7/8쪽입니다. +20HU는0.5px 경계를 넘으므로 종전 이월을 유지합니다. 이 자료는 수동 변경한 용량 계약 진단이며 정상 한컴 출력 증거나 새 정식 회귀 함수로 취급하지 않습니다.
- 최종 기존 집중 검사 **57 PASS/3183 SKIP**, 별도17번 **1 PASS/215 SKIP**입니다. fmt/check·Native Clippy·WASM lib Clippy·workspace build·workspace all-targets Clippy·고정base `d6cf1605327ced1c276f17a529192a743a766209` manifest/unit-tier 검사·release-test CLI·fresh WASM은 모두 exit0입니다. WASM은 Mac 로컬 `--no-opt` 대체 빌드입니다. 루트pkg/Studio js·wasm 해시가 같습니다.

| 원문/정상 기준 | 전쪽수→후쪽수/기준 | Native 전체 최저 | fresh WASM 전체 최저 | 판정 |
| --- | --- | --- | --- | --- |
| stored_frame_page_larger_rowbreak.hwpx / 동일이름-2020.pdf | 8→7/7 | 95.81586% | 95.81586% | 두 backend 전7쪽 gate PASS |
| nested_table_fragment_cut.hwp / 동일이름-2020.pdf | 6→6/6 | 92.83181% | 92.83181% | 두 backend 전6쪽 gate PASS |

- 대상5쪽 review/overlay의 하단 내용,6쪽 동의서,대조군4~5쪽 중첩 표와WASM6쪽 후속 내용을 직접 대조했습니다. 대상5쪽 외곽 하단에는 약6.5px 차이,6쪽 동의서 상단에는 약1.8px 차이가 남습니다. HWP 대조군5쪽 제목 상자의 차이도 남으며6쪽 모두 render tree가 수정 전후 byte 단위로 같습니다. 점수100%인 페이지도 픽셀 일치나 전체 피델리티 완성으로 승격하지 않습니다. 앞 보정68의 시험 문서 쪽번호2/1 차이도 해결했다고 보고하지 않습니다.
- source/runtime/입력·정상 PDF·글꼴·페이지별 수치·명령·대표 PNG 해시는 [보정69 검증 기록](../assets/pr7382_20260926/stage69_validation.json)에 연결합니다. 산출물 원본은 `output/pr-review/planet6897-7382-20260926/stage69-stored7336/visual-final-native`와 `visual-final-wasm`입니다. 원본 문서·PDF를 유지하고 `.log`, 임시 진단 HWPX,SVG 폰트 payload,pkg/generated 파일은 커밋하지 않습니다.
- 이번 단계는17번만 개별 해결로 갱신합니다. 집중 묶음에서 통과한 다른 대기 함수는 개별 분석·실행·기록을 완료하기 전 해결 수에 넣지 않습니다. 고정38은12해결/1이관/25대기이며 다음은18번 쪽수 검사입니다. 유지37의 개별 재실행·전체 nextest·Native Skia3·최종 통합 head 검증이 남아 **PR 준비 보류**입니다.

![Native 5쪽에서 복구된 마지막 행](../assets/pr7382_20260926/stage69_stored7336_native_review_005.png)

![fresh WASM 6쪽 동의서와 정상 PDF](../assets/pr7382_20260926/stage69_stored7336_wasm_review_006.png)

![fresh WASM 중첩 표 대조군5쪽](../assets/pr7382_20260926/stage69_nested7336_wasm_review_005.png)

- 보정69 코드 commit: `b7436944b81d20ad799edcd59ead3b94621902b3`. 고정 CLI/fresh WASM의 생산 소스 해시와 이 commit의 파일 해시가 동일함을 확인했습니다.

- 사용자 추가 요청으로 [사용자 제공 지표 기준 PDF](../../../tests/fixtures/stored_float_anchor_control/1351000_policy_indicators-2020.pdf)를 현재 fixture 경로에서 함께 커밋합니다. 86쪽/792,765byte, SHA-256 `4873b182ad42ffe1cea6ebd1a585691b703f32cf6cda769a5124d715bd850b96`로 앞 보정65에서 채택한 사용자 원본과 같습니다. 이번 추가는 기준 증적 보존이며 별도 검사 추가나 기대값 갱신을 수반하지 않습니다.

## 보정70 사전 분석 — #7336 기존18번 전체 쪽수

- 기준 head `b87c2130ba41aa33b4b92cfa2187eb90f4933c4f`에서 `stored_frame_page_larger_rowbreak_table_is_split_across_pages`만 개별 실행합니다. 기존7쪽은 한컴2020·2024 모두의 정상7쪽과 일치합니다. 보정69에서 마지막 행과동의서 소유를 복구한 같은 원문이므로 기존 기대를 유지하며, 먼저 실제 개별 실행 결과를 확인합니다. 새 함수는 추가하지 않습니다.

## 보정70 결과 — #7336 기존18번 쪽수 개별 확인

- 기존7쪽 기대값은 한컴2020·2024의 정상 PDF 모두7쪽과 일치하므로 유지합니다. 보존된 보정55 전체 실행의 이 함수는 실제8/기대7로 실패했고, 당시와 현재 테스트 파일 SHA-256이 동일합니다. 이번 단계에서는 수정 전 검사를 새로 실행했다고 보고하지 않습니다.
- 보정69 코드 `b7436944b81d20ad799edcd59ead3b94621902b3`를 포함한 `b87c2130ba41aa33b4b92cfa2187eb90f4933c4f`에서 기존18번만 `cargo nextest`로 실행하여 **1 PASS/215 SKIP, exit0**을 확인했습니다. `release-test`, `target/pr-review`, threads8을 사용했고 코드·기대값·함수는 추가 수정하지 않았습니다.
- 같은 원문 전7쪽 Native/fresh WASM 최저95.81586%와 직접 확인한 페이지 소유·잔여 위치 차이는 [보정69](../assets/pr7382_20260926/stage69_validation.json)를 연결합니다. 생산 코드 해시와 코드 commit 이후 문서/증적만 변경된 사실을 다시 확인했습니다. 전체 head 새 캡처나 전체 회귀 통과로 보고하지 않습니다. [보정70 개별 검증](../assets/pr7382_20260926/stage70_validation.json)에 이전 실패·현재 명령·출력·해시를 보존했습니다.
- 고정38은 **13해결/1이관/24대기**입니다. 다음 대기는1번 본문 영역 초과 검사이며, 유지37 각각 재실행·전체 nextest·Native Skia3·최종 통합 head 검증 및 직접 발견된 쪽번호 차이 해소 전 **PR 준비 보류**를 유지합니다.

## 보정71 사전 분석 — 기존1번 본문 영역 초과 partition0

- 기준 head `aa7dc390c6e11e98658926513150be8f93695e7b`에서 `body_overflow_does_not_grow_partition_0`을 먼저 개별 실행합니다. 실패 시 증가 문서와 본문 경계를 넘는 실제 노드를 식별한 뒤 저장 정보·기준 한컴 PDF·같은 페이지 Visual Sweep으로 검사 적절성을 판정합니다. 기존2px 공차와 baseline은 유지하고, 90% 미달 원문은 렌더링부터 개선합니다. 새 검사 함수는 추가하지 않습니다.

- 개별1번은1PASS/228SKIP,현재17건/기존19건입니다. 첫 잘못된 suite005 실행은0검사(exit4)로 제외하고 실제suite025에서 재실행했습니다. 그러나 문서는203/정상205쪽이며39/53쪽Native47.03148/58.70083%로 보류입니다. 첫 페이지 소유 차이는그림7:원문para109의저장vpos300/줄높이21000HU가이전para108의저장끝52982HU 뒤같은본문71154HU에들어가지않지만인라인Picture가reset분기에서제외되어실제21쪽에붙습니다. `apply_stored_paragraph_boundary→advance_column_or_new_page→실제페이지배치`에공통경계를복원하는후보를검증하며,baseline과테스트기대는바꾸지않습니다.

- 사용자 후속 지시로61,810,688byte 생물독 문서를 [#7445 추가 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5868460165)했습니다. 원본은`mydocs/pr/assets/issue7445/1480000-201900698-native-neartop-reset.hwp`로해시동일하게보존하고정상PDF205쪽도유지합니다. 큰문서의전용2함수와원장3행만제거하며,작은#5921검사와#7147나머지2함수 및generic partition함수는유지합니다. 렌더러후보는철회했고후보빌드는exit130으로중단했습니다. 제거를해결/PASS로바꾸지않습니다. 다음실패검사도동일입력의Visual Sweep이90%미만이면#7445등록후관련검사를제거하는사용자지시를적용합니다.

- 이동후크기기반corpus partition0은69개문서/스킵1개로재편되어다른문서`issue6031/3249937_asset_management_rules.hwpx`의1→15건증가로FAIL(exit100)입니다. 제거대상생물독문서의재실행실패가아니며다음개별검토대상입니다. 분할번호만으로이전실패원인과같다고간주하지않습니다. 전체회귀PASS·수용가능으로보고하지않습니다.

## 보정71 결과 — 사용자 승인 생물독 문서 이관

- 제거 commit: `ca499e732a076009496da9f6b5a90ccdc2442698`. 61,810,688byte HWP를100%동일한rename으로보존하고전용2함수·세원장행을제거했습니다. 렌더러코드변경·새검사·ignore/공차완화는없습니다. 독립PDF205쪽·대표Native39/53쪽review/overlay를보존했습니다. [제거·검증 기록](../assets/issue7445/neartop5941_test_removal_validation.json).
- fmt/check, Native Clippy, WASM lib Clippy, workspace build, workspace all-targets Clippy 및고정base `d6cf1605327ced1c276f17a529192a743a766209` manifest 검사는모두exit0입니다. source-side 단위검사는수정하지않았습니다. `.log`,output,pkg,generated파일은커밋하지않습니다.
- 유지한작은#5921은1PASS/213SKIP입니다. body partition0은재배정된#6031 자산관리규정의1→15건증가로1FAIL/231SKIP이며, #7147의다른시장구조조사원문은기존2함수모두pi934표의쪽위치조건으로FAIL입니다. 두함수와helper는제거전HEAD와같고production도불변이므로생물독원문제거를이문서의수정으로보고하지않습니다. 새로확인한두원문의독립PDF와90%조건을이후개별판정합니다.
- 고정38의해결수를올리지않습니다(13개별완료/1함수이관/24대기). 전체 corpus소속과suite소속이바뀌어유지37개함수각각을새소속에서다시실행해야합니다. 새정책은실패문서의실제Visual Sweep이90%미만일때#7445에추가등록한뒤관련검사를제거하는것이며,정상출력승인·피델리티해결로세지않습니다. 다음은현재1번의#6031 원문입니다.

## 보정72 사전 분석 — 현재1번의 #6031 자산관리규정

- 기준head `711cea1ac2f41bb33199dd800a8c983462af7ec6`에서corpus재배정으로드러난`3249937_asset_management_rules.hwpx`의본문초과1→15건을개별검토합니다. 독립PDF는기존정상60쪽이며파일명이2020이어도Creator는Hwp2022로표기됩니다. 버전문자열만으로재생성하지않고실제원문대응을대조합니다. 실제본문초과쪽과기존3·4·6·41·42쪽계약을먼저Visual Sweep으로판독한후90%미만은승인된#7445이관/검사제거경로를적용합니다.

- Native3·4·6·15·41·42쪽은50.62960/49.34186/39.04806/29.16104/25.22086/25.84322%로모두90%미만입니다. 현재59/독립60쪽,본문바닥초과15건이며15쪽문단/제목의배치와41쪽붙임4/신고서의소유차이를직접판독했습니다. [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5868575524)후원본HWPX를이슈자산에해시동일하게이동했습니다. 문서전용2함수·원장3행·공통#7080의이원문입력한개만제거하며,렌더러·다른원문·5개공통함수·최소20개관측·공차는유지합니다. 전체/fresh WASM미실행을통과로승격하지않습니다.

## 보정72 결과 — 사용자 기준에 따른 자산관리규정 검사 이관

- 검사·입력 이관 commit `5a0d2d48ead8f5595f55ce673b7e4957b157c3ef`. 원본 HWPX는100%동일한rename이며 독립PDF60쪽을 유지했습니다. 전용2함수·원장3행·#7080의 해당 입력만 제거했습니다. 렌더러와 다른 원문·공차는 바꾸지 않았습니다. [실제 비교·검사 결과](../assets/issue7445/assets6031_test_removal_validation.json).
- #7080 유지5함수는5PASS입니다. fmt/check·Native/WASM/workspace Clippy·workspace build·고정base manifest 검사는모두exit0입니다. source-side 단위검사변경은없습니다.
- body partition0은70문서/스킵0으로재편됐고 `task2097/75544_pii_bunseok.hwpx`의1→2건으로1FAIL입니다. 자산관리규정 제거를이문서의수정이나전체통과로보고하지않습니다. 다음은이문서의실제초과페이지와독립PDF를개별검토합니다.
- 고정38은13개별완료/1함수이관/24대기를유지합니다. 전체37함수재실행·전체nextest·NativeSkia3 및최종head검증전에는PR준비보류입니다. `.log`·output·파생suite는커밋하지않습니다.

## 보정73 사전 분석 — 현재1번의 개인정보 분석 편람

- 기준head `8672401774d5a173ddcc681ceb844124dad5cf62`에서 `task2097/75544_pii_bunseok.hwpx`의본문초과1→2건을개별검토합니다. 기존 `75544_pii_bunseok-2020.pdf`는cairo출력이므로독립정답지로채택하지않고, 별도한컴PDF `75544_pii_bunseok-hwpx-2020.pdf`66쪽과실제초과쪽·기존59/60쪽내용소유를비교합니다. 실제Creator는Hwp2022이며파일명만으로생성엔진을추정하지않습니다. 렌더러/기대값수정전Native실제출력을확인하고,90%미만이면사용자승인#7445등록/검사제거를적용합니다.

- Native22/46/59/60쪽은93.59725/65.56940/30.66565/30.63875%이며현재70/기준66쪽입니다. #5846전용함수도66쪽기대로1FAIL(exit100)했습니다. 실제46쪽표높이/본문흐름과59쪽의법령/서식내용소유차이를직접확인한뒤 [#7445에추가등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5868672890)했습니다. 원본HWPX를해시동일하게보존하고전용1함수와렌더링/쪽수원장6행만제거합니다. 정적IR/직렬화자료·다른원문·공차는유지하며전체/fresh WASM통과와렌더링해결로세지않습니다.

## 보정73 결과 — 개인정보 분석 편람 이관 후 개별1번 완료

- 검사/입력 이관commit `63a6e9eadb9ef190142f6eeeb1f0394e9b7fdc8b`. 원본HWPX의SHA-256 `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1`은이동전과같고, 두기존PDF·Native대표22/46/59/60쪽PNG와정적IR자료를보존했습니다. [검증결과](../assets/issue7445/pii75544_test_removal_validation.json). 전용1함수와원장6행만제거했으며렌더러와다른원문·공차는불변입니다.
- 재편된body partition0은1PASS, render-page fixture계약기존6함수도6PASS입니다(합계7PASS/442SKIP,exit0). fmt/check·Native/WASM/workspace Clippy·workspace build·고정base manifest는모두exit0입니다. source-side검사수정은없습니다.
- 원래38의1번은승인된미달원문의이관후남은corpus검사통과로개별완료했습니다. 생물독/자산관리규정/개인정보편람의피델리티해결로세지않습니다. 고정38은14개별완료/1함수이관/23대기이며, 다음은기존2번body partition1입니다. 유지37함수각각의최종재실행·전체nextest·NativeSkia3와최종head검증전에PR준비완료로보고하지않습니다. 로그·임시SVG·파생suite는커밋하지않습니다.

## 보정74 사전 분석 — 기존2번 body partition1

- 기준head `24a5c69ea67bf8cbd2e697e87e359a7b25b7737f`의현재corpus와실제suite `regression_suite_025`에서기존2번만실행합니다. 문서이관뒤크기기반partition소속이달라질수있으므로이전partition1실패문서를현재원인으로추정하지않습니다. 실제실패파일·증가노드와독립기준PDF를확인한뒤수정또는승인된90%미만이관을판정하며, baseline공차를완화하거나새검사를추가하지않습니다.

- 기존2번은현재 `hwp3-sample16-hwp5.hwp`1→3건, `issue3637/press_release_split_cell_nested_table.hwpx`2→3건, `rowbreak-problem-pages.hwp`신규1건으로1FAIL입니다. 우선첫원문만판정합니다. 저장제품한컴2024와대응하는한컴PDF64쪽을채택했고현재65쪽입니다.23쪽TextLine3개가본문하한을26.987/54.720/97.547px넘습니다.23·24·64쪽Native를비교해검사수정또는90%미만이관을결정하며다른두원문은같이제거하지않습니다.

- 첫원문Native23/24/64쪽은26.53989/7.09794/15.52702%입니다.23쪽글줄의외곽선/꼬리말넘침과24쪽한줄만남는내용소유차이를직접판독하고 [#7445에등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5868738187)했습니다. 이원문만해시동일하게보존경로로이동하고본문초과/쪽수원장3행만제거합니다. #4680문단style_id파서계약3함수는그대로유지하고경로만갱신합니다. 다른두실패원문·HWP3/다른연도변환본·정적IR·공차는변경하지않으며전체/fresh WASM통과와피델리티해결로세지않습니다.

- 참조전수확인보완:첫workspace lint는source-side `include_bytes!` 누락으로exit101입니다. 메인터너참조조사누락을수정하여root/pilot의활성렌더링전용11함수·관련helper와#2158/source-side의해당입력을제거했습니다. #4680/#3693/#3695/#3744/#4155및진단의IR입력경로는보존자산으로갱신합니다. 제거스크립트가남긴중복 `#[test]`와인자있는helper의test속성은수정했습니다. 이빌드실패는원문렌더링결함증거로세지않습니다. 최종fmt후suite소속을다시생성·해석하고기존유지검사/필수lint/source-side unit-tier를다시실행합니다.

## 보정74 결과 — HWP5 미달 원문 이관과 기존2번 완료

- 이관commit `bd2c2be0f891a13cc2b239301392e3670d9ae143`. HWP5 원문3,043,328byte/SHA-256 `6a3cdf2c148bf39f40ab06e847767e7f722543fce59e6ee4328ef44b835f3f45`는100%동일한rename이며한컴2024PDF64쪽과Native23/24/64쪽PNG를보존했습니다. 전용렌더링/쪽수11함수·관련helper·해당matrix입력/원장3행을제거하고파서/구조/음영/진단경로는보존원문으로갱신했습니다. src변경은cfg(test)안의해당검사/helper제거뿐이고production함수는불변입니다. [참조누락·속성/주석오류의수정과최종검증](../assets/issue7445/sample16_test_removal_validation.json).
- 최종fmt/check·Native/WASM/workspace Clippy·workspace build·고정base manifest와unit-tier는모두exit0입니다. source-side유지2검사는2PASS/4066SKIP입니다. 확대기존검사는45PASS/1FAIL/1492SKIP이며body partition1과#4680/구조/음영/fixture계약은PASS입니다. 실패는이관HWP와다른입력인`hwp3-sample16-hwp5.hwpx`의65/64쪽입니다. 같은내용의다른형식을현재HWP의시각판정으로같이제거하지않습니다.
- 원래38은15개별완료/1함수이관/22대기입니다. 개별1/2의PASS는사용자승인미달원문이관뒤현재corpus에대한결과이고,재배정된다른실패문서나이관원문의피델리티해결로바꾸어보고하지않습니다. #3637보도자료·rowbreak HWP·시장구조조사#7147의기존2함수와새HWPX실패를계획의추가미해결목록에보존했습니다.
- 다음은새HWPX실패의독립PDF/실제Visual Sweep판정입니다. 원래3번partition10이후의개별검증·유지37함수재실행·전체nextest/NativeSkia3·최종head검증을완료해야합니다. 현재PR준비보류이며로그/임시SVG/파생suite는커밋하지않습니다.

## 보정75 사전 분석 — 별도 sample16 HWPX의 쪽수 실패

기준 head `31222d1da`의 #2158 HWPX 검사는 실제65/기대64쪽으로 실패했습니다. HWP5 이관의 판정을 다른 입력으로 옮기지 않고, 저장 한컴2024 HWPX와 기존 독립 한컴2024 PDF64쪽을 대조했습니다. 생산 코드가 보정69 이후 바뀌지 않은 고정 CLI로 Native 음성 증거를 먼저 수집했습니다.

## 보정75 결과 — HWPX 미달 원문 보존과 승인된 검사 제외

- 선택3·5·6·18·23·24·64쪽은44.78343/58.63140/22.67417/67.86130/22.32296/41.72462/11.02735%로 모두90% 미만입니다. 6쪽 본문·표·수식의 세로 위치와64쪽 합의각서 소유 차이를 직접 확인했습니다. 전체/fresh WASM 미실행이며 시각 승인으로 세지 않습니다.
- [#7445 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5869485533)에 별도 원문을 등록한 뒤 전용4함수와 corpus3행만 제거했습니다. 원문은 바이트 동일하게 이관했고 PDF는 기존 경로에 유지했습니다. 다른 입력의 온새미로·HWP3 수식 및 gradient IR 파싱 검사를 유지합니다. 코드 commit `aa8b4c00385f6e28d3b68ae4f1d95f4038cac900`에 새 검사·skip/ignore·공차 완화·생산 코드 변경은 없습니다.
- 유지 집중7PASS/0FAIL/659SKIP, fmt·Native/WASM/전체 target Clippy·workspace build·고정 base manifest/unit-tier는 모두exit0입니다. [검사 제거 근거와 증적](../assets/issue7445/sample16hwpx_test_removal_validation.json), 로컬 `output/pr-review/planet6897-7382-20260926/stage75-sample16hwpx/`에 명령·로그를 기록했습니다.
- 고정38의15검증 완료/1이관/22대기는 유지합니다. 다음3번 개별 검사, 재배정된 별도 실패 문서의 검증, 유지37 재실행 및 전체 nextest·Native Skia·최종 head 시각 검증이 남아 통합 PR 준비는 보류합니다.

## 보정76 결과 — 고정 실패3·4번의 개별 실행

- head `24d83f3a4`에서3번 body partition10을 정확히 선택해 개별1PASS(2.431초/233SKIP,exit0)했습니다. corpus 이관 후 남은 문서의 검사 결과이며 이관 문서의 피델리티 해결로 세지 않습니다.
- 다음4번 body partition15는 개별1FAIL(0.532초/233SKIP,exit100)로 중단했습니다. `hwp3-sample16-hwp5-2010.hwp` 본문초과1→3건, 보도자료 HWPX2→3건, rowbreak HWP 신규1건입니다. 다른 버전 입력의 이관으로 이 세 문서가 해결됐다고 판단하지 않습니다.
- 고정38은16검증 완료/1이관/21대기입니다. `output/pr-review/planet6897-7382-20260926/stage76-individual-pending/individual-results.json` 및 개별 로그에 정확한 suite·명령·종료 코드를 보존했습니다. 다음 단계에서2010 저장본부터 같은 입력의 독립 PDF로 판단합니다.

## 보정77 사전 분석·결과 — 2010 이름 sample16의 독립 피델리티 판단

고정4번의 원문2010 저장본을 별도 검증했습니다. 실제 저장 metadata는 한컴2024이며, 기존 cairo PDF 대신 같은 입력의 독립 한컴2024 PDF64쪽을 사용했습니다. 선택7·22·23·24·64쪽은21.76084/22.15574/26.53989/7.09794/15.52702%, 실제65/기준64쪽으로 미달입니다. 23쪽 본문 넘침과24쪽의 한 줄만 남은 내용 소유를 review/overlay에서 직접 확인했습니다. 전체/fresh WASM은 미실행이며 승인 증거로 세지 않습니다.

[#7445 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5869562651)에 증거를 등록한 뒤 사용자 지시에 따라 #1105 전용2함수와 corpus2행만 제거했습니다. 원문은 바이트 동일하게 보존하고 다른 버전·입력의 함수/공차 및 IR 진단을 유지합니다. 코드 commit `123e7260c107e4f45f04521b395b1906d8d40256`, [보존·검사 제거·검증 근거](../assets/issue7445/sample16_2010_test_removal_validation.json). 새 함수·생산 코드·skip/ignore·허용치 완화는 없습니다.

집중8PASS/0FAIL/865SKIP, body partition15 개별PASS이며 필수 lint·workspace build·고정 base manifest/unit-tier는 모두exit0입니다. 재배정된 보도자료·rowbreak 문제는 해결로 세지 않고 별도 대기합니다. 고정38은17검증 완료/1이관/20대기입니다. 다음5번부터 개별 실행한 뒤 전체검증을 진행합니다.

## 보정79 사전 분석·결과 — sample16 2022 입력의 독립 검증과 제외

고정5번 개별 실패에서2022 입력의 본문초과2→4건과 관제교육39→40건을 구분했습니다. 먼저 같은2022 원문과 독립 한컴2022 PDF64쪽을 검증했습니다. 실제65/기준64쪽, 선택3·6·23·24·64쪽은38.04778/33.70787/26.53989/7.09794/15.52702%로 미달입니다. 3쪽 문단·도형 간격과24쪽 페이지 소유를 직접 판독했습니다. 전체/fresh WASM은 미실행이며 승인 증거가 아닙니다.

[#7445 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5869647971)에 먼저 등록한 뒤 사용자 승인 범위에서 전용6함수와 corpus2행을 제거했습니다. 원문은 바이트 동일하게 보존했고, 다른 버전·입력 및 저장 제품 info/IR 검사는 유지합니다. 코드 `ced1d9ef299af17850c9f43ec3c4d91f914baa6c`, [원문 보존과 검증 근거](../assets/issue7445/sample16_2022_test_removal_validation.json). 새 함수·생산 변경·skip/ignore·공차 완화는 없습니다.

유지8PASS/다른3원문에 대한 body partition2 1FAIL/853SKIP(exit100)이며 필수 fmt·Clippy3종·workspace build·고정 base 정책은exit0입니다. 해당3원문은 보도자료2→3건·관제교육39→40건·rowbreak HWP신규1건입니다. 고정38은17검증 완료/1이관/20대기를 유지하고 다음 보도자료부터 개별 시각 검증합니다. 전체검증과 PR 준비는 미완료입니다.

## 보정80 사전 분석·결과 — 보도자료의 미달 회귀 제외

고정5번의 보도자료HWPX는 본문초과2→3건과 실제13/독립 한컴PDF12쪽으로 실패했습니다. 기존cairo PDF를 독립 기준으로 쓰지 않고 같은 원문의 한컴 출력으로 선택4·5·7·8·12쪽을 비교했습니다. 점수97.23437/95.89646/91.68111/83.85004/36.41033%; 8쪽 중첩 상자/글줄·배경,12쪽 향후계획 표와 마지막 내용의 소유 차이를 직접 확인했습니다. 전체/fresh WASM 미실행이며 시각 승인으로 세지 않습니다.

사용자 지시에 따라 [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5869729786) 후 전용4함수와 corpus3행을 제거했습니다. 원문은 바이트 동일하게 이관하고 PDF를 유지했습니다. 다른 issue3637 입력과 합성 중첩 행 계약·함수·공차를 유지하며 생산 변경/새 함수/skip/ignore가 없습니다. 코드 `521cee34a1d62ae144e2db28ab015f95bb39fd99`, [검증·보존 근거](../assets/issue7445/press3637_test_removal_validation.json).

유지5PASS/다른입력2FAIL/849SKIP(exit100), 필수 lint·workspace build·고정 base 정책은exit0입니다. 실패는 별도 규제영향HWPX 셀 위치 검사와 관제교육/rowbreak HWP의 본문 넘침입니다. 원래 고정38은17검증 완료/1이관/20대기이며 다른 원문도 한 개씩 판정합니다. PR 준비는 전체검증 완료 전까지 미완료입니다.

## 보정81 사전 분석·결과 — rowbreak HWP의 미달 회귀 제외

고정5번의 신규 본문 초과1건은 `samples/rowbreak-problem-pages.hwp` 3쪽 표 조각의2.26667px 초과입니다. 같은 원문의 독립 한컴2024 PDF와 비교한 실제/기준 쪽수는18/18이지만, 선택 Native3·8·12·13·17·18쪽은96.83411/49.39444/80.05503/71.99775/99.82183/88.25163%입니다. 8쪽의 이전 내용 이어받기와 제27조 셀 경계/뒤 항목 소유 차이를 직접 확인했습니다. 선택6쪽 실행은 완료했으며 전체/fresh WASM은 미실행입니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5869840624) 후 HWP 원문을 바이트 동일하게 이관하고, 전용5함수와 공용4함수의 HWP 입력 및 corpus3행만 제거했습니다. HWPX 입력·다른 matrix·기대값·공차는 유지합니다. 순수 #1770 파서 origin-marker 함수는 보존 경로만 수정했습니다. #4967 cache-key 함수는 실제 page tree를 검사하므로 HWP 렌더링 검사 제거 범위입니다. 새 함수/skip/ignore/생산 변경은 없습니다.

초기 전체 target Clippy의 잔여 문서 주석과 단일 입력 loop 오류를 수정해 같은 범위를 다시 검증합니다. 처음 지원하지 않는 Sweep 옵션 호출은 capture 이전 setup 실패이며 올바른 옵션으로 선택6쪽을 완주했습니다. 이 오류를 렌더링 결함 재현으로 세지 않습니다.

최종 최신 배정 집중 검사는37PASS/다른 입력4FAIL/1265SKIP(exit100)이며 fmt·Clippy3종·workspace build·고정 base manifest/unit-tier는exit0입니다. 실패는 별도HWPX의7쪽2함수, exact-face 문서의 추적건수1334/기대1336, 관제교육39→40건입니다. 이번HWP의 제외를 다른 원문의 해결이나 전체 검증 통과로 세지 않습니다. 코드 `66510d325d7c085e684a914f4176d953bf9eef38`, [보존·실행 증거](../assets/issue7445/rowbreak_hwp_test_removal_validation.json). 고정38은17완료/1이관/20대기이며 다음은 고정5번의 관제교육 원문입니다. PR 준비는 미완료입니다.

## 보정82 사전 분석·결과 — 관제교육의 미달 회귀 제외와 기존5번 완료

기존5번의 마지막 관제교육 본문초과39→40건을 독립 한컴 PDF로 검증했습니다. 실제201/기준204쪽, 선택 Native39·103·124·194·201쪽98.12011/37.65331/23.8889/38.95167/11.45426%입니다.103쪽 로드맵과 설명의 소유,201쪽 역량 표와 부록 소유 차이를 review에서 직접 확인했습니다. 전용182·183쪽의 경계 정확성은 이번 선택 캡처로 입증하지 않았고, 전체/fresh WASM 미실행입니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5869952956) 뒤 원문을 바이트 동일하게 보존하고 전용4함수와 해당 원장3행을 제거했습니다. 다른 문서·기대값·공차를 유지하고 새 함수/skip/ignore/생산 변경은 없습니다. 기존5번 개별1PASS/239SKIP(exit0), fmt·Clippy3종·workspace build·고정 base 정책exit0입니다. 코드 `1d91421c4d0b885e3fec99d549d490758876de1c`, [실행·보존 증거](../assets/issue7445/air6764_test_removal_validation.json). 고정38은18완료/1이관/19대기이며 다른 실패와 최종 전체 회귀·fresh WASM/시각은 남아 PR 준비 미완료입니다.

## 보정83 결과 — 기존6번 개별 실행

head `1fddec84c4d05919323e190d0068046be624d373`에서 기존6번 body partition3을 nextest release-test/공유target/8threads로 단독 실행했습니다. 1FAIL/239SKIP(exit100,44.776초)이며 증가 원문은 전기안전관리규정70833의2→3건입니다. 시장구조조사의44.728초는 같은 corpus의 실제 장기 샘플 실행이며 출력 공백을 중단 사유로 삼지 않았습니다. 첫 실패에서 다음 함수를 멈추고 독립 PDF와 해당 원문의 시각 비교를 먼저 수행합니다. 생산·검사·기대값 변경은 없습니다. [개별 실행 근거](../assets/pr7382_20260926/stage83_individual_validation.json). 고정38은18완료/1이관/19대기, PR 준비 미완료입니다.

## 보정84 사전 분석·결과 — 전기안전규정 제외와 기존6번 완료

기존6번의 전기안전규정 본문하단2→3건을 같은 원문/독립 한컴18쪽 PDF로 검증했습니다. 실제18/기준18쪽이나 선택 Native5·6·10·14·18쪽37.05508/35.31196/19.83933/93.80158/20.13671%입니다.5쪽은 앞 표가 반복되고 정상 다음 표가 표시되지 않으며10쪽은 규제 적정성 대신 이전 이해관계자 표/규제목표가 표시됩니다. 해당 review를 직접 확인했고 전체/fresh WASM은 미실행입니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5870144757) 후 원문을 바이트 동일하게 이관하고 HWP 전용2함수와 해당corpus2행만 제거했습니다. 같은case의 춘천HWPX2함수·helper·기대값·공차 유지, 새 함수/skip/ignore/생산 변경 없음. [입력·검증 근거](../assets/issue7445/electrical6854_test_removal_validation.json), 코드 `5dcf8eb2c96da81b4b8054c888d0892ae25d8a2d`입니다. 기존6번 단독1PASS와 유지HWPX2PASS(집중3PASS/466SKIP,exit0), fmt·Clippy3종·workspace build·고정base정책exit0입니다. 원문 피델리티나 다른 입력 해결로 세지 않습니다. 고정38은19완료/1이관/18대기, 최종 전체 검증과PR준비 미완료입니다.

## 보정85 결과 — 기존7번 개별 실행

head `88dea0a3a0ddec32cf59a9280c7bf297cc776672`에서 기존7번 body partition4을 nextest release-test/공유target/8threads로 단독 실행했습니다. 1FAIL/232SKIP(exit100,0.540초), 권익위 의결사항30269의 본문하단 신규3건이 실패 원문입니다. 첫 실패에서 다음 함수를 멈추고 원문 독립 PDF와 실제 비교로 먼저 판정합니다. 생산·검사·기대값 변경 없음. [실행 근거](../assets/pr7382_20260926/stage85_individual_validation.json). 고정38은19완료/1이관/18대기이며PR준비 미완료입니다.

## 보정86 사전 분석·결과 — 권익위 문서 이관과 기존7번 완료

기존7번의 권익위 의결사항 본문하단 신규3건을 독립 한컴 PDF로 검증했습니다. 실제22/정상22쪽이나 선택 Native2·5·6·22쪽100.0/68.7493/29.19654/94.66351%입니다.5쪽의 다음 장 제목이 본문 하단·쪽번호 영역에 미리 표시되고 도형/본문 위치가 다른 것을 review에서 직접 확인했습니다. 목차4쪽의bbox 자체가 잘못된 기대값이라고 확정하지 않으며 전체/fresh WASM 미실행입니다.

#6844와#6023의 두 등록 경로는 SHA-256/470,016byte가 같은 중복 원문입니다. [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5870283939) 후 둘 다 바이트 동일하게 보존하고 전용렌더링5함수를 제거했습니다. #6806의 속성 getter/setter·저장0복원은 유지하며 경로만 이관하고 undo 함수의 SVG 비교는 제거했습니다. IR추출 자료를 보존합니다. 렌더링 원장행은 원래 없으므로 추가/완화가 없습니다. 다른 입력·기대값·공차 유지, 새 함수/skip/ignore/생산 변경 없음.

삭제 전suite 참조와 잔여빈줄의fmt 실패를 수정한 뒤 최신target 배정에서 기존7번1PASS/유지속성2PASS(집중3PASS/477SKIP,exit0), fmt·Clippy3종·workspace build·고정base정책exit0을 확인했습니다. 코드 `6c8a6c8e5e83a992fab14a0d90e3fc31e3e7ad4c`, [실행·보존 증거](../assets/issue7445/anticorruption6844_test_removal_validation.json). 원문 피델리티나 전체 회귀 해결로 세지 않습니다. 고정38은20완료/1이관/17대기, PR준비 미완료입니다.

## 보정87 결과 — 기존8·9번의 개별 실행

head `2b6780510c986a27292925f7cac176fda4c97bb7`에서 기존8번 body partition8은1PASS/244SKIP(exit0,0.600초), 다음9번 body partition9는1FAIL/244SKIP(exit100,0.528초)입니다. 실패 원문은 exam_eng3→5건/hwpctl_API0→1건이며 영어시험부터 원문별 독립 시각 검증합니다. 첫 실패에서 다음 함수를 멈췄고 생산·검사·기대값 변경은 없습니다. [개별 실행 근거](../assets/pr7382_20260926/stage87_individual_validation.json). 고정38은21완료/1이관/16대기, PR준비 미완료입니다.

## 보정88 사전 분석·결과 — 영어시험의 미달 회귀 제외

기존9번의 영어시험 본문 초과3→5건을 같은 원문/독립 한컴 PDF 전체8쪽으로 검증했습니다. 현재8/기준8쪽이나 최저7쪽48.54225%,2~8쪽 모두90%미달입니다.4쪽 문항27 표·문단 위치,7쪽 지문·선택지·상자·각주·머리 쪽번호 차이를 review에서 직접 확인했습니다. 전체 Native는 완료, fresh WASM은 미실행입니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5870452250) 후 원문을 바이트 동일하게 보존하고 전용 렌더링·페이지8함수와 #7061 입력1개·원장3행을 제거했습니다. 순수 HWP→HWPX 총쪽수 필드 파싱·직렬화와 IR진단을 유지합니다. 다른 문서·기대값·공차 유지, 새 함수/skip/ignore/생산 변경 없음. 코드 `2c39cd3ab5fffffc4fa84997027bed1d08f873a3`, [근거](../assets/issue7445/exam_eng_test_removal_validation.json). 유지 검사17PASS이며 기존9번은 재배정된 issue7196의 신규2건으로1FAIL입니다. 집중 ['        FAIL [   2.427s] (17/17) rhwp::regression_suite_019 body_overflow_baseline::body_overflow_does_not_grow_partition_9', '     Summary [   2.456s] 17 tests run: 16 passed, 1 failed, 5156 skipped', '        FAIL [   2.427s] (17/17) rhwp::regression_suite_019 body_overflow_baseline::body_overflow_does_not_grow_partition_9', '     Summary [   0.183s] 1 test run: 1 passed, 215 skipped']; 필수 lint/고정base정책의 정확한 exit를 근거 JSON에 남겼습니다. hwpctl_API 및 다른 대기 문서의 해결이나 PR준비로 세지 않습니다.

보정88 추가 정정: CanvasKit native/browser `renderer_baseline_manifest.json`의 `exam-eng` 항목을 놓친 자체 누락을 코드 `ee9b3ce3cbc49439cef0b7b830254befc40e5903`에서 제거했습니다. 나머지121개 원문이 존재하고 실제 manifest 로딩이 통과했습니다. 다른 항목의 공차·기준은 유지하며 원문 피델리티 통과로 세지 않습니다. 위 보정88 JSON에 정확한 제거 항목·실행 결과를 연결했습니다.

## 보정89 사전 분석·결과 — API 문서의 미달 회귀 제외

기존9번에 검출된 API 본문하단0→1건은74쪽 TextLine14.26667px입니다. 독립 한컴105쪽 PDF와 Native1·28·52·60·74·75·105쪽을 비교했습니다. 현재105/기준105쪽이나74쪽60.06836%, 나머지 선택6쪽은97.85668%이상입니다.74쪽에 앞쪽 소유 movePrevPos/13행이 남고 정상 moveNextPosEx/14행부터 후속 표행·설명이 밀리는 차이를 직접 확인했습니다. 전체105쪽/fresh WASM은 미실행입니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5870586821) 후 원문을 바이트 동일하게 보존하고 전용 렌더링23함수·원장4행·CanvasKit manifest1항목을 제거했습니다. #3695의 순수 개요 구조 파싱/IR과 다른 문서4함수·helper·기대값·공차 및 IR원장은 유지합니다. 새 함수/skip/ignore/생산 변경 없음. 코드 `09c4395e84b7edd23d9f819c52ec6c00d635f582`, [증거](../assets/issue7445/api24_test_removal_validation.json). 집중 ['     Summary [   0.913s] 18 tests run: 18 passed, 1078 skipped']; fmt·Clippy3종·workspace build·고정base정책 exit는 JSON에서 구분합니다. CanvasKit120입력 존재/실제 manifest 로딩 PASS입니다. 원문 피델리티나 전체 회귀 해결로 세지 않습니다.

## 보정90 결과 — 기존24~28번의 개별 실행

head `a82cee49561109052553907231bcd7f2806f165d`에서 기존24·25번 off-canvas partition10·14와26·27번 oracle partition13·15를 순서대로 각각1PASS로 확인했습니다. 다음28번 text-overlap partition0는1FAIL/225SKIP(exit100,2.507초)이며 원문 `issue1937_rowbreak_footnote_overpagination.hwp`의132→135건 증가입니다. 첫 실패에서 멈췄으며 생산·검사·기대값 변경은 없습니다. [개별 명령·summary](../assets/pr7382_20260926/stage90_individual_validation.json). 고정38은26완료/1이관/11대기입니다. 해당 각주 원문의 독립 시각 판정을 다음 단계에서 수행하며 PR준비·최종 전체 검증은 미완료입니다.

## 보정91 사전 분석·결과 — 각주1937 미달 회귀 제외

기존28번의 text-overlap132→135건 원문은 현재51/한컴50쪽입니다. 선택 Native24·42·43·44·45·50쪽92.59251/53.92934/41.51422/14.87881/38.19829/28.18537%입니다.43쪽 이전 표/각주18~39 중첩과44쪽의 한 쪽 밀림·각주 소유 차이를 review에서 직접 확인했습니다. 전체/fresh WASM은 미실행이며 기존45~80쪽 허용 범위를 피델리티 증거로 인정하지 않습니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5870699037) 후 원문 보존/렌더링·페이지2함수와 원장2행을 제외했습니다. 다른 문서6함수·helper·공차·IR 진단, 손상본4입력의 CLI 패닉 방지 계약을 유지합니다. 코드 `f2a4dd1e1da26f6ad1b861d8ef5297fb643a54fb`, [실행·보존 근거](../assets/issue7445/footnote1937_test_removal_validation.json). 집중 ['     Summary [   1.213s] 8 tests run: 8 passed, 455 skipped']; 필수 fmt/Clippy3종/workspace build/고정base 정책은 근거에 구분했습니다. 이관을 원문 개선이나 최종 전체 회귀 통과로 세지 않습니다.

## 보정92 결과 — 기존29번 개별 실행

head `f56788498d63d21e6cafd9c563fab52c4e0cb133`에서 기존29번 text-overlap partition1을 단독 실행했습니다.1FAIL/227SKIP(exit100,1.497초)이며 화학표시기준 `issue6782/1480000-201900042-chemical-labeling-standards.hwp`의 신규2건과 `task1749/saved_bounds_cumulative_page_break.hwpx` 신규9건입니다. 첫 실패에서 멈췄으며 생산·검사·기대값 변경 없음. [명령·summary](../assets/pr7382_20260926/stage92_individual_validation.json). 화학표시 원문부터 한 개씩 독립 시각 비교합니다. 고정38은27완료/1이관/10대기이며 PR준비 미완료입니다.

## 보정93 사전 분석·결과 — 화학표시기준 축소본 미달 회귀 제외

기존29번의 축소본76쪽 신규겹침2건은5.68px의 캡션/앞 문단 충돌입니다. 해시가 다른 전체 원문을 같은 입력으로 묶지 않고 축소본 자체의 독립 한컴 PDF를 사용했습니다. 현재103/기준103쪽이나 Native76·77·78·83·103쪽60.7561/99.27598/53.67396/99.79822/99.98703%입니다.76쪽 캡션/표 위치와78쪽 표 행 높이·후속 캡션 차이를 직접 확인했고 전체/fresh WASM은 미실행입니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5870791302) 후 축소본만 보존/렌더링14함수·matrix2입력·body 원장1행을 제외했습니다. #7048 전체원문3함수와 다른 전체원문 검사·helper·공차·기대값 유지, 새 함수/skip/ignore/생산 변경 없음. 코드 `26777a527892cf17ef3b7e7ffcd5715024506ea7`, [근거](../assets/issue7445/chemical6782_test_removal_validation.json). 집중 ['     Summary [   1.328s] 4 tests run: 4 passed, 450 skipped']; 필수 lint/정책 exit는 근거에 구분했습니다. 입력 피델리티 미달 승인 제외이며 각 개별기대값의 오류확정/원문 개선/전체회귀 완료로 세지 않습니다. saved_bounds HWPX는 별도 미해결입니다.

## 보정94 사전 분석·결과 — saved-bounds HWPX 미달 회귀 제외

기존29번의 HWPX4쪽 신규겹침9건을 실제 공개 결재문서/기존 독립 한컴2024 PDF 전체5쪽으로 검증했습니다. PR#1752/계획#1811/결과#2015의 입력별 대응을 확인했습니다. 현재5/기준5쪽이나 Native1~5쪽94.04792/98.06349/82.65886/72.63884/30.44845%입니다.4쪽 표/문단 겹침과표내용소유,5쪽 이어받기·뒤 설명/표/제목 위치 차이를 직접 확인했습니다. fresh WASM은 미실행입니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5870886007) 뒤 HWPX만 보존/렌더링3함수와 body1행을 제외했습니다. #1811 혼합 함수의 HWPX 페이지/cut 부분만 제외하고 셀52/57 저장 IR 및 기존 HWP5쪽/3유닛컷 대조군을 유지합니다. same-id 역사적IR catalogue/배열계약, 다른 원문·기대값·공차 유지, 새 함수/skip/ignore/생산 변경 없음. 코드 `3f7ea7600d9c1bd4ef4804badcb94b7eafe890cb`, [근거](../assets/issue7445/savedbounds1749_test_removal_validation.json). 집중 ['     Summary [   1.181s] 4 tests run: 4 passed, 628 skipped']; 필수 lint/정책 exit는 근거에서 구분했습니다. IR/HWP 대조군을 HWPX 승인으로 세지 않으며 원문 피델리티/전체회귀 완료로 보고하지 않습니다.

## 보정95 결과 — 기존30번 개별 실행

head `f90b2d3592dbd239f00e4bb5ce5777756da05e48`에서 기존30번 text-overlap partition10을 단독 실행했습니다.1FAIL/227SKIP(exit100,0.615초)이며 `한글문서파일형식_5.0_revision1.3.hwp`의 신규겹침1건입니다. 첫 실패에서 멈췄으며 생산·검사·기대값 변경 없음. [명령·summary](../assets/pr7382_20260926/stage95_individual_validation.json). 동일 원문 독립 시각 비교를 먼저 수행합니다. 고정38은28완료/1이관/9대기이며 PR준비 미완료입니다.

## 보정96 사전 분석·결과 — 공식 형식문서 미달 회귀 제외

기존30번의49쪽 신규겹침1건에 대해 공식CDN의 원문HWP가 저장소파일과 SHA-256 동일함을 확인했습니다.같이 배포되는 PDF의 Creator가 저장제품/빌드2018/10.0.0.7282와 같고71쪽입니다.원문 배포용특성 때문에 중간HWP저장/직접인쇄시도는 실패/보류이며 기준으로 수용하지 않습니다.원문변경/서비스배포 없이 공식짝 자료를 사용합니다.현재69/기준71쪽,선택8쪽 최저69쪽0%,15쪽40.68047/49쪽26.93028%입니다.15/49쪽 표분할·내용소유·머리말 및69쪽발행정보소유차이를 직접확인했습니다.전체Native/freshWASM 미실행입니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5871244705) 후 원문/공식PDF보존,렌더링·페이지4함수/body·offcanvas2행만 제외했습니다.IR표식·프로필/왕복2함수와 #2099 다른입력3함수 유지,진단실제입력경로 갱신,역사적기록 유지.코드 `eb94eba73dfcaa147392ee7b87ab1a58b88f65d6`, [근거](../assets/issue7445/format_spec13_test_removal_validation.json).집중 ['        FAIL [   2.725s] (6/6) rhwp::regression_suite_020 text_overlap_baseline::text_overlaps_do_not_grow_partition_10', '     Summary [   2.731s] 6 tests run: 5 passed, 1 failed, 664 skipped', '        FAIL [   2.725s] (6/6) rhwp::regression_suite_020 text_overlap_baseline::text_overlaps_do_not_grow_partition_10'],필수fmt·Clippy3종·workspace build·고정base정책 exit0.기존30번은corpus재배정의pr4093/outline_navigation_panel_demo.hwpx 신규겹침2건으로 pending입니다.새검사/생산변경/공차완화 없음.이관을피델리티 개선/승인/전체회귀완료로 세지 않습니다.

## 보정97 사전 분석·결과 — 개요 탐색 패널 데모 미달 렌더링 제외

기존30번재배정의pr4093데모신규겹침2건을동일입력/해시가고정된독립한컴PDF전체3쪽으로검증했습니다.한컴빈문서에서Python이본문합성한입력이며생성정보를정상저장본증거로쓰지않습니다.Native100/100/51.51148%,3쪽표셀번호와뒤개요겹침/뒤문단·부칙위치차이직접확인.freshWASM 미실행.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5871352884)후원문바이트보존/생성기출력경로이관및기존혼합함수의페이지·SVG assertion만제외.번호·제목·수준15항목getter와다른최소입력SVG계약유지,생성원문SHA동일.코드 `93f62ee4fd4b9b626ba59ae03e782e866d8e9604`, [근거](../assets/issue7445/outline4093_test_removal_validation.json).집중 ['     Summary [   2.402s] 3 tests run: 3 passed, 447 skipped'],필수fmt·Clippy3종·workspace build·고정base정책exit0.기존30번resolved로갱신,고정38은29완료/1이관/8대기.생산변경/새검사/공차완화없으며이관을피델리티개선으로세지않습니다.

## 보정98 결과 — 기존31~33번 개별 실행

head `14058e212abdb9cb25396924aafb2f8b3686e7c2`에서 기존31·32번은 각1PASS,33번은1FAIL로 첫 실패에서 멈췄습니다. 생산·검사·기대값 변경 없음. [명령·summary](../assets/pr7382_20260926/stage98_individual_validation.json). 고정38은31완료/1이관/6대기입니다. 다음 입력의 독립 PDF 시각 검증부터 진행하며 최종37함수/전체 회귀는 별도입니다.

## 보정99 사전 분석·결과 — 가상융합 시행령 미달 회귀 제외

기존33번의 신규겹침1건은63쪽 참고 상자가 기준PDF보다 약40px 아래로 밀려 하단쪽번호와 충돌한 것입니다. 기존 등록 manifest의 원문/PDF 해시·독립 변환 출처를 확인했습니다. 확장자는HWPX이나 실제HWP5이며 현재74/기준74쪽입니다. 기존 그림 경계18~20쪽과 신규겹침62~64쪽/첫·마지막쪽을 같은head/provenance로 선택8쪽 비교했습니다. 최저74쪽51.68936%,19쪽59.24283/63쪽72.34041%입니다.19·63쪽 review 직접 확인, 전체Native/freshWASM 미실행입니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5871489510) 뒤 원문바이트보존/전용렌더링2함수/body1행 제외와manifest 경로·역할 갱신을 수행했습니다. 다른비TAC입력의 음성대조/기대값/helper/공차 유지. 코드 `7336af268e721bf57934be0800c98bbda338fc29`, [근거](../assets/issue7445/decree6776_test_removal_validation.json). 집중 ['     Summary [   2.611s] 2 tests run: 2 passed, 444 skipped'], 필수fmt·Clippy3종·workspace build·고정base정책 exit0입니다. 새검사/생산변경/허용치완화 없음. 기존33번완료, 고정38은32완료/1이관/5대기이며 이관원문 피델리티 해결이나 최종전체회귀 완료로 세지 않습니다.

## 보정100 결과 — 기존34번 개별 실행

head `2cd02b417d5830858aadb74e9546a99d8cb90967`에서 기존34번은1FAIL(exit100,0.957초)입니다. 보정97에서 유지한 다른입력인 `pr4093/outline_navigation_table_cell_number.hwpx`의 신규겹침2건으로 첫 실패에서 멈췄습니다. 생산·검사·기대값 변경 없음. [명령·summary](../assets/pr7382_20260926/stage100_individual_validation.json). 이 입력의 독립시각승인을 주장한 적은 없으며 같은입력 PDF검증부터 진행합니다. 고정38은32완료/1이관/5대기입니다.

## 보정101 사전 분석·결과 — 최소 개요 입력 미달 SVG 검사 제외

기존34번의다른최소개요합성입력 신규겹침2건을같은입력/변환해시가고정된독립한컴PDF전체1쪽으로검증했습니다. 전체Native85.44776%,표셀번호2.와뒤개요3.요구사항이겹치며PDF두줄간격을잃는것을직접확인했습니다. 합성입력/글꼴예외없음,freshWASM미실행.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5871582903)후원문보존/생성기최소출력경로이관과기존혼합함수의SVG대조부분만제외했습니다.3개/15개번호·제목·수준getter및기존함수이름유지,새함수없음. 코드 `d01ab7eb21f908a2939df889954b8a0f766a175f`, [근거](../assets/issue7445/outline_minimal4093_test_removal_validation.json). 재생성SHA동일,집중 ['     Summary [   0.838s] 3 tests run: 3 passed, 424 skipped'],필수fmt·Clippy3종·workspace build·고정base정책exit0. 기존34번완료,고정38은33완료/1이관/4대기. 생성제품메타데이터를정상생성본증거로쓰지않으며이관을피델리티개선/전체검증완료로세지않습니다.

## 보정102 결과 — 기존35번 개별 실행

head `f9a154de16df07f957368d29416e081ea335c158`에서 기존35번은1FAIL(exit100,0.887초)이며 화학표시기준 전체원문의 신규겹침2건입니다. 첫 실패에서 멈췄고 생산·검사·기대값 변경 없음. [명령·summary](../assets/pr7382_20260926/stage102_individual_validation.json). 전체원문6,521,856byte/SHA398d03a5...는 보정93 축소본과 다릅니다. 축소본 증거를 재사용하지 않고 전체원문의 독립PDF로 검증합니다. 고정38은33완료/1이관/4대기입니다.

## 보정103 사전 분석·결과 — #6782 전체 원문 미달 회귀 제외

기존35번 신규겹침2건의 전체 원문은 보정93 축소본과 다른 해시/6,521,856byte입니다. 기존 manifest 원문해시와 동일하며 독립PDF의 실제Creator2022를 확인했습니다. 현재103/기준103쪽, Native선택8쪽 최저78쪽49.36129%,39쪽50.07638/76쪽61.10728%입니다.39쪽 표 내용/이어받기,76쪽 캡션/본문겹침,78쪽 행위치/그림누락을 직접 확인했습니다. 전체103쪽/freshWASM 미실행입니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5871748601) 후 전체 원문 보존/렌더링19함수·8파일/body1행 제외. 다른입력/공차/기대값/IR역사목록 유지, 새검사/skip/ignore/생산변경 없음. 전체 원문 및 이전 축소본 manifest의 보존경로/보류역할도 수정했습니다. 코드 `85bea0d563a3e53fa20eb0f18b25d6227bcee05b`, [명령·해시·제외근거](../assets/issue7445/chemical_full6782_test_removal_validation.json). 집중 ['     Summary [   2.687s] 1 test run: 1 passed, 223 skipped'], 필수8단계exit0. 기존35번 현재 corpus통과이며 원문 피델리티 개선/최종 전체회귀 완료 아님.

## 보정104 결과·사용자 범위 정정

head `a643e0e6b8bdb40172dbc55386d747b2ea49fd3f`에서 기존36·37번 각1PASS,38번1FAIL입니다.38번은 rowbreak-problem-pages.hwpx text-overlap1→2건입니다. [개별 명령·summary](../assets/pr7382_20260926/stage104_individual_validation.json). 고정38의 현재36완료/1이관/1대기이며 최종재실행/전체회귀 미완료입니다.

사용자 정정에 따라 #7445 처리는 PR을 실제 막는 실패 함수/assertion/corpus 입력으로 제한합니다. 동일 문서90%미달을 이유로 정상 검사까지 일괄 제외하지 않습니다. 보정103에서 제외한 전체원문19함수에는 이전PASS 기록이 있어 복원 후 현재head에서 다시 검사합니다. 앞선 제외 역시 실패 로그와 대조해 정상 검사의 동반 제외를 보정합니다. #7832는 API에서 존재하지 않으며 현재 검토 PR#7382로 우선 진행합니다.

## 보정105 결과 — PR 차단 범위만 보류·정상19함수 복원

사용자 정정을 반영해 보정10319함수와 원문 samples/body원장1행을 복원했습니다. 실제 실패가 확인된 text-overlap 신규2건 원문입력만명시적으로제외하며 정상함수/다른원장 계속검사합니다. 집중 ['     Summary [   3.632s] 20 tests run: 20 passed, 1920 skipped'],필수8단계exit0. 코드 `9efd1823c57b9e9417ef15cf7492269c27f2c8cb`, [근거](../assets/issue7445/chemical_full6782_scope_correction_validation.json). 부정시각증거/보존 원문·PDF 유지, 피델리티승인 아님. 사용자 확인PR#7382입니다. 나머지 이전동반제외를 원래검사/현재실패 증거로 대조합니다.

## 보정106 사전 분석·결과 — 기존 정상 회귀 동반제외 복원

사용자 정정에 따라 원래 검사102함수와혼합assertion/기대값을 복원하고 보존 입력의 동일해시 경로로 연결했습니다.110개 기존함수 각각실행, 초기경로오류6개 수정/단독재실행후94PASS/16실제FAIL입니다. 통과 검사 유지/실패16개는원문별개별보정 대기입니다. 새검사/생산변경/공차완화 없음. 집중19PASS,필수8단계exit0,코드 `37ec7fa3722ab2c9ae42daa63c8803c628666df5`. [명령·전후경로·결과·미해결](../assets/issue7445/scope_restore_validation.json), [#7445범위정정](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5872059905). 기존부분제외서술은역사기록이며최신범위를따릅니다. 공통원장/renderer manifest 동반제외도재대조합니다. 전체회귀/최종head시각검증/PR준비 미완료입니다.

## 보정107 사전 분석·결과 — 각주 쪽번호 검사 전제 교정·유지

복원대조FAIL의24쪽에는독립PDF/Native모두각주가없습니다.따라서각주소실회귀로해석하지않고 실제각주가있는25쪽을대조했습니다.독립PDF의기준선1061.7197265625는기존기대값과같습니다.Native/freshWASM25쪽각각97.33607%,review직접판독에서각주두줄/구분선/본문/쪽번호보존입니다.

기존함수index23→24만교정하고기대값/공차0.6/함수유지,새검사/생산변경/검사제외없음.보정106HWP경로prefix가별도HWPX까지치환했던1건도정상samples경로로복원했습니다.집중9PASS/필수8단계+freshWASMexit0.Mac로컬no-opt이며Docker최적화빌드아닙니다.코드 `ff3a23d06dd38db9096ba621c5a340eb8460c29c`, [수정전FAIL·독립근거·수정후PASS](../assets/pr7382_20260926/stage107_footnote_anchor_validation.json).원문의51/50쪽·뒤쪽미달은 #7445유지하며현재계약만충족입니다.복원대조의실제실패16개중1교정/15대기,기존38번/전체회귀/공통원장및source-unit동반제외 점검대기입니다.

![실제각주25쪽Native](../assets/pr7382_20260926/footnote_anchor_native_review_025.png)

![실제각주25쪽freshWASM](../assets/pr7382_20260926/footnote_anchor_wasm_overlay_025.png)

## 보정108 사전 분석·결과 — 소스 내부 정상 회귀 동반제외 복원

이전제외를 source의테스트영역까지대조해영어시험의표바깥여백·총쪽수2개기존함수와sample16의문서스타일·쪽테두리기존혼합assertion/helper를복원했습니다.같은원문보존해시경로로연결했고새함수/생산변경/기대값·공차완화없음.집중4PASS/필수8단계exit0.코드 `3e965ae01ce3c1e323b80342d93d76bb96063bd2`, [소스·입력해시·각명령·결과](../assets/issue7445/source_unit_scope_restore_validation.json). 원문전체피델리티승인아님.15개실제FAIL 및공통원장/manifest동반제외·기존38번/최종37개/전체회귀는계속미완료입니다.

## 보정109 사전 분석·결과 — API·영어 정상 corpus 범위 복원

사용자께서 확인하신 대상은 PR#7382입니다. API/영어 원문의 실제 본문 넘침 증가(API0→1, 영어3→5)와 다른 정상 검사 축을 구분했습니다. 보존 원문과 같은 바이트의 samples 입력, renderer manifest2항목, 쪽수 oracle/matrix4행과 API text 원장1행을 복원했습니다. 본문 넘침만 명시적 목록으로 보류하며 다른 corpus의 자동 수집과 정상 개별 함수를 유지합니다. 기존 수치·공차·생산코드·함수 수를 바꾸지 않았습니다.

코드 `65a5e2d31e703be24e5b8a14a3c53c399dd7a2e3`, [입력·실행·범위 근거](../assets/issue7445/corpus_api_eng_scope_restore_validation.json). 두 입력이 실제 포함된 off-canvas/cell/text/oracle 분할과 matrix 계약을 실행해11PASS/917SKIP, 필수8단계exit0입니다. renderer manifest122항목의 실제 로드/digest 확인은 통과했지만 backend 재캡처로 보고하지 않습니다. 문서 피델리티 미달은 미해결이며 나머지 corpus의 과잉제외 대조와15개 실제 실패 함수 개별 처리가 남았습니다. 최종 전체회귀/PR 준비는 미완료입니다.

## 보정110 사전 분석·결과 — 나머지 정상 corpus 범위 복원

보정69 이후 이동한22입력 중 API·영어 외20입력도 보존 원문과 바이트/해시가 같은 samples 경로로 복원했습니다. 확인된 본문13입력/text8입력 및 쪽수5입력의 해당 축 보류만 유지하고 다른 축은 원래 기대값으로 검사합니다. neartop의 본문17/19 통과를 실패로 보지 않으며 정상 원장24행(body7/off7/text5/oracle2/matrix2/cell1)을 복원했습니다. 보정69 이전 사용자 지정 거대문서 제외는 대상이 아닙니다. 새 함수·생산 변경·기준값/공차 완화는 없습니다.

코드 `501888148359749aef460291e3282a65ebe1eb96`, [입력·증거·실패 목록·명령](../assets/issue7445/corpus_scope_restore_validation.json). 기존 corpus80분할과 matrix6계약을 실행해63PASS/23FAIL/1086SKIP(exit100,110.901초), lint·정책7단계exit0입니다. 이 실패를 제외 목록에 자동 추가하지 않았습니다. 정상 범위 복원 진단이며 최종 전체회귀 통과가 아닙니다. 기존38 완료 표시는 과거 snapshot이고 최종 재실행이 필요합니다. 복원된 개별검사15실패와 corpus23실패는 같은 원문인지 대조하며 개별 처리합니다. PR준비는 미완료입니다.

## 보정111 사전 분석·결과 — rowbreak HWPX 실제 차단 범위만 보류

HWP 대응본과 구분해 동일 HWPX/기존 정상 한컴 PDF18쪽의 해시를 보정52 근거와 확인했습니다. 두7쪽 개별검사와 text-overlap1→2를 각각1FAIL로 재현했습니다. 독립 PDF7쪽에 제26조가 있지만 현재는8쪽으로 넘어가므로 이 쪽 소유 기대는 근거가 있습니다. 전체 Native18/18쪽 비교 최저9쪽20.18283%,13개쪽90%미달입니다.7/8쪽33.49973/43.01334%,7/8/9/16 review 및7/16 standalone overlay에서표시작의큰빈영역·내용이월·후속표/문단위치차이를직접확인했습니다. fresh WASM 미실행이며 글꼴 예외는 없습니다.

[issuecomment-5872944940](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5872944940) 후 실패2함수와이HWPX의text입력/기준1행만보류했습니다. 다른18함수·HWP대조군·원문/PDF·다른corpus축·쪽수원장·기대값/공차를유지합니다. 영어소개주석과잘못된HWPX소개경로도교정했습니다. 코드 `83402f03e3ce700149dd6099a078dabdc3859b06`, [입력·독립기대·시각·범위·실행](../assets/issue7445/rowbreak_hwpx_blocking_scope_validation.json). 남은18함수18PASS/194SKIP,필수8단계exit0,고정38의원래text partition9는별도1PASS/234SKIP입니다. 현재미달원문은재배정에서partition14였으며원래함수명은최종재실행대상으로유지합니다. 이보류는원문개선/전체회귀통과가아니며다른corpus실패원인·개별15실패및최종37개별/전체검증·PR준비는미완료입니다.

## 보정112 사전 분석·결과 — 정상 검사의 canonical 원문 경로 복원

사용자께서 대상 PR은 #7382로 확인하셨습니다. 보정110에서 복원한 samples 원문22개와 과거 증거 사본의 동일 바이트를 확인했습니다. 정상 검사가 향후 개선 원문을 소비하도록76개 파일의135개 경로를 samples로 복원했습니다. src3개는 cfg(test) 내부이고 함수·기대값·공차·생산 동작은 유지했습니다. 동적 경로3개는 root.join의 전체 상대경로 소비를 대조했습니다. 이번 소개 주석은 한국어로 정리했습니다.

코드 `f48fa6d88d40fbf8fe39b5f76d795653ce48216b`, [원문·변경 전후 소스 해시·경로·명령·결과](../assets/issue7445/normal_reader_scope_restore_validation.json). 기존 집중19PASS/1082SKIP, 소스 내부4PASS/4064SKIP, fmt·Clippy3종·workspace build·고정base manifest/unit 정책 모두exit0입니다. 이전 미달 증거 사본과 사용자가 별도로 제외한 거대 입력은 유지합니다. 개별15실패 및 다른 corpus 원인·최종37개별/전체검증·PR준비는 미완료입니다.

## 보정113 사전 분석·결과 — 영어시험 실제 차단 한 함수만 보류

복원13번 #6030 함수를 현재경로에서 단독1FAIL(셀높이15.9px,기존기대18.5px이상)로재현했습니다. 동일원문/PDF/생산Native binary 및 보정69대비빈src diff로 부정시각증거를재사용했습니다. 전체8쪽최저48.54225%,영향6쪽51.55944%이며6쪽review/standalone overlay에서지시문·상자·본문·선택지위치차이를직접재확인했습니다. 기존19.07px가한컴정답이라는독립근거나 clipping 개선은단정하지않습니다.

[#7445범위정정](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5870452250)뒤유일실패함수/전용helper만보류했습니다. 다른정상함수와source-unit/왕복·원문/PDF·renderer/text/oracle/matrix/공차유지,새함수/생산변경없음. 코드 `0d1ca6970444f0b05ea95cd68ce74b0bf398dace`, [검사범위·해시·명령·결과](../assets/issue7445/exam_eng_6030_blocking_scope_validation.json). 유지8PASS/1296SKIP,source-unit2PASS/4066SKIP,필수lint/정책exit0. upstream은Studio변경만으로 `0e8fd49fb868da0d47ac1294dcbbda81f0211233`로전진해이번정책은gates에기록한새base를사용했습니다. 실제15미해결중1보류/14대기이며다른corpus/최종전체검증/PR준비미완료입니다.

## 보정114 사전 분석·결과 — PII 실제 실패 함수와 혼합 항목만 보류

현재 후보의 #5846 단독 실행은1FAIL(70/66쪽)입니다. 정상 한컴 PDF는66쪽이며, 기존 주석의 cairo 기준은 정답지로 쓰지 않습니다. 정상59쪽은 자동화평가 안내서식으로 기존 신용도판단정보 표 전제와 달라 첫 쪽수 계약과 뒤 내용 소유 계약의 근거를 구분했습니다. 동일 원문/PDF/생산 Native binary 및 보정69 대비 빈 src diff를 확인해 영향4쪽 부정 증거를 재사용했습니다. 최저60쪽30.63875%,59쪽 review/60쪽 standalone overlay에서 법령과 안내서식의 페이지 소유 차이를 직접 재확인했습니다. 전수 Native/fresh WASM 미완료입니다.

유지 검사에서 #3595 두 함수는PASS였고, 혼합 #2097 함수는 같은 PII 항목70/66에서FAIL했습니다. [#7445 범위 갱신](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5868672890) 후 #5846 한 함수/전용helper와 #2097의 PII 쪽수 항목 한 개만 보류했습니다. #2097 함수·다른5개 입력 및 #3595 두 함수·원문/정상PDF·다른 corpus 축·공차는 유지했습니다. 코드 `71610682efde6fa4c0681228caf1f981f78e5d9b`, [독립 근거·실패·좁은 변경·재검증](../assets/issue7445/pii5846_blocking_scope_validation.json). 재실행3PASS/407SKIP 및 필수8단계exit0입니다. 실제 복원15실패 중 두 개를 좁게 보류해13개 대기이며 최종 개별/전체 검증과PR준비는 미완료입니다.

## 보정115 사전 분석·결과 — 자산관리규정 실제 차단 두 함수와 off-canvas 입력

현재 후보에서 #6031·#6409 각각 단독1FAIL(59/60쪽), off-canvas의 이 입력1→12 증가를 재현했습니다. 정상 PDF60쪽의41쪽은붙임4,42쪽은신고서이므로 이 페이지 소유 기대는 독립 근거가 있습니다. 동일 원문/PDF/생산Native binary 및 빈src diff로 선택6쪽 부정 증거를 재사용했고 최저41쪽25.22086%입니다.6쪽review/41쪽standalone overlay에서 문단·제목·뒤 본문과 신고서/붙임4 소유 차이를 직접 재확인했습니다. 전수 Native/freshWASM 미완료입니다.

[#7445 범위 갱신](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5868575524) 후 실제 실패 두 함수/전용helper와 이 입력의 off-canvas 기준1행만 보류했습니다. 원문/PDF와 다른cell/text축 및 모든 다른 문서/공차는 유지했습니다. 코드 `e6a4590f23f00c7756eede591feb0800508d4474`, [독립 근거·범위·실행 결과](../assets/issue7445/assets6031_blocking_scope_validation.json). 집중2개는1PASS/1FAIL이며 새 분할의 실제 남은 실패는2022변환본1건입니다. 이전 같은 분할의 basic nested42065/nopassword HWPX는 재배정되어 이번 결과로 해결을 추정하지 않습니다. lint·정책7단계는exit0입니다. 복원15미해결 중 누적4함수만 보류해11개 대기, 다른corpus/최종전체검증/PR준비 미완료입니다.

## 보정116 사전 분석·결과 — 2022 변환본 실제 세 함수와 두 축만 보류

복원78/79/103번은 현재 후보에서 각각 단독1FAIL(65/64쪽)입니다. 정상 한컴PDF64쪽은 쪽수 기대의 독립 근거이며 뒤 사업자선정/서버요건 내용의 정확성까지 첫 실패로 입증하지 않습니다. text/off에서 이 입력의 신규각1건도 재현했습니다. 같은 text 분할의 교육과정 입력37→90은 추가 분석 대상으로 유지했습니다.

동일 원문/PDF/생산Native binary 및 빈src diff로 선택5쪽 부정 증거를 재사용했습니다. 최저24쪽7.09794%,24쪽review는 정상 본문/표 대신 한줄만 남고3쪽standalone overlay는 문단시작·줄간격·도형이 다릅니다. 전체Native/freshWASM 미완료입니다. [#7445 범위 갱신](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5869647971) 뒤 실제3함수와text/off의이입력만 보류했습니다. 다른 버전 정상 검사·원문/PDF·모든다른축/문서/기대값공차는 유지했습니다. #1105 영어소개주석도 한글로 변경했습니다.

코드 `3c01d2ebbc03388ac01dc4eff5d60caeb0af3f95`, [입력별 범위·개별 재현·직접 판독·전수분할 결과](../assets/issue7445/sample16_2022_blocking_scope_validation.json). 유지11함수PASS, 변경한text/off32분할24PASS/8FAIL로 합계35PASS/8FAIL/851SKIP(1slow)입니다. 2022 입력은 남은 실패 목록에 없고 다른 실패8함수는 자동 제외하지 않습니다. lint·정책7단계exit0입니다. 자체 쉼표 누락으로 초기fmt가 실패했으나 같은 범위에서 수정해prepare부터 재실행했으며 렌더링 회귀로 세지 않습니다. 복원 실제15미해결 중7함수만 좁게 보류해8개 대기, 다른축/최종전체검증/PR준비 미완료입니다.

## 보정117 사전 분석·결과 — 동일 HWP 두 이름의 실제 차단 범위

기본원문과-2010이름은동일SHA/바이트이며서로다른저장버전으로간주하지않습니다. 실제metadata는한컴2024입니다. 각정상PDF는64쪽이고파일해시는다르지만72DPI래스터는64쪽모두동일합니다. 이PDF대PDF관측은Native/freshWASM승인과구분합니다. 현재후보의실패5함수를각각단독1FAIL(65/64쪽)로재현했고두이름의text/off신규각1건도각분할에서재현했습니다.

동일원문/각PDF/생산Native binary 및빈src diff로부정증거를재사용했습니다.24쪽review는기준본문/표대신한줄만남고23쪽standalone overlay는하단글자가외곽/꼬리말아래로나오며줄·문단위치가다릅니다. 영향24쪽7.09794%,전수Native/freshWASM미완료입니다. [기본원문#7445](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5868738187)·[동일별칭#7445](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5869562651)에등록한뒤이실패5함수와text/off의두이름입력만보류했습니다. 같은원문의정상그림·목차·IR/source-unit·다른버전·모든다른축/문서·기대값공차는유지했습니다.

코드 `7d64760ec7ad4ba78d35eebac00fe934326cc7ef`, [별칭·독립PDF64쪽동일성·개별실패·유지/전수분할](../assets/issue7445/sample16_aliases_blocking_scope_validation.json). 정상14함수PASS, text/off32분할24PASS/8FAIL로합계38PASS/8FAIL/850SKIP(1slow)입니다.두이름입력은실패목록에없고다른8실패함수를자동제외하지않았습니다.lint·정책7단계exit0.복원15미해결중누적12함수를좁게보류해3개대기이며다른축/최종전체검증/PR준비미완료입니다.

## 보정118 사전 분석·결과 — HWPX의 쪽수만 보류하고 정상 좌표 유지

#6706과 #2158을 각각 단독1FAIL로 재현했습니다. 동일 HWPX/한컴2024 PDF·생산 코드/Native binary 해시로 기존 선택 부정증거(18쪽67.8613%,64쪽11.02735%,65/64쪽)를 재사용했습니다. 18쪽 review에서 그림 내부 색/글자 및 글줄 차이를 직접 확인했으며 전체Native/freshWASM 승인이 아닙니다. 진단 CLI의 첫 페이지 인덱스 오류는 올바른0기준17 및 정확한 public DocumentCore 관측으로 수정했고 렌더링 회귀로 세지 않습니다.

#6706의 뒤 assertion은 모두 통과 범위입니다. 제목/그림 각1개, 제목x107.0933/baseline318.48, 그림x108.0533/y340.16, 오른쪽끝716.5333<페이지793.7067, 뒤제목770.9333>그림끝752.0533입니다. 동일입력 한컴PDF18쪽 baseline318.56px/그림107.9413·339.7894px도 기존값/0.7공차의 근거를 지지합니다. [#7445 범위 갱신](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5869485533) 후 쪽수 assertion 하나만 보류하고 모든 위치·개수·소유 assertion과 함수를 유지했습니다. #2158은 쪽수 전용 한함수만 보류하고 온새미로 함수 및 정상 수식/따옴표/IR·모든 다른 corpus·원문/PDF를 유지했습니다.

코드 `838c6387bf3e988d2fdd5bd6e5f88b85c48b59b8`, [개별 실패·정상 좌표·독립 근거·좁은 변경·검증](../assets/issue7445/sample16_hwpx_blocking_scope_validation.json). 기존10검사PASS/1091SKIP, 필수8단계exit0입니다. 기대값/공차 완화·신규 회귀/생산 변경 없음. 남은 dedicated는 #1749 혼합 검사 한개이며 다른축·최종37개별/전체검증·PR준비 미완료입니다.

## 보정119 사전 분석·결과 — 저장 bounds의 실패 컷 assertion만 보류

#1811 혼합 함수를 단독1FAIL로 재현했습니다. 실제 실패는 HWPX pi52 컷[3]이며 현재[2]입니다. 같은 함수의5쪽 유지/host먼저소비·HWP5쪽/컷[3]·HWPX IR52 및 다른두함수의pi26/IR57은정상입니다. 동일HWPX/독립한컴2024 PDF·생산Native binary 해시와빈src diff로기존전체Native5쪽 부정증거를재사용했습니다. 영향4쪽72.63884%,최저5쪽30.44845%입니다.4쪽review의사회기여 글줄/표겹침·표내용소유/뒤문단차이를직접확인했습니다. freshWASM미실행이며 HWP통과를HWPX승인으로쓰지않습니다.

[#7445 범위 갱신](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5870886007) 후 HWPX 컷 assertion 하나만 보류했고 함수/쪽수/순서·HWP·IR/다른2함수/다른축·원문/PDF를유지했습니다. [3]을현재[2]로완화하지않았습니다.

코드 `f1083037faac517f7dde23bba4a1f97e5eb8ed64`, [개별실패·컷/소비 순서·IR/HWP 대조·좁은 변경·검증](../assets/issue7445/savedbounds1749_blocking_scope_validation.json). 기존3함수PASS/205SKIP, 필수8단계exit0입니다. 신규회귀/생산변경없음. 복원16실패는1각주독립보정/13실패전용함수 보류/2혼합함수 실제assertion만보류로정상부분유지했습니다. corpus·최종37개별/전체검증·PR준비는미완료입니다.

## 보정120 사전 분석·결과 — near-top 원문의 실제 text 증가만 보류

현재원문text10→19건을분할단독1FAIL로재현했습니다. 동일원문/정상한컴PDF/생산binary와빈src diff로기존부정증거39쪽47.03148%·53쪽58.70083%/203대205쪽을재사용했고39쪽review에서문단·표내용소유/외곽/뒤문단 차이를직접확인했습니다. 전체205쪽Native/freshWASM미완료입니다. [#7445 범위갱신](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5868460165) 후이입력의text축과10건원장행만보류했습니다. 정상203쪽/작은1쪽/개체높이반례 및모든다른축·문서·원문/PDF유지,10을19로완화하지않았습니다.

코드 `9d1b255c4759b02c29545bfa8b9d5f7dcdde9602`, [단독실패·부정증거·좁은범위·전수분할](../assets/issue7445/neartop5941_text_blocking_scope_validation.json). 정상3PASS, text16분할12PASS/4FAIL로합계15PASS/4FAIL/642SKIP입니다. 다른4실패는제외하지않았고필수lint/정책7단계exit0입니다. 다른off/body/cell/oracle축 또는최종개별/전체검증을대체하지않으며PR준비미완료입니다.

## 보정121 사전 분석·결과 — 관제 원문의 실제 text 증가만 보류

현재원문text7→11건을분할단독1FAIL로재현했습니다. 동일원문/정상한컴PDF/생산binary와빈src diff로기존부정증거103쪽37.65331%·201쪽11.45426%/201대204쪽을재사용했고103쪽review에서정상3단계확산/그림24 대신이전본문이있는페이지내용소유 차이를직접확인했습니다. 전체204쪽Native/freshWASM미완료입니다. [#7445 범위갱신](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5869952956) 후이입력의text축과7건원장행만보류했습니다. 정상관제4함수 및모든다른축·문서·원문/PDF유지,7을11로완화하지않았습니다.

코드 `4979472991320a35a622d580245d676d12498b1c`, [단독실패·부정증거·좁은범위·전수분할](../assets/issue7445/air6764_text_blocking_scope_validation.json). 정상4PASS, text16분할13PASS/3FAIL로합계17PASS/3FAIL/854SKIP입니다. 다른3실패는제외하지않았고필수lint/정책7단계exit0입니다. 다른off/body/cell/oracle축 또는최종개별/전체검증을대체하지않으며PR준비미완료입니다.

## 보정122 사전 분석·결과 — 사이버대학 원문의 실제 text 축만 보류

text2→3건의분할단독1FAIL을재현했습니다. 기존#6804/#7274의같은원문/정상한컴PDF 대응과현재해시를확인했고현재45/45쪽입니다. 현재생산Native로실제겹침9·29쪽및분할표28–32쪽을새로비교했습니다. 선택6쪽complete/exit1/re_review_required이며9쪽98.86518%,표구간최저31쪽49.74703%/영향29쪽60.6783%입니다.29쪽review/standalone overlay의내부표행높이·글줄·뒤행위치차이와31쪽review의첫행소유/표종료차이를직접확인했습니다. 전수45쪽/freshWASM은미실행입니다. 정상기존좌표핀통과를PDF정답/전체피델리티로승격하지않습니다.

[#7445 추가등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5874150745) 후text입력하나/기존2건행만보류했습니다. 기존#6795다섯함수/#6981반례 및모든다른축·문서·원문/PDF 유지,2를3으로완화하지않았습니다. 테스트자체가틀렸다고확정하지않습니다. metrics JSON을dict로잘못읽은진단오류는최종manifest/overlay의6쪽지표수집으로바로잡았고렌더링회귀로세지않습니다.

코드 `0572a2d899cc58225d9476386d61225de9193eff`, [독립원문/PDF·새Sweep·좁은범위·전수분할](../assets/issue7445/cyber6795_text_blocking_scope_validation.json). 정상6PASS, text16분할15PASS/1FAIL로합계21PASS/1FAIL/664SKIP입니다. 한실패분할의다른두입력(교육37→90/진안신규5)은유지해개별처리하며필수lint/정책7단계exit0입니다. 다른off/body/cell/oracle축 또는최종개별/전체검증을대체하지않으며PR준비미완료입니다.

## 보정123 사전 분석·결과 — 교육과정 원문의 실제 text 축만 보류

text37→90건의분할단독1FAIL을재현했습니다. 같은원문/정상한컴PDF415쪽의#7244해시/출처를대조했고현재413쪽입니다. 현재생산Native로25·124·131·361·366·377쪽을새로비교했습니다. 선택6쪽모두90미만/complete/exit1/re_review_required,최저366쪽68.47008%입니다.366쪽review에서행내용소유·학교급/교과/성취기준 코드와표경계가다르고131쪽standalone overlay에서도행높이·글줄·셀내용차이를직접확인했습니다. 전수415쪽/freshWASM미실행입니다.

[#7445 추가등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5874264216) 후text입력하나/기존37건행만보류했습니다. 기존실물/IR함수 및모든다른축·문서·원문/PDF유지,37을90으로완화하지않았습니다. oracle415/413은기존원장값이며현재실패가관측되지않아보류대상에포함하지않았습니다. 테스트자체의오류나문서개선완료로세지않습니다.

코드 `5117536ee0e1cee0b9ac23547fd3ce0bebeb9e02`, [독립원문/PDF·새Sweep·좁은범위·전수분할](../assets/issue7445/curriculum2287_text_blocking_scope_validation.json). text16분할15PASS/1FAIL(진안),기존대조18PASS/별도시장구조조사#6761 1FAIL로합계33PASS/2FAIL/1092SKIP입니다. #6761은다른원문의317/315쪽실패이며이번에제외하지않고기존시장#7147두실패와함께후속개별분석합니다. 필수lint/정책7단계exit0이며다른축/최종검증·PR준비미완료입니다.


## 보정124 사전 분석·결과 — 선택 쪽 Native 내보내기 용량 개선

선택6쪽 비교가 전체413쪽 SVG 24,890,164,162바이트를 생성하는 원인을 확인했습니다. Native export에서
선택 쪽을 CLI의0기준 인덱스로 전달하고 resume의 빠진 쪽만 추가하도록 개선했습니다. 전체 문서 쪽수413과
산출물6개를 별도로 보존합니다. 전체 내보내기·단일쪽 파일명·누락 산출물 복구·범위 밖 요청도 확인했습니다.

코드 `ee3c9aecec58ac432c2ea7d072d74e7fba65572a`, [기존 검사·실제 파일 동일성·정리 결과](../assets/pr7382_20260926/selected_native_export_validation.json).
기존 Python77검사PASS, 전체/선택 SVG·render tree12개 및 비교PNG30개 SHA가 모두 같습니다. 선택SVG는
289,774,983바이트이며 실루엣점수와기존보류판정/exit1은변하지않았습니다. Rust생산/회귀함수 추가변경없음.
사용자 승인 범위에서 보정123의 미사용SVG407개/24,600,389,179바이트를 정리했고 선택6쪽SVG·모든PNG/JSON·원문/PDF는
보존했습니다. 보정123의export413기록은 당시 실행사실이며 정리후파일개수와구분합니다. freshWASM/최종전체검증/PR준비는미완료입니다.


## 보정125 사전 분석·결과 — 진안군 신청서의 신규 겹침 입력만 보류

신규5건의 text 분할을 단독1FAIL로 재현했습니다. 저장소/다운로드 원문 바이트가 같고 한컴2022 저장본입니다.
지정 변환 서비스의 engine2020으로 같은 원문의 [한컴 기준 PDF2쪽](../../../pdf/task2319/20544835_jinan_apt_form-hwp-2020.pdf)를 생성했습니다.
현재 Native 전체2쪽은1쪽51.96856%/2쪽71.44508%입니다. 1쪽review의 행 높이·구비서류 경계와 수신자 글줄 겹침,
2쪽overlay의 처리절차 상자 높이 및 지원제외대상/본문 위치 차이를 직접 확인했습니다. freshWASM은 미실행이며
Native 부정증거를 승인으로 쓰지 않습니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5874475817) 후 이 원문의 실패한 text 입력 하나만
보류했습니다. baseline 신규5건 허용값은 추가하지 않았습니다. 기존 정상2함수·다른 축/문서·원문은 유지하고 새 PDF를 커밋했습니다.
코드 `45edbe87fcde65296760f7d79e4907dfcc73e97d`, [개별 재현·독립 PDF·전체 Native·유지/분할 검증](../assets/issue7445/jinan2319_text_blocking_scope_validation.json).
text16분할16PASS와 정상2PASS, 합계18PASS/399SKIP입니다. 필수lint/정책7단계exit0, 신규 회귀/생산 변경 없음.
다른 축·시장#6761/#7147·최종37개별/전체회귀·원PR Native/freshWASM 검증은 남아 있으며 PR 준비 미완료입니다.


## 보정126 사전 분석·결과 — 재활용 보도자료의 실제 off-canvas 입력만 보류

신규1건의 off 분할을 단독1FAIL로 재현했습니다. 7쪽Table5 경계가 용지 오른쪽으로444.9067px 나갑니다.
#6917의 같은원문/기여자 한컴PDF 대응을 확인했고 현재8/PDF8쪽입니다. 현재 Native6–8쪽은99.74887%/
83.9489%/99.94084%입니다. 7쪽review/standalone overlay에서 하단 배박활용·감귤박활용 제목과 두 그림이
Native에는 없는 것을 직접 확인했습니다. 탐지 경계 하나를 모든 누락의 확정 원인으로 보고하지 않습니다.
전체8쪽/freshWASM은 미실행입니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5874555016) 후 이입력의 off축만 보류했습니다.
baseline 신규1건은 추가하지 않았습니다. 기존앵커2함수/다른모든축·문서·원문PDF를 유지했습니다.
코드 `ab0fe1cb4e68b09258615147632beaaef3293dd5`, [개별실패·직접판독·좁은범위·분할/대조 결과](../assets/issue7445/recycling6892_off_blocking_scope_validation.json).
off16분할14PASS/2FAIL와앵커1PASS/다른입력대조1FAIL로합계15PASS/3FAIL/409SKIP입니다. 다른두 off실패
(HWP5-nopassword 신규1, basic issue2007 신규2) 및 pic-in-table-01 그림y38.3067/기대39.7 대조실패는 제외하지
않았습니다. 필수lint/정책7단계exit0이며 다른축·최종검증/PR준비는 미완료입니다.


## 보정127 사전 분석·결과 — 다른 입력의 잘못된 그림 좌표 기대값 교정

pic-in-table-01 대조 함수를 단독1FAIL로 재현했습니다. 기존39.7/50.9px는 수정 전 바이너리의 값이며
독립 정본의 좌표가 아닙니다. 한컴2022 저장본/같은Creator 버전의 [기존 한컴PDF](../../../pdf/pic-in-table-01-2022.pdf)
1쪽 그림y는28.76899pt/37.15897pt, 96DPI로38.35865px/49.54529px입니다. 현재출력의 첫그림38.3067px가
정본에 맞으며 기존39.7px 핀이 정상 출력을 실패시킵니다.

현재 Native와 루트fresh WASM의 영향1쪽은 모두99.95104%, 실제rhwp PNG SHA도 같습니다. 양쪽review/standalone
 overlay를 직접 열어 두 머리말 그림·표외곽·뒤본문 배치를 확인했습니다. 타이틀 굵기 등 미세잔여가 있어
완전픽셀일치로세지않습니다. 전체22쪽이나 다른 경로의 피델리티 승인은 미검증입니다. rootpkg/Studio JS·WASM
SHA가 같고 생산소스는 보정69와 동일합니다. 빌드는 Mac로컬no-opt 대체이며 Docker최적화 통과로 보고하지 않습니다.

코드 `ecbaeff9afcecbe50ed97248ad6ac2bd44b892dc`에서 두 y기대값만38.36/49.55로 교정하고 함수2개/그림개수/높이/0.5공차/모든다른축을 유지했습니다.
[#검증 근거](../assets/pr7382_20260926/first_cell_float_expectation_validation.json), 기존2PASS/216SKIP 및필수lint/정책7단계exit0입니다.
#7445로 보류하지 않았으며 신규회귀/생산변경없습니다. 다른 off2입력·body/cell/oracle·시장실패 및최종개별/전체검증은
남아 있어PR준비미완료입니다. 진단의Python환경/PNG파일명 추정 오류는 venv와실제파일목록으로수정했으며회귀로세지않습니다.

![첫 문단 그림 Native1쪽 review](../assets/pr7382_20260926/first_cell_float_native_review_001.png)
![첫 문단 그림 fresh WASM1쪽 overlay](../assets/pr7382_20260926/first_cell_float_wasm_overlay_001.png)


## 보정128 사전 분석·결과 — HWP5 HWPX 원문의 실제 off-canvas 입력만 보류

신규1건의 off 분할을 단독1FAIL로 재현했습니다. 14쪽TextLine39가 종이 아래6.56px 나갑니다. 한컴2024 저장본을
확인하고 지정 서비스engine2024로 같은원문의 [한컴PDF24쪽](../../../pdf/HWP5-nopassword-123456-hwpx-2024.pdf)를 생성했습니다.
Creator/Producer의Hancom PDF 표기를출력무효로오인하지않았으며출처와실제판독을확인했습니다. 현재24/PDF24쪽입니다.
현재Native1/13/14/15쪽은63.35754/47.58076/41.8722/54.90896%입니다. 14쪽review에서 문단소유·줄/문단시작이
다르고 원고지기능 내용이 종이 아래로 내려가며13쪽overlay에도소유·줄바꿈·단락간격차이가있습니다. 전체24쪽/
freshWASM미실행이고Native부정증거로만썼습니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5874791787) 후 이HWPX off입력만 보류했습니다.
신규1건 baseline은추가하지않았습니다. 기존밝기/대비·왕복2함수 및다른정상함수·HWP대조군/모든다른축·문서·원문과새PDF를유지했습니다.
코드 `6961bd48748e2c65895272cc9cb413add71e515b`, [개별실패·변환출처·직접판독·분할/대조 결과](../assets/issue7445/hwp5_off_blocking_scope_validation.json).
off16분할15PASS/1FAIL와정상2PASS로합계17PASS/1FAIL/434SKIP입니다. 남은 basic issue2007 신규2건은동반제외하지
않았고필수lint/정책7단계exit0입니다. 다른축·최종검증/PR준비미완료입니다.


## 보정129 사전 분석·결과 — basic2007의 실제 두 차단 축만 보류

동일 원문의 off-canvas 신규2건과 oracle17→21쪽 실패를 각각 단독 재현했습니다.
최신 devel의 원래 검사에서는 off0건/17쪽이며 정상5함수도 통과합니다.
Native10/11/12/13/14/16/17쪽 비교는62.50548/60.79677/1.04782/49.05945/3.45708/28.06232/13.81874%입니다.
12쪽 review와14쪽 overlay에서 본문 누락을 직접 확인했습니다. 저장 사양이나 정상 PDF의 연도표기로
입력을 무효 판정하지 않았고, 전수/freshWASM은 미실행인 부정 증거로 구분했습니다.

[#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5874913489) 후 off입력과
oracle17/17행만 보류했습니다. 기대17을21로 완화하지 않았으며 정상세로정렬5함수/다른모든축·문서/원문PDF는 유지했습니다.
코드 `6288398853c9cf0ca945ac285baddd7c17cb6fd8`, [범위·전후실패·검증 근거](../assets/issue7445/basic2007_blocking_scope_validation.json).
집중37개는32PASS/5FAIL이고 off16개와 정상5개는 모두PASS입니다. 남은5개oracle실패를 함께 제외하지 않았습니다.
필수lint·정책7단계exit0이며 전체회귀·최종시각/PR준비는 미완료입니다.

![basic2007 12쪽 review](../assets/issue7445/basic2007_native_review_012.png)
![basic2007 14쪽 overlay](../assets/issue7445/basic2007_native_overlay_014.png)


## 보정130 범위 재검토 — #7382를 실제 막는 검사만 처리

사용자 지시로 새 회귀 보정을 멈추고 최신devel과 현재 후보를 대조했습니다.
최신 devel `0e8fd49fb868da0d47ac1294dcbbda81f0211233`에서 기존 변경 검사와 관련 원장·정상 대조군 51개 case는
277+29=**306PASS/0FAIL**입니다. 전체devel회귀의 통과 주장과 구분합니다.
후보 생산소스는 보정69와동일하며 봉인Native SHA `ad4f63371a5b205b3696d4e653526cfbda9ec894748ba9de24c5f35f463f14c7`로
동일바이트29입력의 모든페이지 진단을 재실행했습니다. 현재보류37개입력·축은 **모두 기존baseline초과 및devel대비증가**입니다.
제외한oracle6행도17→21,64→65(4원문),66→70의 실제쪽수증가입니다.
보류한개별15함수는 기존 후보의 단독exit100/실패요약과 이번devel원래함수PASS를 연결했습니다.
저장된 단독결과의 생산소스는 지금과 같아 재사용하며 추가원본후보harness재빌드는 실행하지 않았습니다.
사용자직접지정거대문서/이전근거있는계약보정/이미만든검사는 별도로보존합니다. 진단값을시각통과나피델리티개선으로보고하지않습니다.

차단해결에 불필요했던 경로helper재작성3건을 원래형태로복원했습니다.
또한 `render_page_samples.tsv`의sample16-HWP5와PII 두행은형태계약을막지않으며
PR필수실패게이트의제외근거도없어 **동반제거를취소**했습니다. 실제oracle실패행의보류와이자료목록을구분합니다.
원래두행/다른바이트와정상검사를보존했고한국어범위주석도유지했습니다.
코드 `b20c22377e0ca1ab9b5f49f176534812505ef226`, [개별devel·후보수치·실패출처·복원근거](../assets/pr7382_20260926/blocking_scope_audit_validation.json).
복원후 기존17함수와자료계약6함수 **23PASS/0FAIL**, 필수fmt·Clippy3종·workspacebuild·base고정manifest/unit정책exit0입니다.
새함수·생산변경·기대값/공차완화없습니다. 새대조worktree만실행종료확인후제거했고공유target/다른작업은유지했습니다.

이후에도후보에서실패한함수·assertion·입력/축의원인과devel대조부터확인합니다.
낮은점수만으로다른정상검사를동반제외하지않습니다. 보정129의oracle5실패와기존다른실패는남아있으며
최종개별/전체nextest·Native/freshWASM·PR준비완료로판정하지않습니다.


## 보정131 사전 분석·결과 — 전직시험면제 표의 실제 쪽수만 보류

- 사전 head `17140921c2cf68285a2e5ca29c5ea7f1c3ae2520`, 코드 커밋 `dcbdf1a8762a1ad064443f68804332928eb8ac01`. [독립 PDF·원문·실패·명령·검사](../assets/issue7445/jeonjik2146_blocking_scope_validation.json).
- 최신 devel 기존 oracle는6쪽PASS, 후보7쪽. 관련기존19검사15PASS/4FAIL이며 네실패모두선행쪽수입니다. 전체PDF6쪽Native86.44992/49.90645/59.72521/44.29839/39.57099/37.39924%이고행소속/표끝/뒤설명이달라집니다. 추가7쪽의말미설명도직접확인했습니다.
- [#7445](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5875379798)에 oracle이입력행, #2097이입력항목, #7028/HWP #7032쪽수assertion만 보류했습니다. 정상함수·첫여섯쪽머리행/대각선/빈문단/소유/SVG·HWPX6쪽핀/다른두입력·render_page_samples행/다른축·원문PDF 유지. baseline6→7완화/새함수/생산코드변경없음.
- 유지검사를실제로재실행해선행assertion뒤도통과했습니다. 관련18PASS+oracle12PASS/4FAIL=30PASS/4FAIL/842SKIP; 다른oracle4실패는그대로입니다. fmt/Native·WASM·alltargetsClippy/workspacebuild/manifest·unitfixedbase정책 모두exit0.
- 최종개별/전체회귀·원PR Native/freshWASM·PR준비는미완료입니다. 이건좁은실패범위후속이관이며원문개선/피델리티통과가아닙니다.

![전직시험면제6쪽 review](../assets/issue7445/jeonjik2146_native_review_006.png)
![전직시험면제2쪽 overlay](../assets/issue7445/jeonjik2146_native_overlay_002.png)


## 보정132 사전 분석·결과 — 규제영향 중첩 표의 실제 쪽수·소속만 보류

- 사전 head `b77d41894bc373d93addfef8d864c521f0e45a14`, 코드 `467b2835f47a58f596af9b281c166c5bf06c55f1`. [동일입력/PDF·실패재현·유지검사·명령](../assets/issue7445/regulatory3637_blocking_scope_validation.json). ZIP HWPX·마지막저장한컴2020이며paired정상한컴2022 PDF31쪽을사용했습니다. cairo current-2020은독립기준아닙니다.
- devel기존함수/oracle는PASS, 후보32쪽과26/27줄소속은실제FAIL입니다. 선행쪽수뒤의26임금존재·27사업체존재·27임금부재를기존Rust함수로각각exit100재현했습니다. 선택1·25–31Native최저24.2785%이며큰물리쪽소속차이를직접확인했고추가32쪽에도말미표/설명이넘어갑니다.
- [#7445](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5875495573)에 이입력oracle31/31행 및실제실패assertion4개만후속이관했습니다. 기존함수·정상26다음내용부재/28숨은줄/전체넘침상한1100px·다른입력/축/원문/PDF 유지. baseline31→32완화·새함수·생산변경없음.
- 유지4PASS+oracle13PASS/3FAIL=17PASS/3FAIL/648SKIP. 필수fmt/Native·WASM·alltargetsClippy/workspacebuild/manifest·unit정책fixedbase모두exit0. 다른3oracle실패는유지했습니다.
- 전31쪽/freshWASM/최종전체회귀·원PR시각증거·PR준비는미완료이며피델리티개선으로보고하지않습니다.

![규제영향26쪽 review](../assets/issue7445/regulatory3637_native_review_026.png)
![규제영향28쪽 overlay](../assets/issue7445/regulatory3637_native_overlay_028.png)


## 보정133 사전 분석·결과 — 파라미터 명세의 실제 oracle 행만 보류

- 사전 head `b7f8fc8e4`, 코드 `59c1aa39f943bd5c89060c4e3b4afe3f0dc54b77`. [독립PDF·단독실패·정상대조·명령](../assets/issue7445/parameter_control_blocking_scope_validation.json). 원문한컴2018저장본이고정상한컴2022 PDF74쪽은기존#6656독립좌표실측근거입니다.
- devel기존oraclePASS,후보75쪽FAIL. #6307/#6656기존정상3함수PASS. Native선택1·3·11·12·73·74쪽최저29.90644%이고PDF73Type표가Native74,PDF74HWPUNIT·URC가Native75로이월됨을직접확인했습니다.
- [#7445](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5875602474)에oracle이입력74/74행하나만이관했습니다. 기존함수/기대값·공차/모든다른축/입력/원문PDF유지. 정상3PASS+oracle14PASS/2FAIL=17PASS/2FAIL/640SKIP,필수lint/정책7exit0. 다른2실패는유지했습니다.
- 전74쪽/freshWASM/최종개별·전체회귀·PR준비는미완료이며원문피델리티개선으로보고하지않습니다.

![파라미터 명세74쪽 review](../assets/issue7445/parameter_control_native_review_074.png)
![파라미터 명세73쪽 overlay](../assets/issue7445/parameter_control_native_overlay_073.png)


## 보정134 사전 분석·결과 — 편람 HWP의 실제 쪽수·본문 소속만 보류

- 사전 head `2f76234555ef3e31ed6239cc22f810b2194a492b`, 코드 `4a5e66b91101909d120349a0da1345493d549561`. [단독 실패·독립 PDF·devel 대조·유지검사와 명령](../assets/issue7445/handbook_hwp_blocking_scope_validation.json). 원문/기준은한컴2024이며384쪽 정상PDF를 사용했습니다. cairo383쪽 PDF는 독립기준으로 삼지 않았습니다.
- 고정devel `0e8fd49fb868da0d47ac1294dcbbda81f0211233`의 원래 #7009·#3931 8개는 모두 PASS입니다. 후보 HWP의384→383쪽 및 선행쪽수 뒤 간지313→312/본문끝311→310 assertion도 각각 기존 Rust 함수에서 실제 FAIL로 재현했습니다.
- Native선택158/159/310–313/383쪽 최저6.48027%입니다. PDF310 간지가Native312,PDF384판권이Native383으로 옮겨져 있으며 선택contact sheet/312review/383overlay/기준384PNG를 직접 확인했습니다. 양쪽 빈311쪽100%를 피델리티 승인으로 해석하지 않습니다.
- [#7445](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5875738025)에 HWP oracle행·#3931 쪽수전용한함수·#7009실패assertion3곳만 후속이관했습니다. 기존HWP함수/정상부록71쪽/랜드마크 존재·저장줄/분할·다른입력/축/원문PDF 유지. 현재HWPX/포맷비교3실패는 별도검증 전까지 함께 제외하지 않았습니다. 새함수/생산변경/공차완화없음.
- 집중21PASS/4FAIL/855SKIP이며 HWP유지6PASS와oracle15PASS/다른1FAIL입니다. 필수fmt/Clippy3단계/workspacebuild/manifest·unitfixedbase정책 모두exit0. 소유한clean devel대조워크트리4.4GB는 Cargo종료를확인한뒤제거했고 공유target·로그·결과는 유지했습니다.
- 전384쪽/freshWASM/최종전체회귀·PR준비는 미완료입니다. 다음은 HWPX 별도 비교와 실패 범위 판정입니다.

![편람 HWP 부록312쪽 review](../assets/issue7445/handbook_hwp_native_review_312.png)
![편람 HWP 말미383쪽 overlay](../assets/issue7445/handbook_hwp_native_overlay_383.png)


## 보정135 사전 분석·결과 — 편람 HWPX의 실제 실패 핀만 보류

- 사전 head `496f6d6b7`, 코드 `2085dfdce823ef7eaf2b122b599a2c82705db4e4`. [단독 실패·독립PDF·선행assertion뒤검사·유지범위/명령](../assets/issue7445/handbook_hwpx_blocking_scope_validation.json). HWP판정과 분리해 한컴2024 저장 HWPX와정상한컴2024 PDF384쪽을 직접 비교했습니다. 고정devel의원래두case8PASS는보정134근거를재사용합니다.
- 후보385쪽/간지311쪽,기존핀382쪽/간지308쪽,독립PDF384쪽/간지310쪽입니다. 기존구조격차는줄지만선택158/159/310–312/383/384쪽의실제소속차이가있고310–312쪽은한쪽만빈페이지로0%입니다. 선택contact sheet·311review/384overlay·추가385판권을직접확인했습니다.
- 선행쪽수뒤 #7009간지assertion도실제FAIL이며 #3931문답같은물리쪽계약은실제로PASS입니다. [#7445](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5875912864)에쪽수/간지assertion3곳과비대칭전용한함수만후속이관했습니다. #7009기존두함수/정상부록71·74쪽/랜드마크존재·#3931문답/저장줄/분할·다른입력/축/원문PDF를유지했습니다. HWPXoracle는기준차2→현재차1로개선되어차단이아니므로원장행그대로입니다.
- 집중23PASS/1FAIL/601SKIP: 유지8PASS+oracle15PASS/정책연구HWP1분할FAIL. 필수fmt/Clippy3/workspacebuild/manifest·unitfixedbase정책모두exit0. 새함수/생산변경/공차완화없습니다.
- 전384쪽/freshWASM 및다른body/cell/시장/font검사군·최종전체회귀/원PR시각증거·PR준비는미완료입니다.

![편람 HWPX311쪽 review](../assets/issue7445/handbook_hwpx_native_review_311.png)
![편람 HWPX384쪽 overlay](../assets/issue7445/handbook_hwpx_native_overlay_384.png)


## 보정136 사전 분석·결과 — 90% 미만 정책연구 문서의 실제 차단 검사 제거

- 사용자의 재지시를 적용해 렌더러 개선을 중지하고 이번 `entry.rs` 변경은 모두 원복했습니다. `src/**`·`crates/**` 변경은 없습니다. 사전 head `5deb943bfc7fe7e8926f5dd2aae6741a24f703f4`, 코드 커밋 `2e76743a7ef2899428da359806827da4f86f080e`입니다.
- [원문·독립 PDF·실패 목록·제거/보존 범위·명령](../assets/issue7445/policy_report_blocking_scope_validation.json). 같은 정책연구 HWP/HWPX 원문과 정상 한컴2024 PDF는 보존합니다. 기준 두 형식 모두215쪽/현재216쪽이며 Native 선택 HWP6쪽 최저32.05171%, HWPX4쪽 최저31.53863%입니다. 각66쪽 review·67쪽 overlay에서 본문·표·각주 소속 차이를 직접 확인했습니다.
- 수정 전140검사20PASS/120FAIL을 재현했습니다. [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5876239548) 후 실제 실패118함수(#7379 78/#3738 32/캡션·테두리·저장프레임8), HWP oracle한행, #4882의215쪽 전제 한곳만 제거했습니다. HWPX oracle는 기존 격차6→현재1로 차단이 아니므로 유지했습니다.
- 기존 통과20검사의 함수 본문은 삭제 전후 정확히 동일합니다. #4882 기존 함수는 원본==왕복 쪽수 등식과5개 IR검사를 유지하고 이름을 현재 계약에 맞췄습니다. 새 함수/기준값 완화/원문 삭제/다른 입력·축 일괄 제외는 없습니다.
- 제거 후 유지37PASS/0FAIL(14+1+6+oracle16)입니다. 필수fmt/Clippy3/workspacebuild/manifest·unitfixedbase정책 모두exit0입니다. 기준 base `0e8fd49fb868da0d47ac1294dcbbda81f0211233`입니다.
- 이것은 회귀 실행 범위의 후속 이관입니다. 문서 출력 개선·90% 시각 gate 통과·최종 전체 검증/PR준비로 보고하지 않습니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage136-policy-report-hwp/`에만 보관합니다.

![정책연구 HWP66쪽 review](../assets/issue7445/policy-report-hwp_native_review_066.png)
![정책연구 HWPX67쪽 overlay](../assets/issue7445/policy-report-hwpx_native_overlay_067.png)


## 검증137 — 전체 nextest 완료, 60개 실패를 개별 판정

- 검증 head `0a069b1f1c1b0541f52f281e2d4f3a316616f982`, [정확한명령·전체실패목록·결과](../assets/pr7382_20260926/stage137_full_regression.json). release-test·공유target/pr-review·8스레드·no-fail-fast로 전체10250검사를 끝까지 실행했습니다. **10190PASS/60FAIL, exit100**, 819.225초입니다.
- #7382 원 PR은OPEN/head `81a402179dc556cce781d844d4b9252be36ba8af` 그대로입니다. 최신devel고정base `0e8fd49fb868da0d47ac1294dcbbda81f0211233`은 현재브랜치보다31커밋앞서므로 최종통합검증도 남습니다.
- 실패에는 동일정책연구HWP source 내부3검사, cell/body축, 다른쪽수/레이아웃, SVG snapshot, 비밀번호fixture계약 등이 포함됩니다. 전부7445로이관하지 않습니다. 사용자의90%미만제외조건과실제차단범위를문서별로판정합니다. 다음은이미직접확인한정책연구source3함수이며그영향6쪽최저18.00186%입니다.
- 보정136 삭제목록118함수의 원본행번호·해시를 원본head의구문트리와다시대조해 검증원장에 보완했습니다. 테스트/생산동작변경은 없습니다. 전체검증FAIL을 승인·PR준비완료로보고하지않습니다.


## 보정138 사전 분석·결과 — 정책연구 source 내부 실제 실패 3개 제외

- 전체검증137에서 실제FAIL한 동일정책연구HWP source 내부3함수만 [#7445에 추가](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5876621658)했습니다. 사전head `fb73f24b7`, 코드 `76e6834b1037f8ef37e6e7fef5a746ccf9a157ae`. [원래함수/해시·제거·보존·명령·결과](../assets/issue7445/policy_report_source_unit_validation.json).
- 영향94/95/106/107/156/182쪽은 정상한컴2024 PDF와Native54.77737/51.96505/33.20293/31.07231/28.45017/18.00186%입니다. contact sheet·182review·107overlay에서 본문/표/그림의 물리쪽소속 차이를 직접 확인했습니다. 사용자90%미만제외지시를적용하며렌더러는개선하지않습니다.
- 다른55검사본문은원래그대로보존했습니다. **54PASS/0FAIL**, 기존 `test_552_passage_box_top_gap_p2_4_6`의`#[ignore]`1건은그대로입니다. 필수fmt/Native·WASM·alltargetsClippy/workspacebuild/manifest·unitfixedbase정책7단계exit0. 원문/PDF·생산동작을유지했습니다.
- 전체60FAIL중 이3곳만처리했으며다른57실패의통과로보고하지않습니다. 다음은비밀번호fixture의복호화/오류계약과페이지수전제를분리검토합니다. 문서피델리티승인·최종PR준비는미완료입니다.

![정책연구 source검사182쪽 review](../assets/issue7445/policy_report_source_unit_native_review_182.png)
![정책연구 source검사107쪽 overlay](../assets/issue7445/policy_report_source_unit_native_overlay_107.png)


## 보정139 사전 분석·결과 — 암호 HWPX의 실패한 쪽수 전제만 이관

- 사전 head `e8d4e8089`, 코드 `a4ecead9ed345f48337acfbbbffab968a23c3b98`. [개별 실패·독립 기준·변경 범위·명령·결과](../assets/issue7445/password_page_pin_blocking_scope_validation.json). 전체137의 암호 관련 네 함수는 모두 같은 HWPX의23쪽 고정에서 실패했습니다. 앞선 복호화·IR·오류 계약과 HWP3/HWP5 정상 대조군은 통과했습니다.
- 동일 문서 평문에 대응하는 정상 한컴2024 PDF는24쪽이며 현재 Native도24쪽입니다. 보정128의1/13/14/15쪽63.35754/47.58076/41.87220/54.90896%와14쪽 review의 본문 소속·줄바꿈·용지 아래 차이를 다시 확인했습니다. 원문/평문/PDF의 현재 커밋 바이트도 동일합니다.
- [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5883321298) 후 `hwpx_password_fixture` 공개 API·CLI의 쪽수 핀 두 곳과 `mcp_password_contract`·`armor_cli_contract`의 HWPX 쪽수 전제만 제거했습니다. 기존11개 함수, 복호화·IR·오류·비노출·세션 읽기/닫기·응답 봉투, 다른 두 포맷 정상24/64쪽 핀과 원문/PDF는 유지했습니다. 23을 현재24로 바꾸지 않았고 새 함수·생산 변경은 없습니다.
- 수정 전7PASS/4FAIL, 수정 후 최종11PASS/0FAIL입니다. 포맷 뒤 파생 묶음 불일치는 재생성한 뒤 같은 검사와 필수fmt/Clippy3/workspacebuild/manifest·unit 고정base 정책을 다시 실행해 모두exit0을 확인했습니다.
- 전체137의60FAIL 중 보정138의3곳과 이번4곳만 처리했습니다. 다른53개는 개별 검토 대상이며 현재 전체 통과·피델리티 승인·PR준비 완료로 보고하지 않습니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage139-password-page-pin/`에만 저장합니다.


## 보정140 사전 분석·결과 — #6358 합성 입력의 여백 활성화 조건 바로잡기

- 사전 head `ab22b5e59`, 코드 `82c28e524ee3536753f829c4888b79cfa41be1f2`. [사양·기존/현재 전체시각·실패·변경·명령](../assets/pr7382_20260926/stage140_padding_contract_validation.json). 기존 두 함수는 `apply_inner_margin=false`인데 셀 양수32/141HU를 적용하길 기대했습니다. `hasMargin=false`는 표 기본값을 쓰는 사양과 정상 한컴 출력에 어긋납니다.
- #6101 독립 PDF3쪽의 표 외곽700.035–850.269px와 `hasMargin=0`/표 기본0/셀 보존141HU가 규칙의 근거입니다. 모델 소스는 보정68과 byte 단위로 같습니다. 과거 전체 Native/fresh WASM 최저92.26525%를 규칙 근거로 연결하고, 현재 고정 Native로 전체11쪽을 새로 비교해 같은 최저92.26525%/90% gate passed를 확인했습니다. 전체 contact sheet·3쪽 review/overlay에서 표 높이와 뒤 본문을 직접 확인했습니다. 큰제목 글꼴·굵기 차이는 남아 완전 일치로 보고하지 않습니다.
- 음수 결측 폴백과 활성 셀 양수 보존을 실제로 검사하도록 합성 입력의 플래그 두 곳만 true로 바꾸고 설명을 바로잡았습니다. 기대값/공차/기존 두 함수는 유지합니다. 비활성 보존값이0을 덮어쓰지 않는 정상 #1785 대조군도 그대로 실행했습니다. 원문/PDF·생산 코드는 변경하지 않았고 새 함수·#7445 일괄 이관은 없습니다.
- 수정 전4PASS/2FAIL, 수정 후6PASS/0FAIL입니다. fmt/Clippy3/workspacebuild/manifest·unit 고정base 정책7단계 exit0입니다. 전체137의60FAIL 중 앞선7곳과 이번2곳을 처리했으며, 다른51개 대상은 개별검토가 남습니다. 최종 전체 검증·PR준비 완료는 아닙니다.
- 전체 비교 산출물: `output/pr-review/planet6897-7382-20260926/stage140-cell-padding-contract/visual-native/firefighter6101/`. 이번 fresh WASM은 미재실행이며 과거 결과와 현재 Native 결과를 구분합니다.

![셀 여백 정상 대조군3쪽 review](../assets/pr7382_20260926/stage140_padding_native_review_003.png)
![셀 여백 정상 대조군3쪽 overlay](../assets/pr7382_20260926/stage140_padding_native_overlay_003.png)


## 보정141 사전 분석·결과 — basic2007의 실제 차단 검사8곳만 이관

- 사전 head `47cf60a36`, 코드 `2d90e389f6c9cca6b178195526a6dbe14b8db4ea`. [단독 실패·독립 근거·제거/보존·명령](../assets/issue7445/basic2007_render_tests_blocking_scope_validation.json). 동일 HWP 원문/PDF 현재 커밋 바이트와 보정129 증거를 대조했습니다. 정상 PDF17쪽/현재21쪽이며 Native 선택7쪽 최저1.04782%입니다. 기존12쪽 review에서 현재 빈 본문과 PDF 법령 본문 차이를 다시 직접 확인했습니다.
- 수정 전22개14PASS/8FAIL입니다. [#7445 추가등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5883519989) 후 실제 실패한 #2007 렌더링6함수만 제거했습니다. #4252·#4272 두 함수는17쪽 전제만 제거하고 원본IR·모든쪽 경로해석·중첩 셀 선택/복사/HTML·응답API를 유지했습니다. 이 두 기능 검사는 제거 후에도 실제 PASS입니다.
- 기존 정상14함수의 본문은 이전 head와 byte 단위로 동일합니다. 원문/PDF·모든 다른 문서/축도 유지하고17을21로 바꾸지 않았습니다. 생산 변경·새 함수·기준 완화는 없습니다.
- 수정 후16PASS/0FAIL, 필수fmt/Clippy3/workspacebuild/manifest·unit 고정base 정책7단계exit0입니다. 전체137의60FAIL 중 앞선9곳과 이번8곳만 처리했으며 다른43개 대상은 개별검토가 남습니다. 최종 전체회귀·PR준비 완료·원문 피델리티 개선은 아닙니다.
- 로그: `output/pr-review/planet6897-7382-20260926/stage141-basic2007-render-tests/`. 부정시각 증거는 같은 생산 코드의 보정129 자료를 재사용하며 이번 fresh WASM은 미실행입니다.


## 보정142 사전 분석·결과 — 시장구조조사의 실제 차단4곳만 이관

- 사전 head `80de5d458`, 코드 `39c4042fbfbd16e38eb6c49e388679d91ebc82e1`. [독립 기준·선택시각·실패·제거/보존·명령](../assets/issue7445/market2070_blocking_scope_validation.json). 원문/PDF 현재 커밋 바이트는 동일하며 정상 PDF315/현재317쪽입니다. 기존6검사는2PASS/4FAIL입니다.
- 현재 고정 Native의 영향4/5/94/95/96/315쪽은98.79298/45.12739/1.83707/34.28600/33.34729/67.87910%입니다. 94review의 본문/표 누락·물리쪽 소속,95standalone overlay의 표 조각·뒤 본문,5review의 목차 말미 이월·줄 간격 차이를 직접 확인했습니다. 6/6 출력 완료 뒤 exit1은90% gate 미달의 부정 증거이며 실행 장애가 아닙니다. 4쪽 점수를 전체 피델리티 승인으로 사용하지 않습니다.
- [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5883629635) 후 쪽수 전용 #2070 파일의한함수와 #7147의두실패함수만 제거했습니다. #6761은315쪽 선행전제 한곳만 제거하고본문4/5쪽 되감김 검사와정상교육문서 반례를유지했습니다. 본문 검사는 실제 PASS이며, 원래 정상2함수본문도byte단위로동일합니다. 원문/PDF·모든 다른입력/축은유지했고315를317로갱신하지않았습니다.
- 유지3PASS/0FAIL, 필수fmt/Clippy3/workspacebuild/manifest·unit 고정base 정책7단계exit0입니다. 생산변경·새함수·기준완화없음. 전체137의60FAIL 중21곳을처리했으며 나머지39개는개별검토대상입니다. 최종전체회귀·PR준비/원문피델리티완료는아닙니다.
- 로그·비교: `output/pr-review/planet6897-7382-20260926/stage142-market2070/`. 선택6쪽 Native부정증거이며전315쪽/freshWASM은미실행입니다.

![시장구조조사94쪽 review](../assets/issue7445/market2070_native_review_094.png)
![시장구조조사95쪽 overlay](../assets/issue7445/market2070_native_overlay_095.png)


## 보정143 사전 분석·결과 — 복학원서의 실제 실패2함수만 이관

- 사전 head `5e199f8bb`, 코드 `2234af963e60f7cc855592be0493e6282858314c`. [원문·독립 기준·실패·범위·명령](../assets/issue7445/bokhak938_blocking_scope_validation.json). 원문/PDF SHA는 #7212 기록과 같고 현재 커밋 바이트도 같습니다. 현재 Native 전체1쪽은77.85291%로90%에 미달합니다. review·overlay에서 제목·표 시작/칸높이·뒤 본문·접수 상자 차이를 직접 확인했습니다. 출력1/1 완료 뒤 exit1은 시각 gate 미달 판정입니다.
- 실패 원인은 #938 layer 검사에서 PNG 선적용 뒤에도 원래 grayScale/-50/70 효과를 요구하는 기대와 SVG 기준 차이입니다. 하지만 원문 전체가90% 미만이므로 메타데이터·golden을 현재값으로 고치지 않았습니다. [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5883744483) 후 실제 FAIL한 layer 한 함수와 같은 문서의 snapshot 한 함수만 제거했습니다. 렌더러 개선·새 함수는 없습니다.
- 수정 전2PASS/2FAIL, 유지2PASS/0FAIL입니다. SVG·overlay 워터마크 두 정상 함수와 다른5개 snapshot·determinism, 원문/PDF·기존 golden·다른 입력/축을 보존했습니다. 모든 남은 함수 본문과 기존 golden은 이전 head와 동일합니다. 진단 actual.svg만 SHA를 확인해 output으로 옮겼습니다.
- 필수fmt/Clippy3/workspace build/manifest·unit 고정base 정책7단계 exit0입니다. 전체137의60FAIL 중23곳을 처리했으며 나머지37개는 개별 검토 대상입니다. 현재 전체 회귀 통과·PR준비 완료·원문 피델리티 개선으로 보고하지 않습니다.
- 비교/로그: `output/pr-review/planet6897-7382-20260926/stage143-bokhak938/`. 이번 fresh WASM은 미실행이며 Native 부정 증거만 사용했습니다.

![복학원서 전체1쪽 review](../assets/issue7445/bokhak938_native_review_001.png)
![복학원서 전체1쪽 overlay](../assets/issue7445/bokhak938_native_overlay_001.png)


## 보정144 사전 분석·결과 — 정책연구 그림5 검사의 전체쪽수 전제만 이관

- 사전 head `9ae787bb9`, 코드 `00f0aa74ebca0e04a8c59de4b9469c3fb7666a87`. [독립 근거·실패·범위·명령](../assets/issue7445/policy_caption_page_pin_blocking_scope_validation.json). 보정136과 같은 원문/PDF의 커밋 바이트를 확인하고11쪽 review를 다시 직접 판독했습니다. 11쪽46.45331%이며 그림6·뒤본문·각주가 누락됩니다. 전체215/현재216쪽입니다.
- 단독 실행5PASS/1FAIL이며 그림5 셀2개/행높이/캡션267.621053px는 이미 통과했습니다. [#7445 추가등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5883869473) 후 마지막 전체215쪽 단정 한곳만 제거했습니다. 같은 함수의 그림·캡션 및 다른5개 공개4쪽 입력 검사·원문/PDF·모든 다른입력/축을 보존했습니다. 216으로 재고정하거나 렌더러를 보정하지 않았습니다.
- 유지6PASS/0FAIL, 필수fmt/Clippy3/workspacebuild/고정base manifest·unit 정책7단계exit0. 전체137의60FAIL 중24곳 처리이며 나머지36개는 개별검토 대상입니다. 최신 전체통과·PR준비 완료가 아닙니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage144-policy-caption-page-pin/`입니다. 이번 fresh WASM 미실행입니다.


## 보정145 사전 분석·결과 — 편람 HWP의 #3930 쪽수전제 한곳만 이관

- 사전 head `9879dd922`, 코드 `b7551d228c36d6fdef4b3287bfb1edf4af6c6e2c`. [실패·독립근거·범위·명령](../assets/issue7445/handbook_hwp_page_pin_blocking_scope_validation.json). 동일 원문/기준PDF의 커밋 바이트를 확인하고 기존312쪽 review를 직접 다시 판독했습니다. 정상384/현재383쪽이며 선택Native 최저6.48027%입니다. 부록의 물리쪽 소속 차이가 있습니다.
- 단독1FAIL은 Q8 표제 검사 통과 뒤 마지막384쪽 전제에서 발생했습니다. [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5883899549) 후 전제한곳만 제거해 같은함수의Q8검사를 유지했고 다른HWPX·바탕쪽·IR 및원문/PDF/모든다른입력·축도 유지했습니다. 현재383으로 재고정하지 않았습니다.
- 유지1PASS/0FAIL·필수fmt/Clippy3/workspacebuild/고정base manifest·unit 정책7단계exit0입니다. 생산변경·새함수·공차완화없음. 전체137의60FAIL 중25곳 처리이며 나머지35개는개별검토대상입니다. 최종전체/PR준비 미완료이며 이번 freshWASM 미실행입니다. 로그: `output/pr-review/planet6897-7382-20260926/stage145-handbook-hwp-page-pin/`.


## 보정146 사전 분석·결과 — 편람 HWPX의 실제 차단 쪽수·물리쪽 전제만 이관

- 사전 head `829a31516`, 코드 `2b6355bf430510e9bde6064f80a4efbea6fbe10e`. [독립근거·단독실패·연속진단·범위·명령](../assets/issue7445/handbook_hwpx_owner_blocking_scope_validation.json). 원문/정상PDF의커밋바이트는 동일합니다. PDF384/현재385쪽, 기존부록/말미0%와 이번Native272쪽84.03568%로 전체피델리티가 부적격입니다. 추가294/295/296쪽은97.17009/96.20827/97.61333%입니다. 272·294review 및296standalone overlay를 직접판독했습니다. 일부높은값으로전체통과를선언하지않습니다.
- 단독3PASS/2FAIL입니다. Q27/Q29/Q30의 물리쪽단정과 이후전체쪽수3전제는 하나씩 실제FAIL을실행해확인했습니다. #3930의0기반293(물리294)에 Q27/Q29가 있어야한다는 요구와달리현재/PDF294에는 Q22/Q23이 있고296에는Q27/Q29가 있습니다. [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5883936903) 후 쪽수전용#5801 한함수 및혼합#3930의실패6단정만 제거했습니다. 현재숫자로 기준을갱신하지않았습니다.
- 기존정상3함수본문은동일합니다. #3930의이전통과배치·p144셀넘침·저장전후13쪽tree동일성·SectionDef0/19·Odd슬롯/짝수상속·BorderFill·그림extra/밝기/대비도최종실행PASS로 유지했습니다. 원문/PDF·다른입력/축 및차단하지않는HWPX oracle원장행은 그대로입니다.
- 최종집중4PASS/0FAIL·필수fmt/Clippy3/workspacebuild/고정base manifest·unit 정책7단계exit0입니다. 처음파생suite를준비하지않은진단0tests/exit4는보존하고통과로세지않았으며 prepare뒤정상 재실행했습니다. 생산변경·새함수·공차완화없음. 전체137의60FAIL 중27곳 처리이며나머지33개는개별검토대상입니다. 최종전체/PR준비 미완료입니다.
- 로그/시각: `output/pr-review/planet6897-7382-20260926/stage146-handbook-hwpx-owner/`. 이번선택4쪽Native증거이며freshWASM 미실행입니다.

![편람 HWPX272쪽 review](../assets/issue7445/handbook_hwpx_owner_native_review_272.png)
![편람 HWPX294쪽 review](../assets/issue7445/handbook_hwpx_owner_native_review_294.png)
![편람 HWPX296쪽 overlay](../assets/issue7445/handbook_hwpx_owner_native_overlay_296.png)


## 보정147 사전 분석·결과 — 배포용 HWPX의 실제 차단2함수만 이관

- 사전 head `9f1d8886d`, 코드 `c13a8021a5b411184aff76a9eca28dfd07f6e9f9`. [원문·독립시각·실패·범위·명령](../assets/issue7445/distribution7160_blocking_scope_validation.json). 동일원문/PDF 커밋바이트를확인하고현재고정Native 전체3쪽을새로비교했습니다. 1/2/3쪽95.31683/89.38903/60.66529%로최저90미만입니다. 3쪽review를직접판독해표위단위문구가표아래로내려가고표원점이위/왼쪽으로옮겨진것을확인했습니다. 전체3/3출력완료 뒤exit1은gate미달입니다.
- 단독4PASS/2FAIL입니다. [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5884037641) 후실제실패한줄소유/표왼쪽원점 두함수만제거했습니다. 말미공백·단위문구가로위치·같은줄표·소제목과목록소유의기존정상4본문은byte단위동일하며원문/PDF·모든다른입력·축도유지했습니다. 생산변경·새함수·현재값재고정없음.
- 유지4PASS/0FAIL·필수fmt/Clippy3/workspacebuild/고정base manifest·unit 정책7단계exit0입니다. 전체137의60FAIL 중29곳처리며나머지31개는개별검토대상입니다. 최종전체회귀/PR준비 완료가아닙니다.
- 비교/로그: `output/pr-review/planet6897-7382-20260926/stage147-distribution7160/`. pdftotext bbox가exit-6으로문항marker분석만생략했으며raster/review/overlay3/3은정상완료했습니다. 이번freshWASM은미실행입니다.

![배포용 HWPX3쪽 review](../assets/issue7445/distribution7160_native_review_003.png)
![배포용 HWPX3쪽 overlay](../assets/issue7445/distribution7160_native_overlay_003.png)


## 보정148 사전 분석·결과 — 저슬랙 원문의 실제 차단2함수만 이관

- 사전 head `ac94ab661`, 코드 `6fd829478a0e11a93113006a08ddc38b724aefe6`. [독립근거·쪽수중단·시각·실패·범위·명령](../assets/issue7445/low_slack6535_blocking_scope_validation.json). 동일원문/PDF의 커밋바이트를확인했습니다. 정상PDF1/현재2쪽이며 전체sweep은SVG/tree2·PDF1의쪽수불일치로비교전중단됐습니다. 이를통과또는시각gate완료로세지않았습니다.
- 별도같은물리1쪽비교69.33089%이며review·standalone overlay를직접판독했습니다. 기준1쪽의서명·결재선·주소/전화블록이현재1쪽에서빠졌습니다. 1쪽부정증거를전체2쪽비교완료로보고하지않습니다.
- 단독2PASS/2FAIL은모두1쪽전제에서현재2쪽으로실패합니다. [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5884097204) 후실제쪽수전용한함수파일과같은입력의하단위치한함수만제거했습니다. 공통helper·다른페이지앵커/초과근무의정상2본문·모든원문/PDF·다른입력/축은byte단위동일하게유지했습니다. 현재2쪽으로재고정·생산변경·새함수없음.
- 유지2PASS/0FAIL·필수fmt/Clippy3/workspacebuild/고정base manifest·unit 정책7단계exit0입니다. 전체137의60FAIL 중31곳처리며나머지29개는개별검토대상입니다. 최종전체회귀/PR준비 완료가아닙니다.
- 비교/로그: `output/pr-review/planet6897-7382-20260926/stage148-low-slack6535/`. 이번freshWASM은미실행입니다.

![저슬랙1쪽 review](../assets/issue7445/low_slack6535_native_review_001.png)
![저슬랙1쪽 overlay](../assets/issue7445/low_slack6535_native_overlay_001.png)


## 보정149 사전 분석·결과 — #7288 잘못된 조회쪽만 독립 PDF로 교정

- 사전 head `94411a3de`, 코드 `38830266cfa0206bcd1b8ca55630076cb7a4a07a`. [독립쪽/좌표·실패·시각·범위·명령](../assets/pr7382_20260926/stage149_stale_frame_validation.json). 원문HWP/기준PDF는현재커밋바이트와같으며정상/현재모두242쪽입니다. Native영향63/64/65쪽99.8046/99.99042/99.97124%이고freshWASM도같습니다. 선택3쪽모두90이상이며Native63review·WASM63standalone overlay를직접판독해표외곽·뒤본문·누락/줄바꿈을확인했습니다. 글자획/굵기잔여차이가있으므로픽셀완전일치를주장하지않습니다.
- 단독1PASS/1FAIL은대상문단을못찾은검사설정오류입니다. 독립PDF의주)대각선문단은물리63쪽에있고64쪽에는없습니다. 현재Native63쪽문단1374첫줄364.1px와PDF첫글자상단272.893677pt=363.858236px는기존저장기대364.12px에맞습니다. 기존PAGE=63(물리64쪽)을62(물리63쪽)로만교정했습니다. 기대좌표·공차1px·문단번호·본문넘침상한2px·두함수본문은그대로입니다.
- 유지2PASS/0FAIL·필수fmt/Clippy3/workspacebuild/고정base manifest·unit 정책7단계exit0입니다. 생산변경·새함수·허용치완화·#7445검사제거없음. 전체137의60FAIL 중32곳처리며나머지28개는개별검토대상입니다. 최종전체회귀/PR준비완료가아닙니다.
- freshWASM은루트에서공유target/pr-review를써새로빌드한no-opt로컬대체입니다. rootpkg/Studio의JS/WASM SHA동일성을확인했고새WASM은기존생산동일패키지SHA와같습니다. Docker최적화빌드통과로보고하지않습니다. 비교/로그: `output/pr-review/planet6897-7382-20260926/stage149-stale-frame7288/`. 전체242쪽시각일치를주장하지않습니다.

![#7288 물리63쪽 Native review](../assets/pr7382_20260926/stage149_stale_frame_native_review_063.png)
![#7288 물리63쪽 fresh WASM overlay](../assets/pr7382_20260926/stage149_stale_frame_wasm_overlay_063.png)


## 보정150 사전 분석·결과 — 폰트 추적의 공개 HWP 렌더핀만 별도 이관

- 사전 head `6ac946c5f`, 코드 `d2dd864b68efc7bba55e59a2a20483e7e9a36733`. [원인경로·독립기준·단독실패·범위·명령](../assets/issue7445/fonttrace4961_blocking_scope_validation.json). 단독7검사6PASS/1FAIL에서exact-face문자수1336/1334가다릅니다. devel0e8와현재고정Native194run을직접대조해각주번호의뒤공백1개및각주공백run의1개가달라진것을확인했습니다. `stored_footnote_number_prefix → layout_footnote_area → collect_runs/run.text → counts/layoutHash`경로이며본문문자누락이아닙니다.
- 같은원본은한컴2010저장8.5.6.1133으로info를확인해engine2020으로정상PDF를생성했습니다. job succeeded/download SHA고정, 새PDF는원문과함께커밋했습니다. 정상/현재모두1쪽이나전체Native88.61634%입니다. review직접판독에서OLE그래프·오른쪽열흐름차이를확인했습니다. 90미만이므로현재값으로핀을재고정하지않습니다.
- [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5884283688) 후exact-facecounts한핀, 그뒤별도실제로FAIL한layoutHash한핀만데이터에서명시적으로이관했습니다. 혼합함수/폰트프로필/다른5문서핀·형식동일성·기능탐지·limit·backend검사·다른정상함수본문은유지했습니다. 명시적이관선언이없으면핀누락도여전히assertFAIL이며문서ID를Rust에하드코딩하지않았습니다. 현재trace에서exact-face프로필모든필드일치도readonly로대조했으나정식전체e2ePASS로승격하지않습니다.
- **최종집중6PASS/1FAIL/exit100**입니다. 다른문서missing-face의layoutHash실패가드러났으며이는다음개별판정대상으로남겼습니다. 혼합함수는끝까지통과하지않았고전체137의60FAIL중해결수는32/남은28로유지합니다. 필수fmt/Clippy3/workspacebuild/고정base manifest·unit 정책7단계는exit0입니다.
- 초기출력래퍼가nextest실패를전달하지않아lint앞단표시0이됐지만원시JSON/요약exit100을확인했고현재래퍼를수정해실제exit100으로재실행했습니다. 기존10개집중after결과의실제exit0도감사했습니다. 이실패를통과로보고하지않습니다. 생산변경·새함수·핀/공차완화없음. 로그: `output/pr-review/planet6897-7382-20260926/stage150-fonttrace4961/`. 이번freshWASM미실행입니다.

![폰트 추적 공개 HWP1쪽 review](../assets/issue7445/fonttrace4961_native_review_001.png)
![폰트 추적 공개 HWP1쪽 overlay](../assets/issue7445/fonttrace4961_native_overlay_001.png)


## 보정151 분석·보정152 결과 — 붉은 면으로 덮인 WMF 그림 복원

- 사용자께서 확인한 `CLP000030900001.wmf`(원본 속성541×311px)는 월간 수출입 보도자료의 그림입니다. 같은 HWP `BinData/BIN0003.wmf` 원본92,032바이트를 추출했습니다. [원인·명령·입력/PDF/WMF·전후증거](../assets/pr7382_20260926/stage152_wmf_vector_mask_validation.json). 코드 `c3f748325f9ebe7f1d774d378f4159ae1226ae20`. OLE 차트 종류를 바꾸지 않고 실제 WMF 재생 경로를 수정했습니다.
- 원본의510/523번 PATINVERT가 각각붉은사각형으로출력되어517번마스크윤곽을덮었습니다. 독립 MS-WMF 규칙 `(D xor P) and M xor P`는 흰마스크에서배경D, 검은마스크에서원래색P입니다. `poly_polygon → R2_MASKPEN/흑백팔레트/동일clip·영역 → 마지막동일XOR → finish`로연결하며완성된쌍만역마스크윤곽으로합성합니다. 문서ID/특정좌표예외는없습니다.
- 기존 #6865함수에 이 그림의원본바이트와다색팔레트/마지막XOR변경반례를연결했습니다. 함수추가0, 기존15함수유지. 정상 #6469·WMF/EMF golden도포함해수정전17PASS/1FAIL(그림마스크누락), 수정후18PASS/0FAIL입니다. 기대golden/공차는변경하지않았습니다. 필수fmt·Clippy3·workspacebuild·고정base manifest/unit정책7단계exit0입니다.
- 새한컴PDF19쪽을같이커밋했습니다. 영향1쪽 Native/freshWASM은둘다99.82783%, 그림전체영역은43.17549→98.93048%입니다. Native review와freshWASM standaloneoverlay/그림PNG를직접읽어붉은꺾은선·범례와배경복원을확인했습니다. 다른글꼴/잉크굵기의작은차이는남깁니다. root freshWASM --no-opt/Studio동기화SHA도확인했으며Docker최적화빌드로보고하지않습니다.
- 전체글꼴SVG가380MB여서초기2회는브라우저navigation30초제한으로PNG0장/미완료였습니다. 대기한도만 `RHWP_VISUAL_RASTER_TIMEOUT_MS=180000`으로설정해정상재실행했습니다. 글꼴/좌표/viewport/실제load·fontready·screenshot조건은같습니다. 진단73.85초·canonical Native/freshWASMexit0을구분해보존했습니다.
- 수정후전체19쪽Native는계속실행중이며완료판정하지않습니다. 보정151의이전전체19쪽최저73.34034%를최종head통과로재사용하지않습니다. #4961 다른문서hash와전체137의남은28개개별검토/최종전체회귀/원PR시각게이트는미완료이며통합PR준비보류입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage152-wmf-vector-mask/`에만둡니다.

![WMF 그림복원 Native1쪽 review](../assets/pr7382_20260926/stage152_wmf_native_review_001.png)
![WMF 그림복원 fresh WASM1쪽 overlay](../assets/pr7382_20260926/stage152_wmf_wasm_overlay_001.png)
![복원된 WMF 꺾은선 그림](../assets/pr7382_20260926/stage152_wmf_curve.png)


## 보정153 사전 분석·결과 — 월간 수출입 HWP의 실패한 해시 한 핀만 이관

- 사전 head `a7e0b91a0`, 코드 `911063c54234e3ad206f17b71cce098ab1afc0cc`. [독립 PDF·새 전체 시각·실패/보존 범위·명령](../assets/issue7445/monthly_trade_fonttrace_blocking_scope_validation.json). 사용자 지정 WMF 복원 후의 고정 Native로 전체19/19쪽 compare/overlay/review를 완료했습니다. 정상/현재19쪽, 최저73.34034%(8쪽), 4/6/8/9쪽이90미만이며 exit1은 완료된 시각 gate 실패입니다. 직접4/8쪽 review에서 표/후속본문 위치 차이를 확인했습니다. freshWASM 영향1쪽과 WMF 복원 검증은 보정152에 연결하며 전체19쪽 freshWASM은 미실행입니다.
- 사전 집중6PASS/1FAIL에서 `missing-face`의 해시만 실패했습니다. counts607/run136·status는 통과했습니다. [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5884848639) 후 `expectedLayoutHash` 한 핀만 명시적으로 이관하고 기존함수·counts·다른5문서·profiles/options/comparisons를 보존했습니다. XY는 해시의 직접 입력이 아니며 정확한 해시 원인은 baseline 실행 재현이 부족해 미검증으로 남겼습니다. 현재 해시로 기대값을 재고정하지 않았습니다.
- 수정 후 **6PASS/1FAIL/exit100**이며 다른 `subst-counterpart-hwp`의 해시가 다음 실패입니다. 혼합함수 전체 통과로 보고하지 않습니다. 현재 missing-face 프로필9필드의 readonly 일치는 확인했고 필수fmt/Clippy3/workspacebuild/고정base manifest/unit정책7단계는exit0입니다. 생산변경/새함수0. 원래60개 중32곳처리/28개개별검토대기를 유지합니다. 최종전체회귀/PR준비미완료. 로그는 `output/pr-review/planet6897-7382-20260926/stage153-fonttrace-monthly-trade/`입니다.

![월간 수출입4쪽 review](../assets/issue7445/monthly_trade_fonttrace_native_review_004.png)
![월간 수출입8쪽 review](../assets/issue7445/monthly_trade_fonttrace_native_review_008.png)
![월간 수출입8쪽 overlay](../assets/issue7445/monthly_trade_fonttrace_native_overlay_008.png)


## 보정154 사전 분석·결과 — 온새미로 HWP의 실패 해시 한 핀만 이관

- 사전 head `0e7a35743`, 코드 `2bbbdc6a733d85b0f09b538a1f4693a47982d038`. [원본/기준·해시 원인·시각·범위·명령](../assets/issue7445/onsaemiro_hwp_fonttrace_blocking_scope_validation.json). 기존 oracle 대응행과 PDF rename100% 바이트 이력을 확인했습니다. 전체 비교는 rhwp47/PDF46쪽으로 사전 차단돼 compare0장입니다. 별도 Native/freshWASM1/2/6/46쪽을 새로 캡처했고6쪽43.40896%, 46쪽37.41779%로 미달했습니다. 1쪽92.25199%를 문서 전체 통과로 사용하지 않습니다. review1/6/46 및WASM6 overlay 직접 판독에서 같은 내용·정상 기준 출력, 보기상자/줄바꿈/뒤본문과 물리쪽 소속 차이를 확인했습니다.
- 집중6PASS/1FAIL은 `subst-counterpart-hwp` 해시만 실패합니다. 원본 CharShape89/90/124/125의 상대크기는 모두100%입니다. `resolve_single_char_style(font_size*100/100) → 실제 run font_size → heuristic 폭 → px_to_hwpunit 절삭 → layoutMetric/layoutHash` 경로에서 549/1099 등1HWPUNIT 감소가 있습니다. 원본 base_size에서 상대100%의 원래 크기로 독립 폭을 재구성하면 기존 기대 해시와 정확히 일치합니다. 이 계산 증거를 수정 전 baseline 실행으로 보고하지 않습니다.
- [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5885044173) 후 실제 HWP 해시 한 핀만 명시적으로 이관했습니다. 현재 해시로 재고정하거나 생산 보정하지 않았고 counts46run/41문자·status·다른5문서/HWPX·모든프로필/형식동일성/기능탐지·기존함수는 보존했습니다. 최종6PASS/다른 `document-substitute-hwpx` 해시1FAIL/exit100, 필수fmt/Clippy3/workspacebuild/base고정 manifest/unit정책7단계exit0입니다. 원래32곳처리/28개검토대기는 유지합니다. 최종전체회귀/PR준비미완료입니다.
- 생산은 보정152와 같고 같은 fresh no-opt root WASM으로 새 캡처했습니다. Docker 최적화 검증 아님. 로그: `output/pr-review/planet6897-7382-20260926/stage154-fonttrace-onsaemiro-hwp/`.

![온새미로 HWP6쪽 review](../assets/issue7445/onsaemiro_hwp_fonttrace_native_review_006.png)
![온새미로 HWP46쪽 review](../assets/issue7445/onsaemiro_hwp_fonttrace_native_review_046.png)
![온새미로 HWP6쪽 fresh WASM overlay](../assets/issue7445/onsaemiro_hwp_fonttrace_wasm_overlay_006.png)


## 보정155 사전 분석·결과 — 온새미로 HWPX의 실패 해시 한 핀만 이관

- 사전 head `ba4ab4a37`, 코드 `8d4233f0f9ed89b997d91f6a1ee0c5f782141fa0`. [독립 원문/기준·원인·새 시각·범위·명령](../assets/issue7445/onsaemiro_hwpx_fonttrace_blocking_scope_validation.json). HWP 판정을 대신 사용하지 않고 기존 oracle의 같은 HWPX 대응PDF로 선택1/2/6/46쪽을 Native/freshWASM에서 새로 비교했습니다. 정상46/현재47쪽, 최저37.74556%, 6쪽43.45823%입니다. 직접 Native6 review/WASM6 standalone overlay에서 보기상자 높이·줄바꿈·뒤본문 배치 차이를 확인했습니다. 전체47쪽 raster비교는 이번에 실행하지 않았으며 선택4쪽으로 범위를 제한합니다.
- 실제 `document-substitute-hwpx` 해시만 실패했고 counts46run/41문자와status는 통과했습니다. HWPX 원본 charPr height/relSz100%에서 원래크기/폭/자간/절삭 규칙으로 재구성한 해시는 기존 기대와 정확히 일치합니다. relative100% 곱셈/나눗셈의 미세 오차→정수절삭→trace metric 경로이며 readonly 계산을 baseline 실행 증거로 보고하지 않습니다. 최초진단의 HWP용필드 조회 KeyError는 원본HWPX필드를 확인해 수정했고 source/test변경없습니다.
- [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5885116951) 후 실제 실패한 HWPX 해시 한 핀만 명시적으로 이관했습니다. 생산/현재해시재고정/새함수0. 모든 기존함수·counts/status·다른5문서·프로필·형식동일성·substFont 비대칭 기능탐지·limit/backend/determinism을 유지하고 **최종7PASS/0FAIL/exit0**을 확인했습니다. 필수fmt/Clippy3/workspacebuild/base고정 manifest/unit정책7단계exit0입니다. #4961 원래혼합실패1건의 유지검사 통과로 전체137의60FAIL중33곳처리/27개검토대기로 갱신합니다. 문서피델리티를해결한것은아니며최종전체/PR준비미완료입니다.
- 생산은 보정152와 같고 같은 root fresh no-opt WASM으로 새 캡처했습니다. Docker 최적화 검증 아님. 로그: `output/pr-review/planet6897-7382-20260926/stage155-fonttrace-onsaemiro-hwpx/`.

![온새미로 HWPX6쪽 review](../assets/issue7445/onsaemiro_hwpx_fonttrace_native_review_006.png)
![온새미로 HWPX6쪽 fresh WASM overlay](../assets/issue7445/onsaemiro_hwpx_fonttrace_wasm_overlay_006.png)


## 검증156 — 남은27개 검토 대상의 현행 head 개별 실행

- 검사 head `e13abf3b1`에서 남은27개를 각1함수씩, 고정target/locked release-test/threads8/--no-fail-fast로 순차 실행했습니다. **27개 개별 명령 완료: 0PASS/27FAIL**, 각명령은 실제1검사를실행했습니다. [대상·현행target해석·명령·exit·요약](../assets/pr7382_20260926/stage156_remaining_individual.json). source/test변경·새검사·허용치완화없음. Cargo진행중에는source/test를수정하지않았습니다.
- 원래60개 중33곳처리/남은27개는 이제 현행 head에서도 실패가 재현된 대상입니다. 전체10250검사 실행을 대체하지 않으며 다음은 각 입력별 독립시각/원인 판정 후 개별 보정입니다. #1100의 기존 PDF는A4인데 원본은771×1117pt로용지가달라 재산출·배율을 먼저 확인합니다. 미달점수만으로 해당함수를제거하지않았고 사용자지시대로 새한컴PDF 비교를진행합니다.
- 변경분의 src/tests 추가주석도감사해순수영문추가주석0을확인했습니다. 기존upstream주석은별도입니다. 로그: `output/pr-review/planet6897-7382-20260926/stage156-remaining-individual/`. 최종전체/PR준비미완료입니다.


## 보정157 사전 분석·결과 — 시험지 4쪽 개선과 의미 회귀 교정

- 사전 head `8b9783013`, 코드 `4e50d2317e83d408ee12d6adc92d7e9a37aa8fbd`. [원인·소비 경로·독립 PDF·명령·전체 결과](../assets/pr7382_20260926/stage157_exam_social_validation.json). 원본은 [exam_social.hwpx](../../../samples/hwpx/exam_social.hwpx)이며, 이전 A4 기준595×841pt가 원문의771.02×1116.85pt와 달랐습니다. 같은 원문을 실제 저장 제품에 맞는 engine2020으로 재출력한 [정상 한컴 PDF](../../../pdf/exam_social-hwpx-revalidated-2020.pdf)는4쪽/771×1116pt, SHA `68a1c41533ebd0e3408799e642e70a2cd04ebf20bb599763e2c3a7331df00b95`입니다. 이전 A4 점수를 회귀 제거 근거로 사용하지 않습니다. 사용자 지정 개선으로 [#7445 이관을 철회](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5885313883)했으며 기존3함수를 유지합니다.
- **2쪽**: 10번 표 앞 공백 줄의 `text_height500 + spacing200 = 다음 vpos700HU`가 빠져 표와 뒤본문이 앞당겨졌습니다. `stored_tac_lines → stored_tac::prepare`에서 저장 소유줄의 top/occupied_end/end와 가용 높이를 함께 확인합니다. `StoredTacPlan::placement → commit_stored_tac_control → InlineBoxPlacement → layout_table_control_block → flow_table_y`가 같은 원점을 사용하며 advance_end로 뒤 간격을 다시 더하지 않습니다. 순수공백/마지막 단일TAC표·저장 비편집·실측높이 일치·전체끝fit만 수용하고, 캡션·각주·불일치는 일반 경로입니다. 최종 원점795.9→805.3px, 독립 PDF804.675px이며 폭·높이는 유지됩니다. 특정 문서ID나 위치 예외를 넣지 않았습니다.
- **3쪽**: 쪽번호의 표시문자열은 `3`인데 말미 공백 판정은 원모델의 자리표시자 ` `를 제외하여 우측 정렬 번호가 글상자 밖으로 밀렸습니다. 폭 측정과 같은 `effective_text_for_metrics`로 말미 공백을 판정하고 모델 문자 위치/TAC 경계는 보존했습니다. `shape_layout`의 글상자 정렬과 `paragraph_layout`의 줄 정렬이 이 판정을 소비합니다. 정상 뒤공백·셀 정렬과 자동번호 대조군을 실행했습니다.
- **4쪽**: 원본 paraPr25/43의 번호 형식은 HANGUL_JAMO이고 사양 표41의 값10은 ㄱ/ㄴ/ㄷ입니다. `numbering_format_to_number_format`의 누락된10분기 때문에 Digit로 출력되었습니다. `expand_numbering_format → format_number`에 자모 형식을 연결해16/18번 보기 ㄱ~ㄹ을 복원했습니다. 자동 쪽번호의 별도 코드축은 유지했습니다. **1쪽**은 본문·그림·표 소속과 전체 배치를 직접 확인했으며 원문 근거 없는 위치 조정을 하지 않았습니다.
- 사용자 지시대로 기존3함수의 절대 x/y·bbox 핀을 없애고 각 쪽 문항1~5/6~10/11~15/16~20의 순서·단일 소속, 10번 자료의2쪽 소속, 16/18번 자모, 머리말/바탕쪽 번호1회 치환·뒤 figure-space와 인쇄묶음32 보존을 검사합니다. 새 `#[test]` 함수는0개입니다. 원래 함수의 수정 전2PASS/좌표1FAIL과 새 의미 검사 수정 전2PASS/자모1FAIL을 구분해 보존했습니다. **수정 후 기존3+정상 대조군12=15PASS/0FAIL**, 기존번호 단위2PASS/0FAIL입니다. 자모 검사만 수정 전 의도한 결함을 검출했고, 표 원점·번호 잘림 개선은 아래 독립 시각 증거로 입증합니다.

| 쪽 | 정상 용지 재출력 후 수정 전 | 최종 Native | 최종 fresh WASM | 직접 확인한 결과 |
| --- | ---: | ---: | ---: | --- |
| 1 | 93.21648% | 93.21648% | 93.21648% | 문항1~5·표/그림·바탕쪽 보존 |
| 2 | 87.65269% | 96.78390% | 96.78390% | 10번 표·자료와 후속 선택지의 위치 개선 |
| 3 | 91.97659% | 92.12235% | 92.12235% | 머리말3의 잘림 제거 |
| 4 | 93.32797% | 95.00799% | 95.00799% | 16/18번 보기 자모 복원 |

- 최종 Native/freshWASM 각4/4쪽 compare/standalone overlay/review 완료·exit0, 최저92.12235%·gate passed·글꼴 예외 없음입니다. Native review1~4와 freshWASM standalone overlay2~4를 직접 읽었고 각 쪽 실제 렌더 PNG의 두 backend SHA가 같습니다. 제목/본문 글자형·굵기, 특수기호와 작은 위치 차이는 남습니다. PDF Type3 정보만으로 원래 글꼴 대응은 미검증이며 엄격 픽셀/잉크 완전 일치나 전체 피델리티100%로 보고하지 않습니다.
- 필수fmt·Native/WASM/workspace-alltargets Clippy·workspace build·고정base `0e8fd49fb868da0d47ac1294dcbbda81f0211233` manifest/unit 정책7단계는 모두exit0입니다. root fresh WASM --no-opt와 Studio 복사본 SHA `d0880e5e146d3e778d2a9e69f7d0a45a5772d8facd185d6d9fc1ecec1c877711`가 같습니다. 로컬 대체 빌드이며 Docker 최적화나 Studio 브라우저 실행 검증으로 보고하지 않습니다.
- 초기 집중 명령에 `--tests`를 함께 넣어 선택 외 전체 대상까지 빌드되는 자신의 오류를 확인했습니다. 소유 nextest/Cargo를 SIGINT로 중단한 실행은exit-2·통과 아님으로 보존했고, 선택한 `--test` 대상으로 정상 재실행해15PASS를 확인했습니다. 출력 공백 때문에 중단한 것이 아니며 캐시를 삭제하지 않았습니다.
- AGENTS/CLAUDE/CONTRIBUTING 및 canonical 증적 지침에 실물 문서의 의미 계약과 독립 시각 판독 기준을 반영했습니다. 원래60개 중34개 처리/26개 개별 검토 대기이며, 현재 전체 nextest·Native Skia3·원 PR 전체 최종 시각 검증/통합PR 준비는 아직 미완료입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage157-exam-social-header/`에만 있으며 커밋하지 않습니다. 전체 PNG는 그 아래 `visual-final-native/exam1100-header/`와 `visual-final-wasm/exam1100-header/`에서 확인할 수 있습니다.

![시험지2쪽 보정 전 review](../assets/pr7382_20260926/stage157_before_native_review_002.png)
![시험지2쪽 최종 Native review](../assets/pr7382_20260926/stage157_native_review_002.png)
![시험지2쪽 최종 fresh WASM overlay](../assets/pr7382_20260926/stage157_wasm_overlay_002.png)
![시험지3쪽 최종 Native review](../assets/pr7382_20260926/stage157_native_review_003.png)
![시험지4쪽 최종 Native review](../assets/pr7382_20260926/stage157_native_review_004.png)


## 보정158 — CGMP 평가표의 실제 차단 셀 넘침 입력만 이관

- 코드 `0e02483d2aec01174b6fe07a1754b399a6ac2350`, [원인·정상 PDF·명령·범위·결과](../assets/issue7445/cgmp6035_cell_blocking_scope_validation.json). `build_page_render_tree → take_overflow_cell_lines → 신규 원장 판정`의 신규7줄을 현재 head에서도 단독 재현했습니다. 같은 원문을 저장제품2022에 맞는 engine2020으로 정상 출력한 PDF48쪽/rhwp50쪽입니다. 원문/PDF는 커밋했고 PDF의 신청서 앞/뒷면44/45쪽과 rhwp46쪽의 소속이 다르며 뒤내용이 쪽 밖으로 나감을 직접 확인했습니다.
- Native/freshWASM1/44/45/46/47/48쪽 각6개 compare/overlay/review 완료·exit1(gate 실패), 최저0%입니다. Native46쪽1.39325%/WASM46쪽1.36340%, 직접 review/standalone overlay로 판독했습니다. 전체50쪽 시각 통과로 쓰지 않습니다. [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5886337869) 후 이 입력의 셀 넘침 assertion만 보류했습니다. 수집·분할과 관측은 유지해 다른입력 소속을 바꾸지 않았고, 같은문서 정상2함수·다른원장·원문·baseline/공차·기존16분할을 유지했습니다. 새함수/생산변경0입니다.
- 수정 후 해당분할1+정상2함수 **3PASS/0FAIL**, fmt/diff check 완료입니다. 필수lint 묶음은 남은 개별 보정 뒤 최종head에서 수행 예정이며 지금 PASS로 세지 않습니다. 원래60개 중35개 처리/25개 검토 대기입니다. [현재26개 진행 원장](../assets/pr7382_20260926/remaining26_validation_progress.json). 최종전체/통합PR 준비 미완료. 로그는 `output/pr-review/planet6897-7382-20260926/stage158-cell6035/`입니다.

![CGMP 셀넘침 Native46쪽 review](../assets/issue7445/cgmp6035_cell_native_review_046.png)
![CGMP 셀넘침 freshWASM46쪽 overlay](../assets/issue7445/cgmp6035_cell_wasm_overlay_046.png)


## 보정159 사전 분석·결과 — 심사지표 8쪽의 중첩 표와 쪽 소유 개선

- 사용자 최신 범위에 따라 소수 페이지의 보정은 현재 브랜치에서 해결하고, 대용량·복잡한 전체 피델리티 문제만 #7445로 이관합니다. 이번 문서는 이관하거나 검사에서 제외하지 않았습니다. 코드 `0dc722184`, [입력·소스 해시·원인·명령·시각 결과](../assets/pr7382_20260926/stage159_simsa2097_validation.json).
- 대상은 [21217935_simsa_jipyo.hwp](../../../samples/task2097/21217935_simsa_jipyo.hwp)입니다. 저장 제품에 맞는 engine2020으로 재산출한 [정상 한컴 PDF](../../../pdf/21217935_simsa_jipyo-hwp-2020.pdf)는 8쪽·192480바이트, SHA `87e4ccc02ce169facee5393394637101cbc1cb874cc1735d31bf8b66a50f84d2`입니다. 원문은 그대로 보존했습니다.
- **사전 원인**: `overflow_cell_lines_do_not_grow_partition_8`이 기존 허용24줄 대비26줄로 실패했습니다. 글줄이 TAC 자식 표 높이를 이미 담는데 legacy 투영에서 같은 높이를 다시 더했고, 블록 컷은 셀 로컬 높이로 선택한 뒤 실제 행합을 더 크게 그렸습니다. 저장 병합 셀의 빈 밴드와 내용 컷도 혼동해 뒤 항목이 쪽 밖으로 나오거나 다음 쪽으로 밀렸습니다. Native/fresh WASM의 이전 전체8쪽 최저는32.58232%였습니다.
- **공통 높이와 실제 호출 경로**: 저장 줄의 제어문자 소유와 닫힌 원본 물리 상자만 canonical 유닛에 연결합니다. `cell_units → RowBlockQuery::fragment_height(MeasuredTable) → SelectedBlockCut::occupied_height/complete_row_boundary_band → scan의 끝 행 높이 → emit의 동일 측정 원장과 다음 조각 높이 → table_partial의 실제 행 상자`가 같은 컷·빈 밴드·행 높이를 소비합니다. 마지막 코드 대조에서 `stored_rowspan_page_frame`의 별도 행 높이 재계산도 제거하고 실제 분할에 사용한 `MeasuredTable`을 전달했습니다. 걸침 내용이 남거나 저장 경계가 행 내부에 있으면 완전 행 정규화를 적용하지 않습니다. 편집·재조판·유효하지 않은 저장 줄에는 저장 상자 보정을 적용하지 않습니다. 문서ID 분기·좌표 덮어쓰기·클리핑으로 내용을 감추는 보정은 추가하지 않았습니다.
- **1쪽 중첩 표**: 호스트 문단의 마지막 커서 뒤에 offset을 더하던 중복 예약을 제거했습니다. `stored_nested_float_placement → ParagraphFloatPlacement::from_stored_host`의 실제 제어문자 앵커·표 원점·점유 끝점을 유닛 측정과 통째/분할 paint가 함께 사용합니다. 원본을 다시 저장한 한컴 출력의 첫쪽은 원본 PDF raster와 같고, offset만 바꾼 독립 한컴 출력에서 실제 앵커 이동을 확인했습니다. 원본을 변형본으로 대체하지 않았습니다.
- **3→4쪽 문단**: 원본 문단의 고아 줄 방지·다음 문단과 함께·문단 전체 유지가 모두 꺼져 있고, 다음 저장 줄이 새 프레임 첫 슬롯0HU이면 실제 1+1줄 분할을 보존합니다. K-water 대조군의 다음 줄1652HU는 앞 줄 자리까지 예약된 경우라 기존 보호를 유지합니다. style만으로 모든 reset을 허용한 후보는19PASS/1FAIL로 기각했고 최종 대조군은 통과했습니다. 4쪽 가운데 정렬에서는 위쪽 정렬 전용 lazy 경로가 내용 높이를0으로 만들지 않도록 같은 선택 내용 메트릭을 소비합니다.
- **기존 회귀 교정**: `issue_2097_block_band_fill_page_pins`의 기존5문서 쪽수는 유지하고, 심사지표의 문단 쪽 소속·순서·단일 출력만 추가했습니다. 절대 픽셀/좌표·이미지 해시를 assertion에 고정하지 않았으며 새 `#[test]` 함수는0개입니다. 독립 PDF의 전체8쪽 내용이 쪽 안에 들어가는 것을 확인하고 실제 셀 넘침0줄을 측정하여, 해당 문서의 기존24줄 허용 행을 제거해 기본0줄 기준으로 **강화**했습니다. 다른 baseline·공차·문서·검사 함수는 그대로입니다. 저장된 수정 전 렌더트리에 같은 의미 계약을 적용하면6항목 중4개가 어긋나고 수정 후0개입니다. 이를 수정 전 새 Rust 테스트 실행으로 보고하지 않습니다.

| 쪽 | 보정 전 | 최종 Native | 최종 fresh WASM |
| --- | ---: | ---: | ---: |
| 1 | 64.73626% | 99.15259% | 99.15259% |
| 2 | 47.15617% | 98.63195% | 98.63195% |
| 3 | 46.42543% | 95.79791% | 95.79791% |
| 4 | 87.66419% | 95.58934% | 95.58934% |
| 5 | 54.56021% | 92.26478% | 92.26478% |
| 6 | 55.04512% | 91.85001% | 91.85001% |
| 7 | 44.30630% | 95.70257% | 95.70257% |
| 8 | 32.58232% | 98.06308% | 98.06308% |

- **최종 검증**: 기존 집중/대조군20PASS + 실제 물리 예산·끝 유닛·이어받기·뒤 문단의 기존 경계15PASS = **35PASS/0FAIL**입니다. 강화한0줄 분할8도 통과했고 전8쪽 셀 넘침은 각0줄입니다. Native/fresh WASM 각8/8쪽 compare·standalone overlay·review 완료/exit0, 최저91.85001%·gate passed·글꼴 예외 없음입니다. 최종 Native1/3/4/6 review와 WASM3/4/6 standalone overlay를 직접 확인했습니다. 전체8쪽 두 backend PNG SHA가 같습니다. 글자형·굵기·그리드 명암 및 약1px 용지/경계 차이는 남으며 엄격 픽셀 완전 일치로 보고하지 않습니다.
- root fresh WASM `--no-opt`와 Studio 복사본의 JS/WASM SHA가 각각 같습니다. Mac 로컬 대체 빌드이며 Docker 최적화·Studio 브라우저 실행 검증은 아닙니다. fmt/diff check 완료입니다. 최종head의 필수lint·정책 묶음, 전체nextest, Native Skia3 및 원 PR 전체 시각 검증은 남아 있습니다. **통합PR 준비 미완료**입니다.
- [남은26개 진행 원장](../assets/pr7382_20260926/remaining26_validation_progress.json)은2건 처리/24건 대기로 갱신했습니다. 원래60실패 기준으로는36건 처리/24건 대기이며 현재 전체 회귀 결과로 대체하지 않습니다. 다음은 #6697의31/32쪽 차단 건입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage159-cell2097/`에만 있습니다. 최종 전체 PNG는 그 아래 `visual-candidate-shared-frame-native/simsa2097/`와 `visual-candidate-shared-frame-wasm/simsa2097/`입니다. 승인된 이전 SVG만 정리하고 PDF·PNG·결과 기록·최신 산출물을 보존했습니다.

![심사지표1쪽 보정 전 review](../assets/pr7382_20260926/stage159_before_native_review_001.png)
![심사지표1쪽 최종 Native review](../assets/pr7382_20260926/stage159_native_review_001.png)
![심사지표3쪽 최종 Native review](../assets/pr7382_20260926/stage159_native_review_003.png)
![심사지표4쪽 최종 Native review](../assets/pr7382_20260926/stage159_native_review_004.png)
![심사지표6쪽 최종 Native review](../assets/pr7382_20260926/stage159_native_review_006.png)
![심사지표3쪽 최종 fresh WASM overlay](../assets/pr7382_20260926/stage159_wasm_overlay_003.png)
![심사지표4쪽 최종 fresh WASM overlay](../assets/pr7382_20260926/stage159_wasm_overlay_004.png)
![심사지표6쪽 최종 fresh WASM overlay](../assets/pr7382_20260926/stage159_wasm_overlay_006.png)


## 보정160 — 31쪽 복잡 중첩 표의 실제 캡션 차단 함수만 이관

- 사전 head `f8c3b0e9a`, 코드 `0eeb4eb76`, [원문·정상 PDF·원인 계층·선택 시각·개별 명령](../assets/issue7445/host6697_blocking_scope_validation.json). 대상은 [80550 HWPX](../../../samples/issue6697/80550-agricultural-machinery-act-amendment.hwpx)이고 [기존 정상 한컴2020 PDF](../../../pdf/issue7382-regression-review/80550-agricultural-machinery-act-amendment-2020.pdf)는31쪽/565939바이트입니다. 동일 원문·용지와30쪽 캡션/31쪽 말미 정상 내용을 직접 대조해 재사용했습니다. 현재 출력은32쪽이며 **31쪽 기대값이 틀린 것으로 판정하지 않았습니다**.
- 기존 캡션 함수는 현재 소스에서도0PASS/1FAIL로 재현했습니다. 18×4 바깥 표의 여러 쪽1×1 중첩 표 안에12×3·18×3·13×7 등의 자식 표가 이어받기됩니다. 렌더트리32쪽을 조사하고 Native30/31 review·freshWASM30 standalone overlay에서 큰 빈 밴드·내용 이월·말미 표와 뒤 내용의 소속 차이를 확인했습니다. 다음 쪽에 남는 자식의 음수 원점 자체만으로 결함을 판정하지 않았습니다. source 줄/개체 컷과 실제 물리 조각을 함께 복원해야 하는 범위입니다. 기존 offset3062HU/음수 보호/가운데 정렬3함수는 모두 통과해 단일 offset 수치 보정으로 처리하지 않았습니다.
- Native/freshWASM1/29/30/31쪽 각선택4개 compare/overlay/review 완료·exit1입니다. 두 backend의 점수는 **98.44961/37.68457/25.15773/29.21124%**이고 selected PNG는 각각같습니다. 글꼴 예외0이며 전체32쪽 PNG 검증으로 보고하지 않습니다. 정상 첫쪽만으로 문서 전체 통과를 선언하지 않습니다.
- [#7445 추가 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5889107858)·UTF-8 본문 readback 확인 뒤 실제 실패 함수 `cell_host_paragraph_caption_is_drawn_on_its_page` 한개만 정식 회귀에서 제거했습니다. 한 함수만 있던 해당 source는 제거하고, 원문/manifest/정상PDF·다른 문서·별도offset/음수/Center의 기존3함수는 유지합니다. 전체 피델리티 복원 뒤31쪽·캡션·뒤 내용 계약을 회복할 후속 항목입니다. 사용자 최신 범위의 복잡한 다쪽 문서 이관이며, 소수 페이지 보정의 일괄 이관은 수행하지 않았습니다.
- 정상3함수는 **제거 전3PASS/제거 후3PASS**입니다. 생산 코드·새 테스트 함수·기준값/공차 변경0, fmt/diff check 및 고정base `0e8fd49fb868da0d47ac1294dcbbda81f0211233` manifest 검사 통과입니다. 필수lint 전체묶음과 최종전체/원PR시각은 남아 있습니다. 남은26개 중3건 처리/23건 대기(원래60중37처리/23대기)로 갱신하며 통합PR 준비 미완료입니다. 로그: `output/pr-review/planet6897-7382-20260926/stage160-host6697/`.

![80550 Native29쪽 review](../assets/issue7445/host6697_native_review_029.png)
![80550 Native30쪽 review](../assets/issue7445/host6697_native_review_030.png)
![80550 Native31쪽 review](../assets/issue7445/host6697_native_review_031.png)
![80550 fresh WASM30쪽 overlay](../assets/issue7445/host6697_wasm_overlay_030.png)


### 보정161 — #7358 86712의 실제 차단 함수1개와 정상 글꼴 PDF

- 사전 분석 뒤 단독1FAIL을 재현했습니다. 위 여백 검사는 통과했고 후속 표543.733/기존PDF541.488px의 약2.25px 차이가 실제 실패였습니다. 위 여백 누락으로 단정하거나 기대좌표/공차를 완화하지 않았습니다.
- 원문2024 저장본·기존64쪽 PDF를 보존하고, 실제 KoPub 글꼴을 공급한 동일원문 정상 한컴2024 PDF도 커밋했습니다. 기존PDF/새PDF를 각각Native/freshWASM1·27·28·29쪽으로 직접 대조했습니다. 기존 최저80.03785%, 새PDF최저28.67074%이며 두 backend의8쌍PNG는해시동일합니다. 새PDF28쪽의수치표가27쪽으로옮겨지고후속표/말미내용소속도달라지는64쪽·28행4열/중첩표전체피델리티문제입니다. 단순좌표 보정이나 글꼴예외로통과처리하지않습니다.
- [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5889443550) 후 실제 실패1함수만 제거했습니다. 원문·두PDF·다른정상검사·생산·baseline·공차를 보존했고 기존#1133 HWP정상대조군1PASS, fmt/고정base manifest/diff0을확인했습니다. 새테스트0입니다. [명령·자료해시·부정시각·보존범위](../assets/issue7445/outer7358_blocking_scope_validation.json).
- 코드`b0aed8322`. 남은26개중4처리/22대기(과거전체60실패중38처리/22대기)이며 현재전체결과가아닙니다. 최종lint·전체nextest·NativeSkia·원PR최종전체시각·통합PR준비는미완료입니다.


### 보정162 — 작은 결재문서 #2243 네 개의 흐름과 기존 의미 회귀

- **사전 분석**: 기존 #2243 검사1개는 세운3쪽의 심사항목 기준선이 PDF보다 약16px 아래라 실패했습니다. Native/fresh WASM 전14쪽 대조에서 컨설팅3쪽63.60066%, 세운3쪽47.04651%와 표/후속 문단 이동을 직접 확인했습니다. 5/3/5/1쪽의 작은 문서이므로 현재 브랜치에서 해결하고 #7445로 이관하지 않았습니다. 원문과 기준 PDF4쌍의 해시는 보존했습니다.
- **원인과 수정**: 합성 줄의 소폭 전방 이동을 거부한 뒤 활성 vpos 원점이 남아 다음 저장 문단에서 같은 이동이 다시 적용됐습니다. `HeightCursor`의 기존 거부 조건에서 원점도 함께 갱신했습니다. 컨설팅의 저장 TAC 표 후행 간격은 측정 상한만 절반으로 소비해 뒤 computed 표가 약13px 위로 배치됐습니다. `composer::tac_host_trailing_spacing`을 측정 `tac_reconcile::measure`와 paint가 공유합니다. 현재 생성한 단일 RowBreak 소유 프레임의 기존 반간격 및 음수 Fixed 경로는 보존했습니다. `상한→current_height→computed placement→paint 원점`의 실제 호출 연결과 정상/비적용 경계는 아래 근거에 있습니다.
- **기존 회귀 개선**: 기존 #2243의 한 함수와 네 문서 쪽수는 유지했습니다. 절대 x/y·기준선 핀을 제거하고, PDF에서 확인한 문단/표 쪽 소속·앞뒤 순서·본문 내부 점유, 원문 행열·모든 셀의 누락/중복 및 선언 줄간격 보존을 검사합니다. 기존 compositor 단위 함수와 스윕 함수만 강화했고 새 테스트 함수는0개입니다. 같은 의미 검사는 보정 전 코드에서 선언 줄간격 손실로 **1FAIL**, 보정 후 **1PASS**입니다. 파생 묶음 미갱신으로0개가 선택된 실행은 재현 근거에서 제외하고 `--prepare` 후 다시 확인했습니다.
- **검증 결과**: 기존 회귀/정상 대조군 **7PASS/0FAIL**, HeightCursor와 소유 줄의 기존 단위 **56PASS/0FAIL**, 기존 스윕 **5PASS/0FAIL**입니다. fmt, 고정 base manifest/unit 정책, diff check도 통과했습니다. Native/fresh WASM 각14쪽 compare·standalone overlay·review 완료/exit0, 각14쪽 PNG SHA도 같습니다. 컨설팅3쪽은98.81065%, 세운3쪽은96.43610%입니다. 글꼴 예외 없이4문서 모두 gate passed입니다. 선 굵기·명암·글자형과 약1px 경계 차이는 남아 엄격 픽셀 완전 일치로 보고하지 않습니다.

| 문서 | 한컴/rhwp 쪽수 | 최종 Native/fresh WASM 최저 | 판정 |
| --- | --- | --- | --- |
| 36395325 컨설팅 | 5/5 | 95.62058% / 95.62058% | 충족 |
| 36382819 교통 | 3/3 | 91.61629% / 91.61629% | 충족 |
| 36386907 세운 | 5/5 | 96.43610% / 96.43610% | 충족 |
| 156631374 택시 | 1/1 | 94.56346% / 94.56346% | 충족 |

- **스윕 보정**: 전체 단일 쪽 export의 SVG 파일명 문서번호를 쪽 번호로 해석하는 오류를 발견했습니다. 실제 번호가 있는 render tree와 PDF가 같은 한 쪽일 때 그 번호를 사용합니다. 보정 전156631374/후1을 실행으로 확인하고 택시 Native1쪽을 다시 산출했습니다. 명시적7쪽 선택은 유지하는 기존 검사도 통과했습니다. 과거 잘못 표시된 산출물을 수동으로 고치거나 새 결과로 재사용하지 않았습니다.
- **직접 판독**: Native 컨설팅3/5·세운3/4·교통3 review와 네 문서 전14쪽 contact sheet, WASM 컨설팅3·세운3 standalone overlay에서 제목·표 외곽·뒤 문단·본문 하단을 확인했습니다. Mac root fresh WASM `--no-opt`와 Studio 복사본 SHA가 같으며 Docker 최적화/브라우저 실행 결과로 대체하지 않습니다.
- **기록**: 코드 `037a0ec508e71cae7127ca43d9b2b12d2a200b1b`, [입력/PDF·호출 경로·전후 검증·명령·PNG 해시 근거](../assets/pr7382_20260926/stage162_consult2243_validation.json). 전체 PNG는 `output/pr-review/planet6897-7382-20260926/stage162-consult2243/visual-candidate2-native/` 및 `visual-candidate2-wasm/`, 택시의 수정한 Native1쪽은 `visual-candidate2-taxi-native/`입니다. 로그는 output에만 두며 커밋하지 않습니다.
- 남은26중 **5처리/21대기**입니다. 원래60실패 기준39처리/21대기이며 현재 전체 검증 결과로 대신하지 않습니다. 최종 정확한 head의 필수 lint·정책·전체 nextest·Native Skia3·원 PR 시각 검증은 남아 있습니다. **통합 PR 준비 미완료**이며 다음은 저장 표의 문자 테두리 검사입니다.

![컨설팅3쪽 최종 Native review](../assets/pr7382_20260926/stage162_native_consult3_review.png)
![세운3쪽 최종 Native review](../assets/pr7382_20260926/stage162_native_sewoon3_review.png)
![컨설팅3쪽 최종 fresh WASM overlay](../assets/pr7382_20260926/stage162_wasm_consult3_overlay.png)
![세운3쪽 최종 fresh WASM overlay](../assets/pr7382_20260926/stage162_wasm_sewoon3_overlay.png)


### 보정163 — 작은 저장 표의 문자 테두리와 100% 글꼴 크기

- **사전 분석**: `hancom_saved_object_row_keeps_its_character_border`가 단독 실패했습니다. 대상은 정상 한컴 저장본 `samples/stored-table-text-tail/native-8-0.hwpx`와 기존 한컴 PDF `pdf/pr7242/native-8-0-2020.pdf`의 1쪽입니다. 이전 Native/fresh WASM46.21901%에서 표·셀·뒤 문장·테두리가 약8px 아래임을 직접 확인했습니다. 작은 문서이므로 현 브랜치에서 보정했으며 이관/원문 교체는 없습니다.
- **원인과 소비 경로**: `style_resolver::resolve_single_char_style`이 원래13.333333333333334px에100을 곱한 뒤 나누어13.333333333333336px를 만들었습니다. 측정 `composed_line_max_font_size → typeset/paragraph/format`은 원래 크기를 사용하지만 paint `ComposedRun::text_style → paragraph_layout → corrected_line_metrics_for_source → corrected_line_metrics`는 이 미세 증가를 실제 큰 글꼴로 보고 저장0간격을160%의8px로 바꾸었습니다. 상대 크기를 비율로 먼저 바꿔100%의 항등성을 보존했습니다. 좌표 clamp·문서ID 조건·메트릭 수용 공차는 추가하지 않았고, 진짜80%/125% 상대 크기는 기존 단위 함수에서 확인했습니다. 다른 상대 크기의 측정/paint 일반화까지 완료했다는 주장은 하지 않습니다.
- **기존 회귀 교정**: 절대 y/폭 핀을 제거하고 정상 저장 공백 줄의 점유/0간격, 표·뒤 공백의 문자 테두리가 Footer를 소유하지 않는 것을 검사합니다. 같은 변경 검사와 기존 글꼴 크기 단위 함수는 보정 전 각각1FAIL, 보정 후 PASS입니다. 입력/PDF 출처와 정상 재저장 계보는 샘플 README에 그대로 보존했습니다. 새 테스트 함수0개입니다.
- **결과**: 기존 테두리/정상 대조군10PASS + #2243 회귀1PASS + 기존 style resolver 단위26PASS = **37PASS/0FAIL**입니다. Native/fresh WASM1쪽 각96.86991%·gate passed·PNG SHA동일이며, Native review와 WASM standalone overlay에서 표·Cell1..8·Footer·문자/문단 테두리를 직접 확인했습니다. 선 명암/굵기와 약1px 경계 차이는 남습니다.
- 공통 글꼴 계산의 영향이 있는 #2243 네 문서 전체14쪽도 다시 산출했습니다. Native/fresh WASM 모두 gate passed·14쪽 PNG SHA동일이며 최저91.61509%입니다. 컨설팅3쪽 review와 교통/세운 contact sheet에서 표·뒤 문장·쪽 소속을 다시 확인했습니다. root fresh WASM `--no-opt` 및 Studio 복사본 SHA일치, fmt/고정base manifest/unit 정책/diff0입니다. 초기 fmt 뒤 파생 묶음 drift는 재준비 후 정책 통과로 해결했고 생성 파일은 커밋하지 않았습니다.
- 코드 `6fcb36f24`, [입력/PDF·전후 실패·명령·해시·시각 근거](../assets/pr7382_20260926/stage163_tail_border_validation.json). 전체 PNG/로그는 `output/pr-review/planet6897-7382-20260926/stage163-tail-border/`에 있습니다. 남은26중 **6처리/20대기**이며 최종 정확한 head의 lint3종·workspace·전체nextest·NativeSkia·원PR전체 시각 및 통합PR 준비는 미완료입니다. 다음은 같은 문서의 Footer 검사이며 별도로 판단합니다.

![보정163 최종 Native review](../assets/pr7382_20260926/stage163_native_review.png)
![보정163 최종 fresh WASM overlay](../assets/pr7382_20260926/stage163_wasm_overlay.png)


### 보정164 — 같은 1쪽 문서의 Footer 회귀 의미 교정

- 기존 Footer 단독 실패는 보정163의 공통100% 글꼴 계산으로 해소됐고 현재 원래 검사1PASS입니다. 정상 한컴1쪽/PDF와 Native/freshWASM96.86991% 자료의 입력·생산·runtime artifact 해시가 그대로이므로 그 시각 증거를 재사용합니다. 현재 브랜치에서 검사 의미를 교정하며 이관/원문 교체/생산 변경/새함수0입니다.
- 기존 절대 baseline/x/표 y핀을 제거하고 Footer의 문단/본문 소속·단일 출력·표 다음 순서·원문 object/footer LineSeg의 기준선 차, Cell1..8의 순서·단일 표시·기준선 점유를 검사합니다. 글꼴의 여유 상자 끝을 실제 줄 기준선과 혼동하지 않습니다. 표/뒤 문장이 함께 이동한 결함은 보정163 테두리/100% 단위가 검출하며, 본 함수는 독립적인 상대 흐름 계약입니다. **같은 최종 Footer 검사는 보정 전100% 생산 코드에서도1PASS**이므로 그 결함의 검출 증거로 주장하지 않습니다.
- 필드명 오기로 끝난 컴파일101과 여유bbox 초안의FAIL은 재현에서 제외하고 수정·재실행했습니다. 최종 기존 Footer/대조군 **11PASS/0FAIL**, fmt/고정base manifest/unit 정책/diff0입니다. 코드 `e230327f2`, [전후 판별·명령·재사용 해시·범위](../assets/pr7382_20260926/stage164_tail_footer_validation.json). 로그는 `output/pr-review/planet6897-7382-20260926/stage164-tail-footer/`에만 있습니다.
- 남은26중 **7처리/19대기**이며 최종 전체/통합PR 준비는 미완료입니다. 다음은 같은 문단을 여러 페이지 항목으로 나눴을 때 외곽 테두리 소유 검사입니다.


### 보정165 — 같은 문단의 텍스트/표/텍스트 테두리 소유

- 기존 외곽 절대 좌표 검사는 보정163 후 단독1PASS로 실패가 해소됐습니다. 작은 정상1쪽에서 기존 함수를 실제 본문 `partialParagraph → table → partialParagraph` 호출 경로·원문 테두리 연결 꺼짐·표와 Footer를 소유하는 단일 외곽 검사로 바꿨습니다. 새함수/생산/원문/PDF/이관 변경0입니다.
- 동일 계약을 보정 전/후의 실제 저장 렌더트리와 다시 실행한 페이지 항목에 적용하면 둘 다 단일 소유를 유지합니다. 원래 실패는 전체 위치 이동이며, 이 함수의 소속 계약을 그 글꼴 결함의 검출 증거로 주장하지 않습니다. 보정 전 새 Rust 함수 실행으로도 쓰지 않습니다. 최종 기존12PASS/0FAIL·fmt/고정base manifest/unit 정책/diff0입니다.
- 코드 `c5113cb5c`, [원인·실제 항목·명령·범위·해시](../assets/pr7382_20260926/stage165_tail_outline_validation.json). 생산/Native/WASM runtime artifact 해시가 동일하므로 보정163의 정상PDF/Native/freshWASM96.86991% 시각 근거를 재사용합니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage165-tail-outline/`입니다. 남은26중 **8처리/18대기**이며 최종 전체/통합PR 준비는 미완료입니다. 다음 본문 넘침 검사도 먼저 현재 단독 실패 여부부터 확인합니다.


## 보정166 — 작은 #7196 10쪽의 저장 경계·TAC 제목 줄·글꼴 공급

- **사전 분석**: 본문 넘침 분할11은 단독1FAIL이었습니다. 첫 NO_LS 호스트를 합성한 뒤 구역 전체 vpos를 재계산하면서, 정상 저장 pi18의 문단 내부 쪽 경계와 pi29의 명시 쪽나눔 원점700HU, pi41/72의 TAC 다음 프레임0을 지웠습니다. 한컴 PDF10쪽에 대해 rhwp11쪽이 되어 3쪽 연락처 표가 이월되고 5/9쪽 표가 본문 아래로 넘쳤습니다. 작은 문서이므로 현재 브랜치에서 보정하고 #7445로 이관하지 않았습니다. [원문·정상 PDF·원인·명령·결과·자료 해시](../assets/pr7382_20260926/stage166_body11_validation.json).
- **원인 계층과 소비 경로**: `DocumentCore::reflow_zero_height_paragraphs`에서 합성하지 않은 유효 저장 줄의 프레임을 보존하여 측정과 paint가 같은 변환 사다리를 소비하게 했습니다. 명시적 쪽나눔의 양수 원점, 자기 저장 줄을 소유한 TAC의 물리 fit 경계, 순수 텍스트 문단 내부의 물리 fit 경계를 구분합니다. 누적 축 생성본의 명시 PageBreak0까지 보존한 후보는 #1749의 호스트 소속을 깨뜨려 기각했습니다. 문서 ID·절대 좌표 clamp·새 공차는 추가하지 않았습니다.
- **빈 TAC 제목 줄**: `stored_first_tac_line`은 가시 글자 부재와 줄 점유 부재를 구분합니다. 실제 원문 제목 호스트의2630HU는 표2348+상하 바깥여백141+141HU이며, 첫 원점700HU의 위 간격을 보존해야 합니다. `layout_composed_paragraph → TextLine.bbox.y → 중첩 TAC table_anchor_y → 기존 바깥여백/기준선 분기`에서 같은 소유 줄의 실제 배치 원점을 소비하도록 수정했습니다. 정렬 높이에 같은 간격을 다시 더한 후보는 전체 셀 내용을 약4.67px 위로 밀어 기각했습니다. 기존 콘텐츠 높이/정렬·후속 저장 사다리 소비는 유지하며, 표 원점과 뒤 문단을 직접 대조했습니다.
- **실제 글꼴**: 9/10쪽의 한컴 윤고딕230에 Noto Sans KR ExtraLight가 공급되는 것을 실물 파일의 이름/해시로 확인했습니다. Windows의 정상 `HANYGO230.ttf`를 검증 경로에 공급하고, 공식 파일명 후보 및 기존 Haan YGodic230 메트릭의 한글 이름 규칙을 정식 변경 세트로 연결했습니다. 동일 파일의 이름·upem·Latin/한글 전진폭과 기존 독립 font oracle 기록이 근거입니다. 글꼴 바이너리는 커밋하지 않습니다. 새 메트릭 규칙1개 외 네 projection의 의미 해시는 그대로입니다. 기존 정책 함수에서 도입·교체 이력과 정확한 규칙ID를 검사하도록 교정했으며 봉인된 초기 migration은 보존합니다.
- **캡처 종료 보정**: 캡처와 임시 브라우저 정리는 끝났지만 Chrome 출력 PipeWrap이 남아 Node가 대기했습니다. 정상 `browser.close()` 뒤 그 실행이 소유한 stdio만 정리합니다. 중단한 WASM 실행은 통과에서 제외하고 Native/freshWASM 전체를 다시 산출했습니다. Native 수정 전후10개 PNG의 해시가 같아 종료 처리의 이미지 불변을 확인했습니다.
- **검증**: 본문 분할11 **1PASS**, 기존 정상/첨부 **22PASS**, 변경한 소유 줄·글꼴 단위 **3PASS**, 기존 글꼴 정책·이력·캡처 **55PASS**입니다. fmt·고정 base manifest/unit 정책·기본/WASM lib/workspace all-targets Clippy·workspace build 모두 통과했습니다. 신규 테스트 함수0, baseline/공차 완화0입니다. 별도 #1749 혼합 함수의 HWP 컷[3] 검사는 [2]로 **1FAIL**이며 보정 전후 HWP 전체 페이지 덤프가 같습니다. 이 검사를 삭제하거나 [2]로 완화하지 않았고, 최종 전체 전 현재 브랜치에서 다시 판단합니다. 위26PASS를 모든 대조군0FAIL로 보고하지 않습니다.
- **전체 시각**: Native/freshWASM 각10/10쪽 compare·standalone overlay·review 완료/exit0, 두 backend 최저 **91.53858%**, gate passed·글꼴 예외0입니다. 전10쪽 contact와 Native7 review/10 overlay, WASM4/10 overlay를 직접 판독했습니다. 큰 쪽 소속·누락·겹침은 해소됐고 일부 글자형·단어 폭 및 약1~4px 경계 차이는 남습니다. Native/WASM 점수·PNG가 완전히 같다고 주장하지 않습니다. 본문 아래 넘침은2→0, 쪽수11→10입니다. 기존 표 오른쪽1.8933px2건은 남아 전체 anomaly0으로 보고하지 않습니다.
- 코드 `74b70903b`. 남은26개 원장은 **9처리/17대기**, 이전 처리 항목의 #1749 재검토1건을 별도로 기록했습니다. 최종 전체 nextest·NativeSkia3·원PR최종 전체 시각·통합PR 준비는 미완료입니다. root fresh WASM/Studio 복사본 SHA는 같으며 Mac `--no-opt` 대체 빌드입니다. Docker 최적화/Studio 브라우저 검증으로 보고하지 않습니다. 로그와 전체 PNG: `output/pr-review/planet6897-7382-20260926/stage166-body11/visual-candidate5-final-native/trim7196/`, `visual-candidate5-final-wasm/trim7196/`. 승인된 이전 후보 SVG95개 약6.79GiB를 정리하고 원문/PDF/PNG/결과/최신 산출물을 보존했습니다.

![#7196 3쪽 보정 전 Native review](../assets/pr7382_20260926/stage166_before_native_review_003.png)
![#7196 3쪽 최종 Native review](../assets/pr7382_20260926/stage166_native_review_003.png)
![#7196 9쪽 최종 Native review](../assets/pr7382_20260926/stage166_native_review_009.png)
![#7196 10쪽 최종 Native review](../assets/pr7382_20260926/stage166_native_review_010.png)
![#7196 3쪽 fresh WASM overlay](../assets/pr7382_20260926/stage166_wasm_overlay_003.png)
![#7196 9쪽 fresh WASM overlay](../assets/pr7382_20260926/stage166_wasm_overlay_009.png)
![#7196 10쪽 fresh WASM overlay](../assets/pr7382_20260926/stage166_wasm_overlay_010.png)


## 보정167 — 기존 첨부 함수의 쪽 소속과 원문 줄 관계

- 원장25번의 기존 첨부 함수는 보정166 이전에 첨부 표가 첫 흐름 항목이 아니어서1FAIL, 생산 보정 후 정상 대조군과 함께2PASS였습니다. 작은 동일10쪽 문서이며 현재 브랜치에서 해소됐습니다. [사전 분석·정식 명령·소스/런타임 불변·검증](../assets/pr7382_20260926/stage167_attachment_validation.json).
- 기존 함수2개를 유지하면서 첨부 함수에 정상 한컴의10쪽/9쪽 첨부·8쪽 시작 `□ 아울러`와 끝 `감사합니다.` 소속을 검사합니다. 느슨한 상대 픽셀 상한은 원문XML pi72의 저장 프레임 원점0과 위 바깥여백141HU의 관계로 교정했습니다. 전10쪽 본문 아래 점유를 확인하고 기존 음수 Percent 정상 함수는 유지했습니다. 옛 헤더의10쪽/간격1920HU 설명을 정상8쪽/빈 밴드4420HU(1920+2500)의 실제 저장 근거로 수정했습니다.
- 강화한 기존 첨부/정상2PASS와 본문 분할11 1PASS, 합계 **3PASS/0FAIL**입니다. fmt·고정base manifest/unit 정책·workspace all-targets Clippy가 통과했습니다. 생산·원문/PDF·새 함수·이관·허용 공차 완화0입니다. 생산/입력/Native binary/JS/WASM의 해시가 같아 보정166의 Native/freshWASM 전10쪽 최저91.53858%·직접 판독 증거를 재사용합니다. 강화한 새 assertion을 이전 Rust 코드에서 실행한 것으로 보고하지 않습니다.
- 코드 `5d136a5bc`. 남은26개 원장은 **10처리/16대기**, 이전 처리 #1749 HWP 컷의 재검토1건은 별도입니다. 최종 전체 nextest·NativeSkia·원PR최종시각·통합PR 준비는 미완료입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage167-attachment7196/`에만 있습니다. 최종 캡처 확인 뒤 이전 SVG86개 약6.24GiB를 추가 정리하고 원문/PDF/PNG/JSON·최신 두 backend 산출물을 보존했습니다.


## 보정168 — 작은 #1749 HWP의 세 글줄 컷·종료 빈 공간·실제 글꼴

- **사전 분석**: 기존 HWP 컷[3]이 [2]로 실패한 1건을 재검토했습니다. 보정166 전후 전체 페이지 덤프가 같아 그 단계에서 생긴 회귀는 아닙니다. 실물 HWP와 같은 이름의 정상 한컴2024 PDF5쪽을 재사용했습니다. 4쪽의 세 번째 별표줄10개가 5쪽으로 밀리고, 마지막 표 뒤 문단도 너무 위에 있었습니다. 작은5쪽은 현재 브랜치에서 보정하며 원문·PDF를 교체하거나 #7445로 신규 이관하지 않았습니다.
- **컷의 공통 결과**: 셀의 원래 시작 원점500HU로 되돌아가는 저장 줄은 0으로 돌아가는 경우처럼 물리 프레임을 닫습니다. 같은 첫 원점으로 엄격하게 rewind한 실물 줄에만 종료 줄간격을 제외하고, 다른 양수 로컬 재시작650HU·개체 소유·합성 태그는 제외합니다. `native_multirow_saved_reset_trailing_trim → row_cut_content_height`의 공통 결과를 scanner와 paint가 사용하여 실제 마지막 줄79.2px를 수용합니다. 공차·절대 좌표·문서ID 조건은 추가하지 않았습니다.
- **분할 프레임의 물리 공간**: 분할 행 전에서 완결된 제목 rowspan은 해당 행의 첫 프레임 보존을 막지 않습니다. 원본 전체 행합23109HU−첫 개체8555HU=종료14554HU가 다음 빈 문단53의 저장 원점14554HU를 정확히 닫습니다. `stored_rowbreak_closing_frame_height → emit → end/next_start_row_height_override → 기존 continuation scan/paint`에서 같은 첫114.0667px/종료194.0533px 공간을 소비합니다. 원래 내용 컷[3]·아홉 별표줄·뒤 본문을 보존하며, 내용으로 표현되지 않는 종료 빈 밴드를 빠뜨리지 않습니다. 저장 정보·미편집·미재조판·무캡션/각주 등의 적용 경계를 제한하고 적용되지 않는 정상 사례를 재실행했습니다.
- **글꼴 보정**: 영문 HCR/Haansoft 이름에 실제 설치 파일을 대응시켰습니다. Chrome에서 구형 글꼴의 cmap 마지막 비문자 U+FFFF가 없는 글리프65535를 가리켜 OTS가 파일을 거부하던 원인도 확인했습니다. [OpenType cmap 사양](https://learn.microsoft.com/en-us/typography/opentype/spec/cmap)에 따라 브라우저 임베드 사본에서 잘못된 종료 매핑만 missingGlyph0으로 연결합니다. 원본 디스크·정상 문자 매핑·윤곽선·advance·글꼴 메트릭은 바꾸지 않습니다. 실제 세 글꼴의 정상 문자/윤곽/수평 메트릭 불변과 Chrome의 Haansoft Batang 실사용·오류0을 확인했습니다. 글꼴 바이너리는 커밋하지 않습니다.
- **기존 회귀 교정**: 새 테스트 함수0입니다. 기존 HWPX 정상 쪽수·순서·IR 검사는 유지하고, HWP는 PDF4쪽의 `[50,67,10]`과 5쪽의 `[67,22,67,16,67,4]` 글줄 소유·뒤 추진계획·원문 종료 공간을 검사합니다. 컷[3]을[2]로 완화하지 않았습니다. 원래 함수의 수정 전FAIL과 실제 전후 캡처를 연결하지만 강화한 새 Rust assertion을 이전 소스에서 실행했다고 주장하지 않습니다. HWPX의 역사적 첫 컷 보류도 이 HWP 통과로 해소하지 않습니다.
- **최종 검증**: 기존 회귀24PASS+변경 단위3PASS=**27PASS/0FAIL**입니다. fmt·고정base manifest/unit 정책·세 Clippy·workspace build가 통과했습니다. Native/freshWASM 대상 각5/5쪽은 최저 **90.62519%**(수정 전18.75798%), #2243 네 문서14쪽과 #7196 10쪽의 각 backend 대조군은 최저 **91.53858%**, 모두 complete/gate passed·글꼴 예외0입니다. 전쪽 contact와 대상 review/overlay를 직접 확인했습니다. 대상 최종 Native/WASM PNG5개가 같고 Native 후보4/5도 같습니다. Native 대조24쪽은 lint에서 `contains`로만 바꾼 이전 후보4, finalWASM24쪽과 finalNative5쪽은 최종 소스입니다. 두 자료의 정확한 binary/소스 해시를 분리했습니다.
- **한계와 기록**: 표지 굵기·기존 연결 그림 표식·일부 별표줄2~5px 차이는 남으며 완전 pixel match가 아닙니다. root freshWASM/Studio 사본 SHA는 같고 Mac `--no-opt` 대체 빌드입니다. Docker/Studio UI 검증으로 보고하지 않습니다. 코드 `aee3bd2b4`, [입력·전후·호출 경로·명령·해시·시각 근거](../assets/pr7382_20260926/stage168_savedbounds1749_validation.json). 전체 PNG/로그는 `output/pr-review/planet6897-7382-20260926/stage168-savedbounds1749/visual-candidate5-native/`, `visual-candidate5-wasm/`, `counter-current-native/`, `counter-final-wasm/`입니다. 승인된 이전 SVG25개 약1.39GiB만 정리했습니다.
- 원래26개 원장은 **10처리/16대기**, 별도 이전 처리 #1749 재검토는 해결입니다. 최종 전체 nextest·NativeSkia·원PR최종전체시각·최신base 통합PR 준비는 미완료이며 다음은 #1853 본문 넘침 분할5를 개별 검토합니다.

![#1749 5쪽 보정 전 Native review](../assets/pr7382_20260926/stage168_before_native_review_005.png)
![#1749 3쪽 최종 Native review](../assets/pr7382_20260926/stage168_native_review_003.png)
![#1749 4쪽 최종 Native review](../assets/pr7382_20260926/stage168_native_review_004.png)
![#1749 5쪽 최종 Native review](../assets/pr7382_20260926/stage168_native_review_005.png)
![#1749 4쪽 fresh WASM overlay](../assets/pr7382_20260926/stage168_wasm_overlay_004.png)
![#1749 5쪽 fresh WASM overlay](../assets/pr7382_20260926/stage168_wasm_overlay_005.png)


## 보정169 — #1853 52쪽 법률문서의 실제 본문 넘침 범위

- **사전 분석/현재 재현**: 본문 분할5를 현재 소스에서 단독 실행해0→2건으로1FAIL, 원래 캡션/52쪽 대조군2PASS를 확인했습니다.14쪽의 실제 `○ 유사 입법례(참고)` 문단이7.9467px,44쪽의 표가2.3333px 초과합니다. 독립 한컴2024 PDF52쪽과 원문을 그대로 재사용했습니다. 옛32% 캡처를 현재 결과로 대신하지 않았습니다.
- **간단한 보정 여부**: Native13/14/44쪽 review와 freshWASM14쪽 standalone overlay에서13쪽 표의 마지막 내용이14쪽으로 밀려 첫 이어받기 내용·표 높이·뒤 본문이 다름을 직접 확인했습니다. 원문pi105 2×2 CellBreak와 pi371 TAC캡션+3×2 CellBreak의 서로 다른 이어받기 경로입니다. 마지막 y clamp·공차 증가로 표 소유를 복원할 수 없으므로 전체52쪽 피델리티 후속으로 판정했습니다. 작은 문서를 점수만으로 일괄 이관한 것이 아닙니다.
- **좁은 이관/보존**: [#7445 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5894072744) 후 실제 차단인 본문 넘침의 이 입력1개와 대응 baseline0행만 이관했습니다. `issue_1853`의 캡션 같은 쪽 분할·52쪽 정상2함수와 원문/PDF·기존16본문함수·다른입력/축은 유지합니다. 생산/새테스트/기대값·허용치 완화0입니다. 두 backend 선택13/14/43/44/52쪽은62.38007/32.88144/88.03127/93.34014/96.82559%, 모두 complete지만 gate는 re_review_required입니다. 이관이 해당 문서의 피델리티 해결이나 시각 통과를 뜻하지 않습니다.
- **재분할 확인**: 입력 제외로 크기 기준 greedy분할이 바뀌므로 기존 본문16함수를 모두 재실행했습니다. **12PASS/4FAIL**, 정상 #1853 **2PASS**입니다. #1853은 실패 목록에서 사라졌으며 다른 실패는 traffic-safety-designated-routes1건·k-water2024 1건·virtual-convergence1→2건·animal-welfare1건입니다. 원래 분할7/10 이름을 현재 입력 소유와 혼동하지 않고 이후 각각 검토합니다. fmt·고정base 정책·세Clippy·workspace build는0입니다. 전체 회귀 통과로 보고하지 않습니다.
- 코드 `61ef48a08`, [정상 PDF·전후 실행·실제 노드·원문 경계·명령·시각](../assets/issue7445/caption1853_blocking_scope_validation.json). 전체 PNG/로그는 `output/pr-review/planet6897-7382-20260926/stage169-body5/`입니다. 남은26원장 **11처리/15대기**, 별도 #1749 재검토 해결이며 본문 전수의 네 실패도 후속 검토에 포함합니다. 최종 전체 nextest·NativeSkia·원PR최종시각·통합PR 준비는 미완료입니다.


## 보정170 — #2019 기존 본문 증가의 현재 해소 확인

- 원장12의 과거 분할7 실패는 #2019 HWPX2→3건 증가였습니다. 현재 같은 원문의 전18쪽 anomaly는 기존 baseline과 같은2건이고, 보정169의 전체본문16함수 재실행에서도 이 입력의 증가는 나타나지 않습니다. greedy분할번호와 실제 입력 소유를 구분했습니다.
- 현재 partition7과 원래 과분할 정상 함수를 각각 재실행해 **2PASS/0FAIL**입니다. 생산/검사행동/기준값/새함수/원문/PDF 변경과 #7445 이관0입니다. 기존 baseline2와 정상 검사를 유지하며 실패가 없는 입력을 점수 미검증만으로 제거하지 않았습니다. 전체18쪽 시각≥90와 #2019 전체피델리티 해결은 미검증입니다.
- [현재 단독 명령·실제 본문 노드·범위](../assets/pr7382_20260926/stage170_body7_validation.json). 로그는 `output/pr-review/planet6897-7382-20260926/stage170-body7/`입니다. 원장 **12처리/14대기**, 본문 전체의 별도 네 실패는 아직 남습니다. 최종 전체/PR준비는 미완료입니다. 다음은 #6145 폭 검사 한 개를 개별 분석합니다.


## 보정171 — #6145 저장 간격과 누름틀 인쇄 비교

- **사전 분석**: 원장13의 기존 #6145 표 셀 폭 검사는 단독 실패했습니다. 실제 원본 `samples/issue6145/worklife_balance_index_156607916.hwpx`와 독립 한컴2020 PDF는6쪽인데 수정 전 rhwp는7쪽이어서 최종 표가 이월됐습니다. 원본 표 폭9906HU와 좌우 안쪽여백 각283HU, 저장 줄·앞뒤 문단·그림 위치를 대조했습니다. 작은6쪽 문서이므로 현 브랜치에서 처리하고 #7445로 이관하지 않았습니다.
- **원인과 보정**: HWPX 저장 일반 글줄의 실제 쪽 상대0 원점이 누적 좌표로 덮였고, 필드·글자취급 그림 뒤의 원문 간격과 글자취급 표 앞500HU가 재조판에서 사라졌습니다. 반대로 일반 글줄에 이미 담긴 문단 위 간격과 그림 위 간격은 측정·배치에서 다시 소비됐습니다. `reflow_zero_height_paragraphs`의 저장 좌표·원본 앵커 → 쪽 경계·줄 측정 → `height_cursor`의 간격 소비 → 최종 표·그림 원점 경로를 맞췄습니다. 앞서 실제 저장0 원점이 없는 #1749 누적 좌표 대조군에는 리셋을 적용하지 않으며, 문서ID·좌표 clamp·새 공차를 사용하지 않았습니다.
- **기존 회귀**: 기존 #6145 폭 검사는 최종 표의6쪽 소속과 셀 폭의 원문 관계를 확인하도록 교정했고, 기존 왼쪽 검사도 함께 **2PASS**입니다. #1749 쪽 소속 검사1PASS, 기존 누름틀 인쇄 프로필 검사3PASS입니다. 새 검사 함수0, baseline·허용치 완화0입니다. 남은 #6145 왼쪽 검사의 절대 x 기대값은 원장14에서 별도로 검토합니다.
- **Visual Sweep 인쇄 프로필**: 한컴 PDF에 없는 빈 누름틀 안내문은 SVG 생성 시 Native와 fresh WASM 양쪽에서 인쇄 프로필로 제외합니다. 입력된 필드 본문은 계속 출력하고 쪽수·실루엣 비교에도 포함합니다. 비교 PNG를 마스킹하지 않습니다. `visual_sweep.py`·WASM exporter·CLI 조합 호출과 [Visual Sweep 가이드](../../manual/verification/visual_sweep_guide.md#pdf와-같은-인쇄-프로필), [시각 검증 거버넌스](../../manual/verification/visual_verification_governance.md), [fixture 증적 지침](../../manual/pr_review/visual_fixture_evidence.md), `CLAUDE.md`·`CONTRIBUTING.md`를 함께 맞췄습니다.
- **최종 시각**: 독립 PDF6쪽 대비 Native/fresh WASM 모두6쪽, 전6쪽 gate passed, 각각 `[96.8384, 99.83973, 95.5266, 95.67837, 97.80197, 98.89183]%`, 최저95.5266%, 글꼴 예외0입니다. 두 backend의 해당 PNG 해시는 전6쪽 동일합니다. 전체 contact sheet와 아래3·4·5쪽 review 및3·5쪽 overlay에서 본문·그림·표와 앞뒤 내용의 소속을 직접 대조했습니다. Mac root fresh WASM `--no-opt` 로컬 대체 빌드이며 Docker 최적화 검증으로 보고하지 않습니다.
- **최종 검증과 범위**: `cargo fmt`, 기본·WASM lib·workspace all-targets Clippy, workspace build, 고정 base `0e8fd49fb868da0d47ac1294dcbbda81f0211233` 대비 manifest/unit 정책 검사를 통과했습니다. 전체 `cargo nextest --tests --test-threads 8 --no-fail-fast`는 **10,229실행/10,186PASS/43FAIL/50SKIP**입니다. 직전 중간 후보의44FAIL에서 #1749 한 건이 해소됐고 새 실패 함수 이름은0건입니다. #7226/#6535 출력은 보정168 기준 바이너리와 최종 후보의 render tree가 동일해 이 단계에서 새로 만든 차이는 아닙니다. 다른43건은 개별 검토가 필요하며 전체 PR 승인·제출 판정은 보류합니다.
- [원본·PDF/실행 산출물 SHA·수정 전후·명령·쪽별 점수·잔여 실패](../assets/pr7382_20260926/stage171_width6145_validation.json). 전체 SVG/PNG·로그는 `output/pr-review/planet6897-7382-20260926/stage171-width6145/`에만 둡니다. 원장 **13처리/13대기**이며 다음은 #6145 왼쪽 검사를 개별 판단합니다.

![#6145 3쪽 최종 Native review](../assets/pr7382_20260926/stage171_native_review_003.png)
![#6145 4쪽 최종 Native review](../assets/pr7382_20260926/stage171_native_review_004.png)
![#6145 5쪽 최종 Native review](../assets/pr7382_20260926/stage171_native_review_005.png)
![#6145 3쪽 최종 fresh WASM overlay](../assets/pr7382_20260926/stage171_wasm_overlay_003.png)
![#6145 5쪽 최종 fresh WASM overlay](../assets/pr7382_20260926/stage171_wasm_overlay_005.png)


## 보정172 — #6145 왼쪽 여백 검사의 셀 상대 관계

- **사전 분석**: 원장14의 기존 왼쪽 여백 검사는 보정171 생산 코드에서1PASS이나, 6쪽 전체 TextRun을 고정 x 범위로 문자열 검색했습니다. 표가 함께 이동하거나 이웃 문장이 같은 범위에 들어오면 잘못된 노드를 검사할 수 있습니다. 정상 한컴2020 PDF6쪽과 원문 `cellSz=9906HU`, 좌우 안쪽여백 각283HU, 저장 줄 폭9340HU는 보정171의 독립 근거 그대로입니다.
- **교정**: 기존 함수 하나를 유지하고 렌더 트리의 원문 표 문단 pi56 → 21행 마지막 셀 → `담당조직 형태 만점` 글줄 순서로 대상 소유를 확인합니다. 글줄의 실제 run 좌단과 **그 셀**의 좌단 차이가 선언 여백을 지키는지 검사하므로 페이지 절대 x에 의존하지 않습니다. 기존 폭 검사도 같은 표·셀 탐색 함수를 공유하며 여백·폭 기대값은 완화하지 않았습니다. 생산 코드·원문·PDF·새 테스트 함수 변경0입니다.
- **검증**: 기존 #6145 두 함수 **2PASS/0FAIL**입니다. fmt, Native/WASM lib/workspace all-targets Clippy, workspace build, 고정 base manifest/unit 정책 통과입니다. 마지막 검사 소스 수정 뒤 파생 suite drift가 발견돼 `--prepare`를 다시 실행하고 정책 검사를 통과시켰습니다. 보정171에서 확인한 동일 생산 출력의 Native/fresh WASM 전6쪽 최저95.5266% 증거를 재사용합니다. 보정171의 전체 nextest43FAIL 뒤 이 단계에서 전체 재실행은 하지 않았으며, 최종 code head의 전체 검증은 남았습니다.
- [원본·검사 변경·명령·결과](../assets/pr7382_20260926/stage172_left6145_validation.json). 로그는 `output/pr-review/planet6897-7382-20260926/stage172-left6145/`에만 보관합니다. 원장 **14처리/12대기**, 통합 PR 준비는 미완료입니다.


## 보정173 — #6032 이월 표 뒤의 저장 프레임과 호스트 간격

- **사전 분석**: 원장15의 기존 #6032 검사는 한컴 쪽수2를 기대했으나 현재 rhwp는3쪽이며, 작성요령 표와 뒤의 `용지규격` 문장이3쪽으로 밀렸습니다. 원문 `samples/issue6032/2912695_civil_petition_form.hwp`를 한컴2020 엔진으로 다시 변환해 독립 PDF2쪽을 확보했습니다. 수정 전 Native 2쪽 Visual Sweep은 **89.78092%**로 gate가 `re_review_required`였습니다. 작은2쪽 문서이므로 현 브랜치에서 해결했습니다.
- **원인과 보정**: 앞의 큰 표는2쪽으로 이월돼 몸체 상대 y108.5~875.3에 그려지며, 후행 빈 호스트 줄간격이 흐름 커서를781.507px까지 늘렸습니다. 다음 표의 저장 프레임 시작780.4px은 커서보다 앞이지만 실제로 그려진 앞 표의 끝 뒤에 있습니다. `original_control_frame` → 예약 표 흐름 판정 → 쪽 이월 → 실제 표 배치 경로에서 커서 역전만으로 물리 겹침이라고 판정하던 조건을 고쳤습니다. HWP5 미편집 저장 줄, 빈 단일 표 호스트, 이월된 앞 표의 선언 높이·윗여백·후행 줄간격과 실제 커서가 일치하는 경우에만 빈 간격을 사용합니다. 실제 겹침·편집 재조판·다른 표 경로는 기존 판단을 유지합니다. 문서ID 분기나 좌표 clamp는 없습니다.
- **기존 회귀**: 기존 함수 하나를 유지하되 SVG 가로선의 절대 y 대신 렌더 트리에서 두 원문 표의 소속과 뒤 문장을 찾고 `큰 표 하단 ≤ 작성요령 표 상단 ≤ 뒤 문장 상단`의 물리 관계를 검사했습니다. 기존 함수 **1PASS/0FAIL**, 새 함수0, baseline·허용치 완화0입니다. 수정 전3쪽에서 수정 후2쪽이며 작성요령 표와 뒤 문장 모두2쪽에 있습니다.
- **시각·검증**: 독립 PDF2쪽 대비 Native/fresh WASM 모두2쪽, 전체 gate `passed`, 1·2쪽 각각 **98.39585%·99.3826%**입니다. 두 backend의 PNG 해시가 두 쪽 모두 같고, 2쪽 review·overlay에서 표 외곽·후속 문장 소속을 직접 확인했습니다. Mac root fresh WASM `--no-opt` 로컬 대체 빌드입니다. fmt, Native/WASM lib/workspace all-targets Clippy, workspace build, 고정 base `0e8fd49fb868da0d47ac1294dcbbda81f0211233` 대비 manifest/unit 정책 검사를 통과했습니다. 최종 전체 nextest는 **10,229실행/10,187PASS/42FAIL/50SKIP**로 직전43FAIL에서 이 함수만 해소되고 새 실패 함수는0입니다.
- [원문·정상 PDF·전후 페이지와 점수·명령·SHA·전체 실패 대조](../assets/pr7382_20260926/stage173_anchor6032_validation.json). PDF는 [`2912695_civil_petition_form-2020.pdf`](../../../pdf/2912695_civil_petition_form-2020.pdf)로 증적 커밋에 포함합니다. 전체 SVG/PNG·로그는 `output/pr-review/planet6897-7382-20260926/stage173-anchor6032/`에 둡니다. 원장 **15처리/11대기**이며 다음은 원장16을 개별 분석합니다. 전체 PR 준비는 미완료입니다.

![#6032 수정 전 2쪽 Native review](../assets/pr7382_20260926/stage173_before_native_review_002.png)
![#6032 수정 후 1쪽 Native review](../assets/pr7382_20260926/stage173_native_review_001.png)
![#6032 수정 후 2쪽 Native review](../assets/pr7382_20260926/stage173_native_review_002.png)
![#6032 수정 후 2쪽 fresh WASM overlay](../assets/pr7382_20260926/stage173_wasm_overlay_002.png)


## 보정174 — #1133 검사에 남은 #86712 변형 입력 단정 분리

- **사전 분석**: 원장16의 `hwpx_captionless_rowbreak_keeps_independent_frame_origins` 실패는 #1133 원문 자체가 아니라 함수 끝에 붙인 64쪽 `86712_regulatory_analysis.hwp`의 제어된 가로 기준 변형에서 발생했습니다. 종료 조각의 위여백 검사는 통과하지만 뒤 표 y543.733px과 과거 PDF y541.488px의 차이2.245px로 실패합니다. 동일 64쪽 원문의 한컴 PDF와 실제 글꼴을 공급한 재산출 PDF 모두 앞서 직접 비교했고, 선택 페이지 최저80.03785%·28.67074%에 표/후속 내용 쪽 소속 차이가 있습니다. 이 입력의 본래 차단 함수는 [#7445에 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5889443550)하고 보정161에서 제거했으나, #1133 HWPX 검사 안의 변형 입력 단정은 남아 있었습니다.
- **검사 범위 정리**: 기존 HWPX 함수에서 64쪽 변형 입력 호출과 그 전용 helper만 제거했습니다. #1133 원본 HWP/HWPX의 표 조각 소유, start/end cut, 인쇄 괘선, 뒤 빈 줄의 실제 소속 검사는 그대로 유지합니다. 원본 문서·PDF와 기존 두 함수는 보존하며, 생산 코드·새 함수·baseline·공차 변경은 없습니다. 이는 #86712의 출력 해결이나 기준 좌표 완화를 뜻하지 않습니다.
- **현재 독립 검증**: 현재 생산 코드로 #1133 HWP/HWPX의 기존 두 함수 **2PASS/0FAIL**입니다. Native/fresh WASM에서 각 원본 전3쪽, 합계12쪽을 새로 비교했고 HWP 점수는 `[96.24979, 99.272, 100]%`, HWPX 점수는 `[96.1041, 99.25401, 100]%`로 네 sweep 모두 gate `passed`입니다. 같은 페이지의 Native/WASM PNG 해시는 모두 일치하며 대표2·3쪽 review와 overlay에서 분할 표·후속 문장·쪽번호를 직접 대조했습니다. Mac fresh WASM은 `--no-opt` 로컬 대체 빌드입니다.
- **정책·잔여 범위**: fmt, 세 Clippy, workspace build와 고정 base `0e8fd49fb868da0d47ac1294dcbbda81f0211233` 대비 manifest/unit 정책 검사를 통과했습니다. 보정173 전체 nextest **42FAIL** 뒤 이 검사 전용 변경에 대해서는 전수 재실행하지 않았습니다. [검사 범위·독립 PDF·명령·현재 시각·보존 사항](../assets/pr7382_20260926/stage174_captionless1133_validation.json), 전체 로그/PNG `output/pr-review/planet6897-7382-20260926/stage174-captionless1133/`. 원장 **16처리/10대기**, 통합 PR 준비는 미완료입니다.

![#1133 HWP 2쪽 Native review](../assets/pr7382_20260926/stage174_hwp_native_review_002.png)
![#1133 HWPX 3쪽 Native review](../assets/pr7382_20260926/stage174_hwpx_native_review_003.png)
![#1133 HWPX 2쪽 fresh WASM overlay](../assets/pr7382_20260926/stage174_hwpx_wasm_overlay_002.png)


## 보정175 — #6778 재출력 PDF와 본문 넘침 입력의 개별 이관

- **사전 분석**: 원장17의 본문 넘침 분할10은 `issue6778/156757920-animal-welfare-husbandry-guidelines.hwp` 2쪽 문단35 글줄이 본문 하단을23.76px 넘는 신규1건으로 실패했습니다. 원문은 한컴오피스2024 저장본·12쪽이며 `printMethod=4`, `printMethodImpliesNup=true`입니다. rhwp는 이 특수 인쇄 방식을 출력에 반영하지 않습니다. 기존 한컴 PDF도12쪽이지만 Native/fresh WASM 전12쪽에서 최저1.62198%/1.58065%, 모든 쪽90% 미만이었습니다. 1·2쪽 PNG에서 본문 배율·문단 위치가 크게 달라 단순 폰트 예외로 처리하지 않았습니다.
- **PDF 재출력**: 사용자 지시에 따라 같은 원문을 `hwp2024-mcp-convert` `engine=2024`로 다시 변환해 [재출력 PDF](../../../pdf/issue7382-regression-review/156757920-animal-welfare-husbandry-guidelines-2024.pdf)를 별도로 보존했습니다. 새 PDF SHA-256은 `d2373f7d40c0b54953b4a55e6cce6dc488171452744934c21b3584fc171f07c3`, A4·12쪽·877608바이트입니다. 기존 PDF와 새 PDF의 2쪽 텍스트 크기·좌표 및96dpi 래스터가 동일합니다. 재출력 후 전12쪽 Native 최저17.50287%, fresh WASM 최저17.50287%이며 모두90% 미만입니다. 1·2·4쪽 review와2쪽 overlay를 직접 확인했고, PDF와 rhwp의 본문·그림 크기·위치 차이는 남습니다. 재출력 자체를 피델리티 개선으로 보고하지 않습니다.
- **검사 범위와 후속**: [#7445에 원문·재출력·전쪽 결과·복원 조건을 등록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5904847678)했습니다. 현재 브랜치에서는 이 원문 한 개만 본문 넘침 래칫 자동 수집에서 보류했습니다. 원문·기존 PDF·새 PDF와 다른 회귀/입력은 유지하며 새 검사·baseline·공차 완화·생산 코드 변경0입니다. 입력 제외로 분할 배정이 바뀌므로 본문 넘침16함수를 전부 재실행해 **14PASS/2FAIL**입니다. 남은 두 함수는 다른 원문의 기존 증가(#6776, #6756, k-water)이고 #6778 신규1건은 사라졌습니다. fmt, 세 Clippy, workspace build, 고정 base manifest/unit 정책 검사 통과입니다. 보정173 전체42FAIL 뒤 보정174·175를 포함한 최종 전체 nextest는 남았습니다.
- [원문·두 PDF 해시·재출력 job·12쪽 점수·명령·다른 실패 범위](../assets/pr7382_20260926/stage175_animal6778_validation.json). 로그와 전체 PNG는 `output/pr-review/planet6897-7382-20260926/stage175-body10-animal/`에만 둡니다. [Visual Sweep 가이드](../../manual/verification/visual_sweep_guide.md#pdf와-같은-인쇄-프로필)에 특수 인쇄 방식 대조 절차를 보완했습니다. 원장 **17처리/9대기**, 통합 PR 준비는 미완료입니다.

![#6778 재출력 PDF 1쪽 Native review](../assets/pr7382_20260926/stage175_animal_native_review_001.png)
![#6778 재출력 PDF 2쪽 Native review](../assets/pr7382_20260926/stage175_animal_native_review_002.png)
![#6778 재출력 PDF 4쪽 Native review](../assets/pr7382_20260926/stage175_animal_native_review_004.png)
![#6778 재출력 PDF 2쪽 fresh WASM overlay](../assets/pr7382_20260926/stage175_animal_wasm_overlay_002.png)


## 보정176 — 저장 중첩 셀 앵커 검사의 현재 통과 확인

- 원장18의 `nested_cell_alignment_uses_the_sequential_anchor_when_stored_positions_reset`은 보정173 전체 nextest에서 이미 PASS였습니다. 보정175 이후 최신 head에서 기존 함수를 정확히 하나만 다시 실행해 **1PASS/0FAIL**을 확인했습니다. 원래 실패 목록의 차단 함수가 아니므로 테스트·생산 코드·기준값·공차를 수정하거나 #7445로 옮기지 않았습니다.
- 이 검사는 합성 입력의 순차 앵커 계약이며, 통과만으로 별도의 한컴 PDF 전페이지 시각 일치를 주장하지 않습니다. [현재 함수·실행 명령과 결과](../assets/pr7382_20260926/stage176_nested_alignment_validation.json). 로그는 `output/pr-review/planet6897-7382-20260926/stage176-nested-alignment/`에만 있습니다. 원장 **18처리/8대기**, 전체 검증과 PR 준비는 미완료입니다.


## 보정177 — #157 SVG 문자열 스냅샷의 의미 검사 교정

- **사전 분석**: 원장19의 `svg_snapshot::issue_157_page_1`은 golden과 실제 SVG의 바이트가 달라 실패했습니다. XML 노드523개와 태그·문자·속성 키는 모두 같으며 숫자 속성61개의 최대 차이는 `5.684341886080802e-14px`입니다. 실제 조판 차이로 판정할 수 없는 부동소수 직렬화입니다. 원본 `samples/hwpx/issue_157.hwpx`와 기존 독립 한컴 PDF는 모두2쪽입니다.
- **검사 교정**: 기존 함수 하나를 유지하고 2쪽의 원문 표 문단 pi7·pi25와 각각의 앞뒤 문장이 실제 렌더 트리에 있으며 표가 그 문장을 덮지 않는지를 검사합니다. 페이지 절대 픽셀 위치와 SVG 문자열은 사용하지 않습니다. 오래된 golden SVG는 갱신하지 않았고 원문·PDF·생산 코드·새 검사 함수·baseline·공차 변경0입니다.
- **검증**: Native/fresh WASM 전2쪽 Visual Sweep 점수는 둘 다 `[98.98123, 93.09137]%`, gate `passed`입니다. 2쪽 review·overlay에서 표·문단 소속을 직접 확인했습니다. 교정한 기존 함수 **1PASS**, SVG 검사 전체7함수는 **6PASS/1FAIL**이며 남은 실패는 별도 원장23의 #617입니다. fmt, 세 Clippy, workspace build, 고정 base manifest/unit 정책 검사 통과입니다. [전후 SVG 구조·시각·명령·검사 범위](../assets/pr7382_20260926/stage177_svg157_validation.json). 로그와 전체 PNG는 `output/pr-review/planet6897-7382-20260926/stage177-svg157/`에 둡니다. 원장 **19처리/7대기**, 전체 nextest와 PR 준비는 남았습니다.

![#157 1쪽 Native review](../assets/pr7382_20260926/stage177_issue157_native_review_001.png)
![#157 2쪽 Native review](../assets/pr7382_20260926/stage177_issue157_native_review_002.png)
![#157 2쪽 fresh WASM overlay](../assets/pr7382_20260926/stage177_issue157_wasm_overlay_002.png)


## 보정178 — 현재 통과 중인 SVG 검사 세 건의 차단 여부 확인

- 원장20~22의 `form_002_page_0`, `issue_267_ktx_toc_page`, `issue_147_aift_page3`은 보정177의 현재 SVG 검사 묶음에서 각각 PASS였습니다. 세 함수는 최신 전체 실패42건의 차단 항목이 아니므로 원본·검사·golden·생산 코드를 수정하거나 #7445로 이관하지 않았습니다.
- 이 확인은 스냅샷 검사의 현재 통과 여부만 뜻하며 세 문서의 전페이지 한컴 PDF 시각 일치를 주장하지 않습니다. [실행 결과](../assets/pr7382_20260926/stage178_svg_existing_pass_validation.json)는 보정177의 검사 로그를 가리킵니다. 원장 **22처리/4대기**, 전체 검증과 통합 PR 준비는 미완료입니다. 다음 실제 실패는 원장23의 `issue_617_exam_kor_page5`입니다.


## 보정179 — #617 시험 문서 PDF 재출력 및 DPI 원인 대조

- `samples/exam_kor.hwp`는 한컴오피스2022 저장본·20쪽 A3입니다. 사용자 요청에 따라 MCP `engine=2020`으로 동일 원문을 재출력하여 [새 PDF](../../../pdf/issue7382-regression-review/exam_kor-2020-reprint.pdf)를 커밋 대상으로 보존했습니다. job `47943141-efc4-40d7-ae6f-4e62e20474d8`은 성공했고 서버/수신 SHA가 일치했습니다. 기존 `pdf/exam_kor-hwp-2020.pdf`와 새 PDF는 해시만 다르며 96dpi 전20쪽 래스터의 크기와 모든 픽셀이 동일합니다. PDF 차이는 생성 시각 메타데이터입니다. 재출력으로 조판 차이가 해결되었다고 보고하지 않습니다. 앞서 재출력한 #6778 PDF도 이미 `pdf/issue7382-regression-review/`에 커밋되어 있습니다.
- 기존 #617 SVG 스냅샷 실패는 XML 노드1905개에서 구조·문자 차이0, 수치 직렬화50속성의 최대 차이 `1.14×10⁻¹³px`입니다. Native/fresh WASM 전20쪽 비교에서 6쪽은 두 backend 모두94.8256%, 17쪽은84.84018%로 전체 gate가 `re_review_required`입니다. 17쪽 review에서 제목과 본문 글자폭·위치 차이를 직접 확인했습니다. 기존 2022 PDF의17쪽도84.84174%, 실제 글꼴을 공급한 재검사는85.09605%이므로 PDF DPI나 단순 글꼴 공급만으로 해결되지 않습니다. 글꼴 예외는 사용하지 않았습니다.
- 사용자가 20쪽 문서는 현 브랜치에서 해결하도록 정했으므로 #7445 이관과 #617 검사 제거를 취소했습니다. 이 단계는 PDF 재출력·원인 진단만 기록하며 원장23은 아직 대기입니다. [명령·원본/재출력 SHA·전쪽 결과](../assets/pr7382_20260926/stage179_exam617_pdf_reprint_validation.json), 로그와 PNG는 `output/pr-review/planet6897-7382-20260926/stage179-exam617/`에 있습니다. 생산 코드·검사·golden 변경은 없고, 전체 검증과 통합 PR 준비는 미완료입니다.

## 보정180 — #617 시험지 조판과 고정 픽셀 회귀 검사 수정

- 기준은 `samples/exam_kor.hwp`와 보정179에서 재출력한 20쪽 PDF입니다. 17쪽 `홀수형` 상자의 저장 줄 vpos 708HU는 높이 4389HU인 상자 안의 위치인데, 기존 쪽 오프셋 680HU를 빼서 글자를 위로 붙였습니다. 상자 높이보다 작은 오프셋은 쪽 좌표로 재기저화하지 않게 했습니다. `신명 신그래픽` 제목은 대체 글꼴의 잘못된 괄호 폭·자간으로 오른쪽으로 밀렸습니다. 한컴 PDF의 제목 전진폭을 기준으로 측정 글꼴·자간을 고쳤고, 같은 PDF의 동일 글자 79개에서 확인한 `HY신명조` 표시 기준선 약 1.5px 차이를 0.1em 보정했습니다. 줄 상자와 전진폭은 유지합니다.
- 기존 #617 SVG 바이트 스냅샷 실패는 XML 구조·글자 차이 없이 부동소수 직렬화 오차 최대 `1.14×10⁻¹³px`뿐이었습니다. 검사를 6쪽 보기 문단의 셀 내부 여백, 17쪽 `홀수형`의 상자 내부 중앙 배치로 바꿨습니다. 고정 px 좌표 대신 셀·상자의 상대 비율을 검사합니다. 다른 실패였던 #6646은 `exam_eng.hwp` 1쪽 7번 문항만 검증 대상으로 하여 `15~17px` 글자 원점 핀을 제거하고, 원문 공백 제어문자가 같은 글줄에 있으며 묶음 빈칸이 일반 공백의 측정 갈래와 양의 전진폭을 쓰는지 검사합니다. 해당 1쪽은 Visual Sweep 96.82579%이고, 이 검사는 나머지 7쪽의 시각 일치를 주장하지 않습니다.
- Native와 fresh WASM의 `exam_kor` 전20쪽 Visual Sweep은 모두 gate `passed`, 최저 17쪽 **90.66068%**이며 6쪽은 98.30348%입니다. 글꼴 예외를 쓰지 않았고 6·17쪽의 표·제목·문단을 review/overlay로 직접 확인했습니다. 두 기존 집중 검사 2PASS, 전체 nextest **10,192PASS / 37FAIL / 50SKIP**로 이전 38FAIL에서 #6646 한 건을 해소했습니다. 남은 37건은 별도 원장 순서로 검토하며 전체 회귀 통과나 PR 준비 완료로 보고하지 않습니다. fmt, Native/WASM/전체 target Clippy, workspace build, 고정 base manifest·unit 정책 검사는 통과했습니다. Mac의 fresh WASM은 로컬 대체 빌드로 기록합니다. [해시·전20쪽 점수·명령·제한](../assets/pr7382_20260926/stage180_exam617_validation.json), 전체 로그·PNG는 `output/pr-review/planet6897-7382-20260926/stage179-exam617/`에 있습니다.

![#617 6쪽 Native 기준 PDF 비교](../assets/pr7382_20260926/stage180_exam617_native_review_006.png)

![#617 17쪽 Native 기준 PDF 비교](../assets/pr7382_20260926/stage180_exam617_native_review_017.png)

![#617 17쪽 fresh WASM overlay](../assets/pr7382_20260926/stage180_exam617_wasm_overlay_017.png)

## 보정181 — #1772 본문 시작 회귀 검사의 절대 좌표 제거

- 기존 `issue_1772_body_first_line_respects_table_outer_margin_bottom` 검사는 본문 첫 줄을 `306.7±1px`에 고정하여 현재 `308.60px`에서 실패했습니다. 원본 `samples/task1772/table_outer_margin_common_sync.hwpx`와 한컴 PDF의 단일 1쪽을 Native Visual Sweep으로 비교한 결과 내용 실루엣 **100.00%**, gate `passed`였습니다. review에서 결재 헤더 표·`1. 관련` 문단·하단 표의 상대 배치를 직접 확인했습니다.
- 검사는 원본 표의 `outer_margin_bottom=852HU`를 읽어 상단 표의 실제 하단과 `관련: 총무과`가 포함된 첫 본문 글줄의 간격을 비교하도록 고쳤습니다. 관측 간격은 11.4px, 원본 여백은 96dpi에서 11.36px로 비율 1.0035입니다. 절대 쪽 x/y 좌표와 기존 `306.7px` 핀은 제거했습니다. 해당 검사와 원본 여백의 IR 동기화 검사는 **2PASS**, fmt, Native/WASM/전체 target Clippy, workspace build, 고정 base manifest 검사가 통과했습니다. 이 단계의 전체 nextest는 실행하지 않았고, 남은 다른 실패 및 PR 준비 완료로 보지 않습니다. [입력·PDF 해시와 검증 결과](../assets/pr7382_20260926/stage181_task1772_validation.json), 로그·전체 PNG는 `output/pr-review/planet6897-7382-20260926/stage181-task1772*`에 있습니다.

![#1772 1쪽 Native 기준 PDF 비교](../assets/pr7382_20260926/stage181_task1772_native_review_001.png)

## 보정182 — #6078 HWP3 서식의 미달 회귀 검사만 이관

- 기존 `issue_6078_paper_spec_line_stays_inside_the_page`는 용지 규격 줄의 y를 `994.5±1px`에 고정하여 현재 981.3px에서 실패합니다. 원본 `samples/hwp3-table-caption.hwp`와 보존된 한컴 2020 기준 PDF의 전1쪽 Native Visual Sweep은 내용 실루엣 **30.47054%**, gate `re_review_required`입니다. review에서 표 전체의 가로 원점·셀 경계·본문 위치 차이를 직접 확인했으므로 한 줄의 기대 좌표만 갱신하지 않습니다. 글꼴 예외는 없습니다.
- 사용자 지정 90% 기준에 따라 PR #7382를 막는 해당 검사 한 함수만 제거하고 [#7445 후속 항목](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5908796733)에 전체 피델리티 개선·회귀 재구축을 기록했습니다. 원본 HWP, 한컴 PDF, 생산 코드, 같은 원본의 유효한 `issue_5563_hwpx_lineseg_axis::hwp3_sectiondef_fallback_extends_the_comparable_lineseg_axis`는 그대로 유지하며 후자는 1PASS입니다. fmt, Native/WASM/전체 target Clippy, workspace build, 고정 base manifest 검사 통과 및 파생 suite에서 제거 함수 부재를 확인했습니다. 이 단계의 전체 nextest는 실행하지 않았고, 결함 해결이나 PR 준비 완료를 주장하지 않습니다. [원본·PDF 해시와 검증 범위](../assets/pr7382_20260926/stage182_issue6078_validation.json), 로그·전체 PNG는 `output/pr-review/planet6897-7382-20260926/stage182-issue6078*`에 있습니다.

![#6078 1쪽 Native 기준 PDF 비교](../assets/pr7382_20260926/stage182_issue6078_native_review_001.png)

## 보정184 — #6078 한 쪽 HWP3 표·캡션·체크박스 피델리티 복원

- **보정182 판정 변경**: 사용자의 10쪽 미만 문서 방침에 따라 #6078(1쪽)을 #7445 이관 대상에서 되돌리고 현 브랜치에서 해결했습니다. 원본 HWP와 한컴 2020 PDF는 그대로 사용했습니다. 보정182의 30.47054%는 수정 전 실패 증거로 유지합니다.
- **원인과 수정**: HWP3 표 정보의 캡션 세로 크기(원본 463단위)를 파서가 버려 표가 캡션과 겹쳤습니다. 위·아래 캡션에서 남는 물리 높이와 저장 줄간격을 캡션 간격으로 전달해 측정과 배치가 함께 사용합니다. 좌·우 캡션의 가로 간격은 변경하지 않습니다. 표는 두 번째 저장 글줄에 있는데 앞 글줄의 긴 공백까지 폭에 합산하여 가운데 정렬을 잃었습니다. 표 소유 글줄의 내어쓰기·정렬·선언 폭으로 원점을 계산합니다. 추천기관의 HWP3 원시 문자 `0x2F00` 세 개는 미지원 처리로 사라졌고, 한컴 PDF에 추출되는 `□`로 복원했습니다.
- **검사와 결과**: 고정 `994.5±1px` 회귀 핀을 제거하고 기존 #6078 검사를 복구했습니다. 한 쪽에서 캡션·표·후행 용지 규격 문단의 순서와 추천기관 체크박스 세 개를 확인합니다. 새 검사와 문자 매핑 검사 **2PASS**, 같은 HWP3 원문의 #5563 및 별도 HWP3 캡션 표 #6874 **2PASS**입니다. Native/fresh WASM 한 쪽 Visual Sweep은 각각 **91.47674%**, gate `passed`이며 출력 PNG SHA-256이 일치합니다. PDF와의 표 외곽·셀 경계·본문·체크박스 위치를 review/overlay에서 직접 확인했습니다. 글꼴 예외는 쓰지 않았습니다. Mac WASM은 `--no-opt` 로컬 대체 빌드입니다.
- **제출 게이트**: fmt, Native/WASM/전체 target Clippy, workspace build, 고정 base `0e8fd49fb868da0d47ac1294dcbbda81f0211233` 대비 manifest/unit 정책 검사 통과입니다. 이 개별 단계에서 전체 nextest는 다시 실행하지 않았으며 다른 #7382 차단 함수 해결 및 최종 전체 검증은 남아 있습니다. [입력·기준 PDF 해시, 원인 경로, 명령·검증 증적](../assets/pr7382_20260926/stage184_issue6078_validation.json). 로그와 전체 PNG는 `output/pr-review/planet6897-7382-20260926/stage184-issue6078/`에만 있습니다.

![#6078 체크박스 복구 후 한 쪽 Native 기준 PDF 비교](../assets/pr7382_20260926/stage184_issue6078_native_review_001.png)

![#6078 한 쪽 Native overlay](../assets/pr7382_20260926/stage184_issue6078_native_overlay_001.png)

![#6078 한 쪽 fresh WASM overlay](../assets/pr7382_20260926/stage184_issue6078_wasm_overlay_001.png)

## 보정185 — #3308 저장 행 소유·연속 하이픈 1차 보정

- **입력과 사전 분석**: 9쪽 원본 [`issue3307_outline_number.hwpx`](../../../samples/task3307/issue3307_outline_number.hwpx)의 SHA-256은 `28a60bd05e152fd5292bdc8b75c6dbd980a04375c0d8890b752e32017fd1731d`입니다. 기존 `pdf/task3307/beopryeong_3307-2020.pdf`는 Cairo 출력이어서 같은 원문을 승인된 MCP `engine=2020`으로 다시 출력했습니다(job `a6b3ca58-08f3-4301-b1d9-88041be1c303`). 새 [한컴 PDF](../../../pdf/task3307/issue3307_outline_number-2020.pdf)는 9쪽 A4·237518바이트, SHA-256 `0b176d3020e89d6a896b4efcab87bb9683298ea10b8593ad799bf99009668acb`, Creator `Hwp 2020 0.0.0.0`/Producer `Hancom PDF 1.3.0.550`입니다. 새 PDF와 원본을 직접 비교한 보정 전 Native 최저는 9쪽 41.63119%, 8쪽 69.77392%, 2쪽 60.87358%였습니다. 8쪽 왼쪽 셀에 `3. <신 설>`이 남고 오른쪽 본문은 쪽 경계에서 갈라졌습니다.
- **원인·경로와 보정**: 저장 HWPX RowBreak의 한 줄 셀과 다줄 셀이 함께 있는 행에서 첫 유닛만 앞쪽에 남으면 `row_step.rs`의 컷을 행 시작으로 되돌립니다. 저장 vpos 재설정·문단 간 reset·명시 컷·기존 특수 보존 경로는 제외합니다. HWPX `PARA/PARA` 앵커 중 저장된 폭 0의 표 전용 글줄이 첫 글줄 아래에 있는 경우만 표의 바깥 위 여백을 엽니다. 공백 스타일은 일반 본문에만 쓰고 표 셀에는 쓰지 않는 14pt 한양신명조에 한정합니다. 연속 하이픈의 원 글리프가 저장 간격을 넘어 실선처럼 겹칠 때만 공통 글자별 짧은 획을 SVG·Canvas·Skia에 사용합니다. 입력→컷/측정→예산→배치→paint는 위 함수와 `svg.rs`, `web_canvas.rs`, `skia/text_replay.rs`에 연결됩니다.
- **시각 검증 상태**: 최초 후보는 새 PDF와 전 9쪽을 Native/fresh WASM 96dpi로 비교해 양쪽 모두 `[93.42898, 98.22721, 98.18999, 97.96030, 98.36466, 99.89148, 96.59063, 98.85387, 99.76733]%`였습니다. 그러나 이 후보의 전체 nextest는 10,177 PASS/52 FAIL/50 SKIP으로 직전 head의 37 FAIL보다 신규 실패가 18건 늘어 **폐기했습니다**. 현재 코드에서는 여백·행 이월의 적용 조건을 좁혔으며 전 9쪽 Native/fresh WASM은 다시 실행해야 합니다. 최초 후보의 PNG는 `output/pr-review/planet6897-7382-20260926/stage185-issue3308/visual-dash-origin-{native,wasm}/`에 진단 자료로만 보존합니다. 이 점수를 현재 코드나 PR 제출 증거로 사용하지 않습니다.
- **기존 회귀 검사 교정**: `issue_3308_nested_table_width`는 기준 PDF의 오른쪽 셀 경계 x599.2를 `직인` TextRun의 x598.7로 잘못 고정해 현재 정상 TextRun x613.7에서 실패했습니다. 수정 전 focused nextest **1FAIL**을 확인했습니다. 이제 부모 대비 중첩 표 폭·가운데 간격·오른쪽 셀의 직인 포함 관계를 검사하고, 산출 JSON은 `output/pr-review/tests/`에서만 만듭니다. 기존 `issue_3307_outline_default_numbering`은 3번 항목 양쪽 셀의 9쪽 소유를 검사합니다. 최신 후보의 두 함수는 **2PASS/0FAIL**입니다. 새 검사 파일·golden·baseline·공차 상향은 없습니다.
- **현재 집중 검증과 남은 일**: 좁힌 후보에서 #6950 연속 조각, #1949 첫 조각 bbox, #2215 선택 좌표 2건, #6860 저장 reset, #3307 행 소유, #2214 페이지 조회 2건, 차트 11~13쪽, SVG 스냅샷 및 넘침/겹침 일부가 통과했습니다. `#2004` 그림 y는 아직 기존 회귀보다 +3.8px이고, 본문 넘침 기준의 #6697·#3637은 각각 1건 증가했습니다. 이 3건은 다음 보정에서 독립 PDF와 실제 출력으로 판정하며 기준값을 올리지 않습니다. 현재 후보의 전체 nextest, 최종 시각 검사, lint·정책 검사는 미실행이므로 PR 준비 판정은 보류합니다. 이전 단계의 재생성 가능한 산출물을 정리해 `output`을 464GiB에서 33GiB로 줄였고, 최신 입력·기준 PDF·진단 PNG는 남겼습니다. 실행 로그는 `output/pr-review/planet6897-7382-20260926/stage185-issue3308/`에만 있습니다.

## 보정186 — #2004 그림 회귀의 절대 좌표 검사 교정

- **분석**: 보정185 뒤 기존 `issue_2004_projection_preserves_each_picture_identity_and_final_bounds`는 HWPX 4쪽 그림 y의 과거 고정값 124.9px에 대해 128.66px을 관측해 실패했습니다. 기존 검사 주석은 HWP5 조각의 바깥 위 여백만 인정하고 HWPX에는 이 변화를 적용하지 않는다고 가정했지만, 이 픽셀 상수는 그림의 쪽·셀 소유 계약을 나타내지 않습니다. HWP/HWPX 원본과 각 한컴 PDF는 모두 8쪽입니다. HWPX SHA-256은 `e91cf067de71c6c567e27974415bc67462f3e777997d41853db949fea4265fa7`, HWP는 `c633fd086323075ab25777d8ee10357869d037bd17c9e74d444d5b8297c40d50`입니다. [HWPX 기준 PDF](../../../pdf/issue2004_cell_image_stack-hwpx-2020.pdf)는 `ed5fea4e1fa6e21134088f905c717eb51396d398d6398636754b49d8ff83477b`, [HWP 기준 PDF](../../../pdf/issue2004_cell_image_stack-hwp-2020.pdf)는 `b5e2f962620d8d7cdd2ec8b10571c53fbe959eb9ce7b3428ff09a595f12fee90`입니다. 두 PDF의 Creator는 파일명과 달리 `Hwp 2022 0.0.0.0`입니다.
- **독립 시각 판정**: 보정185의 renderer head `78a1dfc65`를 Native와 새로 빌드한 Mac 로컬 대체 fresh WASM(`--no-opt`)으로 실행했습니다. 각 형식의 **전 8쪽**을 해당 PDF와 96dpi Visual Sweep으로 비교해 총 32개 페이지 비교 모두 gate `passed`, 네 경로 각각의 최저 2px 이웃 관용 내용 실루엣 일치율은 **95.82931%**입니다. HWPX 4쪽은 99.93866%이며 review에서 그림·셀 괘선과 뒤 문단의 소속·누락을 직접 확인했습니다. 1쪽 글자 모양·자간 잔차는 남습니다. Native 출력은 `output/pr-review/planet6897-7382-20260926/stage186-issue2004/visual-{native,hwp-native}/`, fresh WASM 출력은 같은 위치의 `visual-{wasm,hwp-wasm}/`에 있습니다. 대표 PNG의 SHA-256은 아래 순서대로 `8b66c1f5b33069b12c77e36fe465d01c543c8d10f9350db379777d593092aa1d`, `d151f31b749e266c58b15c4e6294245d6afa2344ea28d7328f797e40b355319d`, `a09a772c6dfc42a27abfd9cd067d1310298ad86a9a6efc77347e41940a2af5fe`, `437ad32e1157411ebed3b62776cb690b821cdfc3057ada6df3aa9aafe2b8ae64`입니다.
- **회귀 검사 수정과 결과**: 새 파일·golden·baseline·허용치 증가는 없습니다. 기존 검사에서 그림의 x/y/폭/높이 픽셀 상수를 제거하고, HWP와 HWPX의 4~8쪽마다 그림이 정확히 한 번 나타나며 원본 binData ID 순서를 유지하고 소유 표 셀 안에 완전히 들어가는지 검사합니다. 수정 전 focused nextest는 #2004 **1FAIL**, 수정 후 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --test regression_suite_013 --test-threads 8 --no-fail-fast -E 'test(issue_2004_projection_preserves_each_picture_identity_and_final_bounds)'`는 **1PASS/0FAIL**입니다. 이 단계는 테스트 의미 교정이며 renderer 코드는 바꾸지 않았습니다. #6697·#3637 본문 넘침 증가 및 현재 head의 전체 nextest·lint는 다음 단계에서 검증합니다.

![#2004 HWPX 1쪽 Native 기준 PDF 비교](../assets/pr7382_20260926/stage186_issue2004_hwpx_native_review_001.png)
![#2004 HWPX 4쪽 Native 기준 PDF 비교](../assets/pr7382_20260926/stage186_issue2004_hwpx_native_review_004.png)
![#2004 HWP 4쪽 Native 기준 PDF 비교](../assets/pr7382_20260926/stage186_issue2004_hwp_native_review_004.png)
![#2004 HWPX 4쪽 fresh WASM overlay](../assets/pr7382_20260926/stage186_issue2004_hwpx_wasm_overlay_004.png)

## 보정187 — #6697 본문 넘침 회귀 대상만 #7445 이관

- **선행 판정과 새 증가분**: 보정160에서 [#7445에 등록한 같은 원본](../assets/issue7445/host6697_blocking_scope_validation.json)은 한컴 PDF **31쪽**·rhwp **32쪽**이고, 여러 쪽의 중첩 표·캡션 소유가 달라 관련 실패 함수 1개만 이미 제거했습니다. 이번 보정185의 본문 넘침 원장에는 이 원본의 25쪽 `PartialTable`이 본문 바닥 1046.9px을 약 3.03px 넘어 **기존 2건→현재 3건**으로 추가됐습니다. 원본 [`80550 HWPX`](../../../samples/issue6697/80550-agricultural-machinery-act-amendment.hwpx)의 SHA-256은 `e7b147f7cea66c97bed79085a3d89c2656037e0f711232f659ed3c7344984f62`, [한컴 PDF](../../../pdf/issue7382-regression-review/80550-agricultural-machinery-act-amendment-2020.pdf)는 `4e3e656a70bc1a0ba1314f949b0f35ae057266b6a25921c6dc417c8c26f587ad`입니다.
- **현재 head의 시각 재확인**: renderer 코드 `78a1dfc65`와 SHA-256 `31b3acf8f721b52662c84c21a967bc55ba46a1e6ba41731e974cedf851de89db`인 검토 전용 Native CLI로 29~31쪽을 다시 비교했습니다. 2px 이웃 관용 내용 실루엣 일치율은 **38.03058/24.18179/28.81844%**, gate는 `re_review_required`입니다. 30쪽 review에서 한컴의 뒤 표·문단이 rhwp의 앞 표와 다른 쪽 소속으로 나타남을 직접 확인했습니다. 전체 32쪽 통과나 fresh WASM 재검증으로 보고하지 않으며, 기존 보정160의 fresh WASM 실패 증거도 유지합니다. 새 review PNG의 SHA-256은 `0721653b3e4d6ab6b0acfaf28e5d5056e249309057ebf2d9446ebf4fa65c46c4`입니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage187-issue6697/visual-native/`입니다.
- **검사 범위 수정**: `tests/cases/body_overflow_baseline.rs`의 이미 존재하는 #7445 보류 원본 목록에 **이 한 HWPX만** 넣고, 해당 원본의 `tests/fixtures/body_overflow_baseline.tsv` 행만 제거했습니다. 원문·PDF와 다른 회귀·문서의 넘침 기준 및 공차는 유지합니다. 16개 분할 전체를 재실행한 결과 **12PASS/4FAIL**이며 #6697이 있던 분할5는 통과했습니다. 남은 실패 원본은 #3637, #6756, #6776, `k-water-rfp-2024.hwp`로 각각 별도 판정합니다. 이 단계의 분할 전체 통과나 #6697 피델리티 해결을 주장하지 않습니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage187-issue6697-body-overflow-nextest.log`입니다.

![#6697 현재 Native 30쪽과 한컴 PDF의 소속 차이](../assets/issue7445/host6697_stage187_native_review_030.png)

## 보정188 — #3637의 중복 원본 경로를 같은 #7445 범위로 판정

- **원문과 독립 기준**: [`issue3637/regulatory_impact_nested_table_escape.hwpx`](../../../samples/issue3637/regulatory_impact_nested_table_escape.hwpx)는 확장자대로 ZIP HWPX이며, 보정187의 #6697 원본과 SHA-256 `e7b147f7cea66c97bed79085a3d89c2656037e0f711232f659ed3c7344984f62`로 **바이트까지 동일**합니다. 이 경로의 별도 [한컴 PDF](../../../pdf/issue3637/regulatory_impact_nested_table_escape-hwpx-2020.pdf)는 SHA-256 `4628627c8d6e41dea85aa022232221da982e28538e6a514a24b8ca1af3f21699`, Creator `Hwp 2022 0.0.0.0`, 31쪽입니다. 현재 rhwp는 같은 원본을 32쪽으로 출력합니다. 이전 기록의 다른 #3637 보도자료·HWP5 파일과 혼동하지 않습니다.
- **현재 시각·회귀 증거**: renderer 코드 `78a1dfc65`의 Native CLI로 이 PDF의 29~31쪽을 직접 비교했습니다. 점수는 **38.09851/23.93271/28.73217%**, gate `re_review_required`이며 30쪽 review에서 #6697과 같은 뒤 표·문단의 쪽 소속 차이를 확인했습니다. 전체 32쪽/fresh WASM을 새로 통과했다고 주장하지 않습니다. 대표 PNG SHA-256은 `c7c313a6eff01798f549715b7ce0082151ba5e10a158381b80a85b7e5c666ca2`이고 출력은 `output/pr-review/planet6897-7382-20260926/stage188-issue3637/visual-native/`입니다.
- **검사 범위**: #7445로 이미 보류한 동일 입력의 **이 경로만** 공용 본문 넘침 원장의 보류 목록에 넣고 해당 TSV 행을 제거했습니다. 다른 샘플·래칫·공차는 유지합니다. 16개 분할 재실행은 **14PASS/2FAIL**이며 남은 증가/신규 원본은 #6756, #6776, `k-water-rfp-2024.hwp`입니다. [#7445 보충 기록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5915130154)에 두 경로의 동일 SHA·새 25쪽 넘침·복귀 조건을 적고 API readback으로 한글과 BOM 없음도 확인했습니다. 이 보정은 피델리티 복원이 아니고, 전체 PR 검증은 남아 있습니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage188-issue3637-body-overflow-nextest.log`에 있습니다.

![#3637 중복 원본의 현재 Native 30쪽과 별도 한컴 PDF](../assets/issue7445/host3637_stage188_native_review_030.png)

## 보정189 — #6756 짧은 원문의 기존 실패와 표 조각 경계 확인

- **실패 출처**: 보정179의 전체 nextest는 이미 37건 실패했고, `body_overflow_does_not_grow_partition_3` 안에 `issue6756/17253153-traffic-safety-designated-routes.hwp` 신규 본문 넘침 1건과 `k-water-rfp-2024.hwp` 1건이 있었습니다. 따라서 #6756을 보정185에서 새로 발생한 회귀라고 분류하지 않습니다. 원문은 5쪽이라 이 브랜치에서 조판을 고칠 대상입니다.
- **독립 기준과 현재 출력**: [원본 HWP](../../../samples/issue6756/17253153-traffic-safety-designated-routes.hwp) SHA-256 `0bb29b403355463d16c8dab2107431cb258ec73f8ada8ec564f2200fd8aa2721`, [한컴 PDF](../../../pdf/17253153-traffic-safety-designated-routes-2020.pdf) SHA-256 `deb6aad07fde6198443c95360a2dcbaea7781dce66e769c4754c0fb8fca8f6be`입니다. 양쪽 모두 5쪽이나 현재 Native 전쪽 점수는 **89.21882/83.49429/77.78808/86.93917/34.61325%**로 gate가 `re_review_required`입니다. 1·4·5쪽 review를 직접 확인했습니다. 4쪽 마지막 표 셀의 `3. 북위 34도50분53초…`는 한컴 PDF에서는 4쪽에 있고 rhwp에서는 5쪽 첫 줄입니다. 5쪽의 한컴 출력에는 다음 `4. 북위 34도50분32초…`만 있습니다. 이는 5쪽의 낮은 점수를 설명하는 실제 내용 소유 차이입니다.
- **스캐너 관측과 검사 한계**: 현재 `dump-pages`는 4쪽 조각을 `startRow=20,endRow=28,endCut=[2,3]`, 사용 높이 982.84px/본문 1009.15px, 5쪽을 `startRow=27,startCut=[2,3]`으로 기록합니다. 1쪽의 표 상자는 본문 바닥을 약 32.89px 넘습니다. 기존 `issue_6756_rowbreak_cut_index_in_rowspan_block`의 두 검사는 **2PASS**지만 전체 글자 수 범위와 2쪽의 용지 경계만 보므로 이 차이를 검출하지 못합니다. 이 단계에서는 검사 추가·baseline 완화·코드 변경을 하지 않았습니다. 4쪽 행 컷의 예약 높이와 실제 paint 높이를 다음 단계에서 대조하고 수정 후 Native/fresh WASM 전체 5쪽을 다시 비교합니다. 산출물은 `output/pr-review/planet6897-7382-20260926/stage189-issue6756/`과 `stage189-issue6756-pages.json`, `stage189-issue6756-scan.log`, `stage189-issue6756-diag.log`에 있습니다.

## 보정190 — #6756 마지막 행 컷의 높이와 저장 문단 경계 분해

- **생산·소비 경로**: `scan_ordinary_row_step`이 `advance_row_cut_with_mixed_nested_reserve`의 컷을 받고, `advance_row_cut_inner`가 셀 유닛 예산을 판정하며, 선택한 `end_cut`을 `row_cut_content_height`와 `table_partial`의 표시 창이 소비합니다. 4쪽의 마지막 행 27에서 내용 예산은 **107.91px**이고 현재 `[2,3]` 컷의 소비 높이는 **81.60px**입니다. 다음 `[2,4]` 컷은 패딩 포함 **123.893px**로 행의 남은 물리 공간 **123.000px**보다 **0.893px** 큽니다. 한컴 PDF는 그 다음 좌표 줄까지 4쪽에 둡니다.
- **시험과 반례**: 이 줄은 동일 문단의 연속 줄이 아니라 각각 별도 문단의 저장 유닛입니다. 끝 문단의 저장 `line_spacing=840HU`, `line_height=1200HU`이며 다음 문단에 `hard_break_before=true`가 있습니다. 같은 문단 줄간격만 빼는 첫 시험은 컷을 바꾸지 못했습니다. 마지막 행의 문단 간격을 빼는 두 번째 진단도 최종 `endCut=[2,3]`, 5쪽의 두 줄, Native 5쪽 최저 **34.61325%**를 그대로 남겼습니다. 두 시험 코드는 제거했고 저장소에는 검증된 동작 변경을 남기지 않았습니다. 다음 단계는 컷 결과가 `advance_row_cut_inner`에서 나오며 그 뒤 예약·source-tail 선택에서 덮이는지 각각 계측합니다. 진단 JSON·로그는 `output/pr-review/planet6897-7382-20260926/stage190-issue6756-*`에 있습니다.

## 보정191 — #6756 저장 문단 재시작의 줄간격 소유 복원

- **원인**: 행 27에서 `advance_row_cut_inner`가 이미 `[2,3]`을 반환하며 중첩 예약도 그대로 유지합니다. 저장 줄은 `vpos=6120HU` 뒤 다음 문단에서 **0으로 되감기고** hard break를 가집니다. 기존 `native_multirow_saved_reset_trailing_trim`은 이 물리 경계의 마지막 줄간격을 제외할 수 있지만, 표 감싸기 방식이 `TopAndBottom`이어야 한다는 조건에서 빠졌습니다. 감싸기 방식은 저장 문단의 쪽 소유를 바꾸지 않으므로 이 조건만 제거했습니다. 컷 선택·높이 조회·표시 창이 같은 트림 결과를 소비하며 다른 수치·baseline은 바꾸지 않았습니다.
- **관측 결과와 잔여**: 원본 5쪽을 유지하고, 4쪽 행 27의 `endCut`은 **[2,3]→[2,4]**, 5쪽 `startCut`도 **[2,4]**가 됐습니다. 4쪽에 `3. 북위 34도50분53초…`, 5쪽에는 `4. 북위 34도50분32초…`만 남아 한컴 PDF의 줄 소유와 맞습니다. Native 5쪽 Visual Sweep은 수정 전 **89.21882/83.49429/77.78808/86.93917/34.61325%**, 수정 후 **89.21882/83.49429/78.68970/87.45207/71.85641%**입니다. `H2MJSM.TTF`를 명시적으로 전체 임베딩해 다시 캡처해도 동일한 점수였으므로 이 차이를 글꼴 예외로 처리하지 않습니다. 1쪽의 `PartialTable` 본문 하한 초과 약 32.7px와 분할 쪽 괘선 차이가 남았고 gate는 여전히 `re_review_required`입니다.
- **검증 범위**: `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_6756_rowbreak_cut_index_in_rowspan_block) | test(issue_6549_square_rowbreak_extension_bound)' --no-fail-fast` **3PASS**입니다. 4·5쪽 review를 직접 확인했습니다. 낮은 점수이므로 기존 #6756 검사와 baseline을 아직 갱신하지 않았고 새 회귀도 추가하지 않았습니다. `output/pr-review/planet6897-7382-20260926/stage191-issue6756/` 및 `stage191-issue6756-focused-nextest.log`가 진단 산출물입니다. fresh WASM·전체 nextest 및 전쪽 ≥90%는 이후 변경 완료 후 재검증할 항목입니다.

## 보정192 — 분할 행 경계선의 상반된 기준 출력 확인

- **시험**: #6756의 1~5쪽 한컴 PDF에서는 쪽 사이 분할 행에 새 가로 괘선이 보이지 않습니다. `h_edges`의 분할 조각 시작·끝 선을 비우는 시험을 했고, Native 5쪽 점수는 **89.21882/83.49429/78.68970/87.45207/71.85641% → 90.22355/84.76893/79.51000/88.90431/93.67188%**로 변했습니다. 2·5쪽 review에서도 불필요한 가로선이 사라졌습니다.
- **반례와 판정**: 같은 RowBreak 분할을 쓰는 #5885의 독립 PDF는 첫 조각 하단 **1002.902667px**에 실제 닫힌 괘선이 있습니다. 집중 nextest **24개 중 23PASS, 1FAIL**로 이 선이 제거되어 마지막 괘선이 **982.026667px**가 됐음을 확인했습니다. 분할 컷만으로 선의 소유를 판정할 수 없으므로 시험 코드를 되돌렸습니다. 한컴 기준에서 두 표의 경계선 조건을 더 구분하기 전에는 광역 삭제를 채택하지 않습니다. 시험 증적은 `output/pr-review/planet6897-7382-20260926/stage192-issue6756/visual-native/` 및 `stage192-issue6756-focused-nextest.log`에 보존했습니다. #6756의 2~5쪽 90% 미달과 gate `re_review_required`는 계속 남습니다.

## 보정193 — #6756 분할 셀의 글자 원점 계측

- 2쪽 행 14 열 2의 저장 `valign=Center`는 분할 컷 때문에 실제 배치에서 `Top`으로 바뀝니다. 조각 상자는 y=**977.480px**, 높이=**139.893px**, 안 여백 상·하 각 **7.547px**, 조각 내용 높이 **81.600px**, 글자 시작 y=**985.027px**입니다. Center를 단순 복원하면 남는 높이의 절반 **21.600px**을 추가해 한컴 PDF에서 관찰한 약 9px 차이보다 과하게 내려갑니다.
- 3쪽 행 20 열 2도 저장 Center→실제 Top이며 상자 y=**891.987px**, 높이=**167.093px**, 글자 시작 y=**899.533px**입니다. `composition_window` 경로의 내용 높이 계측값은 0이지만 실제 글줄은 표시됩니다. 따라서 그 0을 이용한 가운데 정렬은 근거가 없습니다. 임시 진단 코드는 제거했으며 로그는 `output/pr-review/planet6897-7382-20260926/stage193-issue6756-tree.log`에 남겼습니다. 다음 단계에서는 저장 줄 원점과 문단 간격을 실제 글줄 배치 소비 지점까지 추적해 두 행의 공통 차이를 찾습니다.

## 보정194 — #4966 폰트 규칙 회귀 검사의 기록된 추가분 반영

- **원인**: `issue_4966_font_rule_projection` 검사는 봉인된 v1 규칙 830개에서 기록된 두 `retire-and-replace`만 반영했습니다. 이미 적용된 #7196의 `issue-7196-ygodic230-metric-name.json`은 독립 한컴 PDF와 실폰트 계측을 근거로 `rust-layout-metric` 규칙 한 건을 `add-rule`로 추가했습니다. 현재 정본 레지스트리는 활성 **831개**, metric projection **68개**가 맞으며, 실패의 830/67 기대값은 과거 상태입니다.
- **수정**: v1 봉인은 유지하고 변경 기록의 종류·투영·활성 증가량을 검사한 뒤 해당 규칙만 기대 의미와 metric 투영 순서에 추가했습니다. 기존 모든 규칙의 의미·순서 비교와 공개 `find_metric` 호출 검사는 유지합니다. 렌더러 동작이나 시각 기준값은 바꾸지 않았습니다.
- **집중 검증**: `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_4966_font_rule_projection)' --no-fail-fast`에서 **3PASS/0FAIL**입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage194-issue4966-nextest.log`입니다. 전체 nextest 결과로 확대 해석하지 않습니다.

## 보정195 — #4068 검사 실패의 페이지 이동 확인

- 기존 두 검사는 `hwpx_sample2.hwp`의 **19쪽(0-based 18)**, 셀 y=955~975px에서 1×2 중첩 표를 찾습니다. 현재 렌더 트리에서는 19쪽에 해당 칸이 없고 **20쪽(0-based 19) y=963.6px**에 두 칸이 있습니다. 독립 한컴 2020 PDF는 **29쪽**, 현재 rhwp는 **30쪽**입니다. 첫 쪽에서부터 한컴 PDF의 `신 청 안 내` 아래 항목이 rhwp의 다음 쪽으로 이월되어, 19쪽 대상 이동은 한 칸의 좌표 오차가 아니라 앞선 페이지 증가의 결과입니다.
- 따라서 고정 페이지·y 범위를 20쪽으로 바꿔 통과시키지 않았습니다. 쪽수 자체가 다르므로 90% 이상 Visual Sweep 근거도 아직 성립하지 않습니다. 원본과 PDF의 페이지별 텍스트 및 19·20쪽 렌더 트리는 `output/pr-review/planet6897-7382-20260926/stage195-issue4068-*`에 저장했습니다. 다음 판단은 첫 쪽 이월의 조판 원인을 확인하고, 전체 페이지를 독립 PDF와 재비교한 뒤 관계 기반 검사로 고치는 것입니다.

## 보정196 — #4068 첫 쪽 Visual Sweep으로 이월 범위 확인

- 동일 HWP와 `pdf/hwpx_sample2-2020.pdf`를 96dpi Native Visual Sweep으로 직접 비교했습니다. **1쪽 68.42791%**, gate `re_review_required`입니다. `output/pr-review/planet6897-7382-20260926/stage196-issue4068-visual/hwpx-sample2-p1/review/review_001.png`의 한컴 쪽에는 `신 청 안 내` 제목 아래 신청 조건·표·일정이 모두 있으나 rhwp 쪽은 제목까지만 있고 나머지를 2쪽으로 이월합니다. 제목 위의 상단 내용·외곽선은 대체로 같은 위치여서 폰트만의 차이가 아닙니다.
- 현재 #4068 검사를 페이지 20의 새 좌표로 고치거나 삭제하면 이 실제 이월을 가립니다. first-page table 배치 원인과 devel 대조를 먼저 확인해야 합니다. Visual Sweep 산출물은 `output/pr-review/planet6897-7382-20260926/stage196-issue4068-visual/`에 보존했고, 새 회귀·baseline 변경은 하지 않았습니다.

## 보정197 — #4068 이월의 devel 대조

- 보관된 대조 작업트리 `output/pr-review/planet6897-7382-20260926/control-stage19`의 source **`eb9142dd7`**로 별도 debug CLI를 빌드했습니다. 동일 SHA의 `samples/hwpx_sample2.hwp`를 이 CLI가 **29쪽**으로 배치하고, 현 브랜치 CLI는 **30쪽**으로 배치합니다. 이는 기준 PDF 29쪽과도 일치하므로 첫 쪽 이월은 이번 브랜치에서 새로 도입한 차단 회귀입니다. 대조 출력은 `stage197-issue4068-control-p1.json`, `stage197-issue4068-control-p1-text.json`에 보존했습니다.
- 오래된 CLI는 현재 Visual Sweep이 전달하는 `export-svg --profile print --font-style` 조합을 지원하지 않아 대조 Visual Sweep은 실행되지 않았습니다. 대조군에서 확인된 것은 쪽수와 페이지별 텍스트이며, 픽셀 점수는 아직 없습니다. 현재 브랜치의 **1쪽 68.42791%** 증거와 분리합니다. 다음 단계에서는 첫 쪽 신청 안내 표의 paginator 예약과 실제 높이를 두 source에서 비교합니다.

## 보정198 — #4068 표 행 컷의 두 source 비교

- 공용 `target/pr-review`의 실행 파일을 대조 작업트리 빌드가 덮은 직후 현재 source에서 `cargo build`만 실행하면 캐시가 재컴파일을 건너뛰어 대조 실행 파일이 남았습니다. 이때 얻은 현재 29쪽이라는 값은 폐기했습니다. 현재 source 파일의 mtime을 갱신해 명시적으로 다시 빌드한 CLI에서 **30쪽**을 재확인했습니다. 소스 내용·작업 트리는 변경하지 않았습니다.
- 한컴과 같은 **29쪽**인 devel 대조 `eb9142dd7`은 1쪽의 문단 4 표(2행×1열)를 `rows 0..2, endCut=[32]`까지 담아 본문 **1045.09px**를 씁니다. 현 브랜치는 같은 표에서 `rows 0..1, endCut=[]`로 제목행만 담아 **332.40px**를 쓰고 나머지를 2쪽으로 이월합니다. 현재 `RHWP_DIAG_SCAN`에서 행 1 첫 컷은 `budget=710.8px, consumed=722.5px, endCut=[32]`이며, 선택 뒤 예산 재검사에서 버려집니다. 저장 컷 선택→paint 요구 높이→재시도 경로를 다음 단계에서 분해합니다. 근거는 `stage197-issue4068-control-p1.json`, `stage198-issue4068-current-p1.json`, `stage198-issue4068-diag.log`입니다.

## 보정199 — #4068 첫 실패 커밋 격리

- 앞선 전체 nextest에서 보정137 source `0a069b1f1`은 #4068의 두 함수가 PASS였고, 보정171에서는 FAIL이었습니다. 해당 구간에서 별도 소스 빌드로 이분했습니다. `0dc722184`는 동일 HWP **29쪽**, 문단 4 시작의 `cur_h=303.4px`, 행 1 컷 `[32]`의 `budget=715.5px / consumed=713.7px`입니다. 직후 `037a0ec50`은 **30쪽**, `cur_h=308.1px`, 동일 컷의 `budget=710.8px / consumed=722.5px`입니다. 첫 실패는 `037a0ec50`의 TAC 후행 간격·합성 줄 원점 변경 구간에 있습니다.
- `037a0ec50`은 #2243의 기존 문제를 고친 변경이므로 전체 커밋을 되돌리는 것은 해결이 아닙니다. 저장 TAC 줄의 간격이 상위 표 예약과 실제 배치에 각각 어떻게 포함되는지 확인하고, 두 사례의 독립 PDF와 맞는 공통 계산을 찾습니다. 이분 빌드 로그는 `output/pr-review/planet6897-7382-20260926/stage199-*`에 있습니다. 별도 격리 작업트리는 계측 뒤 제거합니다.

## 보정200 — HWP5 저장 TAC 간격의 공통 소비 시험

- `037a0ec50`에서 저장 HWP5 TAC의 양수 후행 간격을 측정·배치 모두 전량 소비하도록 바꾼 것이 #4068 첫 쪽의 `cur_h`와 셀 컷 높이를 함께 키웠습니다. 시험 후보는 저장 원본 HWP5 줄에만 개체 프레임 뒤 간격의 절반을 측정과 실제 배치에서 **같이** 소비하고, 편집·재조판 줄과 #2243의 HWPX 경로는 그대로 둡니다. 문서 ID·픽셀 보정 상수로 분기하지 않았습니다.
- 이 시험에서 `hwpx_sample2.hwp`는 **30→29쪽**, #5885는 7쪽, #6756은 5쪽을 유지합니다. #4068의 19쪽 중첩 셀 두 개가 y=**957.8px**에 돌아왔습니다. 첫 쪽 Native Visual Sweep은 **68.42791→83.17919%**이고 신청 안내 본문·표·일정이 기준 PDF와 같은 쪽에 나타납니다. 아직 90% 미만이므로 시각 gate는 보류입니다. 산출물은 `output/pr-review/planet6897-7382-20260926/stage200-issue4068-candidate-visual/`에 있습니다. 집중 회귀와 다른 HWP5 문서 영향은 별도 확인합니다.
- **집중 회귀**: `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_4068_unclipped_cell_honors_valign) | test(issue_2243) | test(issue_5885_nested_host_row_ladder_end) | test(issue_6756_rowbreak_cut_index_in_rowspan_block)' --no-fail-fast`에서 **10PASS/0FAIL**입니다. #2243 HWPX, #5885 원본 하단선, #6756 기존 컷 검사를 함께 보존했습니다. 로그는 `stage200-focused-nextest.log`입니다. 이 10개로 전체 HWP5 영향이나 29쪽 시각 gate를 통과했다고 주장하지 않습니다.

## 보정201 — TAC 공통 간격 수정 뒤 #6756 전쪽 재확인

- `fde7f496b`로 Native 5쪽 Visual Sweep을 다시 수행했습니다. 점수는 **89.21882/83.49429/78.68970/87.45207/71.85641%**로 보정191과 동일하고, 쪽수도 5쪽입니다. 따라서 TAC 공통 간격 수정은 #6756의 남은 괘선·셀 내부 글자 원점·1쪽 본문 하한을 해결하지 않았습니다. #6756의 90% 미달은 계속 독립 차단 항목으로 둡니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage201-issue6756-visual/`입니다.

## 보정202 — #4068 첫 쪽 글꼴 공급 반례

- 한컴 2020 기준 PDF 첫 쪽에는 `HCRDotum`, `MalgunGothic`, `MalgunGothicBold`가 내장돼 있습니다. Mac에는 `HCR Dotum`·맑은 고딕 실폰트가 있으며, 별도 보존된 `HCRDotum.ttf`와 Library/Fonts를 `--embed-fonts=full --font-path`로 명시해 Native 첫 쪽을 다시 캡처했습니다. 선택된 글꼴 파일·SHA는 `stage202-issue4068-font-visual/hwpx-sample2-fontcheck-p1/run_manifest.json`에 있습니다.
- 명시적 전체 임베딩 뒤 점수는 **83.17919%로 정확히 동일**합니다. 따라서 이 사례의 90% 미달을 단순 글꼴 공급 문제나 글꼴 예외로 분류하지 않습니다. 표·문단의 남은 위치 차이와 본문 내용 실루엣을 계속 확인해야 합니다. 이번 단계는 글꼴 설치나 코드 변경 없이 증거만 남겼습니다.

## 보정203 — #5820의 절대 좌표 검사 교체

- 실패한 기존 검사는 `156560092_ecard_meeting_press.hwpx` 둘째 쪽 로고 글상자 y를 **385~392px**에 고정했습니다. 현재 값은 **359.0px**이고, 그 검사 주석 자체에 적힌 한컴 2022 PDF의 글상자 위치는 **358.3px**였습니다. 현재 출력이 독립 기준에 가까운 상황에서 저장 사다리 추정값을 정답으로 둔 것이 실패 원인입니다.
- 동일 원본과 `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue5820_ecard_meeting_press_156560092_ecard_meeting_press-2020.pdf`를 Native 전쪽 비교했습니다. HWPX와 PDF는 모두 **2쪽**, 1쪽 **96.59961%**, 2쪽 **98.17717%**이며 둘째 쪽 review에서 본문·로고·바닥글의 순서와 위치를 확인했습니다. 산출물은 `output/pr-review/planet6897-7382-20260926/stage203-issue5820-visual/`입니다.
- 기존 검사만 수정해 둘째 쪽의 이어지는 마지막 문장, 로고 글상자와 내부 내용, 바닥글의 **소속·순서·포함 관계**를 확인합니다. 절대 픽셀 범위와 SVG 문자열 파서는 제거했습니다. `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_5820_partial_page_start_keeps_body_and_logo_frame_order)'`는 **1PASS**입니다. fresh WASM 및 최종 전체 검증은 아직 수행하지 않았습니다.

## 보정204 — #1658 한 쪽 고정 틀의 실제 회귀 재확인

- `36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx`는 현재 **2쪽**, 독립 `pdf/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177)-2024.pdf`는 **1쪽**입니다. 기존 검사가 보고한 본문 표 상단 **635.400px**은 PDF **411.071px**과 약 **224px** 다릅니다.
- 현재 Native 첫 쪽 Visual Sweep은 **33.72635% / `re_review_required`**입니다. `output/pr-review/planet6897-7382-20260926/stage204-issue1658-visual/issue1658-stage204-native/review/review_001.png`에서 표가 PDF보다 크게 아래에 놓이고 끝 문단과 하단 고정 내용도 뒤로 밀린 것을 확인했습니다. 이전 보정57의 **97.60662%**를 현 head의 통과 증거로 재사용할 수 없습니다.
- 페이지 수와 본문 표 위치의 독립 기준이 모두 현재 실패를 뒷받침하므로 테스트 기대값을 변경하지 않습니다. 다음 단계에서 고정 틀 높이의 배타 예약과 본문 표 측정·배치 소비를 추적합니다. 코드는 아직 변경하지 않았습니다.

## 보정205 — #1658의 저장 vpos 전방 스냅 계측

- `dump-pages`의 첫 쪽은 제목 표 pi0(**222.04px**)와 본문 표 pi4(**188.52px**), 하단 고정 틀 pi5(**247.28px**)를 갖고, 두 번째 하단 틀 pi6(**357.24px**)는 다음 쪽으로 이월됩니다. 실제 render tree에서는 pi0 제목과 pi1~3 본문은 독립 PDF와 비슷한 높이지만 pi4 본문 표만 **635.4px**에 놓입니다. 기대 **411.071px**와의 차이 **224.3px**는 제목 표 높이와 가깝습니다.
- `RHWP_DIAG_SNAPALL=1`에서는 pi1 진입 시 분할기의 `current_height`가 **252.7→488.0px**로 **235.3px** 전방 스냅하고, pi2~3은 추가 점프가 없습니다. 실제 paint의 pi1은 약 **283px**에서 시작하므로 이 스냅은 출력 글줄의 공통 원점이 아닙니다. pi5 고정 틀 fit은 `cur_h=817.83, avail_after=837.44`, pi6은 `cur_h=817.83, avail_after=727.48`로 뒤 틀만 이월합니다. 고정 틀 코드의 결과를 바꾸기 전에 `HeightCursor::vpos_adjust → vpos_snap_current_height → 표 fit/실제 paint`의 원점 소비를 이전 정상 head와 대조합니다.
- 로그와 JSON은 `output/pr-review/planet6897-7382-20260926/stage205-issue1658-{snap,diag}*`에 보존했습니다. 기존 검사·기준값·코드는 아직 수정하지 않았습니다.

## 보정206 — 구역 첫 문단의 가짜 쪽 경계 제거

- 동일 원본을 이전 정상 `1c82df86f`, 중간 `1005da62e`·`037a0ec50`·`6fcb36f24`, 첫 실패 `74b70903b`에서 각각 별도 빌드·실행했습니다. 앞 네 버전은 **1쪽**이고 pi1 전방 스냅이 없으며 `74b70903b`부터 **2쪽**, pi1 **252.7→488.0px** 스냅입니다. 문제의 HWPX 원본 첫 문단은 `pageBreak="0"`, `columnBreak="0"`이고 저장 줄의 첫 vpos는 **17645HU**입니다. `74b70903b`가 도입한 `source_page_break`는 구역 첫 문단의 Section 표지를 새 쪽 경계로 취급하여 그 양수 vpos를 재구성 원점으로 덮었습니다. 첫 문단에는 이전 쪽이 없으므로 이 조건의 적용 범위를 `pi > 0`으로 제한했습니다. 문서 ID·고정 좌표·테스트 허용치는 사용하지 않았습니다.
- 수정 후 #1658 원본은 다시 **1쪽**, pi1은 **252.7→252.7px**로 유지됩니다. Native 한컴 PDF Visual Sweep은 이전 **33.72635%→99.46309%**, 1쪽 전체를 직접 판독해 본문 표·끝 문단·하단 두 틀의 위치와 내용 소유를 확인했습니다. 정상 대조 PC 셧다운 문서는 **1쪽 / 97.12491%**이며 review에서 표와 하단 틀을 확인했습니다. 산출물은 `output/pr-review/planet6897-7382-20260926/stage206-issue1658-{visual,pc-visual}/`입니다.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_1658_page_bottom_fixed_exclusion) | test(issue_1611_footer_page_bottom_pagination) | test(issue_6145_squeeze_cell_keeps_inner_margin) | test(issue_2243) | test(issue_6535_page_anchored_block_keeps_page)' --no-fail-fast`는 **8PASS**입니다. 현재 검증 head의 전체 회귀·fresh WASM·Rust 필수 lint는 아직 완료하지 않았습니다. 임시 대조 작업트리는 비교 뒤 제거했습니다.

## 보정207 — 현재 head 전체 회귀 결과와 #6950 재판정

- `51e7c1bec`에서 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast`를 완료했습니다. **10,229개 실행 / 10,202 PASS / 27 FAIL / 50 skipped**, 종료 코드 **100**입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage207-full-nextest.log`입니다. #4068, #4966, #5820, #1658, #6535와 #6950의 이전 보고 실패 함수는 현재 PASS이며, #1156은 FAIL입니다. 남은 27개를 통과로 간주하거나 PR 준비 완료로 보고하지 않습니다.
- #6950의 원본 `samples/hwpx/20260909-para-table.hwpx`와 독립 `pdf/hwpx/20260909-para-table-2024.pdf`는 모두 **3쪽**입니다. 현재 Native 전쪽 일치율은 **99.24076/99.24704/99.78614%**, 최저값 **99.24076%**로 gate가 통과했습니다. 2·3쪽 review에서 본문·표와 마지막 내용 소유를 확인했습니다. 이전의 합성 분할 함수 `split_and_deferred_computed_tables_preserve_host_and_paint_inside_frame`도 이 head의 전체 검사에서 PASS여서, 해당 검사·기대값을 수정하지 않습니다. 산출물은 `output/pr-review/planet6897-7382-20260926/stage207-issue6950-visual/`입니다.

## 보정208 — #1156의 35쪽 원본 중 미달 회귀 한 함수만 이관

- 전체 nextest에서 실패한 `synam_001_page5_splits_large_rowspan_block_like_hancom`의 원본은 `samples/synam-001.hwp`(SHA-256 `1dce9356ec316407b6c684d5a11190a44bb26da643a7749626763e781ab0c13b`)입니다. 독립 한컴 PDF `pdf/synam-001-hwp-2020.pdf`와 `pdf/synam-001-2022.pdf`는 모두 **35쪽**이고 같은 4~7쪽 Native 비교 점수를 냈습니다. 4~7쪽은 **94.92654/81.57106/80.90473/99.45301%**입니다. 5·6쪽 review에서 분할 표의 글줄·괘선 위치가 기준과 다르며 90% gate 미달임을 직접 확인했습니다. 산출물은 `output/pr-review/planet6897-7382-20260926/stage208-issue1156-{visual,alt-visual}/`입니다.
- 사용자 지시대로 PR #7382를 막는 이 **한 함수만** 정식 회귀에서 제거하고, 원본 HWP·두 PDF와 #1156 파일의 다른 두 함수는 보존했습니다. 컷 문자열을 새 현재값으로 갈아 끼우거나 허용치를 늘리지 않았습니다. 남은 두 함수를 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_1156_rowbreak_fragment_fit)' --no-fail-fast`로 재실행해 **2PASS**를 확인했습니다. 전체 35쪽과 fresh WASM 검증 및 의미 차이 수정은 #7445로 별도 추적하며, 이번 제외를 렌더링 결함 해결로 보고하지 않습니다.
- #7445에 [35쪽 원본의 후속 검증 조건](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5917747711)을 게시하고 한글 본문을 API로 재확인했습니다.

## 보정209 — #6852 HWP와 HWPX 원본의 시각 기준 분리

- 현재 전체 nextest의 #6852 실패 **5개**는 모두 `samples/issue6797/156160455-social-pig-farm-income.hwp`를 직접 읽거나 그 IR을 변형하며, 5쪽 사각형을 고정 x/y/크기로 찾는 공통 helper에서 `0 != 2`로 중단됩니다. 같은 파일의 HWPX 원본 함수는 PASS입니다. HWP/HWPX 중 어느 쪽도 성공한 함수만으로 한컴 출력과의 일치를 입증하지 않습니다.
- 원본 HWP와 독립 `pdf/156160455-social-pig-farm-income-2020.pdf`는 모두 **11쪽**이나 HWP Native 전쪽 최저는 **29.01913%**(5쪽), 6쪽 **30.73124%**입니다. HWPX 파생본도 같은 PDF와 비교하면 최저 **39.87618%**(6쪽), 5쪽 **42.93059%**입니다. 두 입력 모두 90% gate 미달이며 원본 HWP의 5쪽 review에서 문서 내용과 사각형 위치가 PDF와 크게 어긋납니다. 산출물은 `output/pr-review/planet6897-7382-20260926/stage209-issue6852-{visual,hwpx-visual}/`입니다.
- 11쪽 전체 피델리티를 이 단계에서 완료하지 않습니다. 다음 단계에서는 PR #7382를 막는 **HWP 기반 5함수만** #7445로 이관하고, 현재 PASS인 HWPX 함수와 원본 HWP/HWPX·PDF는 보존합니다. 픽셀 좌표를 현재 출력에 맞춰 이동하지 않습니다.

## 보정210 — #6852의 차단 함수 다섯 개만 이관

- 전쪽 90% 미달인 HWP 원본에 의존하는 기존 함수 `original_hwp_foreground_and_shadow_keep_solid_strokes`, `ordinary_unfilled_rectangle_is_not_a_textbox`, `explicit_no_line_is_not_promoted_to_solid`, `empty_and_whitespace_textboxes_keep_their_strokes`, `existing_textbox_branch_is_unchanged_not_a_new_nonprinting_rule`만 제거했습니다. 실패는 모두 공통 SVG 좌표 선택에서 발생했으며 그 좌표를 현 출력으로 갱신하지 않았습니다. 원본 HWP/HWPX와 PDF는 보존했습니다.
- 같은 테스트 파일의 HWPX 원본 검사는 그대로 유지하고 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_6852_group_rectangle_stroke)' --no-fail-fast`에서 **1PASS**를 확인했습니다. 이 HWPX 함수의 PASS는 위에서 확인한 전쪽 피델리티 미달을 해결했다는 뜻이 아닙니다. 11쪽 전체 HWP/HWPX 렌더링 보정과 의미 기반 회귀 재구축은 #7445에서 후속 처리합니다.

## 보정211 — #7287의 105쪽 문서 좌표 검사 이관

- 현재 전체 nextest에서 실패한 `empty_host_square_float_top_includes_its_outer_margin`은 `samples/hwpctl_API_v2.4.hwp`의 64·93~96쪽 표 윗변을 절대 픽셀 좌표에 고정합니다. 원본 HWP는 **105쪽**(SHA-256 `d11dd1331083be4e8c989dfbd587777626b3d77686d3436c35a2c20da9494603`), 독립 한컴 PDF `pdf/hwpctl_API_v2.4-hwp-2020.pdf`도 **105쪽**(SHA-256 `1d289727dd40ed35e48135bf16df06fe4cd080d967441ff464fb0e0b205fae74`)입니다. 실패 지점인 95쪽 pi2469 표 윗변은 현재 **408.12px**, PDF **409.79px**로 기존 ±1.2px 범위를 0.47px 넘습니다.
- 동일 원본·PDF를 Native Visual Sweep으로 관련 5쪽 비교한 결과 **64쪽 98.65784%, 93쪽 82.09330%, 94쪽 47.11934%, 95쪽 96.26233%, 96쪽 96.87253%**입니다. 93·94쪽 review를 직접 판독했으며 특히 94쪽의 본문 줄과 상자 경계가 어긋납니다. 최저 **47.11934%**이므로 기존 절대 좌표값을 현재 출력으로 바꾸거나 허용치를 늘려 통과시킬 근거가 없습니다. 증적은 `output/pr-review/planet6897-7382-20260926/stage211-square7287/hwpctl-api/`입니다.
- PR #7382의 차단 함수 **하나만** 제거하고 같은 파일의 1쪽 가시 host 대조군과 원본 HWP·PDF는 보존했습니다. `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_7287_square_float_outer_top)' --no-fail-fast`는 **1PASS**입니다. 105쪽 전체 피델리티와 어울림 표 의미 기반 검사는 #7445에서 다루며, [후속 조건을 게시](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5917913377)하고 API로 한글 본문을 확인했습니다.

## 보정212 — #6923의 20쪽 Center 제어군 이관

- 실패한 `midpage_center_control_does_not_claim_the_page_frame`은 `tests/fixtures/issue6923/156645214_240812(조간)_4개_아이돌굿즈_판매사업자_전상법의_위반행위_제재.hwp`의 19쪽 Center 표 윗변을 **217.3±1px**에 고정합니다. 현재 **210.8px**입니다. 원본 HWP(SHA-256 `cb36c862d3d41757d15fb322b1733f0e0ce27bf20142db0a25c57a624739e23b`)와 독립 PDF(SHA-256 `71261bcb218eabc6f1b2af5ce81797a3505f417d1c2482ee5441dec11b6ba943`)는 모두 **20쪽**입니다.
- Native Visual Sweep 전쪽 최저는 **50.61887%**(3쪽), 실패한 **19쪽은 53.73731%**입니다. 19쪽 review에서 제목 아래 본문 여러 줄의 글자 위치·줄간격과 표 경계가 독립 PDF와 다름을 직접 확인했습니다. `output/pr-review/planet6897-7382-20260926/stage212-center6923/center6923/`에 전쪽 compare·review를 보존했습니다. 이 상태에서 기준 좌표나 허용치를 현재 출력에 맞추지 않습니다.
- PR #7382를 막는 20쪽 Center 제어군 함수만 제거했습니다. 별도 7쪽 원본에 의존하는 #6923의 두 통과 함수와 두 HWP·PDF는 그대로입니다. `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_6923_wrapper_fragment_page_frame)' --no-fail-fast`는 **2PASS**입니다. 이 문서의 전체 피델리티와 의미 기반 Center 음성 대조군은 #7445에서 재구축하도록 [후속 조건을 게시](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5917972247)하고 API로 한글 본문을 확인했습니다.

## 보정213 — #1749의 5쪽 HWPX RowBreak 분기 원인 측정

- 남은 `issue_1811_hwpx_pi52_rowbreak_cut_matches_hwp_reference`는 `samples/task1749/saved_bounds_cumulative_page_break.hwpx` 4쪽에서 pi52 host `PartialParagraph`를 요구하지만, 현재 `dump-pages`에는 `PartialTable`만 있습니다. HWPX·독립 한컴 PDF는 모두 **5쪽**입니다. Native 전쪽 일치율은 **94.97706/98.41015/99.27944/87.17904/32.98293%**이고 4·5쪽 review에서 표의 별표 글줄, 표 뒤 본문·다음 표 위치가 차이 납니다. 증적은 `output/pr-review/planet6897-7382-20260926/stage213-hwpx1749/hwpx1749/`입니다.
- 같은 내용의 HWP `samples/task1749/saved_bounds_cumulative_page_break.hwp` 경로는 동일 PDF의 4·5쪽에서 **91.16597/90.42361%**입니다. HWP는 4쪽 pi52 첫 조각 `end_cut=[3]`, 5쪽 이어받기 `start_cut=[3]`; HWPX는 각각 `[2]`입니다. 5쪽 render tree에서 pi52 표는 HWPX **207.2px**, HWP **194.3px**, 뒤 pi54 첫 글줄은 HWPX **355.0px**, HWP **326.1px**입니다. 차이는 표 조각에서 12.9px, 뒤 문단 시작에서 28.9px까지 늘어납니다. 단순 글꼴 예외나 테스트 문구 변경으로 처리하지 않습니다.
- 원본 HWPX는 앞서 #7445에 바이트 동일본을 보존했지만, 사용자의 현행 **10쪽 이하 문서 현재 브랜치 보정** 지시에 따라 이 5쪽 문서는 여기서 90% 이상으로 개선합니다. 이번 짧은 단계는 실패·독립 기준·생산/소비 차이의 측정만 커밋하며, 다음 단계에서 RowBreak 컷 및 후속 문단 원점의 공통 경로를 수정합니다. 테스트나 허용치는 아직 바꾸지 않았습니다.

## 보정214 — #1749 첫 조각 예산과 paint 원점 계측

- 동일한 debug 실행 파일에서 HWPX/HWP에 `RHWP_DIAG_SPLITSCAN=1 RHWP_TABLE_DRIFT=1 rhwp dump-pages`를 적용했습니다. pi52의 `MeasuredTable` 행 높이는 두 경로 모두 **[13.32, 13.32, 281.48]px**, 전체 **308.1px**입니다. 그러나 첫 조각 진입 `cur_h`는 HWPX **670.8px**, HWP **662.8px**여서 offset **144.3px**을 제하면 가용 공간이 **115.4px / 123.4px**로 갈립니다. 첫 조각 실제 소비는 **92.3px / 109.6px**, 컷은 `[2] / [3]`입니다. 다음 쪽 pi52 표는 **207.2px / 194.3px**입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage214-{hwpx,hwp}1749-diag.log`입니다.
- render tree의 첫 차이는 pi50 표 자체가 아니라 그 뒤 빈 pi51에서 납니다. 두 경로 pi50 표는 **y355.2~738.8px**로 같지만 pi51 글줄은 HWPX **754.8px**, HWP **746.8px**입니다. pi52 host 첫 줄은 **766.8px / 764.1px**이고 표 첫 조각은 **y911.1px / 908.4px**입니다. 따라서 컷 숫자만 강제로 바꾸기 전에 `pi50 끝 → pi51 빈 host 간격 → pi52 host/표 예약 → 다음 쪽 실제 표 높이`의 공통 소비를 확인해야 합니다. 구현·검사는 이번 단계에서 변경하지 않았습니다.

## 보정215 — #1749 단일 줄 TAC 가설 기각

- `tac_flow::tac_table_line_index`가 HWPX의 단일 저장 줄을 너무 일찍 제외해 pi50 뒤 8px이 중복된다는 가설로, 원본 HWPX·미편집·유일한 표·가시 텍스트 없는 단일 줄만 허용하는 후보를 실행했습니다. `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_1749_saved_bounds_page_break) | test(issue_3930_hwpx_hwp_save_layout)' --no-fail-fast`는 **5PASS/1FAIL**이고 #1749 pi52의 `PartialParagraph` 부재와 `[2]` 컷은 그대로입니다. #3930의 3개 대조군은 PASS입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage215-tac1749-nextest.log`입니다.
- 효과 없는 후보는 모두 되돌려 현재 source diff가 없습니다. `RHWP_DIAG_TAC=1`의 pi50은 HWP에서 `DIAG_TAC ... table_total=383.6 fmt_total=399.6`, 종료 흐름 **658.8px**이며, HWPX에서는 `DIAG_TBLP pi=50` 블록 표 경로입니다. HWPX pi51 진입 **658.8px**, HWP pi51 진입 **650.8px**입니다. 다음에는 두 경로의 표 뒤 빈 문단 점유를 직접 대조합니다. 기준값이나 실패 검사를 변경하지 않았습니다.

## 보정216 — #1749 합성 단일 TAC 줄의 공통 후행 간격

- 추가 계측에서 HWPX pi50은 `stored_tac::prepare_computed` 단축을 사용했습니다. XML의 pi50은 유일한 TAC 표·`flowWithText=1`·저장 줄 높이 **28769HU**이며 표 선언 높이도 **28769HU**입니다. composer가 이 줄을 합성으로 표시하면서 단축 경로는 `line_spacing=1200HU` **16px 전량**을 표 뒤에 더했습니다. HWP 경로의 일반 TAC 종료 **658.8px**는 뒤 높이 정산에서 **650.8px**로 접혀 pi51에 전달되는데, HWPX 단축은 이 정산을 우회했습니다. 단일 표 줄이 실제 표 높이와 일치할 때 공통 TAC처럼 후행 간격의 반을 소비하도록 `stored_tac` 끝점을 수정했습니다. 문서 ID나 쪽 번호 조건은 없습니다.
- 수정 후 HWPX pi52 첫 조각은 `[2]→[3]`으로, HWP와 같은 컷입니다. Native 전쪽 일치율은 **94.97706/98.41015/99.27944/87.67624/69.16541%**로 5쪽이 **32.98293→69.16541%** 개선됐지만 **90% 미달**입니다. `output/pr-review/planet6897-7382-20260926/stage216-hwpx1749-candidate/`를 직접 확인했고 기준값·테스트는 바꾸지 않았습니다. 다음 단계는 이어받기 표 p5 높이 HWPX **182.9px** 대 HWP **194.3px**, 뒤 pi54 첫 줄 **330.7px** 대 **326.1px**을 보정합니다. 현 단계는 중간 보정이며 승인 가능 판정이 아닙니다.

## 보정217 — #1749 누적 좌표의 행 내부 프레임 판별

- HWPX 원본의 pi52 첫 저장 vpos **50309HU**, 양수 개체 오프셋 **10823HU**, 선언 첫 프레임 높이 **8555HU**의 합이 다음 빈 pi53 저장 vpos **69687HU**와 정확히 같습니다. 이는 페이지가 넘어도 vpos가 되감기지 않는 문서에서 첫 물리 조각의 종료 증거입니다. 다만 pi52 host에는 실제 본문 **4줄**이 있고, 표 마지막 행의 내부 컷 `[3]`에서 끝납니다. 빈 host·온전한 행 경계만 가정해 source frame에 연결한 후보는 처음에 선택되지 않았고, 조건을 바로잡아 선택해도 기존 `nearest_saved_rowbreak_frame_row_end`가 행 내부 프레임을 행 2의 끝으로 잘못 분류하므로 출력은 불변입니다.
- 후보의 네 파일 변경은 모두 되돌렸고 현 source에는 보정216만 남깁니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage217-{source-frame,candidate-split,candidate-build}.log`입니다. 다음에는 `(시작 컷 → 마지막 행 내부 컷 → 첫 조각 물리 프레임 114.07px → 이어받기 남은 높이)`를 공통 결과로 만들고 p4/p5 실제 paint와 PDF를 대조합니다. 현재 #1749 실패와 90% 미달은 해결되지 않았습니다.

## 보정218 — #1749 행 내부 컷의 HWP/HWPX 유닛 대조

- 임시 `advance_row_cut` 계측으로 원본 HWP와 HWPX의 마지막 행 첫 컷을 같은 실행 파일에서 비교했습니다. 두 입력의 가용 높이는 **93.01px**, 첫 3개 유닛 높이는 **30.93/30.93/24.27px**, 결과는 모두 `end_cut=[3]`, 소비 높이 **86.13px**입니다. HWP에는 4번째 유닛의 저장 `hard_break_before`가 있지만 HWPX에는 없습니다. 따라서 보정216 이후 남은 p5 높이 차이는 유닛 내용이나 컷 번호가 아니라 저장된 물리 프레임·패딩을 이어받기에 전달하는 경로에 있습니다. 원본의 첫 프레임 **8555HU=114.07px**와 다음 pi53 원점 **69687HU**는 보정217의 독립 근거입니다.
- 계측 로그는 `output/pr-review/planet6897-7382-20260926/stage218-{hwp,hwpx}-units.log`입니다. 문서 값에 맞춘 임시 진단 코드는 제거했고 이 단계에는 동작 변경이 없습니다. 다음 단계에서 첫 조각의 물리 높이와 이어받는 행의 남은 높이를 공통 결과로 연결한 뒤 p4/p5를 직접 대조합니다. 현재 #1749의 90% 기준과 기존 회귀는 여전히 미충족입니다.

## 보정219 — 첫 조각 프레임 후보와 뒤 빈 문단 간격 반례

- 누적 저장 원점 식이 정확한 미편집 RowBreak 표에만 첫 물리 프레임을 적용하고, 컷 유닛은 그대로 둔 채 마지막 행 높이와 다음 조각의 남은 높이를 함께 변경하는 후보를 시험했습니다. 5쪽 이어받기 표는 **182.9→194.3px**가 되어 HWP 경로와 같아졌고 빈 pi53 시작도 **278.7→290.1px**로 HWP와 일치했습니다. 그러나 pi54 첫 줄은 HWP **326.1px**에 비해 후보 HWPX **342.1px**로 16px 아래에 있습니다. Native 전체 5쪽 비교의 5쪽은 **69.16541→17.45456%**로 악화되므로 후보를 제출하지 않습니다.
- `RHWP_VPOS_DEBUG=1`에서 pi54 직전 빈 pi53은 HWPX의 `vpos_end=72387HU`, 역산 base **53933HU**, 흐름 입력 **230.05px**에서 결과 **246.05px**로 저장 줄간격 **1200HU=16px**를 재가산합니다. HWP 경로는 같은 흐름 입력 **230.05px**가 그대로 결과입니다. 기존 `rendered_spent_trailing`의 적용 여부와 분할 표 뒤 첫 빈 문단의 원점 소유를 다음 단계에서 확인해야 합니다. 후보 코드는 모두 되돌렸고 동작 변경은 없습니다. 로그·5쪽 review는 `output/pr-review/planet6897-7382-20260926/stage219-*`에 보존합니다.

## 보정220 — 누적 표 프레임과 원본 쪽 내부 좌표의 공통 소비

- 원본 HWPX의 pi53·pi54는 누적 모델 좌표 **69687/72387HU**와 쪽 내부 원본 좌표 **14554/17254HU**를 함께 보유합니다. 차이는 모두 **55133HU**이며 pi53 저장 끝과 pi54 시작이 정확히 이어집니다. 첫 조각 프레임의 `(호스트 시작 50309 + 세로 오프셋 10823 + 선언 높이 8555)HU`도 pi53의 누적 시작 **69687HU**와 일치합니다. 이 두 독립 식이 맞는 미편집 저장 경로에서만 첫 물리 프레임과 뒤 빈 문단의 lazy 기준을 연결했습니다. 내용 컷·테스트 기대값·허용치는 바꾸지 않았습니다.
- 수정 후 HWPX 4쪽 첫 컷과 5쪽 시작 컷은 `[3]`, 5쪽 표 **194.3px**, pi53 첫 줄 **290.1px**, pi54 첫 줄 **326.1px**로 HWP 경로와 동일합니다. Native 전체 5쪽 일치율은 **94.97706/98.41015/99.27944/87.65990/88.21033%**로 5쪽이 보정216의 **69.16541%**에서 개선됐으나 4·5쪽은 **90% 미달**입니다. HWP 같은 쪽은 **91.16597/90.42361%**입니다. render tree의 문단·표 외곽은 두 경로가 같지만 표 안 별표 글줄은 HWPX가 4·5쪽에서 **2.1~3.3px 위**이며, pi50 마지막 행 원본 첫 `vpos`도 HWPX **0HU** 대 HWP **500HU**입니다. 다음에는 셀 정렬의 저장 첫 줄·문단 위 간격 회계를 검토합니다. 증적은 `output/pr-review/planet6897-7382-20260926/stage220-{full-visual,hwp-visual,visual}/`입니다.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_1749_saved_bounds_page_break) | test(issue_3930_hwpx_hwp_save_layout)' --no-fail-fast`: **5PASS/1FAIL**. 남은 실패는 기존 `issue_1811_hwpx_pi52_rowbreak_cut_matches_hwp_reference`가 `dump_page_items`에서 `PartialParagraph pi=52`를 요구하기 때문입니다. 실제 4쪽 render tree에는 pi52 본문 4줄과 표가 모두 있고 dump는 `PartialTable pi=52` 한 항목으로 표현합니다. 90% 시각 기준을 채우기 전에는 검사를 완화하거나 fixture를 갱신하지 않습니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage220-nextest.log`입니다. 최종 검증·PR 준비는 미완료입니다.

## 보정221 — HWPX 셀 문단 간격 후보의 비적용 확인

- 4쪽 pi50·5쪽 pi57의 가운데 정렬 별표 셀은 HWPX에서 `content=286.67/96.80px`, 첫 줄 `vpos=0HU`, HWP에서 `content=293.33/103.47px`, 첫 줄 `vpos=500HU`입니다. 선언 셀 높이·패딩과 표 외곽은 같습니다. 첫 문단 스타일의 위 간격 **500HU=6.67px**가 HWP의 저장 줄 사다리에 포함되어 있고 HWPX의 리셋된 사다리에서는 빠져 있다는 가설입니다. 계측 로그는 `output/pr-review/planet6897-7382-20260926/stage221-{hwpx,hwp}-lead.log`입니다.
- 다문단·가운데 정렬·미편집 HWPX TAC 셀에서 첫 위 간격을 내용 측정·배치에 함께 주는 후보를 실행했으나 Native 4·5쪽 점수는 **87.65990/88.21033%로 불변**이었습니다. 적용 조건이나 실제 렌더 경로가 예상과 다르므로 후보는 전부 되돌렸습니다. `stage221-visual/`의 PNG를 보존하며 다음 단계에서 셀 분할/전체 배치 호출 경로와 각 조건의 실제 값을 확인합니다. 동작 변경·테스트 변경은 없습니다.

## 보정222 — HWPX 합성 셀 사다리의 첫 위 간격과 한컴돋움 실폰트

- 실제 pi50·pi57 경로는 `row_filter=None`, 미편집 HWPX TAC, 가운데 정렬, 다문단 셀입니다. HWPX의 각 첫 줄은 `vpos=0`이고 원본 XML에도 줄 세그먼트가 있으나 파서가 구현 속성 비트를 붙였으며 `source_line_seg_vertical_pos`는 없습니다. 보정221의 비적용은 이 비트를 원본 저장 앵커 배제 조건으로 사용한 결과였습니다. 이 줄 사다리는 **좌표 앵커로 신뢰하지 않고**, 문단 모양에 남은 첫 `spacing_before=500HU`를 선언 셀에 들어가는 가운데 정렬 콘텐츠의 높이와 첫 줄 시작에 함께 반영했습니다. 같은 문서의 HWP 저장본은 첫 `vpos=500HU`와 누적 사다리로 같은 간격을 이미 담습니다. 계측은 `output/pr-review/planet6897-7382-20260926/stage222-route*.log`에 있습니다.
- HWPX와 HWP의 별표 글줄·표·본문 render tree가 동일해진 뒤에도 SVG 글꼴명이 `한컴돋움` 대 `Haansoft Dotum`으로 갈렸습니다. 설치된 `/Users/tsjang/Library/Fonts/HDOTUM.TTF`의 이름 테이블은 두 이름을 같은 face로 기록하고 `fc-match`도 두 이름을 그 파일에 연결합니다. SVG의 `한컴돋움` 로컬 후보 첫 자리에 실제 영문 face를 추가했습니다. 이 변경은 표시 이름을 없애지 않고 올바른 설치 파일을 우선합니다.
- Native 전체 5쪽 Visual Sweep 일치율은 **94.97706/98.86521/99.51565/91.11617/90.57406%**, 최저 **90.57406%**입니다. 4·5쪽 review를 직접 확인했고 표 외곽·본문 순서·쪽수 5쪽을 보존했습니다. residual은 주로 글리프 외곽과 마지막 분할 표 내부의 작은 줄 차이입니다. `output/pr-review/planet6897-7382-20260926/stage222-full-visual/`에 전쪽 PNG·manifest를 보존했습니다. 이 단계는 Native 기준 통과이며 **fresh WASM·관련 회귀 갱신·최종 전체 검증 전**이므로 PR 승인 판정은 아직 보류합니다.

## 보정223 — #1749 fresh WASM 전쪽 확인

- 보정222 코드 head `a3f429bb0`에서 저장소 루트의 `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`가 성공했습니다. 로컬 대체 빌드이며 Docker 최적화 빌드 통과로 보고하지 않습니다. 생성한 `pkg/rhwp_bg.wasm`의 SHA-256은 `a23f13f6e7d5fea416df9406bb36e431b8fc272551f86ef7a1d51b56d156fc0f`입니다. `visual_sweep.py --wasm-pkg pkg`의 manifest는 WASM `getPageRenderTree`와 패키지 두 파일의 해시를 기록합니다.
- fresh WASM 전체 5쪽 일치율은 Native와 동일한 **94.97706/98.86521/99.51565/91.11617/90.57406%**이며 `rhwp_004.png`·`rhwp_005.png`의 SHA-256도 Native와 각각 같습니다. 페이지 수는 양쪽 **5쪽**입니다. `output/pr-review/planet6897-7382-20260926/stage223-wasm-build.log`, `stage223-wasm-visual/`에 결과를 보존했습니다. 기존 #1811의 `PartialParagraph` 항목 전제는 여전히 실패하므로, 이제 검사를 실제 4쪽 host 텍스트와 4·5쪽 표 내용·순서로 갱신합니다. 전체 PR 검증은 미완료입니다.

## 보정224 — #1811 기존 회귀의 항목 이름 전제 교정

- 기존 검사는 `dump_page_items`에 별도 `PartialParagraph pi=52` 항목이 있어야 한다고 요구했지만, 현재 render tree는 4쪽의 host 본문 **4줄**을 표보다 앞에 실제 표시하고 4·5쪽 표를 `PartialTable` 항목으로 나눕니다. Native/fresh WASM 전쪽 최저 **90.57406%**와 직접 review를 선행 근거로 삼아, **기존 함수만** render tree의 내용·순서·쪽별 별표 글줄 소유 검사로 고쳤습니다. 원본 HWP/HWPX·독립 PDF·기존 HWP 물리 프레임 검사는 보존했고 새 검사 함수나 픽셀 좌표 기대값은 추가하지 않았습니다.
- `cargo fmt --all -- --check`와 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_1749_saved_bounds_page_break) | test(issue_3930_hwpx_hwp_save_layout)' --no-fail-fast`는 **6PASS/0FAIL**입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage224-nextest.log`입니다. 이후 전체 nextest·lint·최종 head 시각 검증이 남아 있어 PR 준비 판정은 하지 않습니다.

## 보정225 — 현재 head 전체 회귀 실패 19건 재고정

- 코드·기존 검사 보정 head `30dca6c5f`에서 지정된 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast`를 완료했습니다. **10,221실행 / 10,202PASS / 19FAIL / 50SKIP**, 종료 코드 **100**, 로그는 `output/pr-review/planet6897-7382-20260926/stage225-full-nextest.log`입니다. 보정207의 27FAIL에서 제거된 차단 함수 8건과 교정된 #1811 1건을 반영하되, 새로 실패한 **#1658** 1건을 포함합니다. 이전 녹색 결과를 이 head에 재사용하지 않습니다.
- 남은 실패는 #6569·#7049(같은 **7쪽** 원본), #6632, #6795, 본문초과 래칫2, #7203, 텍스트겹침 래칫2, #2287, #6651, #2020, #6660, #7226 두 건, #6648, chart overlap, #1658, #6681입니다. 다음 단계는 #1658이 이번 변경의 실제 회귀인지 먼저 대조하고, 이후 7쪽 #6569·#7049를 독립 PDF 전쪽 Visual Sweep으로 판단합니다. 어느 실패도 아직 승인·제외로 판정하지 않았습니다.

## 보정226 — #1658 표 뒤 문단의 재발 범위 확인

- 현재 head의 한 쪽 원본과 독립 한컴 PDF를 Native Visual Sweep으로 다시 비교했습니다. **1/1쪽, 실루엣 99.39628%, gate 통과**이지만, 실제 review에서 표 뒤 `끝.`이 PDF보다 약 6px 위에 있습니다. 기존 검사는 같은 문단의 기준선 **620.013px / PDF 626.400px**로 FAIL하고, 본문 표 외곽은 이전 정상 위치를 유지합니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage226-issue1658-visual/issue1658-stage226-native/review/review_001.png`입니다. 점수 통과만으로 위치 차이를 승인하지 않습니다.
- #1749용 `HeightCursor` 원본 연속 기준이 글자 없는 표 문단까지 적용된다는 가설을 세우고, 임시로 직전 문단에 개체가 없는 조건을 추가했습니다. 다시 만든 한 쪽의 점수 **99.39628%**와 끝 문단 위치가 모두 불변이므로 이 후보는 **기각하고 원상복구**했습니다. 후보 출력은 `output/pr-review/planet6897-7382-20260926/stage226-issue1658-fixed/`에 있습니다. 테스트·기준값·공차는 수정하지 않았습니다. 다음 단계에서 #1749 셀 측정 변경의 표 뒤 흐름 소비 여부를 분리하겠습니다.

## 보정227 — #1658의 HWPX 셀 첫 간격 후보 분리

- #1749용 HWPX 가운데 정렬 TAC 다문단 셀 첫 위 간격 보정을 **임시로 비활성화**하고 #1658의 render tree를 재생성했습니다. 표 뒤 `끝.`은 여전히 **y=606.4px**이며 현재 head와 같고, 이전 정상 Stage206의 **y=612.8px**로 돌아오지 않습니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage227-no-cell-lead/render_tree_001.json`입니다. 이 후보도 기각하고 코드를 원상복구했습니다.
- 지금까지의 국소 비활성화 둘은 원인을 입증하지 못했습니다. 다음 단계는 보정207의 실제 소스 `51e7c1bec`를 별도 검토 작업트리에서 같은 원본·같은 명령으로 실행해 좌표 차이가 도입된 정확한 코드 경계를 확인하는 것입니다. 기존 #1658 검사와 PDF 좌표는 유지합니다.

## 보정228 — #1658 차이가 나타나는 head 범위 확인

- 분리 작업트리에서 동일 원본을 `51e7c1bec`와 보정220 `46a3bd33b` 각각의 실행 파일로 `export-render-tree`했습니다. `51e7c1bec`의 본문 표 pi4 상단은 **411.5px**, 뒤 `끝.` pi5 줄 상단은 **612.8px**입니다. `46a3bd33b`는 표 상단 **411.5px**를 유지하면서 `끝.`만 **606.4px**가 됩니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage228-{control,stage220}-render-tree/`에 보존했습니다. 독립 PDF의 `끝.` 기준선은 **626.400px**입니다.
- 이 비교만으로 두 head 사이 어느 변경이 원인인지는 확정하지 못했습니다. 보정222의 셀 첫 위 간격은 원인이 아닙니다. 다음 단계는 중간 코드 경로를 분리해 실제 적용 분기를 찾는 것입니다. 기준 PDF·기존 검사·허용치는 유지합니다.

## 보정229 — #1658 뒤 문단 원점 분기 분리

- 분리 작업트리의 보정220 소스에서 `HeightCursor` 한 파일만 이전 정상 `51e7c1bec` 버전으로 되돌려 실행했습니다. 본문 표 상단 **411.5px**, `끝.` 상단 **606.4px**로 보정220과 동일합니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage229-old-cursor-render-tree/`입니다. 이전 커서 코드를 쓰더라도 회복되지 않으므로, 다른 변경을 분리합니다. 후보 코드는 현재 작업 브랜치에 반영하지 않았고 별도 작업트리에서도 복원했습니다.

## 보정230 — #1658 후행 간격 손실의 실제 원인

- 보정220의 표 조각 `emit.rs`와 커서를 각각, 이어 둘 다 이전 정상 코드로 바꿔도 `끝.`은 **606.4px**였습니다. 반면 보정216의 `stored_tac.rs`까지 이전 버전으로 바꾸자 **612.8px**로 회복됐습니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage230-{old-emit,old-both,old-tac}-render-tree/`입니다. 보정216은 `prepare_computed`가 표 뒤 저장 줄간격을 전량에서 절반으로 줄였고, #1658의 **960HU/2=6.4px** 차이와 정확히 일치합니다.
- #1658 원본에서 표 pi4의 저장 시작 **29445HU**, 줄 높이 **14139HU**, 뒤 간격 **960HU**의 합은 다음 `끝.` pi5 시작 **44544HU**와 같습니다. 따라서 이 경계에서 간격 전량을 보존해야 합니다. #1749의 다음 pi51은 빈 문단이므로 같은 결론을 적용할 수 없습니다. 문서 ID가 아닌 **다음 가시 문단의 저장 줄 시작과 직전 줄 끝의 연속성**으로 두 경로를 구분한 뒤, 원본 둘의 전체 시각 출력과 기존 회귀를 검증합니다.

## 보정231 — 다음 가시 줄의 저장 연속성으로 TAC 후행 간격 보존

- `stored_tac::prepare_computed`는 다음 문단이 실제 글자를 갖고 저장 시작이 `현재 시작+줄 높이+후행 간격`과 정확히 이어질 때만 **간격 전량**을 표 뒤 흐름에 전달합니다. 그 외 합성 줄은 기존 공통 TAC의 **절반 간격**을 유지합니다. `try_place_stored_tac_paragraph`가 이미 가진 문단 목록에서 다음 문단을 전달하므로 측정·수용·실제 흐름 끝은 같은 `placement.end`를 소비합니다. #1658 가시 후속 문단은 적용, #1749 pi50 뒤 빈 pi51은 비적용입니다. 문서별 예외·픽셀 상수·검사 공차 변경은 없습니다.
- 수정 전 전체 회귀에서 #1658 기존 함수는 `끝.` 기준선 **620.013px**로 FAIL했습니다. 수정 후 render tree의 `끝.` 줄 상단은 **606.4→612.8px**, 기준 PDF의 기준선 **626.400px**에 다시 맞고 기존 #1658 3함수 모두 PASS입니다. Native Visual Sweep은 원본/PDF **1/1쪽**, **99.46309%**, gate 통과이며 review에서 표 외곽·끝 문단·하단 틀을 직접 확인했습니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage231-issue1658-visual/`입니다.
- #1749 HWPX의 Native 전체 **5/5쪽** 일치율은 **94.97706/98.86521/99.51565/91.11617/90.57406%**, 최저 **90.57406%**로 이전 결과를 유지합니다. 4·5쪽 review에서 분할 표, 뒤 문단과 쪽 소유를 확인했습니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage231-hwpx1749-visual/`입니다. 기존 #1749 3함수와 #3930 3함수도 PASS입니다.
- `cargo fmt --all` 뒤 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_1658_page_bottom_fixed_exclusion) | test(issue_1749_saved_bounds_page_break) | test(issue_3930_hwpx_hwp_save_layout)' --no-fail-fast`는 **9PASS/0FAIL**, 종료 코드 **0**입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage231-focused-nextest.log`입니다. 비교용 임시 작업트리는 제거했습니다. fresh WASM·전체 회귀·필수 Rust lint는 아직 이 코드 head에서 검증하지 않았습니다.

## 보정232 — 보정231 head의 fresh WASM 재현

- 코드 head `0a83091ff`에서 저장소 루트의 `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`가 종료 코드 **0**으로 완료됐습니다. 이는 Mac 로컬 대체 빌드이며 Docker 최적화 빌드 결과가 아닙니다. `pkg/rhwp_bg.wasm` SHA-256은 `6a724904910b0bb11599a3921f70462d8e709d079728d1901d419ff4dfde5ac3`입니다. 빌드 로그는 `output/pr-review/planet6897-7382-20260926/stage232-wasm-build.log`입니다.
- fresh WASM Visual Sweep은 #1658 **1/1쪽 99.46309%**, #1749 **5/5쪽 94.97706/98.86521/99.51565/91.11617/90.57406%**로 양쪽 gate가 통과했습니다. Native와 각 페이지 rhwp PNG SHA-256이 모두 같습니다. #1658 overlay와 #1749 5쪽 review를 직접 판독해 본문 표 뒤 `끝.` 위치와 이어받기 표·뒤 문단을 확인했습니다. 전체 산출물은 `output/pr-review/planet6897-7382-20260926/stage232-{issue1658,hwpx1749}-wasm/`입니다. 대표 PNG를 아래에 보존합니다. 전체 회귀와 필수 Rust lint는 아직 이 head에서 완료하지 않았습니다.

![#1658 fresh WASM 한 쪽 비교](../assets/pr7382_20260926/stage232_gwanak1658_wasm_review_001.png)

![#1658 fresh WASM 겹침](../assets/pr7382_20260926/stage232_gwanak1658_wasm_overlay_001.png)

![#1749 fresh WASM 4쪽 비교](../assets/pr7382_20260926/stage232_savedbounds1749_wasm_review_004.png)

![#1749 fresh WASM 5쪽 비교](../assets/pr7382_20260926/stage232_savedbounds1749_wasm_review_005.png)

![#1749 fresh WASM 5쪽 겹침](../assets/pr7382_20260926/stage232_savedbounds1749_wasm_overlay_005.png)

## 보정233 — #6569·#7049 공통 7쪽 원본의 시각 선행 판정

- 기존 #6569의 제목 칸 검사와 #7049의 표 전용 줄 검사가 사용하는 원본 `samples/issue6542/156678235_mid_para_vpos_rewind.hwp`를 독립 `pdf/issue6542-156678235-mid-para-vpos-rewind-2020.pdf`와 Native 전쪽 비교했습니다. 원본/PDF는 **7/7쪽**이고 페이지별 2px 이웃 관용 내용 실루엣 점수는 **73.97440/88.68655/86.45281/80.74023/93.67208/93.26348/92.10454%**입니다. 최저가 **73.97440%**라 gate는 `re_review_required`입니다. 산출물은 `output/pr-review/planet6897-7382-20260926/stage233-issue6542-visual/`입니다.
- 1쪽 review를 직접 확인하면 제목 상자와 뒤 본문의 기준 위치·글줄이 PDF에서 어긋나며, 4쪽 review에는 차트 색·막대와 표 안 글자의 차이가 함께 있습니다. 기존 #6569 함수는 제목 상단 괘선을 기대 y≈227.8px에서 못 찾고 실제 수평선 목록에 **211.7867px**가 있으며, #7049는 같은 원본의 4쪽 표를 절대 y=537.30px에 묶습니다. 이 단계에서는 어느 검사도 완화·제거하지 않습니다. 7쪽 문서이므로 사용자 지침대로 이 브랜치에서 전쪽 최저 **90% 이상**을 먼저 달성한 뒤, 기존 검사를 내용·소속·관계 중심으로 재검토합니다. 후속 분석에서는 1쪽 제목 상자 원점과 4쪽 차트/표 차이를 독립적으로 분리합니다.

## 보정234 — 저장 TAC 표 연속성의 후보 가설

- 1쪽 원본의 첫 표 pi0 마지막 줄 시작 **180HU** + 높이 **4519HU** + 후행 간격 **1200HU** = 다음 pi1 시작 **5899HU**입니다. pi1 시작 **5899HU** + 높이 **2797HU** + 후행 간격 **1200HU** = 제목 표 pi2 시작 **9896HU**도 정확합니다. 두 다음 문단은 글자는 없어도 TAC 표를 담습니다. 각 후행 간격의 절반 **8px**이 제목 표 상단의 기대/현재 차이 약 **16px**와 맞는다는 후보 가설을 세웠습니다. 다만 이 원본은 **HWP**이고 보정231의 `prepare_computed`는 **HWPX 전용**임을 후속 실행에서 확인했습니다. 이 두 값의 우연한 일치를 원인 증거로 사용하지 않습니다.
- PDF 첫 제목 글자 상단은 `pdftotext -bbox-layout`의 **190.0523pt=253.40px**, 현재 render tree는 **234.9px**입니다. 표 내부 첫 글줄과 표 상단의 상대 간격은 약 **23.1px**로 기존 #6569의 독립 기대 **23.57px**에 가깝지만, 표 전체 원점이 이동해 절대 괘선 탐색이 실패합니다. HWPX 전용 분기의 적용 여부를 확인하지 못한 초기 분석이며, 테스트 기준값은 바꾸지 않았습니다.

## 보정235 — HWPX 전용 간격 후보 기각과 실제 HWP 경로 확인

- `prepare_computed`의 다음 문단 검사에 TAC 표를 포함한 임시 후보를 적용하고 7쪽 Native Visual Sweep을 재실행했지만, **73.97440/88.68655/86.45281/80.74023/93.67208/93.26348/92.10454%**로 전쪽이 완전히 불변이었습니다. 이 함수는 시작에서 HWPX 저장 프로필이 아니면 반환하므로 HWP 원본에는 적용되지 않습니다. 후보 코드를 원상복구했고 출력은 `output/pr-review/planet6897-7382-20260926/stage235-issue6542-visual/`에 보존했습니다.
- HWP 원본의 `RHWP_DIAG_TAC=1 dump-pages`는 pi0에서 표 뒤 간격 **16.0px 전량**을 소비해 `cur_h=78.7px`로 종료하지만, 다음 pi1 표 진입은 **64.6px**입니다. pi1도 **117.9px**로 종료한 뒤 pi2 진입은 **113.6px**입니다. 따라서 빠진 간격은 TAC 단축의 절반 계산이 아니라 **문단 사이 저장 vpos 원점 재조정**에서 발생합니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage235-hwp-tac-diag.log`입니다. 다음 단계는 이 되감기의 생산·측정·배치 소비를 추적합니다. 기존 검사와 기준 PDF는 유지합니다.

## 보정236 — HWP TAC 높이 상한의 실제 되감기 확인

- `section.rs` 각 배치 단계의 임시 계측으로 되감기는 다음 문단 진입 전이 아니라 **현재 표 문단의 `controls::reconcile_tac_height` 안**임을 확인했습니다. pi0의 `DIAG_TAC_END`는 **78.7px**이지만 `place_paragraph_flow` 반환 시 **64.56px**이고, pi1은 **117.9→113.63px**, pi2는 **548.5→543.76px**입니다. `RHWP_DIAG_TACCAP=1`의 pi0 `cap=64.6 < fmt.total_height=78.7`, pi1 `cap=49.1 < 53.3`, pi2 `cap=430.1 < 434.9`와 정확히 대응합니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage236-hwp-{phases,cap}.log`입니다. 임시 계측 코드는 전부 제거했습니다.
- `tac_reconcile::measure`는 표 줄의 `effective_h + tac_host_trailing_spacing`을 모아 cap을 정하고, 뒤 `commit_tac_capped_bottom`이 실제 흐름 끝을 그 cap으로 바꿉니다. HWP5 저장 TAC 줄의 후행 간격 절반과 여러 저장 줄 중 표 소유 줄 선택이 개입합니다. 이 원본 pi0 마지막 저장 줄 끝은 pi1 시작과, pi1 줄 끝은 pi2 시작과 **HU 단위로 연속**이고 둘 다 다음에 실제 표가 있습니다. 다음 단계는 이 연속성의 일반 조건 아래 실제 배치 끝을 cap으로 삭제하지 않는 후보를 시험하고, 7쪽 전체 및 #1658·#1749 정상 대조를 확인합니다. 표 외곽·본문 위치는 독립 PDF로 판단하며 현재 테스트 기대값을 바꾸지 않습니다.

## 보정237 — paginator 상한 단독 수정의 출력 비적용

- HWP5 저장 줄이 다음 표의 첫 줄까지 정확히 연속하면 `tac_reconcile`의 cap을 `fmt.total_height` 이상으로 두는 후보를 시험했습니다. 진단에서 pi0 cap은 **64.6→78.7px**, pi1은 **49.1→53.3px**가 되고 `current_height`의 되감기도 사라졌습니다. 그러나 Native render tree의 제목 표 pi2 상단은 **211.8px로 불변**, 전체 7쪽 일치율도 **73.97440/88.68655/86.45281/80.74023/93.67208/93.26348/92.10454%로 불변**입니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage237-{cap.log,issue6542-visual/}`입니다.
- 실제 `layout.rs`의 TAC 표 배치는 paginator의 최종 `current_height`가 아니라 표별 `inline_placement.advance_end`를 별도로 소비할 수 있습니다. 이 후보는 **측정과 출력이 공유되지 않아 기각**했고 코드를 원상복구했습니다. 다음 단계에서는 `표 flow placement 생산 → TAC cap 소비 → render tree의 advance_end`를 같은 경계에서 대조한 뒤 공통 결과로 수정합니다. 실패 회귀·기준값은 유지하며 이 7쪽은 여전히 보류입니다.

## 보정238 — TAC 줄 소유 선택과 실제 layout 끝점 대조

- 원상복구한 코드의 `RHWP_TABLE_DRIFT=1 export-render-tree`에서 1쪽 pi0 표는 **98.2~150.9px**, 해당 항목 뒤 `LAYOUT_Y`는 **162.7px**입니다. pi1은 **166.5~196.2px → 뒤 208.0px**, pi2는 **211.8~622.1px → 뒤 634.4px**입니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage238-layout-drift.log`와 `stage238-render-tree/`입니다. 보정237이 paginator cap을 바꿔도 이 render tree 좌표는 모두 불변이었습니다.
- `tac_reconcile::measure`는 표 순번 `tac_idx`를 저장 LineSeg 인덱스로 쓰지만, `layout.rs`는 HWP5 저장본에서 `control_line_seg_index`로 컨트롤의 실제 소유 줄을 찾습니다. pi0은 **표 ci3·저장 줄 2개**여서 두 선택이 다를 수 있고, layout의 표 뒤 간격도 `tac_host_trailing_spacing`으로 별도 산출됩니다. 다음에는 표 소유 줄 사영 결과를 계측해 동일한 줄/간격을 측정·배치에서 소비하도록 한 뒤 전쪽 시각으로 검증합니다. 현재 코드/검사 변경은 없습니다.

## 보정239 — 첫 표의 실제 소유 저장 줄 확인

- `tac_reconcile::measure`의 임시 계측에서 1쪽 첫 표 **pi0 ci3**는 TAC 표 순번 **0**, `control_line_seg_index`의 실제 줄은 **1**, 저장 줄 수 **2**입니다. 뒤 pi1·pi2는 각각 순번과 실제 줄이 모두 **0**입니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage239-tac-owner.log`입니다. 즉 첫 표의 상한 측정이 표 앞 별도 저장 줄을 표 줄로 오인한 사실을 확인했습니다. 임시 계측은 제거했습니다.
- 다음 코드 후보는 HWP5 저장 줄에서 측정과 layout이 **같은 표 소유 줄**을 선택하게 하고, 그 마지막 저장 줄 끝에 실제 다음 표가 정확히 이어질 때만 후행 간격을 전량 보존합니다. 합성·편집 줄과 빈 후속 문단은 이 근거에 해당하지 않습니다. 실제 출력과 분할 결과를 전쪽 비교한 뒤 수용 여부를 결정합니다.

## 보정240 — HWP5 TAC 표의 저장 줄 소유와 후행 간격 통일

- `tac_reconcile::measure`가 HWP5 저장본에서 표 순번 대신 실제 컨트롤 소유 LineSeg를 선택하게 했습니다. 저장된 마지막 줄의 끝과 다음 문단의 첫 줄 시작이 `vertical_pos + line_height + line_spacing`으로 정확히 이어지고 다음 문단에도 표가 있을 때만, 측정과 layout 모두 후행 간격 전량을 소비합니다. 편집·재조판 줄과 이 조건에 맞지 않는 문단은 기존 경로를 유지합니다. 근거는 보정235~239의 저장 HU, paginator cap, render tree 실측입니다.
- 변경 전 Native 7쪽 점수는 **73.97440/88.68655/86.45281/80.74023/93.67208/93.26348/92.10454%**였고, 변경 후 **93.49404/88.68655/86.45281/91.01231/96.31566/93.26348/92.10454%**입니다. 1쪽 +19.52pp, 4쪽 +10.27pp이며 2·3쪽은 불변입니다. 입력 7쪽과 기준 PDF 7쪽, 모든 비교 PNG를 산출했고 1·2·3·4쪽 review를 직접 판독했습니다. 1·4쪽의 큰 표/문단 위치 차이는 줄었지만 2·3쪽이 각각 **88.69%, 86.45%**라 전체 gate는 여전히 `re_review_required`입니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage240-issue6542-visual/`입니다.
- 2·3쪽의 글꼴 원인을 분리하려고 설치된 `/Users/tsjang/Library/Fonts`의 실제 글꼴로 `--embed-fonts=full`을 사용한 두 쪽 재출력을 했습니다. 로컬 글꼴 모드 **88.68655/86.45281%**에서 임베딩 후 **88.74580/86.46883%**로 거의 변하지 않았습니다. 따라서 단순 폰트 공급 누락으로 90% 미만을 설명할 수 없습니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage240-issue6542-embedded/`입니다. PDF와 설치 글꼴의 실제 face 버전·행 안 좌표 차이는 다음 단계에서 분석합니다.
- 기존 #6569·#7049 회귀 검사와 PDF 기준값은 아직 변경하지 않았습니다. 전쪽 최저 90%에 도달하기 전에는 이 문서를 새 회귀 근거로 수용하지 않습니다.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_6569) | test(issue_7049) | test(issue_1658) | test(issue_1749)' --no-fail-fast`는 **14/14 PASS**(exit 0)입니다. 여기에는 대상 기존 검사와 HWPX #1658·#1749 대조군이 포함됩니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage240-focused-nextest.log`입니다. 이 통과를 2·3쪽 시각 일치의 근거로 사용하지 않습니다.

## 보정241 — 2·3쪽의 글꼴 버전과 래스터 경로 분리

- 독립 PDF에 임베드된 `BatangChe` subset은 TrueType revision **5.02**이고 설치된 `/Users/tsjang/Library/Fonts/batang.TTC`의 `BatangChe` face는 **2.21**입니다. 이전 검증에서 보유한 revision 5.02 `batang.ttc`·`gulim.ttc`를 `output/pr-review/planet6897-7382-20260926/stage241-font-supply/`에 실제 파일로 복사해 `--font-path`와 `--embed-fonts=full`로 2·3쪽을 재출력했습니다. 점수는 **88.52603/86.46883%**여서 90% 미만입니다. 두 글꼴의 revision이 같다는 사실만으로 PDF subset과 실제 윤곽·사용 face가 같다고 단정하지 않습니다. `embedded_font_check.json`에는 각 쪽의 바탕체·굴림·맑은 고딕 cmap 검사 통과가 기록됐습니다.
- 같은 코드·원본·PDF를 `--svg-rasterizer rsvg`로 재비교한 점수는 **88.42859/87.00390%**입니다. 기본 webfont의 **88.68655/86.45281%**와 차이는 작고 두 경로 모두 보류입니다. 결과는 `output/pr-review/planet6897-7382-20260926/stage241-issue6542-{v502,rsvg}/`와 동명 `.log`에 있습니다. SVG 3쪽 첫 줄 x 시작은 **86.79px**, PDF `pdftotext -bbox-layout`의 **63.60pt=84.80px**이고, 끝도 약 **718px**로 가깝습니다. 큰 글줄 이동보다는 실제 글리프 윤곽·잉크 범위를 다음 단계에서 조사합니다.
- 보정241에서는 렌더러·기존 회귀·기준 PDF를 수정하지 않았습니다. 전쪽 90% 조건과 #7382 보류는 유지합니다.

## 보정242 — 제목 TAC 표 뒤의 저장 줄 연속성 적용

- 2쪽 제목 표 pi17의 저장 줄은 `53960 + 3147 + 1260 = 58367HU`이고, 바로 뒤 빈 문단 pi18의 첫 줄이 **58367HU**에서 시작합니다. 3쪽 제목 표 pi28도 `45000 + 3147 + 1120 = 49267HU`이고, 뒤 pi29의 첫 줄이 **49267HU**입니다. 이전 분기는 다음 문단에 다시 표가 있을 때만 후행 간격 전량을 적용해 이 두 빈 문단 앞에서 각각 약 **8.4px**, **7.5px**을 버렸습니다. 다음 저장 줄이 정확히 연속하는 조건으로 수정해 측정과 실제 layout이 같은 간격을 소비하게 했습니다. 편집·합성 줄은 기존 경로로 남습니다.
- Native 전체 **7/7쪽**을 기존 독립 한컴 2020 PDF와 다시 비교한 2px 이웃 관용 내용 실루엣 점수는 **98.66535/97.29151/99.96978/99.93791/99.94973/100.00000/97.67052%**이고 gate는 `passed`입니다. 입력과 출력 쪽수 모두 7쪽입니다. 1~7쪽 review contact sheet와 1·2·3·4·7쪽 개별 review를 직접 확인했으며 제목 표 다음 문단, 표 외곽, 문서 끝 내용의 위치·누락을 대조했습니다. 산출물은 `output/pr-review/planet6897-7382-20260926/stage242-issue6542-visual/`입니다. 잉크 픽셀 자체의 차이는 남으므로 100% 동일한 래스터 출력이라는 뜻은 아닙니다.

![#6569 1쪽 제목 표 비교](../assets/pr7382_20260926/stage242_issue6542_native_review_001.png)

![#7049 4쪽 표 비교](../assets/pr7382_20260926/stage242_issue6542_native_review_004.png)

![2쪽 제목 표 뒤 문단 겹침](../assets/pr7382_20260926/stage242_issue6542_native_overlay_002.png)

- 대상 기존 회귀 #6569·#7049와 대조군 #1658·#1749의 focused nextest는 **14/14 PASS**(exit 0)입니다. 명령은 보정240과 같고 로그는 `output/pr-review/planet6897-7382-20260926/stage242-focused-nextest.log`입니다. 기존 검사의 절대 y 고정은 다음 단계에서 의미 관계 검사로 교체하고, fresh WASM 전쪽 비교도 별도 수행합니다. Native 시각 통과만으로 최종 PR 승인을 선언하지 않습니다.

## 보정243 — fresh WASM 전쪽 및 기존 회귀의 의미 검사

- 보정242 코드 커밋 `1f69e8909`에서 `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`를 실행해 fresh WASM을 만들었습니다(로컬 no-opt 대체 빌드, exit 0, `rhwp_bg.wasm` SHA-256 `0f111fe0e47180553b941336ddc8e56b6ffd97881ac93017b201fe29a27ef5c1`). `--wasm-pkg pkg`로 한컴 2020 PDF와 **7/7쪽** Visual Sweep을 재실행했고 점수는 Native와 같은 **98.66535/97.29151/99.96978/99.93791/99.94973/100.00000/97.67052%**, gate `passed`입니다. `export-wasm-for-sweep.mjs`가 실제 WASM SVG·render tree를 생성했고 Native/WASM의 rhwp 페이지 PNG SHA-256은 7쪽 모두 같습니다. WASM review contact sheet를 직접 확인했습니다. 산출물은 `output/pr-review/planet6897-7382-20260926/stage243-issue6542-wasm/`입니다.
- 기존 #6569 제목 셀 검사는 상단 괘선의 절대 y 탐색을 제거하고 **1쪽 제목 표 소속·두 글줄 순서·셀 내 첫 문단 여백 관계**를 검사합니다. #7049의 5쪽 중첩 표 검사도 절대 y를 제거하고 **상위 셀·소유 글줄 내부 포함·뒤 본문 순서**를 검사합니다. 독립 PDF의 실제 위치는 위 Native/WASM 전쪽 review가 담당합니다. 두 파일에 대한 focused nextest는 **7/7 PASS**(exit 0)이며 로그는 `output/pr-review/planet6897-7382-20260926/stage243-semantic-nextest.log`입니다.

![fresh WASM 2쪽 제목 표 뒤 문단](../assets/pr7382_20260926/stage243_issue6542_wasm_review_002.png)

![fresh WASM 4쪽 표 비교](../assets/pr7382_20260926/stage243_issue6542_wasm_review_004.png)

- 이 단계는 해당 **7쪽 문서**와 두 기존 회귀의 증거를 닫았습니다. #7382 전체의 나머지 차단 회귀·lint·전수 nextest는 이어서 검증합니다.

## 보정244 — 전체 nextest 결과와 새 페이지 회귀 판별

- head `63a86626b`에서 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast`를 완료했습니다. **10,221개 중 10,196 PASS, 25 FAIL, 50 skipped**, exit **100**이며 로그는 `output/pr-review/planet6897-7382-20260926/stage244-full-nextest.log`입니다. 보정225의 19 FAIL 중 #6569·#7049·#6651·#2020·#6648·#1658 **6건이 해소**됐고, #4068 두 건·#5524·#6653·#5862 세 건·#5863·호환 페이지·oracle 쪽수 세 건 등 **12건이 새로 실패**했습니다. 나머지 13건도 계속 실패합니다. 전체 검증은 통과하지 못했습니다.
- 첫 새 실패 #4068의 원본 `samples/hwpx_sample2.hwp`는 현재 rhwp **30쪽**, 독립 `pdf/hwpx_sample2-hwp-2024.pdf`는 **29쪽**입니다. 현재 렌더 19쪽은 전자계약 문단이고 PDF 19쪽은 주택 소유 기준 표여서 같은 내용을 비교하지 않습니다. 다른 보유 PDF의 19쪽도 같은 표입니다. `pdf/hwpx_sample2-2020.pdf`와의 임시 19쪽 sweep 점수 **38.83619%**는 이 쪽 어긋남이 포함된 값이며, 정상 시각 비교 점수로 사용하지 않습니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage244-hwpxsample2-p19/`입니다. 테스트의 19쪽 셀 전제가 `[]`로 무너진 것은 실제 페이지 증가와 연결해 조사합니다.
- 기존 실패 #6632의 원본 `samples/hwpspec.hwp`는 **178쪽** PDF를 가진 대형 문서입니다. 106쪽 한 쪽 선행 sweep은 **70.54325%**이고 review에서 표·글줄 차이가 보입니다. `pdftotext -bbox-layout`은 이 PDF에서 종료 코드 -6으로 실패해 marker 분석은 미측정입니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage244-hwpspec-p106/`입니다. 이 한 쪽 점수로 전쪽 판정을 대신하지 않고, 현 PR의 차단 여부와 #7445 이관 범위를 추후 별도로 확정합니다.
- 다음 단계는 보정242의 `다음 저장 줄이 연속이면 후행 간격 전량` 가정을 #4068의 쪽수 증가 반례에 대조합니다. 이 단계에서는 실패 검사·기준 PDF·래칫을 수정하지 않았습니다.

## 보정245 — 저장 배치 원점과 분할 예산의 간격 소비 분리

- 보정240 코드 head `6fb3636e6`의 격리 worktree에서 `hwpx_sample2.hwp`는 **29쪽**이었고, 보정242 이후는 **30쪽**이었습니다. 두 `dump-pages --json`을 비교하면 첫 차이는 1쪽 큰 분할 표 pi4의 끝 컷입니다. 이전에는 2행 **cut 32**까지 1쪽에 들어갔지만 보정242는 **1행 끝**에서 잘라 뒤 19쪽의 문서 내용과 테스트 전제를 밀었습니다. `RHWP_DIAG_NATIVE_TAC_NEXT` 임시 계측은 앞선 TAC 표의 저장 줄 연속에서 추가 간격이 발생함을 확인했습니다. 임시 계측은 제거했고 격리 worktree도 정리했습니다. 로그/JSON은 `output/pr-review/planet6897-7382-20260926/stage245-*`입니다.
- 이 과정에서 서로 다른 worktree가 공유하는 `target/pr-review/debug/rhwp`를 Cargo가 오래된 파일로 재사용하는 사례를 확인했습니다. 한 번의 잘못된 **29쪽** 관측은 버리고 현재 worktree 소스를 강제로 재빌드해 재측정했습니다. 이후의 페이지 수·시각 결과는 이 재빌드 바이너리에서 얻었습니다.
- 원인은 저장 좌표의 **배치 원점**에 필요한 전량 간격을 `tac_reconcile::measure`의 **분할 예산**에도 더한 것입니다. 분할 예산은 기존 TAC 표의 물리 점유분을 유지하고, 정확히 이어지는 비텍스트 캐리어(다음 표 또는 내용 없는 저장 줄)의 실제 layout 원점에만 전량 간격을 적용하도록 분리했습니다. 이는 7쪽 문서에서 확인한 저장 HU와 29쪽 문서의 분할 컷 반례를 함께 만족합니다. 원본 ID 분기는 없습니다.
- `hwpx_sample2.hwp`는 다시 PDF와 같은 **29쪽**이고, 1쪽 Native 점수는 전량을 예산에도 더한 후보의 **68.42791%→94.54160%**입니다. 19쪽은 같은 내용·쪽 소속으로 복구됐지만 점수 **88.80412%**라 이 대형 문서 전쪽 시각 완료 근거는 아닙니다. 현재 #4068은 더 이상 차단하지 않아 이 문서를 #7445로 자동 이관하지 않습니다. 7쪽 `issue6542`의 Native 전쪽 점수는 **98.66535/97.29151/99.96978/91.01231/99.94973/100.00000/92.10454%**, gate `passed`입니다. 4쪽의 남은 표/차트 차이는 review에서 직접 확인했고 증적은 `stage245-issue6542-layout-only/`에 있습니다.
- 새 실패군 #4068·#5524·#6653·#5862·#5863·호환/쪽수 검사를 포함한 focused nextest **32/32 PASS**(exit 0)입니다. 명령은 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_4068) | test(issue_5524) | test(issue_6653) | test(issue_5862) | test(issue_5863) | test(compat_2024_changes_pagination) | test(page_counts_do_not_drift_from_hancom_oracle)' --no-fail-fast`이고 로그는 `output/pr-review/planet6897-7382-20260926/stage245-new-failures-nextest.log`입니다. 전체 nextest와 이 코드의 fresh WASM은 후속 단계에서 재검증합니다.

![분할 예산 보정 뒤 29쪽 문서 1쪽](../assets/pr7382_20260926/stage245_hwpxsample2_native_review_001.png)

![7쪽 원본의 현재 4쪽 비교](../assets/pr7382_20260926/stage245_issue6542_native_review_004.png)

## 보정246 — 분할 예산 분리 뒤 전체 회귀 재검증

- 보정245 head `90006da06`에서 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast`를 완료했습니다. **10,221개 중 10,207 PASS, 14 FAIL, 50 skipped**, exit **100**입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage246-full-nextest.log`에만 남겼습니다.
- 보정244의 25개 실패 중 쪽수 증가와 함께 발생한 #4068 두 건, #5524, #6653, #5862 세 건, #5863, 호환 쪽수 및 oracle 쪽수 세 건이 해소됐습니다. #6648 한 건은 보정244에서 통과했으나 이번 전체 실행에서 다시 실패해 별도 원인 확인이 필요합니다. 남은 14건을 하나의 원인으로 묶거나 전체 통과로 보고하지 않습니다.
- 첫 후속 대상은 `samples/hwpspec.hwp`의 #6632 실패 한 건입니다. 이 문서의 독립 PDF는 178쪽이고 106쪽 Native Visual Sweep은 **70.54325%**입니다. 현재 검사는 해당 쪽 글리프의 x·y를 0.7px로 고정하므로 이 근거에서 회귀 계약으로 유지할 수 있는지 개별 판정합니다. 같은 파일의 다른 통과 검사나 원본 문서는 변경 대상으로 삼지 않습니다.

## 보정247 — 178쪽 문서의 잘못 고정된 한 검사 이관

- 검토 head `0f1e646e1`을 Native로 다시 빌드해 `samples/hwpspec.hwp`와 독립 `pdf/hwpspec-2024.pdf`의 **106쪽**을 같은 96dpi에서 직접 대조했습니다. 2px 이웃 관용 내용 실루엣 일치율은 **70.54325%**, gate `re_review_required`입니다. review에서 표·글줄 잔여 차이를 확인했습니다. 이는 178쪽 전체 검증 결과가 아니라 해당 실패 검사 쪽의 선행 판정입니다. 증적은 아래 PNG와 `output/pr-review/planet6897-7382-20260926/stage247-hwpspec-p106/`입니다.
- 실패 검사 `text_after_a_text_and_shape_line_in_a_cell_follows_the_stored_line_height`는 글리프 절대 x/y를 ±0.7px로 고정했고, 이번 전체 회귀의 실제 y는 **536.67px**, 기대값은 **540.7px**입니다. 이 미완성 대형 문서의 위치값을 새 기준으로 바꾸지 않고 해당 검사 **한 건만** 제거했습니다. [#7445 등록 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5920893392)에 원본·PDF SHA-256, 178쪽, 106쪽 점수, 제거 범위와 후속 의미 검사의 조건을 기록했습니다. 추후 문서 피델리티와 관련 페이지의 Native/fresh WASM 일치율을 먼저 개선해야 합니다.
- 원본 HWP와 이미 추적 중인 기준 PDF는 그대로 보존했습니다. 같은 파일의 다른 검사나 다른 문서의 회귀는 변경하지 않았습니다. `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_6632_cell_tac_shape_line_height)' --no-fail-fast`는 남은 `exam_kor.hwp` 그림 관계 대조군 **1/1 PASS**, exit 0입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage247-focused-nextest.log`에 있습니다. 다른 13개 차단은 그대로 남아 다음 단계에서 따로 판정합니다.

![178쪽 원본 중 106쪽 Native와 한컴 PDF 비교](../assets/pr7382_20260926/stage247_hwpspec_native_review_106.png)

## 보정248 — 45쪽 #6795 문서의 절대 하단 검사 이관

- 검토 head `c017a108f`에서 `samples/issue6795/1341000-201100013-cyber-university-application.hwp`의 분할 조각·형제 표·후속 표가 있는 **31~33쪽**을 독립 한컴 2020 PDF와 Native Visual Sweep으로 비교했습니다. 점수는 **66.99083/76.57976/98.23984%**, gate `re_review_required`입니다. 31쪽 review에서 표 내부 행 높이·글줄이 기준 출력과 크게 다릅니다. 이는 45쪽 전체 일치율을 측정한 결과가 아닙니다. 증적은 아래 PNG와 `output/pr-review/planet6897-7382-20260926/stage248-cyber6795/`입니다.
- 차단 검사 `split_fragment_page_holds_only_the_fragment`는 분할 조각이 혼자 있다는 계약에 과거 하단 **560.1px ±8px**를 결합했습니다. 현재 같은 쪽에는 해당 조각만 있고 형제 표는 다음 쪽에 있지만 하단은 **778.2px**입니다. 31쪽이 90% 미만인 상태에서 어느 좌표도 새 정답으로 고정하지 않고, 이 실패 함수 **한 건만** 제거했습니다. [#7445 추가 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5920926127)에 원본·PDF 해시, 45쪽, 관련 쪽 점수와 후속 피델리티 검증 조건을 기록했습니다.
- 원본 HWP·기준 PDF 및 #6795의 다른 네 검사는 유지했습니다. `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_6795_split_float_sibling_gets_its_own_page)' --no-fail-fast`는 형제의 자기 쪽 소유·본문 포함, 다음 표 순서, 겹침 금지, #2813 대조군 **4/4 PASS**, exit 0입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage248-focused-nextest.log`입니다. 다른 차단은 다음 단계에서 개별 판정합니다.

![45쪽 원본 중 분할 조각 31쪽 Native와 한컴 PDF 비교](../assets/pr7382_20260926/stage248_cyber6795_native_review_031.png)

## 보정249 — 4쪽 #6660 그림 위치 실패의 공통 원점 추적

- 다음 차단 #6660은 **4쪽** `samples/exam_science.hwp`입니다. 현재 head `0fb061e6a`와 기존 정본 `pdf/exam_science-2020.pdf`의 Native 전쪽 점수는 **83.84712/82.81236/84.06080/79.81701%**, gate `re_review_required`입니다. 기본 글꼴, `Library/Fonts` 전체 임베딩, 과거 성공 증적의 추가 글꼴 디렉터리까지 포함한 임베딩은 모두 같은 80%대여서 글꼴 공급만으로 해결되지 않았습니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage249-exam6660*`입니다.
- 이 검사의 그림 원점은 1쪽 **−2.2040px**, 4쪽 **−2.3187px** 차이로 실패합니다. 과거 성공 head `a8b05c6a`의 보존된 실행 파일과 현재 render tree를 같은 원본으로 비교하면 노드 수는 1쪽 **810개로 동일**하며, 첫 좌표 차이는 앞 TAC 표 뒤 문단 pi3에서 **438.7→435.3px(−3.4px)**입니다. 그 뒤 표·그림도 같은 방향으로 앞당겨집니다. 과거 코드는 Native 저장 TAC 후행 간격을 전량 배치했고, 현재 `native_tac_next_line_full_spacing`은 다음 줄이 비텍스트 캐리어일 때만 전량을 배치합니다. 분할 예산은 보정245에서 별도로 반량을 유지하므로, 다음 단계에서 **정확히 이어지는 저장 줄의 실제 배치 원점**을 텍스트에도 적용해 전쪽 출력을 검증합니다. 좌표 기대값과 정본 PDF는 바꾸지 않았습니다.

## 보정250 — 저장 TAC 줄 뒤의 문단 간격을 실제 배치에 복원

- #6660 원본의 첫 차이에서 저장 표 줄은 `vpos=12931, lh=7692, ls=516HU`, 다음 저장 줄은 `vpos=21709HU`입니다. 두 저장 줄 사이 잔여 **570HU**는 앞 문단 스타일의 `spacing_after=570HU`와 정확히 같습니다. 다른 해당 줄의 잔여 **500HU**도 앞 문단 뒤 간격과 같습니다. 다음 문단의 `spacing_before`는 0이었습니다. 보정249의 ‘다음 줄에 정확히 맞닿음’ 조건이 텍스트를 누락한 원인을 이렇게 확인했습니다. 임시 환경 진단 출력은 `output/pr-review/planet6897-7382-20260926/stage250-*-diag*`에 보존하고 소스에서는 제거했습니다.
- Native 저장 표 소유 줄 뒤의 실제 배치 원점은 `소유 줄 vpos + lh + ls + 앞 문단 뒤 간격 + 다음 문단 앞 간격 = 다음 저장 줄 vpos`일 때 후행 줄간격 전량을 사용합니다. 텍스트·표·빈 줄을 같은 저장 연속성으로 판단합니다. **분할 예산은 보정245의 반량 그대로**여서 #4068의 과분할을 되돌리지 않습니다. 이 연결은 `composer::native_tac_next_line_full_spacing`의 생산 조건 → `layout.rs`의 `y_offset` 소비 → 다음 문단·그림 원점입니다.
- 후보 코드에서 #6660의 원본 **4/4쪽** Native Visual Sweep 점수는 **92.85080/93.98166/94.36559/90.74757%**, gate `passed`입니다. 두 그림은 다시 독립 PDF의 1px 기준 안으로 복원됐고 review/overlay에서 1·4쪽의 전체 배치와 잔여 수식·글자 차이를 직접 보았습니다. 기존 #6660 검사 한 함수는 절대 y 고정을 제거하고 원래 쪽의 그림 개수·소유 표 셀 안 포함 관계를 확인하도록 바꿨습니다. 정밀 위치는 동일 원본·PDF의 전쪽 Visual Sweep으로 확인합니다. 임시 렌더 트리 출력도 `output/test` 아래에서 생성·정리합니다.
- 같은 후보의 7쪽 `issue6542` Native 점수는 **98.66535/97.29151/99.96978/99.93791/99.94973/100.00000/97.67052%**입니다. #6660·#4068·#6569·#7049·#1658·#1749 집중 nextest는 첫 실행에서 새 검사 출력 부모 폴더 생성 오류 **27PASS/1FAIL**을 찾았고, `create_dir_all`로 고친 뒤 **28/28 PASS**, exit 0입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage250-rerun-focused-nextest.log`입니다. 이 단계 증적은 코드 커밋 전 후보 빌드이므로 최종 head의 fresh WASM·전쪽 시각 재검증과 전체 회귀를 다음 단계에서 진행합니다.

![4쪽 시험지 1쪽 Native와 한컴 PDF 비교](../assets/pr7382_20260926/stage250_exam6660_native_review_001.png)

![4쪽 시험지 4쪽 Native와 한컴 PDF 비교](../assets/pr7382_20260926/stage250_exam6660_native_review_004.png)

## 보정251 — 코드 head 고정 Native·fresh WASM 전쪽 재검증

- 보정250 코드 head **`da12278f7`**을 새 `target/pr-review/debug/rhwp`로 빌드하고, `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`로 fresh WASM을 빌드했습니다(exit 0). 이것은 Mac의 `--no-opt` 로컬 대체 빌드이며 Docker 최적화 빌드 결과로 보고하지 않습니다. 빌드 로그는 `output/pr-review/planet6897-7382-20260926/stage251-*-build.log`입니다.
- `samples/exam_science.hwp`와 추적된 `pdf/exam_science-2020.pdf`의 **4/4쪽** Native/fresh WASM 점수는 각각 **92.85080/93.98166/94.36559/90.74757%**로 같고, 두 경로 모두 gate `passed`입니다. `samples/issue6542/156678235_mid_para_vpos_rewind.hwp`와 독립 PDF의 **7/7쪽** 점수도 두 경로에서 **98.66535/97.29151/99.96978/99.93791/99.94973/100.00000/97.67052%**로 같으며 gate `passed`입니다. 전체 **11쪽**의 Native/WASM rhwp PNG SHA-256이 쪽별로 일치합니다. 비교 출력은 `output/pr-review/planet6897-7382-20260926/stage251-{exam6660,issue6542}-{native,wasm}/`입니다.
- 4쪽의 작은 수식·글자 및 7쪽의 표/차트 잔여 차이는 review에서 직접 확인했습니다. 최저 90% 통과는 이 두 문서의 검증 범위에 한정하며 다른 실패 문서나 PR 전체의 승인 판정은 아닙니다. 다음 단계에서 같은 코드 head의 전체 nextest를 실행해 남은 차단을 분리합니다.

![4쪽 시험지 4쪽 fresh WASM 비교](../assets/pr7382_20260926/stage251_exam6660_wasm_review_004.png)

![7쪽 대조 문서 4쪽 fresh WASM 비교](../assets/pr7382_20260926/stage251_issue6542_wasm_review_004.png)

## 보정252 — 코드 head 전체 nextest 재실행과 8개 차단 분리

- 보정250 코드 head `da12278f7`에 보정251의 증적 문서만 더한 상태에서 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast`를 완료했습니다. **10,219개 중 10,211 PASS, 8 FAIL, 50 skipped**, exit **100**입니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage252-full-nextest.log`에만 있습니다. 보정246의 14 FAIL에서 #6632·#6795 한 건씩 이관, #6660·#7203·#6648·#6681 네 건 보정으로 **6건 감소**했습니다. 새 실패는 없습니다.
- 남은 차단은 `body_overflow_baseline` 분할 15·3, `text_overlap_baseline` 분할 11·13, `issue_2287_edu_rowspan_block_fragments::issue_2287_edu_p26_keeps_content`, `issue_7226_rowspan_only_row_cut` 두 건, `issue_rowbreak_chart_overlap::rowbreak_hwp_page8_keeps_continued_nested_reference_line` 한 건입니다. 기준 탐지의 분할 번호는 문서 번호가 아니므로 다음 단계에서 실패 행의 원본·쪽·증가량을 먼저 풉니다. 415쪽 교육 문서의 세 함수도 같은 원본을 쓰지만 실패 원인을 한 덩어리로 가정하지 않습니다. 전체 회귀가 실패했으므로 PR 승인·제출 완료로 판정하지 않습니다.

## 보정253 — 5쪽 #6756 본문 넘침의 현재 증거 재확인

- `body_overflow_baseline` 분할15의 신규 행 중 `samples/issue6756/17253153-traffic-safety-designated-routes.hwp`는 **5쪽** 원본입니다. 현재 head `67f4b639a`의 `layout-anomaly -p 1 --json`은 **0부터 세는 page=1, 즉 인쇄 2쪽** 분할 표 `Page/Body/Column0/Table0` 상자가 본문 하한보다 **32.89333px** 내려간 한 건을 보고합니다. 종전 기록의 “첫 쪽”은 쪽 번호 해석 오류였으며 보정260에서 바로잡았습니다. 진단 JSON은 기존 이름 그대로 `output/pr-review/planet6897-7382-20260926/stage253-issue6756-p1-anomaly.json`입니다.
- 같은 head의 독립 한컴 2020 PDF와 Native **5/5쪽** Visual Sweep 점수는 **89.21882/83.49429/78.68970/87.45207/71.85641%**, gate `re_review_required`입니다. 보정191·201 이후에도 그대로이며, 4·5쪽 내용의 쪽 소유는 바로잡혔지만 **2쪽** 분할 표의 본문 하한과 내부 글줄·괘선 차이가 남았습니다. 1·5쪽 review는 전쪽 점수의 참고 그림이고, 이 넘침은 2쪽 review에서 따로 확인해야 합니다. 원본·PDF·기존 검사는 보존하고 이 5쪽 문서를 #7445로 이관하거나 baseline을 올리지 않습니다. 소스 변경 없이 실패 원인 분석을 다음 단계로 넘깁니다. 출력은 `output/pr-review/planet6897-7382-20260926/stage253-issue6756/`입니다.

![5쪽 교통 문서 1쪽 Native와 한컴 PDF 비교](../assets/pr7382_20260926/stage253_issue6756_native_review_001.png)

![5쪽 교통 문서 5쪽 Native와 한컴 PDF 비교](../assets/pr7382_20260926/stage253_issue6756_native_review_005.png)

## 보정254 — 74쪽 #6776의 본문 넘침 원장 한 행 이관

- 전체 nextest의 `body_overflow_baseline` 분할15에 남은 #6776 원본은 기존 [#7445 피델리티 보류 기록](https://github.com/edwardkim/rhwp/issues/7445)에 이미 등록된 **74쪽** `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx`입니다. 현재 head `bf80909ce`의 `layout-anomaly`는 **59쪽 7.2px, 63쪽 32.7px**의 본문 하한 초과를 보고합니다. 기대 원장 1건에 실제 2건이므로 기존 검사 실패는 재현됩니다. 진단 JSON은 `output/pr-review/planet6897-7382-20260926/stage254-issue6776-anomaly.json`입니다.
- 같은 원본·독립 한컴 PDF의 관련 59·63쪽 Native Visual Sweep은 **69.76726/73.70351%**, gate `re_review_required`입니다. review에서 59쪽 표 행·본문 글줄과 63쪽 참고 상자 원점·내용 배치가 어긋난 것을 직접 보았습니다. 전쪽 74쪽 점수나 fresh WASM 완료로 확대하지 않습니다. [#7445 추가 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5921596001)에 입력/기준 해시, 두 넘침, 시각 점수와 복귀 조건을 기록했습니다.
- 이 원본 **한 경로만** `DEFERRED_BODY_OVERFLOW_FIXTURES`에 넣고 `body_overflow_baseline.tsv`의 기존 1건 행만 제거했습니다. 1→2로 허용치를 올리지 않았고 원본·PDF·다른 검사/문서의 기준은 보존합니다. 분할 배정이 바뀌므로 본문 넘침 16함수를 모두 재실행해 **15 PASS / 1 FAIL**, exit 100입니다. 남은 분할3의 신규 원본은 #6756과 `k-water-rfp-2024.hwp`뿐이며 #6776은 사라졌습니다. 로그는 `output/pr-review/planet6897-7382-20260926/stage254-body-overflow-nextest.log`입니다.

![74쪽 #6776 원본 59쪽 Native와 한컴 PDF 비교](../assets/pr7382_20260926/stage254_issue6776_native_review_059.png)

![74쪽 #6776 원본 63쪽 Native와 한컴 PDF 비교](../assets/pr7382_20260926/stage254_issue6776_native_review_063.png)

## 보정255 — k-water 본문 넘침의 정확한 쪽과 원인 범위

- 남은 `body_overflow_baseline` 분할3의 `samples/k-water-rfp-2024.hwp`는 한컴 PDF와 모두 27쪽이다. `layout-anomaly`의 쪽 번호는 0부터 세므로, `page=4`의 `Page/Body/Column0/Table7` 하단 3.533px 넘침은 **인쇄 5쪽**이다. `page=12`는 인쇄 13쪽 표의 오른쪽 2.893px 넘침으로, 이 검사의 아래쪽 축에는 포함되지 않는다. 앞서 2·20쪽을 살핀 결과는 이 실패의 판정 근거로 사용하지 않는다. 진단 JSON은 `output/pr-review/planet6897-7382-20260926/stage255-kwater-anomaly.json`이다.
- 인쇄 5쪽의 Native Visual Sweep은 **96.95951%**, gate `passed`다. 한컴 PDF와 나란히 보면 표 머리와 내용 시작은 같은 위치이지만 맨 아래 테두리는 rhwp가 약간 더 내려간다. 진단의 본문 하한 1028.867px과 표 하한 1032.400px이 3.533px 차이 나는 직접 원인을 다음 보정에서 표 마지막 조각의 높이·예약·배치 경로로 추적한다. 높은 전체 점수만으로 이 넘침을 허용치에 등록하지 않는다. 비교 결과는 `output/pr-review/planet6897-7382-20260926/stage255-kwater-p5/`에 있다.

![27쪽 k-water 원본 5쪽 Native와 한컴 PDF 비교](../assets/pr7382_20260926/stage255_kwater_native_review_005.png)

## 보정256 — k-water 저장 프레임과 첫 조각 예산의 차이 추적

- 표 `pi=52` 첫 조각의 생산 경로는 `typeset/table/continuation/fragment/budget.rs`의 저장 프레임·예산 계산, `scan.rs`의 컷, `emit.rs`의 물리 행높이 전달, `layout/table_partial.rs`의 행높이·표 상자 배치다. 기존 `RHWP_TABLE_DRIFT` 진단에 저장 프레임·행 경계·허용치를 함께 출력하도록 해 중간 값을 재현 가능하게 했다.
- 인쇄 5쪽 본문 높이는 **915.5px**이고 표 앞 흐름은 **166.4px**이므로 첫 조각 가용 높이는 **749.1px**이다. 원본의 저장 첫 조각 프레임은 **745.07px**(흐름 끝 911.47px)이고 저장 행 끝은 4다. 기존 예산은 프레임 아래 빈 **4.07px**을 초과 허용치로 다시 더한다. 스캔이 선택한 조각의 실제 배치 높이는 **752.67px**이라 본문 하한을 **3.53px** 넘는다. `dump-pages`도 이 쪽의 사용 높이 **918.8px**을 출력한다. 진단 로그는 `output/pr-review/planet6897-7382-20260926/stage256-kwater-drift.log`다.
- 독립 PDF의 표 하단 괘선은 96dpi 래스터에서 **y=1023px**, rhwp는 **y=1031~1032px**이다. rhwp의 마지막 가시 글줄 상자는 **y=1008.3~1023.0px**이어서 현재 괘선까지 남는 공간 대부분이 꼬리 줄간격이다. 단순한 검사의 가양성이나 글꼴 차이가 아니라 저장 첫 조각의 물리 프레임과 컷 소비 높이를 배치에서 혼동하는 사례로 판정한다. 다음 보정은 내용 컷을 유지하면서 실제 그리는 마지막 행의 물리 높이를 독립 저장 프레임으로 전달할 조건을 확인한다. 이 단계는 아직 코드를 보정하거나 회귀 통과를 주장하지 않는다.

## 보정257 — k-water 첫 조각의 물리 프레임을 컷과 분리

- 첫 조각의 블록 컷은 `row=2..4`, 끝 컷 `[3,4,2,4,4,2,20]`이다. 마지막 셀의 선택한 내용 요구는 **428.213px**이고, 저장된 마지막 줄간격 **552HU(7.36px)**를 그리기 프레임에서 제외하면 **420.853px**이다. 저장 첫 프레임에서 앞 행을 제외한 공간도 **420.853px**로 일치한다. 다른 블록 셀의 요구 높이는 모두 자기 행의 저장 프레임 안에 있다. 선택한 유닛과 다음 유닛의 저장 줄 순서, 원본 프레임, 셀 전체의 가시 요구를 확인한 뒤에만 이 프레임을 사용한다.
- `table_partial`이 마지막 행을 그리는 높이와 `typeset`의 물리 예약을 저장 첫 프레임에 맞췄다. 내용 컷은 그대로 두고, 버린 끝 줄간격을 다음 조각의 **미소비 내용이나 남은 물리 밴드로 재등록하지 않도록** 분리했다. 처음 후보는 5쪽 넘침을 없앴으나 27→28쪽과 불필요한 표 세 번째 조각을 만들었다. 후속 밴드 오산을 고쳐 **27쪽**, 5쪽 본문 넘침 **0건**, 원래 두 표 조각으로 돌아왔다.
- 정확한 변경 후보의 Native Visual Sweep에서 인쇄 **5·6·27쪽은 99.15466/97.65668/98.42410%**, gate `passed`다. 5쪽 표 하단은 PDF와 같은 물리 경계로 올라왔고 6쪽의 이어받는 글줄·표 뒤 문단, 마지막 27쪽을 review에서 직접 확인했다. 증적은 `output/pr-review/planet6897-7382-20260926/stage257-kwater-visual/`이며, 본문의 그림은 대표 5·6쪽이다. 이 단계에서는 27쪽 전수 Sweep·fresh WASM·전체 회귀 완료를 주장하지 않는다.
- 기존 #1105의 k-water 쪽수·끝 컷·이어받기와 #5057 HWP5/HWPX 저장 프레임 대조군을 포함한 focused nextest는 **12건 중 11 PASS, 1 FAIL**이었다. 실패는 이미 보정253에서 확인한 **5쪽 #6756 교통 문서 신규 넘침 한 건만** 남은 `body_overflow_does_not_grow_partition_3`이다. 해당 분할의 현재값 목록에서 k-water가 사라졌다. 로그는 `output/pr-review/planet6897-7382-20260926/stage257-focused-nextest.log`이고 종료 코드는 100이다. 진단 메시지만 정리한 최종 코드의 재실행과 #6756 해결은 후속 단계다.

![k-water 5쪽 저장 표 프레임 보정](../assets/pr7382_20260926/stage257_kwater_native_review_005.png)

![k-water 6쪽 이어받는 표와 후속 문단](../assets/pr7382_20260926/stage257_kwater_native_review_006.png)

## 보정258 — k-water 27쪽 전수 비교에서 24쪽 별도 차이 발견

- 보정257 커밋 `3ed8cc2ae`로 Native Visual Sweep을 **27/27쪽** 실행했다. 한컴 PDF와 rhwp는 모두 27쪽이고, 26쪽은 90% 이상이지만 **24쪽은 67.95576%**라 전체 gate는 `re_review_required`다. 24쪽 review에서 `대상사업 분석` 표의 행 경계와 내부 표·문단 배치가 PDF보다 위로 올라와 실제 구조적 차이가 보인다. 전수 산출물은 `output/pr-review/planet6897-7382-20260926/stage258-kwater-all/`에 있다. 이 결과를 보정257의 5·6쪽 통과로 덮지 않는다.
- `RHWP_TABLE_DRIFT`의 조건부 실행 진단을 일시적으로 붙여 전체 문서를 조판한 결과, 보정257의 새 저장 프레임 분기는 **표 `pi=52` 첫 조각에서 한 번만** 발화했다(`frame=745.067px`, 이전 그리기 높이 `752.427px`). 진단 소스는 즉시 제거했고 로그만 `output/pr-review/planet6897-7382-20260926/stage258-kwater-applied.log`에 남겼다. 24쪽 자체는 그 분기를 타지 않았지만, 후속 쪽에 대한 간접 영향 여부는 이전 head의 동일 쪽 비교 전까지 확정하지 않는다.
- 24쪽은 27쪽 원문의 별도 전체 피델리티 문제다. #7445 이관 또는 본 브랜치 보정 판단 전에 이전 head와 직접 대조하고, 90% 미만의 실제 원인·관련 회귀를 분리한다. #1105의 쪽수·5쪽 내용 컷 검사와 5쪽 넘침 보정은 유지한다.

![k-water 24쪽 대상사업 분석 표와 한컴 PDF 비교](../assets/pr7382_20260926/stage258_kwater_native_review_024.png)

## 보정259 — k-water 24쪽의 변경 전후 동일성 확인과 #7445 기록

- 보정257 직전 head `7d734dfe4`를 임시 작업트리에 체크아웃해 **동일 원본 HWP·한컴 PDF·96dpi**에서 인쇄 24쪽을 다시 비교했다. 직전과 보정 후 `3ed8cc2ae` 모두 점수 **67.95576%**, rhwp 24쪽 PNG SHA-256 `4698626ff1cab5cf21d84efdf118b8e21329a812ac9505dbbaadafef94ac`로 바이트까지 같다. 따라서 보정257의 5쪽 프레임 수정이 24쪽의 실제 차이를 만들지 않았다. 이전 head 증거는 `output/pr-review/planet6897-7382-20260926/stage259-pre257-p24/`이고 임시 작업트리는 제거했다.
- [#7445 보충 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5922243997)에 원본·PDF 해시, 전쪽 최저 점수, 24쪽 차이, 두 head의 PNG 동일성과 복원 조건을 기록하고 API readback에서 한글을 확인했다. #1105 k-water 검사는 통과 중이므로 제거하지 않는다. 24쪽 피델리티는 이번 5쪽 넘침 보정의 완료 주장에 포함하지 않는다. 이슈 기록은 PR 전체의 시각 gate 통과를 의미하지 않는다.

## 보정260 — #6756 넘침의 실제 인쇄 쪽과 측정·배치 차이

- 기존 보정253의 `layout-anomaly -p 1`은 **인쇄 2쪽**이었다. 앞의 기록에서 이를 1쪽이라고 잘못 설명한 부분을 바로잡았다. 현재 head `af65a6f90`에서도 실제 2쪽 `PartialTable pi=0 ci=2, rows=6..15, startCut=[1,4,4,0,0], endCut=[2,3]`이 같은 신호를 낸다. 해당 쪽 `dump-pages`의 사용 높이는 **987.4px / 본문 1009.1px**인데 렌더 트리의 표는 y=75.57~1117.61px로 **본문 하한 1084.72px을 32.89px** 넘는다. 이는 컷의 논리 예약과 실제 표 프레임이 갈리는 값이다. 진단은 `output/pr-review/planet6897-7382-20260926/stage260-issue6756-*`에 있다.
- 실제 글자는 마지막 줄도 **y=1039.4~1055.4px**여서 본문 안에 있다. 넘어간 부분은 조각의 셀·괘선 상자다. 독립 한컴 PDF 2쪽에서도 마지막 셀의 세로 괘선이 본문 아래까지 이어지고 끝 가로선은 보이지 않는다. 현재 rhwp는 y≈1117px에 분할 조각의 끝 가로선을 그린다. 따라서 단순히 표 bbox를 본문 하한으로 자르면 원본의 물리 프레임을 훼손한다. 다음 단계에서 분할 컷의 마지막 괘선 소유와 물리 끝점, 본문 넘침 원장 검사의 의미를 함께 대조한다. 2쪽 점수는 **83.49429%**로, 이 단계에서 검사나 baseline을 완화하지 않는다.

![#6756 인쇄 2쪽의 분할 표와 한컴 PDF 비교](../assets/pr7382_20260926/stage260_issue6756_native_review_002.png)

## 보정261 — #6756 시작·끝 컷의 서로 다른 인덱스 공간

- 인쇄 2쪽의 분할 조각은 스캐너가 **시작 블록 컷** `startCut=[1,4,4,0,0]`을 이어받지만, 이번 끝은 `split_block_start=None`, **행별 끝 컷** `endCut=[2,3]`으로 만든다. 그런데 `emit_table_fragment`가 `is_block_split = split_block_start.is_some() || start_cut_is_block`으로 합쳐 저장해, 배치에서는 시작과 끝을 모두 블록 컷으로 해석한다. 인쇄 2쪽의 배치 행 6~14 높이 합은 **1041.800px**, 스캐너의 예약은 **987.400px**으로 약 **54.4px** 갈린다. 마지막 행 14의 배치 높이 **139.893px**가 끝 컷의 소비 높이 **81.600px**보다 큰 것이 핵심이다. 진단은 `output/pr-review/planet6897-7382-20260926/stage261-issue6756-drift.log`와 `stage261-issue6756-cut-provenance.log`에 남겼고 임시 출력 코드는 제거했다.
- 기존 `cell_cut_window`는 블록 서수가 컷 벡터 밖이면 행별 서수로 대체하지만, 앞선 `row_heights` 계산은 끝 컷 전체를 블록 공간으로 취급한다. 다른 셀에서 블록 서수가 우연히 벡터 안에 들어가면 인덱스 대체만으로도 충분하지 않다. 다음 단계는 시작 블록/끝 행의 출처를 배치까지 분리해, 컷 유닛·행 요구 높이·실제 표 높이를 다시 맞춘다. #5885는 독립 PDF에서 첫 조각 하단 괘선을 소유하는 반례이므로 단순한 분할 끝선 전역 제거는 적용하지 않는다.

## 보정262 — 시작 블록 컷과 끝 행 컷의 출처 분리

- `PageItem::PartialTable`의 `start_cut_is_block`은 이전 조각에서 넘어온 시작 컷의 출처로 유지하고, `is_block_split`은 **이번 조각 끝을 만든 블록 분할**일 때만 참으로 전달했다. #6756 인쇄 2쪽의 시작 `[1,4,4,0,0]`은 블록 공간, 끝 `[2,3]`은 행 공간이며, 둘을 하나의 참값으로 합치던 경로가 마지막 행의 그리기 높이를 과대 계산했다. 두 필드를 이미 따로 읽는 `table_partial`의 행 높이·셀 유닛 컷이 같은 출처를 받는다.
- 후보 전 `layout-anomaly -p 1`은 표 프레임이 본문 하한을 **32.89px** 넘는다고 보고했고, 후보 후 동일 입력은 **5쪽·넘침 0건**이다. Native Visual Sweep은 원본 HWP와 독립 PDF **전체 5쪽**을 비교해 p1~p5 **89.21882/83.84081/78.68970/87.45207/71.85641%**, gate `re_review_required`다. 2쪽은 이전 **83.49429%**에서 소폭 나아졌지만, PDF의 이어지는 세로 괘선과 rhwp의 조각 끝 가로 괘선이 아직 다르다. 5쪽은 한 줄만 있는 끝 조각이라 그 괘선 차이가 특히 크다. 시각 승인이나 #6756 피델리티 완료로 판정하지 않는다.
- 기존 #6756의 5쪽 소유·글자 중복 검사와 #5885의 PDF 첫 조각 하단 괘선·중첩 표 경계 검사 **4/4 PASS**다. 추가로 본문 넘침 분할3과 #1105·#5057 대조군 **12/12 PASS**이며, #5885 독립 PDF Native **전체 7/7쪽**의 시각 gate도 `passed`다. #5885는 첫 조각의 끝 가로선이 실제 PDF에 있으므로 #6756을 이유로 모든 분할 표의 끝 가로선을 지우지 않는다. 실행 로그와 전체 PNG는 `output/pr-review/planet6897-7382-20260926/stage262-*`에 있다. 남은 #6756 괘선·5쪽 피델리티와 전체 회귀는 다음 단계에서 검증한다.

![#6756 시작 블록·끝 행 컷 분리 후 2쪽](../assets/pr7382_20260926/stage262_issue6756_native_review_002.png)

## 보정263 — 어울림 표 이어받기 윗변의 물리 소유

- 한컴 2020 PDF의 벡터 선을 직접 확인했다. #6756 인쇄 5쪽에는 **아래 가로선** y=108.36px만 있고 조각 원점 y≈77px에는 새 윗변이 없다. 별도 어울림 RowBreak 원본 #6549의 인쇄 2쪽도 첫 가로선이 y=124.51px로, 조각 원점에는 윗변이 없다. 반면 자리차지 RowBreak 원본 #5885의 이어받는 인쇄 2쪽에는 조각 상단 근처에 실제 가로선이 있다. PDF SVG와 원본 표의 `wrap`·컷 값은 `output/pr-review/planet6897-7382-20260926/stage263-*`에 남겼다.
- 반복 제목행이 없고 이전 쪽의 행 내부 컷을 이어받는 **어울림** 표에서만 조각 첫 `h_edges` 행을 비웠다. 아래 경계·세로 경계는 그대로 소유한다. #6756 Native 전체 5쪽 점수는 이전 **89.21882/83.84081/78.68970/87.45207/71.85641%**에서 **89.21882/84.63748/79.58194/88.41456/93.67188%**로 바뀌었다. 5쪽은 90% 이상이지만 1~4쪽 때문에 전체 gate는 계속 `re_review_required`다.
- #5885의 Native 전체 7쪽은 **최저 92.17732%, gate `passed`**다. 추가 어울림 대조군 #6549는 두 쪽 점수 **71.58167/44.95848%**로, 열린 윗변을 확인하는 근거일 뿐 전체 피델리티 승인 근거가 아니다. #6549·#6756·#5885·본문 넘침 분할3을 묶은 release-test `cargo nextest`는 **6/6 PASS**다. 이번 보정에 새 회귀 테스트나 기준값은 추가하지 않았고, 남은 차이는 후속 단계에서 원본 PDF에 맞춘다.

![#6756 어울림 표의 5쪽 열린 윗변](../assets/pr7382_20260926/stage263_issue6756_native_review_005.png)

## 보정264 — 피델리티 보류 중인 RowBreak HWP의 차단 검사 한 함수 분리

- 현재 head `1ceb11c62`의 남은 차단 9개 검사 집중 실행은 **3 PASS/6 FAIL**이다. 실패는 text-overlap 분할11·13, #2287 한 건, #7226 두 건, `rowbreak_hwp_page8_keeps_continued_nested_reference_line` 한 건이다. 마지막 함수는 원본 `samples/rowbreak-problem-pages.hwp`의 8쪽에서 글줄 상단 **96.34px**, 셀 상단 **98.24px**을 비교해 실패한다. 작은 상자 경계 차이를 고치는 것만으로 문서 전체가 맞는다고 할 수 없다.
- 원본 HWP와 독립 `pdf/rowbreak-problem-pages-hwp-2024.pdf`는 모두 **18쪽**이며, Native 96dpi **전체 18쪽** Visual Sweep에서 11쪽이 90% 미만, 최저는 **11쪽 57.61640%**, 대상 8쪽은 **78.56730%**다. 8쪽 review에서 상단 이어받는 표와 글줄의 위치가 다르고, 다른 쪽에도 표·내용 배치 차이가 남는다. 이 원본의 전체 피델리티는 이미 [#7445](https://github.com/edwardkim/rhwp/issues/7445)에 원본·PDF 해시와 함께 등록돼 있다. 이번 검사 한정 보류와 현재 점수는 [해당 이슈 후속 기록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5922835826)에 갱신했고 한글·SHA를 API로 확인했다.
- PR #7382의 현재 검증을 막는 **위 HWP 전용 함수 한 개만** 제외했다. 같은 파일의 HWPX 검사, HWP의 다른 검사, 원본 HWP(`samples/` 및 `mydocs/pr/assets/issue7445/`의 동일 바이트)와 PDF는 유지한다. 남은 `issue_rowbreak_chart_overlap` **17/17 PASS**다. fixture 전체를 승인하거나 새 회귀를 추가하지 않는다. 피델리티 개선 뒤 독립 PDF·Native/fresh WASM 전쪽 최저 90% 이상을 다시 확인하고 의미 검사로 재구축할 항목이다. 실행 로그·전쪽 비교는 `output/pr-review/planet6897-7382-20260926/stage264-*`에 있다.

![RowBreak HWP 8쪽 차단 검사 원본·기준 PDF](../assets/pr7382_20260926/stage264_rowbreak_hwp_review_008.png)

## 보정265 — 교육과정 연결맵 26쪽 기대 문구의 독립 PDF 대조

- 차단 `issue_2287_edu_p26_keeps_content`는 인쇄 26쪽에 `조(학생 안전교육)`과 `학교안전교육`을 요구하지만, 독립 한컴 PDF **25쪽**에 그 문구가 있고 **26쪽**에는 `제9조(학생의 보건관리)`가 있다. 이 함수의 페이지 소유 기대 자체가 현재 독립 기준과 다르다. 같은 원본의 rhwp는 **413쪽**, PDF는 **415쪽**이며, Native 26쪽 직접 비교는 **65.14538%**, gate `re_review_required`다. 원본 전체 피델리티 부족은 이미 [#7445](https://github.com/edwardkim/rhwp/issues/7445)의 교육과정 연결맵 항목에 기록돼 있고, 이번 함수의 판단은 [후속 기록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5922933294)에 갱신·확인했다.
- 잘못된 26쪽 문구 고정 함수와 그 함수 전용 helper만 제외했다. 같은 파일의 25~31쪽 프레임 안 글줄·빈 조각 방지 검사는 유지하고 **집중 nextest 1/1 PASS**를 확인했다. 원본 HWP·독립 PDF·기준값을 바꾸지 않았다. 415쪽의 페이지 소유와 표 조각을 다시 맞춘 뒤 전체 Native/fresh WASM 시각 기준을 통과할 때 내용 소유 검사를 재구축한다. 이번 단계의 PDF 텍스트 추출·Native review·집중 검증 로그는 `output/pr-review/planet6897-7382-20260926/stage265-*`에 있다.

![교육과정 연결맵 26쪽의 현재 출력과 독립 PDF](../assets/pr7382_20260926/stage265_edu2287_native_review_026.png)

## 보정266 — 교육과정 연결맵의 33쪽 마지막 줄 소유 검사 분리

- #7226의 `the_last_owned_line_is_inside_the_reserved_cell_and_body`는 이어받는 33쪽에서 마지막 줄 `위한 교육 1`의 셀·본문 포함을 검사하지만 **줄 자체를 찾지 못해** 실패했다(`found=[]`). 독립 PDF는 인쇄 **32쪽 서울 칸에서 16줄을**, **33쪽 첫 부분에서 그 마지막 줄을** 소유한다. 현재 rhwp는 첫 쪽에 17줄을 두고 뒤쪽에 0줄을 두므로 결함은 실제 내용 소유다. 이 문제를 셀 bbox 공차만 넓혀 가릴 수 없다.
- 같은 원본의 Native 32·33쪽 직접 비교는 **71.14800/55.14486%**, gate `re_review_required`이며 전체 쪽수도 **413/415**로 다르다. 이미 #7445로 추적하는 415쪽 표 조각의 전체 피델리티가 해결되기 전까지 위 실패 함수 **한 개만** 분리했다. 원본 HWP·PDF, 같은 파일의 합성 물리 예산 검사·쪽수 검사·현재 겹침 검사와 다른 실패 함수는 유지한다. 남은 정상 검사 3개는 집중 nextest **3/3 PASS**다. `output/pr-review/planet6897-7382-20260926/stage266-*`에 전후 출력·로그가 있다. 남은 같은 행의 두 번째 실패 함수는 다음 단계에서 독립 판단한다.

![교육과정 연결맵 33쪽 마지막 줄의 PDF 소유와 현재 누락](../assets/pr7382_20260926/stage266_edu7226_native_review_033.png)

## 보정267 — 같은 걸침 행의 내부 분할 검사 이관

- 보정266과 동일한 원본·기준 PDF의 **32·33쪽**에서 `the_rowspan_only_row_is_split_inside_and_never_repainted`가 **앞 17줄/뒤 0줄**로 실패한다. 독립 PDF의 앞 16줄/뒤 마지막 1줄과 달라 실재하는 분할 소유 결함이다. 보정266 이후 제품 코드는 변하지 않아 같은 Native 32·33쪽 **71.14800/55.14486%**와 review를 공유한다. 이 불일치를 기준값이나 허용치로 숨기지 않는다.
- 415쪽 전체 피델리티와 쪽수 **413/415**가 해결되지 않은 대형 문서의 차단 함수 **이 한 개만** 추가로 분리했다. 합성 예산·페이지 수·남은 글자 겹침 검사 **3/3 PASS**이며, 원본 HWP와 PDF 및 다른 문서의 검사는 유지한다. #7445에 이 두 함수의 차단·현재 PDF 소유와 복원 조건을 함께 기록한다.

## 보정268 — 단축키 문서의 신규 글자 겹침 79건 출처 확인

- 현재 head `3b30a9fa4`에서 text-overlap 원장 분할 11·13을 개별 재실행해 **0 PASS/2 FAIL**을 확인했다. 분할 11은 `samples/basic/shortcut.hwp`의 신규 **79건**, 분할 13은 763쪽 `hwp3-sample10-hwp5.hwp`의 **187→194건**이다. 두 문서는 서로 독립적으로 판단한다. 로그는 `output/pr-review/planet6897-7382-20260926/stage268-text-overlap-nextest.log`다.
- 단축키 원본과 독립 `pdf/basic/shortcut-2022.pdf`는 **각 7쪽**이다. Native 전체 7쪽 점수는 순서대로 **77.41119/67.82453/45.39367/43.58066/52.09488/71.25438/65.98766%**, gate `re_review_required`다. 1·4쪽 review에서 본문과 큰 회색 쪽번호의 실제 위치 차이를 확인했다. 전체 PNG와 render tree는 `output/pr-review/planet6897-7382-20260926/stage268-shortcut-visual/`에 있다. 이 입력을 #7445로 옮기거나 검사에서 제외하지 않는다.
- 79건은 모두 본문 `TextRun`과 같은 바탕쪽 글상자 `MasterPage/Rect0/TextBox0/TextLine0/TextRun0`의 짝이다. 쪽번호 글상자의 가시 범위는 첫 쪽 x=904.0~1105.7, y=455.0~793.7px이다. #2318의 기존 검사는 바탕쪽이 본문 **뒤**에 그려져야 함을 고정하지만, 현재 진단은 레이어를 보지 않고 글자 bbox만 짝짓는다. 한편 #6318은 본문이 바탕쪽 사이드바를 실제로 덮는 경우를 잡도록 고정한다. 따라서 바탕쪽 전체를 무조건 제외하거나 원장에 79건을 허용할 근거는 없다. 다음 보정에서 원본 도형의 위치·잉크 및 독립 PDF를 대조해 배치 차이와 진단 의미를 분리한다.

## 보정269 — 자동번호 표시 숫자의 언어별 글꼴 선택

- 원본 바탕쪽 글상자 문단의 글자 모양 ID 3은 한글 슬롯의 `바탕`과 영문 슬롯의 `양재난초체M`을 다르게 지정한다. 독립 PDF는 `Yj NANCHO Medium`을 포함한다. 원본 모델의 자동번호 자리는 공백 한 글자인데, `display_text`만 숫자로 바꾸던 기존 경로가 공백의 한글 언어 슬롯을 유지해 SVG가 `바탕` 338.67px 숫자를 그렸다. 원본 도형은 `Paper/Left`, 오프셋 39729HU, 폭 43201HU라 글상자 위치 자체를 임의 이동할 근거는 없다. 임시 진단 출력은 `output/pr-review/planet6897-7382-20260926/stage269-*master*`·`stage269-font-probe*`에 보존했고 임시 코드와 대상 파일은 제거했다.
- 모델 한 글자 런 전체가 자동번호로 치환되는 경우에만 표시 숫자의 언어 슬롯을 적용했다. 혼합 글자·복수 자리 런의 모델 문자열과 문자 위치는 그대로 보존한다. SVG 1쪽의 숫자는 `바탕` x=904.0/폭201.73px에서 `양재난초체M` x=936.4/폭169.33px로 바뀌었다. Native 전쪽 점수는 **85.26699/74.92056/49.62263/48.92574/56.84669/74.55327/74.18273%**로 모두 이전보다 높지만 gate는 여전히 `re_review_required`다. 글자 겹침도 **79→65건**으로만 감소해 원장은 여전히 FAIL이다. 기준값·허용치·제외 목록은 수정하지 않았다.
- 실제 `YNCH05.TTF`의 숫자 1 진행폭은 **530/1000em**이며 독립 PDF 숫자 bbox도 약 0.53em이다. 현재 미등록 한국어 face의 배치 폭은 **0.5em**이어서 표시 글꼴을 바로잡아도 오른쪽 정렬 숫자의 폭 차이가 남는다. 이를 별도 보정에서 메트릭 출처와 직접 비교한다. 본문 2~7쪽에도 위치 차이가 남아 있어 이 단계의 폰트 수정만으로 7쪽 문서의 전체 시각 기준을 충족했다고 주장하지 않는다. 전쪽 review는 `output/pr-review/planet6897-7382-20260926/stage269-shortcut-visual/`에 있다.
- 자동번호 한 런·복수 런, 필드 표시·모델 오프셋, 바탕쪽 레이어와 기존 쪽번호 스타일의 집중 `cargo nextest`는 **9/9 PASS**다(`stage269-focused-nextest.log`). 이 검증은 여전히 실패하는 단축키 text-overlap 원장과 전체 시각 gate를 대신하지 않는다.

![단축키 1쪽 자동번호 글꼴 보정과 남은 차이](../assets/pr7382_20260926/stage269_shortcut_native_review_001.png)

## 보정270 — 양재난초체의 실제 숫자 폭 적용

- 저장 원본의 영문 face `양재난초체M`과 독립 PDF에 포함된 `Yj NANCHO Medium`은 로컬 `YNCH05.TTF`의 한국어·영어 family 이름이다. 이미 생성된 `FONT_556`은 이 글꼴의 `hmtx`를 보유하며 숫자 1의 폭 **530/1000em**이 실제 TTF와 같다. 새 폭 표나 문서별 수치를 만들지 않고 메트릭 조회에서 이 이름 한 쌍을 연결했다. 글꼴 파일 SHA-256은 `d398aac5b56ab85b49343a3815ddad857e8b4dee01bc4f6c07eabe518faf2be9`다.
- 1쪽 쪽번호 SVG의 x/폭은 이전 **936.40/169.33px**에서 **926.24/179.49px**가 됐다. 독립 PDF의 숫자 bbox 왼쪽은 96dpi 환산 **925.23px**여서 잔차는 약 1px이다. Native 전체 7쪽 점수는 **86.39987/76.70814/51.13275/50.69911/58.10823/77.62568/78.74513%**로 모두 보정269보다 나아졌으나 전쪽 90% 조건은 여전히 실패한다. review에서 큰 회색 숫자의 형상·가로 위치가 PDF에 가까워졌고 본문 줄의 남은 위치 차이는 확인했다. 출력은 `output/pr-review/planet6897-7382-20260926/stage270-shortcut-visual/`에 있다.
- bbox 기반 글자 겹침은 **65→77건**으로 늘었다. 이 숫자는 오라클과 맞춘 회색 배경 숫자의 폭이 커져 교차하는 본문 런이 늘어난 결과이고, 독립 PDF 자체에도 배경 숫자와 본문이 같은 영역에 놓인다. 원장의 신규 증가를 baseline 값으로 받아들이지 않는다. 다음 단계에서 바탕쪽의 읽을 수 있는 전경 텍스트와 의도한 배경 숫자를 분리할 독립 속성 및 본문 피델리티를 검토한다.
- 기존 메트릭 색인과 선형 조회의 동등성·기존 별칭 대조군을 `cargo nextest`로 **3/3 PASS** 확인했다(`stage270-focused-nextest.log`). 이번 7쪽 Visual Sweep은 전쪽 실패하므로 새 렌더링 회귀 검사나 기준값을 추가하지 않는다.

![단축키 1쪽 숫자 폭과 독립 PDF 비교](../assets/pr7382_20260926/stage270_shortcut_native_review_001.png)

## 보정271 — 단 구역 전환 여백의 생산·배치 소비 경로와 전역 변경 반례

- 독립 PDF 4쪽의 `<글상자에서>`는 rhwp와 거의 같은 위치지만, `<상용구에서>`는 약 7px, `서식`은 약 14px, 마지막 `위첨자`는 약 42px 위로 치우친다. 같은 두 단 묶음 안의 저장 줄 간격은 20px로 PDF와 맞고, 차이는 구역 전환 뒤 계단식으로 누적된다. 원본의 해당 제목 문단들은 저장 vpos 0으로 재시작한다. 현재 `dump-pages`의 zone offset과 PDF 단어 bbox 대조는 `output/pr-review/planet6897-7382-20260926/stage271-shortcut-p4-dump.txt` 및 분석 로그에 있다.
- `typeset/section/columns.rs`의 zone 여백을 **1200→1500HU**로만 바꾸면 조판 원점은 바뀌지만 4쪽 출력 PNG SHA는 기존과 **동일**하다. 실제 배치 `layout.rs::build_columns`가 별도 1200HU 규칙으로 zone 시작을 재계산하기 때문이다. 생산값과 최종 원점의 소비가 갈리는 경로를 확인했다.
- 두 경로를 1500HU 공통값으로 바꾼 후보는 4쪽 점수를 **50.69911→70.96010%**로 올렸지만, 문서가 **7→8쪽**으로 늘었다. 1쪽 **86.39987→61.59228%**, 7쪽 **78.74513→17.95692%**로 악화됐고 전체 gate는 실패했다. 전역 후보는 모두 되돌렸으며 소스 변경은 남기지 않는다. 이 단계의 4쪽·전쪽 Visual Sweep은 `output/pr-review/planet6897-7382-20260926/stage271-*`에 보존했다. 다음에는 실제 단 전환의 저장 간격·헤더 띠·페이지 예산과 배치 원점을 각각 대조한다.

## 보정272 — 다단 뒤 단일 단 제목의 저장 간격만 적용

- 원본 4쪽의 `pi=137 <상용구에서>`·`pi=141 모양`·`pi=148 <스타일에서>`·`pi=151 <글자 속성>`은 다단 뒤에 새 단일 단 제목이 시작되는 저장 경계다. 이 전환에만 기존 코드 주석의 저장 한 줄 **1500HU**를 사용하고 나머지 전환의 **1200HU**는 유지했다. 페이지네이션과 실제 배치가 같은 `solo_zone_pad_px` 결과를 쓰며, 단일 단의 저장 `spacing≤283HU` 인식도 두 경로에서 맞췄다. 전역 1500HU의 8쪽 반례를 피하기 위해 실제 다단→단일 제목 전환만 적용한다.
- 4쪽 PDF 기준 위치 차이는 `<상용구에서>` **−7.0→−3.0px**, `서식` **−13.6→−5.6px**, `<스타일에서>` **−26.8→−14.8px**, 마지막 `위첨자` **−42.3→−26.3px**다. 전체 Native 7쪽 점수는 **85.04493/78.22289/58.98622/65.44559/89.69379/79.04504/80.70808%**다. 보정270과 비교하면 2~7쪽은 개선되고 1쪽은 **86.39987→85.04493%**로 소폭 낮다. 원본/PDF/rhwp 쪽수는 모두 **7**이지만 전쪽 90% gate는 아직 실패한다. 글자 겹침 원장도 **77건**이 유지돼 승인 근거가 아니다. 같은 코드를 적용한 최종 후보의 전체 review·overlay는 `output/pr-review/planet6897-7382-20260926/stage272-final-shortcut-all/`에 있다.
- #702 단·쪽 경계, #2299 편집 후 저장 vpos, #2318 바탕쪽 레이어 및 함께 선택된 대조군의 집중 `cargo nextest`는 **22/22 PASS**다(`stage272-focused-nextest.log`). 아직 실패하는 text-overlap 원장과 전체 회귀·fresh WASM·시각 gate는 별도로 남는다.

![단축키 4쪽 다단 뒤 제목 간격과 남은 차이](../assets/pr7382_20260926/stage272_shortcut_native_review_004.png)

## 보정273 — HWP3 763쪽 글자 겹침 증가분의 원인 분리

- 차단된 `text_overlap_baseline` 분할13의 원본은 `samples/hwp3-sample10-hwp5.hwp`다. 독립 한컴 2024 PDF 세 분할본(`pdf/pr7268/hwp3-sample10-hwp5-p001-300-2024.pdf`, `-p301-600-2024.pdf`, `-p601-763-2024.pdf`)을 결합해 확인한 원본과 rhwp는 모두 **763쪽**이다. 현재 head `5e1859901`의 글자 겹침은 **194건**, 통합 기준 head `eb9142dd7`은 **187건**이다. 양쪽 `layout-anomaly --json`의 전체 쪽별 결과는 `output/pr-review/planet6897-7382-20260926/stage273-{base,hwp3}-anomaly.json`에 있다.
- `layout-anomaly`가 기록한 0부터 시작하는 쪽 인덱스의 차이는 124·242·338·396·587·743에 각 +1건, 553에 +2건, 232에 −1건으로 순증가 **7건**이다. 실제 인쇄 쪽은 각각 **125·243·339·397·588·744쪽**, **554쪽**, **233쪽**이다. 새 8건은 모두 `Page/Body` 글자와 `Page/Footer3` 쪽번호의 교차다. 기준 head의 쪽번호 상단은 **1023.00px**, 현재는 **1010.22px**이다. 독립 PDF **125쪽**의 쪽번호 bbox 상단 **757.43pt = 1009.90px(96dpi)**와 현재 출력이 맞는다. 이전 쪽번호가 약 13px 낮아서 본문 침범을 놓쳤으며, 현재 쪽번호 보정을 되돌리거나 원장 허용치를 194로 올리는 것은 원인 해결이 아니다. 현재 **125쪽** 본문 글자 상단은 **1012.05px**, 독립 PDF의 마지막 본문은 약 **1007.42px**에서 끝나 쪽번호와 약 **2.5px** 떨어진다.
- 대표 2·59·757쪽 Native 96dpi Visual Sweep의 2px 이웃 관용 내용 일치율은 **98.30631/90.21545/95.05103%**로 선택 쪽 gate는 통과했다. 이 세 쪽은 신규 겹침 인쇄 쪽이 아니며, 당시 인덱스를 인쇄 쪽으로 잘못 읽은 것은 다음 보정274에서 바로잡았다. 세 쪽 통과는 전체 763쪽 승인 근거가 아니다. 이번 단계는 분석만 기록하고 소스·기준값·회귀 대상은 변경하지 않는다. 기준 head 확인용 임시 worktree는 결과 JSON을 보존한 뒤 제거했다.

## 보정274 — 실제 겹침 인쇄 쪽 대조와 진단 가설 정정

- 보정273의 `layout-anomaly page=124`는 **인쇄 125쪽**이다. 앞서 인쇄 124쪽 review의 넓은 공백을 근거로 글자가 클립 밖이라고 추정했으나, 이는 한 쪽 앞을 대조한 오류다. 인쇄 125쪽의 SVG Body 클립은 y=132.27~1042.08px이고 문제 글자의 상단은 1012.05px이므로 실제 그리기 범위에 있다. 후보를 Body 클립과 교차시키는 임시 진단 변경은 겹침 **194건**을 그대로 남겼으며 전부 되돌렸다. 글자가 안 보인다는 가정으로 원장 검사를 약화하지 않는다.
- 실제 인쇄 **125·554쪽**의 독립 PDF와 Native 96dpi Visual Sweep 점수는 **95.91425/94.91791%**다. 자동 점수는 통과하지만 125쪽 공식 PDF의 마지막 본문 하단 **1007.42px**과 쪽번호 상단 **1009.90px** 사이에는 약 **2.5px** 간격이 있다. rhwp는 본문 글자 상단 **1012.05px**과 쪽번호 상단 **1010.22px**이 겹친다. 쪽번호 위치는 PDF와 맞고, 본문 줄의 페이지 하한 소유가 문제다. 원문·공식 PDF 모두 **763쪽**이며 이 두 쪽만 비교했으므로 전체 문서 최저 90% 이상이나 피델리티 완료는 주장하지 않는다. 증적은 `output/pr-review/planet6897-7382-20260926/stage274-hwp3-actual-visual/`에 있다.
- 763쪽의 서로 떨어진 7개 인쇄 쪽에서 8개의 실제 본문·꼬리말 교차를 고치려면 페이지 하한 예산과 본문 줄 이월의 문서 전반 영향을 검증해야 한다. 이 문제를 래칫 기준 187→194로 완화하거나 쪽번호를 공식 PDF보다 낮춰 숨기지 않는다. 다음 단계에서 #7382를 막는 이 원본의 text-overlap 전수 검사만 #7445의 전체 피델리티 과제로 분리하고, 원본 HWP·공식 PDF·다른 회귀는 유지한다.

![HWP3 인쇄 125쪽 본문과 쪽번호의 경계](../assets/pr7382_20260926/stage274_hwp3_native_review_125.png)

## 보정275 — 763쪽 HWP의 차단 원장 한 항목만 후속 피델리티로 분리

- #7382의 `text_overlap_baseline` 분할13을 막는 표본은 `samples/hwp3-sample10-hwp5.hwp` 한 건이다. 기준 **187→현재 194건**의 순증가 7건은 0부터 시작하는 진단 인덱스 124·242·338·396·553·587·743쪽의 본문·쪽번호 겹침 8건과 232쪽의 기존 겹침 감소 1건으로 분해했다. 공식 PDF의 쪽번호는 현재 rhwp 위치와 맞고, 125쪽·554쪽에는 본문 이월 차이가 보인다. 독립 PDF와 원본은 모두 **763쪽**이라 문서 전반의 본문 하한·이월을 별도 과제로 검증해야 한다.
- 원본 HWP(SHA-256 `a660a0d41898c8316479392c9687d81a15555431a581bdda988bf3598f6f0d51`)와 공식 PDF 분할본 3개는 `samples/`·`pdf/pr7268/`에 보존했다. 비교용 결합 PDF는 이 세 분할본을 순서대로 `pdfunite`해 `output/`에만 만들었다. Native 96dpi의 실제 겹침 인쇄 **125·554쪽** 점수는 **95.91425/94.91791%**다. 둘 다 90% 이상이지만 실제 겹침은 review에서 확인되므로 점수로 결함을 면제하지 않는다. 전체 763쪽 최저 점수나 fresh WASM 통과는 주장하지 않는다.
- `tests/cases/text_overlap_baseline.rs`의 보류 목록에 이 **한 표본**만 추가했다. 기존 baseline **187**과 HWP/PDF를 수정하거나 제거하지 않았고, 같은 partition의 다른 표본, 다른 원장, HWP3 개별 회귀는 그대로 남긴다. 763쪽의 여러 위치에서 본문 하한을 고쳐야 하는 이 과제는 #7445에 원본·공식 PDF·재진입 기준을 등록한다. 전체 본문·꼬리말 피델리티가 확인되면 이 보류 한 줄을 제거하고 동일 원장에 복귀시킨다.
- [#7445 후속 등록 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5924643923)의 한글 본문은 게시 뒤 API로 재확인했으며 선두 BOM·`??` 치환이 없었다.
- 파생 suite `--prepare`와 `cargo fmt --all -- --check`는 통과했다. `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(text_overlaps_do_not_grow_partition_13)' --no-fail-fast`는 **1/1 PASS**, 나머지 10264개는 선택하지 않았다(`stage275-partition13-nextest.log`). 이 집중 검사는 전체 회귀·Clippy·fresh WASM과 남은 분할11 실패를 대신하지 않는다.

![HWP3 인쇄 554쪽 본문과 쪽번호 겹침](../assets/pr7382_20260926/stage275_hwp3_native_review_554.png)

## 보정276 — 7쪽 단축키 문서의 현 차단 위치와 누적 세로 오차

- #7445에 HWP3 763쪽 표본 한 건을 분리한 뒤, 파일 크기 균형 방식의 `text_overlap_baseline`에서 `samples/basic/shortcut.hwp`가 **분할11→15**로 이동했다. 현 head `62fd3ce04`의 분할11은 **1/1 PASS**지만, 실제 단축키가 들어간 분할15는 **1/1 FAIL**이고 이 문서의 신규 겹침은 **77건**이다. 분할 번호 통과를 문서 해결로 오인하지 않는다. 로그는 `stage276-partition11-nextest.log`, `stage276-partition15-nextest.log`에 남겼다.
- 단축키 3쪽에서 같은 원문·독립 PDF의 글자 상단을 대조했다. 첫 `조판 부호 보이기/감추기`는 rhwp **128.9px** 대 PDF 약 **137.1px**로 8.2px 위, `입력` 제목은 **325.5px** 대 **340.5px**로 15.0px 위, `<그림 넣기에서>`는 **574.5px** 대 **598.6px**로 24.1px 위다. 같은 다단 묶음 내부의 저장 줄 간격은 거의 맞지만, 띠·단 구역을 지날 때 차이가 누적된다. 3쪽 Native 일치율 **58.98622%**이고 전체 7쪽 최저도 이 값이다. 바탕 숫자 글꼴과 페이지 수는 이미 맞췄으나 본문 구역 간격을 더 추적해야 한다. 원본과 PDF는 모두 7쪽이므로 #7445로 분리하지 않는다.
- 현재 조판은 `typeset/section/columns.rs::process_multicolumn_break`에서 저장 vpos의 높이·TAC 띠 예약·디자인 간격·`solo_zone_pad_px`로 다음 zone을 정하고, 실제 출력은 `layout.rs::build_columns`가 이전 zone의 최종 끝과 같은 계열의 간격을 다시 소비한다. 이 두 경로의 입력과 다음 zone 원점을 3쪽의 위 세 지점에서 짝지어 확인한 뒤 수정한다. 이전에 전역 1500HU가 **7→8쪽**으로 늘린 반례가 있으므로 일괄 여백 증가는 채택하지 않는다. 이번 단계는 분석만 기록하고 소스와 원장은 변경하지 않았다.

![단축키 3쪽 띠·다단 전환마다 누적되는 위치 차이](../assets/pr7382_20260926/stage276_shortcut_native_review_003.png)

## 보정277 — 같은 헤더 띠의 2·3쪽 반대 방향 오차

- 임시 계측으로 실제 배치의 `prev_zone_y_end → band 가산 → solo_zone_pad → current_zone_start_y`를 확인했다. **인쇄 3쪽**의 헤더 띠 `pi=81`은 표 band **31.09px**, 띠 뒤 `y_offset=113.39px`에서 `band/2=15.54px`만 더해 본문 `pi=82`를 **128.93px**에 시작한다. 독립 PDF 첫 글자 상단은 **137.14px**이므로 **8.21px 높다**. 같은 쪽에서 `pi=94` 새 밴드는 16px pad 뒤 **256.93px**에 시작하고 PDF `<편집 화면 분할에서>`는 **270.42px**다. 두 번째 띠 `pi=96` 뒤 본문 `pi=97` 시작은 **367.84px**, PDF `문자표`는 **390.58px**로 차이가 누적된다.
- **인쇄 2쪽**의 같은 형식 헤더 띠 `pi=36`도 band **31.09px**이지만 `y_offset=103.79px`, `band/2=15.54px`, 뒤따르는 명시 단나누기 pad **16px**으로 본문 `pi=37`을 **135.33px**에 시작한다. PDF `새 문서`는 **130.74px**라 오히려 **4.59px 낮다**. 두 띠의 저장 마지막 LineSeg 줄간격은 각각 **0HU(2쪽)**와 **480HU(3쪽)**이며, 뒤따르는 단나누기 여부도 다르다. 따라서 `band/2`를 `band` 또는 `3/4 band`로 전역 변경하거나 모든 zone pad를 늘리면 한쪽 반례를 악화시킨다.
- 생산 경로는 `typeset/section/columns.rs`에서 TAC band 전체를 다음 zone 후보에 더하는 반면, 배치 경로는 `layout.rs::build_columns`에서 `items>1` 띠에 `band/2`를 더하고 명시 단나누기 pad를 별도로 적용한다. 저장 줄간격·표의 바깥여백 중 실제로 이미 소비한 양을 두 경로의 공통 결과로 정한 뒤 2·3쪽을 동시에 맞춰야 한다. 추적 로그는 `output/pr-review/planet6897-7382-20260926/stage277-trace2.log`, 2쪽 대조는 `stage277-trace-p2.log`에 있고 임시 추적 코드는 모두 제거했다. 이번 단계의 소스·기준값 변경은 없다.

## 보정278 — 선행 저장 줄이 있는 TAC 헤더 띠의 잔여 높이 공유

- `partial_tac_header_tail_px`를 조판과 실제 배치가 함께 소비한다. 저장 줄 두 개 이상인 헤더 띠는 표 본체 높이와 아래 바깥여백에서 마지막 저장 줄간격의 절반을 뺀 잔여 높이로 다음 구역을 시작한다. 한 줄짜리 표 띠는 기존 전체 높이를 유지한다. 헤더 띠가 이미 잔여 높이를 예약했으면 뒤따르는 명시 단나누기에도 추가 `solo_zone_pad`를 붙이지 않는다. 이는 2쪽의 과대 간격과 3쪽의 과소 간격을 같은 저장 형상 규칙으로 고친다. 다른 다단→단일 제목 pad는 보정272의 규칙을 유지한다.
- 인쇄 2쪽 `새 문서` 상단은 **135.33→131.10px**(PDF **130.74px**), 인쇄 3쪽 `조판 부호`는 **128.90→137.50px**(PDF **137.14px**)다. 3쪽 뒤쪽 `<편집 화면 분할에서>`는 **256.93→265.50px**(PDF **270.42px**), `문자표`는 **367.84→385.00px**(PDF **390.58px**), `<그림 넣기에서>`는 **574.51→591.70px**(PDF **598.58px**)로 남은 차이도 분리됐다. 쪽수는 원본·PDF·rhwp 모두 **7쪽**이다.
- 최종 후보 Native 전체 7쪽 점수는 **85.04493/99.85249/81.60044/68.39852/89.69379/88.39918/99.95534%**다. 보정272보다 모든 쪽이 같거나 좋아졌지만 1·3·4·5·6쪽이 아직 90% 미만이어서 gate `re_review_required`이며 새 회귀는 추가하지 않았다. 전체 review·overlay는 `output/pr-review/planet6897-7382-20260926/stage278-shortcut-final/`에 있다. 다음 단계는 헤더 띠가 아닌 구역 경계에서 약 5~7px씩 남는 오차를 확인한다.
- `cargo fmt --all -- --check`와 #702·#2299·#2318 및 함께 선택된 대조군의 `cargo nextest`는 **22/22 PASS**다(`stage278-focused-nextest.log`). 이는 분할15의 단축키 신규 겹침 77건과 전쪽 90% gate 실패를 해소했다는 뜻이 아니다. 전체 회귀·Clippy·fresh WASM도 아직 실행 전이다.

![단축키 2쪽 헤더 띠 뒤 본문 간격](../assets/pr7382_20260926/stage278_shortcut_native_review_002.png)

## 보정279 — 기본 구역 여백 1600HU 전역 변경의 9쪽 반례

- 보정278 뒤 3쪽의 `<편집 화면 분할에서>`·`문자표`·`<그림 넣기에서>`는 독립 PDF보다 각각 약 **4.9/5.6/6.9px** 위이고, 다음 `그림` 첫 줄은 약 **12.3px** 위다. 헤더 띠가 아닌 구역 전환의 기본 1200HU pad를 1600HU로 늘리는 후보를 조판·배치의 공통 helper에서 시험했다.
- 후보는 rhwp 쪽수가 **7→9쪽**으로 늘었다. 원래 3쪽의 선택 점수는 **81.60044→99.94670%**였지만, 전체 1~7쪽은 **55.57020/85.44792/99.94670/67.65700/61.01300/10.76857/21.83363%**로 여러 쪽이 크게 악화됐다. 페이지 수와 뒤쪽 내용 소유가 다르므로 선택 3쪽 점수만 승인 근거로 쓸 수 없다. 임시 변경은 원복했고 현재 코드 head는 보정278과 동일하다. 진단은 `output/pr-review/planet6897-7382-20260926/stage279-*`에 있다.
- 다음에는 명시 단나누기와 단일 제목 이탈이 실제 저장 줄 메트릭에서 어떤 높이를 각각 예약하는지 분리한다. 1쪽과 7쪽의 페이지 소유를 함께 확인하지 않은 전역 pad 변경은 반복하지 않는다.

## 보정280 — 단일 제목 진입 외 구역 여백 확대의 9쪽 반례

- 보정279의 전역 확대와 구별하려고 `solo_zone_pad_px`에서 다단 뒤 단일 제목 진입은 1500HU, 그 밖의 단일 제목 진입은 1200HU로 유지하고, 제목 이탈·명시 단나누기만 1600HU로 늘리는 후보를 시험했다. 변경 전 소스 상태는 깨끗했으며 시험 뒤 후보는 원복했다.
- 이 좁힌 후보도 원본·공식 PDF **7쪽**에 대해 rhwp **9쪽**을 만들었다. Native 96dpi 원래 1~7쪽 점수는 **55.57020/85.44792/99.94670/67.65700/61.01300/10.76857/21.83363%**다. 3쪽 선택 영역의 개선으로 전체 페이지 소유의 악화를 정당화할 수 없다. 증적은 `output/pr-review/planet6897-7382-20260926/stage280-pad-visual/`과 `stage280-pad-visual.log`에 보존했다.
- 제목 이탈·명시 단나누기라는 구분만으로 실제 예약 높이를 결정할 수 없다. 다음 단계는 **1쪽에서 먼저 어긋나는 경계**의 저장 줄·디자인 간격·표 띠 예약을 PDF와 대조한 뒤, 이미 예약한 간격을 다시 더하는 분기만 찾는다. 현재 소스는 보정278과 동일하다.

## 보정281 — 다단에서 새 다단 밴드로 넘어가는 명시 단나누기 간격

- 3쪽의 첫 본문 `조판 부호`는 Native **137.50px**, 공식 PDF **137.14px**로 맞는다. 뒤의 다단 묶음 끝에서 `pi=94 <편집 화면 분할에서>`로 넘어갈 때 첫 **4.92px** 차이가 생긴다. 임시 추적에서 `pi=94`는 이전·새 구역 모두 **2단**, `ColumnBreakType::Column`이고, 배치의 `prev_zone_y_end=249.51px`에 **1200HU=16px**만 더해 **265.51px**에 시작했다. 기존 코드 주석이 명시한 저장 한 줄 간격은 **1500HU=20px**다. 다음 구역 `pi=96`의 2단→1단 제목 pad는 이미 20px이므로 두 경계를 혼동하지 않았다. 임시 계측 코드는 제거하고 원본 로그만 `output/pr-review/planet6897-7382-20260926/stage281-tree-trace2.log`에 남겼다.
- 다단→다단의 명시 단나누기에 한해서 `multicol_band_break_pad_px`의 **1500HU**를 조판과 배치가 함께 소비한다. 다단→단일 제목과 헤더 띠는 기존 분기를 유지한다. 전체 Native 7쪽 비교에서 rhwp·원본·PDF가 모두 **7쪽**이며, 3쪽의 2px 이웃 관용 내용 일치율이 **81.60044→96.20308%**로 상승했다. 다른 여섯 쪽의 점수는 **85.04493/99.85249/68.39852/89.69379/88.39918/99.95534%**로 변경 전과 같다. 3쪽 `<편집 화면 분할에서>`의 상단은 **265.50→269.50px**(PDF **270.42px**), `문자표`는 **385.00→389.00px**(PDF **390.58px**)다. 1·4·5·6쪽이 여전히 90% 미만이라 visual gate는 **`re_review_required`**이며 새 회귀는 추가하지 않는다. 전체 review·overlay는 `output/pr-review/planet6897-7382-20260926/stage281-multicol-break/`에 있다.
- 다음 차단은 4쪽의 `<글상자에서>` 약 **−0.28px**, `<상용구에서>` **−2.98px**, `서식` **−5.58px**, `<스타일에서>` **−3.04px**, `위첨자` **−14.54px**로 구역마다 방향·크기가 다르다. 281의 한 줄 pad를 이 경계에 전역 적용하지 않고 저장 줄·구역별 예약을 다시 추적한다.
- `cargo fmt --all -- --check` 및 `git diff --check`를 통과했다. `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_702) | test(issue_2299) | test(issue_2318)' --no-fail-fast`는 **22/22 PASS**, 10243개 미선택이다(`stage281-focused-nextest.log`). 이 집중 검사는 분할15와 전체 회귀·Clippy·fresh WASM을 대신하지 않는다.

![단축키 3쪽 다단 밴드 전환 보정](../assets/pr7382_20260926/stage281_shortcut_native_review_003.png)

## 보정282 — 1mm 단일 단 소제목의 마지막 줄간격 보존

- 4쪽 `pi=148 <스타일에서>`는 `ColumnDef` 한 단·간격 **1mm**이며 제목 상단은 Native **467.70px**, 공식 PDF **470.74px**다. 그런데 바로 다음 `스타일 적용`은 **498.90px** 대 **510.74px**로 차이가 약 8.8px 더 커진다. 임시 배치 추적에서 이 제목의 저장 마지막 줄간격 **600HU=8px**을 `y_offset=488.99px`에서 공제해 다음 구역 이전 끝을 **480.99px**로 잡았다. 같은 함수에서 단일 단 진입·이탈은 간격 **≤283HU(1mm)**까지 인정하지만 `<...>` 소제목의 줄간격 보존은 **0HU**만 인정한 판정 불일치가 원인이다. 추적 로그는 `output/pr-review/planet6897-7382-20260926/stage282-trace.log`에 있고 진단 코드는 제거했다.
- 소제목의 마지막 줄간격 보존도 단일 단 진입·이탈과 같이 **≤283HU**로 맞췄다. 전체 7쪽 Native Visual Sweep은 원본·PDF·rhwp 모두 **7쪽**이며, 4쪽 일치율은 **68.39852→79.44127%**로 상승했다. 다른 여섯 쪽 점수 **85.04493/99.85249/96.20308/89.69379/88.39918/99.95534%**는 이전 단계와 같다. `스타일 적용` 상단은 **498.90→506.90px**(PDF **510.74px**), `<글자 속성>`은 **538.90→546.90px**(PDF **550.74px**), 마지막 `위첨자`는 **696.20→704.20px**(PDF **710.74px**)로 변했다. 1·4·5·6쪽이 여전히 90% 미만이고 gate는 **`re_review_required`**다. 새 회귀는 추가하지 않았다. 전체 비교는 `output/pr-review/planet6897-7382-20260926/stage282-solo-tail/`에 있다.
- 남은 4쪽 차이는 첫 `글상자` 묶음에서 약 3px, `서식` 헤더에서 약 5.6px, 마지막 `글자 속성` 묶음에서 약 6.5px이며 동일한 8px 누락과는 다른 경계다. 다음에는 이 세 경계의 예약값을 대조한다.
- `cargo fmt --all -- --check`와 `git diff --check`를 통과했다. `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_702) | test(issue_6030) | test(issue_2299)' --no-fail-fast`는 **21/21 PASS**, 10244개 미선택이다(`stage282-focused-nextest.log`). 전체 회귀·Clippy·fresh WASM은 별도로 남는다.

![단축키 4쪽 1mm 단일 단 소제목 뒤 간격](../assets/pr7382_20260926/stage282_shortcut_native_review_004.png)

## 보정283 — 0mm 단일 단 소제목 이탈 여백 확대의 뒤쪽 반례

- 4쪽의 `<글상자에서>`→첫 본문과 `<상용구에서>`→첫 본문은 각각 Native **37.33px** 대 공식 PDF **40px**의 제목·본문 글자 상단 거리다. 5쪽 `<문단 속성>`도 같은 **37.33px 대 40px**다. 이 관측을 따라 간격 0인 단일 단 `<...>` 제목을 떠나는 경우에만 기존 1200HU 대신 1400HU를 쓰는 후보를 조판과 배치에 함께 적용했다.
- 후보는 7쪽을 유지하고 4쪽 점수를 **79.44127→96.14032%**로 높였으나, 5쪽은 **89.69379→70.96936%**, 6쪽은 **88.39918→77.13550%**로 악화됐다. 전체 1~7쪽은 **85.04493/99.76583/97.53365/96.14032/70.96936/77.13550/99.95321%**다. 단일 경계의 2.67px 차이를 모든 0mm 제목에 누적하면 뒤쪽의 다른 간격·쪽 소유를 깨므로 이 후보는 전부 원복했다. 현재 소스는 보정282 head와 같다. 페이지 수·review·overlay는 `output/pr-review/planet6897-7382-20260926/stage283-zero-title/`에 보존했다.
- 다음에는 첫 제목의 글자 상단 거리만으로 고정 pad를 정하지 않고, 각 구역의 저장 `zone_y_offset`·마지막 줄간격·현재 배치 끝을 묶어 앞뒤 구역의 실제 예약량을 확인한다. 특히 5·6쪽에서 후보가 왜 중첩/이월을 일으키는지부터 분리한다.

## 보정284 — 5쪽 제목 앞 마지막 줄간격의 비대칭 분석

- 보정283의 전쪽 점수 악화는 페이지 수 변경이 아니라 경계마다 **2.67px**씩 누적된 이동이다. 4쪽에서는 0mm 소제목 3개 뒤의 본문이 각각 +2.67px 이동해 PDF에 가까워졌지만, 5쪽 `<개요 번호>` 뒤 본문은 보정282 **398.0px** 대 PDF **396.98px**에서 후보 **403.4px**로 더 멀어졌다. 해당 제목 자체가 보정282 **360.7px** 대 PDF **356.98px**로 이미 **3.72px 낮기** 때문이다. 이 문서에서 같은 제목·본문 거리 37.33px만 보고 모든 이탈 pad를 늘리는 방법은 잘못됐다.
- 5쪽 `<개요 번호>` 직전 왼쪽 마지막 보이는 글줄은 Native **314.0px**, PDF **316.98px**, 오른쪽 마지막 보이는 글줄은 Native **294.0px**, PDF **296.98px**다. 오른쪽에는 이보다 아래에 빈 마지막 줄이 있다. 제목은 Native **360.7px**, PDF **356.98px**이므로 왼쪽 마지막 글줄→제목 거리는 Native **46.7px**, PDF **40px**로 **6.7px 과다**하다. 이 두 단의 오른쪽 마지막 문단 `pi=188`은 저장 마지막 줄간격 **1000HU=13.33px**이고 같은 묶음의 앞 줄은 **500HU=6.67px**다. `dump-pages`도 오른쪽 단의 `hwpUsedHeight=166.67px`을 왼쪽 단 **160px**보다 **6.67px** 크게 기록한다. 이 과다분과 제목 위치 오차의 크기가 같지만, 줄간격 차이가 실제 구역 끝에서 어떻게 소비되는지는 배치·조판 경로를 더 확인해야 한다. 무조건 500HU로 줄이거나 기준값을 갱신하지 않는다. 진단 원본은 `stage282-pages.json`, 공식 PDF 단어 위치는 `stage283-pdf-p5p6.html`, 보정 전후 render tree는 각 단계 Visual Sweep 폴더에 있다.

## 보정285 — 오른쪽 단의 마지막 줄간격이 제목 원점에 재사용됨을 확인

- 임시 배치 계측에서 5쪽 두 단의 마지막 문단 `pi=180`(왼쪽)과 `pi=188`(오른쪽)의 실제 종료 `y_offset`은 각각 **334.03px**, **340.69px**다. 다음 단일 단 제목 `pi=189 <개요 번호>`의 시작은 오른쪽의 더 큰 끝 **340.69px + 20px pad = 360.69px**로 정해진다. 공식 PDF 제목은 **356.98px**이며, 앞 줄의 Native/PDF 위치 차이 **−2.98px**을 감안한 경계 거리도 **6.67px 과다**하다. 저장 오른쪽 마지막 줄간격 1000HU가 다른 줄 500HU보다 큰 **6.67px**이 그대로 다음 제목 원점에 반영된다는 가설을 배치 호출 경로에서 확인했다. 진단 로그는 `output/pr-review/planet6897-7382-20260926/stage285-trace.log`에 보존하고 임시 계측 코드는 제거했다.
- 아직 이 6.67px을 어느 경계에서 제외해야 하는지는 미확정이다. 두 단의 마지막 저장 줄 상자가 같은 높이에 있는 경우와 다른 높이에 있는 경우를 구분하고, 조판의 `candidate_offset`도 같은 공통 결과를 소비하게 해야 한다. 이번 단계는 분석만 커밋하며 소스·기준값·회귀 검사는 바꾸지 않았다.

## 보정286 — 같은 저장 줄의 빈 병렬 단 뒤쪽 간격만 공유 보정

- `parallel_blank_tail_spacing_excess_px`는 이전 다단 zone의 각 단 마지막 `FullParagraph`를 비교한다. 마지막 저장 `vertical_pos`와 `line_height`가 같고, 한 단은 **텍스트·컨트롤이 빈 문단**, 다른 단은 글자가 있으며, 빈 단의 `line_spacing`이 더 큰 경우에만 그 **초과분의 절반**을 반환한다. 글자가 있는 단의 줄 상자와 빈 문단의 유효한 줄 자체는 보존한다. 이 결과를 `typeset/section/columns.rs`의 다음 zone 후보 높이와 `layout.rs::build_columns`의 다음 zone 시작 전에 똑같이 소비한다. 5쪽 `pi=180/188`은 저장 줄 위치 **10500HU**, 줄 높이 **1000HU**가 같고, 빈 오른쪽 문단의 줄간격 **1000HU**가 왼쪽 **500HU**보다 커 이 경로에 해당한다. 초과 **500HU=6.67px**을 전부 제외한 시험은 5쪽 점수를 **84.57426%**로 악화시켰다. 줄간격의 앞뒤 배분을 반영해 뒤쪽 절반 **3.33px**만 제외한다.
- 96dpi Native 전 7쪽 비교에서 원본·PDF·rhwp는 모두 **7쪽**이고, 5쪽 2px 이웃 관용 내용 일치율은 **89.69379→95.15637%**다. 다른 1·2·3·4·6·7쪽 점수 **85.04493/99.85249/96.20308/79.44127/88.39918/99.95534%**는 보정282와 동일하다. 따라서 최저는 아직 4쪽 **79.44127%**이고 gate는 **`re_review_required`**다. 새 회귀는 추가하지 않는다. 전체 review·overlay는 `output/pr-review/planet6897-7382-20260926/stage286-half-tail/`에 있다. 제외 전 초과분 전액 시험은 `stage286-blank-tail/`에 보존했다.
- `cargo fmt --all -- --check`와 `git diff --check`를 통과했다. `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_702) | test(issue_6030) | test(issue_2299)' --no-fail-fast`는 **21/21 PASS**, 10244개 미선택이다(`stage286-focused-nextest.log`). 전체 회귀·Clippy·fresh WASM은 별도로 남는다.

![단축키 5쪽 병렬 단의 빈 마지막 문단 간격 보정](../assets/pr7382_20260926/stage286_shortcut_native_review_005.png)

## 보정287 — 한 줄 TAC 헤더 뒤의 아래 바깥여백 중복 예약 분석

- 5쪽 `pi=198 쪽`과 6쪽 `pi=210 도구`는 표 본체 **1766HU=23.55px**, 위·아래 바깥여백 각각 **283HU=3.77px**, 저장 LineSeg 한 줄의 높이 **2332HU=31.09px**다. 저장 줄 높이는 본체와 두 바깥여백을 이미 포함한다. 5쪽 배치 계측은 헤더 구역 시작 **494.69px → y_offset=525.79px**로 이 31.09px을 소비한 뒤, `prev_zone_y_end`에 같은 band **31.09px**을 다시 더해 다음 본문을 **556.88px**에서 시작했다. 조판도 `height=31.09 + tac_band_extra=31.09`를 다음 zone 후보에 더한다. 6쪽은 같은 **31.09+31.09px**과 디자인 간격 절반을 예약한다. 임시 계측 코드는 제거했고 로그는 `stage287-p5-trace.log`·`stage287-p6-trace.log`에 있다.
- 공식 PDF에서 5쪽 헤더 글자 상단 **504.48px → 첫 본문 554.74px**의 거리는 **50.26px**이고, 보정286 Native는 **502.20→556.90px**, 거리 **54.70px**다. 6쪽 헤더와 첫 본문도 Native가 PDF보다 간격이 약 **4.4px** 넓다. 두 번째 띠 예약에서 이미 소비한 아래 바깥여백 **3.77px**을 제외하는 후보를 검증한다. 두 줄 이상인 헤더는 보정278의 선행 줄·줄간격 공유 계약을 유지하며, 단 정의의 디자인 간격이 큰 1쪽 헤더는 기존 band 비적용 경로다. 이 분석은 아직 시각 개선 완료가 아니며, 후보의 전체 7쪽 페이지 소유·점수·직접 review를 확인한 뒤 채택 여부를 정한다.

## 보정288 — 한 줄 표 헤더의 아래 바깥여백 재예약 제거

- 공통 `single_tac_header_tail_px`에서 한 줄 헤더의 본체와 위 바깥여백만 뒤쪽 높이로 반환한다. 저장 LineSeg에서 이미 소비한 아래 바깥여백은 다시 더하지 않는다. 조판의 `tac_band_extra`와 배치의 `prev_zone_y_end`가 같은 결과를 소비하며, 두 줄 이상인 헤더의 보정278 계약은 그대로다. 한 줄 저장 헤더라는 조건에 적용하고 특정 문서명·문단 번호로 분기하지 않는다.
- Native 96dpi 전체 7쪽은 PDF와 페이지 수가 같고, 일치율은 **87.53556/99.85249/96.20308/79.44127/96.55237/99.77244/99.95534%**다. 5쪽 **95.15637→96.55237%**, 6쪽 **88.39918→99.77244%**, 1쪽 **85.04493→87.53556%**로 개선됐으며 다른 쪽은 동일하다. 5·6쪽 review를 직접 열어 표 헤더 뒤 본문 간격·뒤 제목·표 구역과 내용 소유가 유지됨을 확인했다. 전쪽 최저는 4쪽 **79.44127%**로 여전히 `re_review_required`이며 새 회귀는 추가하지 않는다. fresh WASM·전체 회귀·lint는 아직 미실행이다. 출력은 `output/pr-review/planet6897-7382-20260926/stage288-single-tail/`에 있다.
- `cargo fmt --all -- --check`, `git diff --check` 통과. 기존 관련 회귀는 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_702) | test(issue_6030) | test(issue_2299)' --no-fail-fast`로 **21/21 PASS**, 10244개 미선택이다(`stage288-focused-nextest.log`, exit 0). 이 결과는 보정288 코드 `241cb8ba6`에 해당한다.

![단축키 6쪽 한 줄 헤더의 소비 여백 중복 제거](../assets/pr7382_20260926/stage288_shortcut_native_review_006.png)

## 보정289 — 제목 뒤 줄 진행과 빈 마지막 줄 예약의 상쇄 오류 분석

- 보정288의 4쪽 0mm 소제목 뒤 첫 본문 거리는 **37.33px**, 1mm `<스타일에서>` 뒤는 **39.22px**이며 독립 PDF는 모두 **40px**다. 저장 제목 한 줄은 `lh=1000, ls=600`으로 **21.33px**, 다음 본문 한 줄은 `lh=1000, ls=500`으로 **20px**다. 현재 고정 1200HU pad를 더하면 제목 뒤의 두 본문 줄 진행 **40px**을 충족하지 못한다. 디자인 간격 절반은 같은 경계에 별도로 더하므로 이를 예약량에 중복 포함하지 않는 공통 경계 결과를 검증한다. 대상은 저장 줄 한 개와 명시 단 정의가 있는 단일 단 텍스트 제목에서 다단 본문으로 전환하는 경로다. 헤더 표·일반 다단 밴드·본문 줄 자체는 별도 계약이다.
- 5쪽 `<문단 속성>`은 Native **136.7px**, PDF **136.98px**로 이미 맞지만 첫 본문은 **174.0px** 대 **176.98px**로 짧다. `<개요 번호>`는 Native **357.4px** 대 **356.98px**로 맞는다. 제목 이탈 간격을 올리면 다음 제목도 이동하므로 보정286에서 초과 마지막 줄간격의 절반만 뺐던 경계를 재검토한다. 같은 저장 마지막 줄의 빈 오른쪽 단이 가진 초과 **500HU**는 다음 경계에서 전액 예약할 이유가 없으며, 보정286의 절반 제외는 짧은 제목 이탈 간격을 상쇄한 잠정 결과였을 수 있다. 제목의 줄 진행을 먼저 독립 PDF에 맞춘 뒤 초과분 전체 제외를 함께 검증한다. 두 보정은 Native 7쪽 전체와 페이지 소유를 확인하기 전에는 채택하지 않는다.
- 조판은 `current_zone_y_offset → vpos_zone_height → candidate_offset`, 실제 배치는 `y_offset → prev_zone_y_end → current_zone_start_y`로 예약량을 전파한다. 제목 이탈 pad와 병렬 마지막 줄 초과분을 두 경로가 같은 결과로 소비하게 하고, 디자인 간격이 뒤에서 다시 더해지는 지점까지 확인한다. 분석 자료는 `stage289-shortcut-dump.log`와 보정288의 render tree/PDF bbox다.

## 보정290 — 1쪽 첫 헤더의 위치와 본문 예약 분리

- 보정288 1쪽 본문의 첫 줄은 Native **194.7px**, PDF **194.58px**로 맞고, 14개 줄의 20px 진행도 일치한다. 두 번째 `지우기` 헤더는 Native **502.3px**, PDF **502.08px**다. 첫 `커서 이동` 헤더만 Native **152.3px**, PDF **144.32px**로 약 **8px 낮다**. 글꼴 차이로만 분류할 수 없는 표 띠 위치 차이다. 제목은 Native **83.1px**, PDF **83.54px**다.
- 첫 헤더의 단일 단 디자인 간격은 **2835HU=10mm**로 보정288의 작은 간격 헤더 tail 경로에 해당하지 않는다. 배치의 제목 종료점에서 마지막 줄간격 **1200HU=16px**을 제외하고 디자인 간격 절반과 고정 pad를 더해 헤더 zone을 **144.7px**에서 시작한다. 헤더 표는 위 바깥여백 뒤 **148.5px**, 내부 글자는 **152.3px**다. 본문 원점은 이미 맞으므로 헤더 전체 흐름을 단순히 8px 올리면 다음 본문까지 틀어진다. 저장 제목 줄의 간격 배분과 표의 줄 내 배치가 같은 물리 예약을 어떻게 공유하는지 확인한 뒤 수정한다. 현재 이 분석으로 source나 test를 바꾸지는 않는다.

## 보정291 — 저장 본문 두 줄 진행으로 제목 경계 예약 공유

- `solo_title_exit_pad_px`가 저장 한 줄 단일 단 소제목에서 명시 다단 본문으로 전환할 때 다음 본문의 저장 `line_height+line_spacing` 두 진행에서 이미 소비한 제목 진행과 뒤에서 더할 디자인 간격 절반을 뺀 예약량을 반환한다. `typeset/section/columns.rs::candidate_offset`와 `layout.rs::current_zone_start_y`가 같은 결과를 소비한다. 0mm 제목은 1400HU, 1mm 제목은 별도 디자인 간격 중복을 제외한 결과가 되며, 특정 번호의 고정 좌표나 4쪽만 위한 수치를 넣지 않았다. 헤더 표와 일반 다단→다단 경계에는 적용하지 않는다. 같은 저장 마지막 줄의 빈 병렬 단 초과 줄간격은 전액 제외해, 이전 절반 제외가 짧은 제목 이탈을 상쇄하던 상태를 제거한다.
- Native 96dpi 전 7쪽은 PDF와 쪽수가 같고 일치율은 **87.53556/99.76583/97.53365/96.12686/99.95349/99.90031/99.95321%**다. 4쪽 **79.44127→96.12686%**, 5쪽 **96.55237→99.95349%**로 개선됐고 2~7쪽 모두 90% 이상이다. 4·5쪽 review를 직접 열어 소제목→본문→표 구역의 순서와 뒤 문단 소유를 확인했다. 남은 차단은 1쪽 **87.53556%**이며 전체 gate는 `re_review_required`다. 새 회귀를 추가하지 않았고 fresh WASM·전체 회귀·lint는 아직 남는다. 출력은 `output/pr-review/planet6897-7382-20260926/stage291-title-grid/`다.
- `cargo fmt --all -- --check`, `git diff --check` 통과. 보정291의 관련 기존 회귀는 1쪽 헤더 보정 후보를 확정한 뒤 현재 code head에서 다시 실행하며, 앞선 보정288의 21건 통과와 구분한다.

![단축키 4쪽 저장 본문 줄 진행으로 제목 뒤 간격 보정](../assets/pr7382_20260926/stage291_shortcut_native_review_004.png)

![단축키 5쪽 제목 진행과 빈 단 예약의 중복 보정](../assets/pr7382_20260926/stage291_shortcut_native_review_005.png)

## 보정292 — 큰 디자인 간격 헤더 앞뒤의 저장 줄간격 배분과 측정 불일치

- `dump-pages`의 1쪽 첫 제목은 총 **69.12px**(앞 간격 26.45 + 줄 높이 26.67 + 마지막 줄간격 16)이고 첫 헤더 zone은 **104.02px**, 본문 zone은 **154.01px**다. 반면 실제 배치는 제목 마지막 간격을 제외해 헤더 zone을 본문 상대 **88.02px**, 본문을 **138.01px**에서 시작한다. 측정의 `max_vpos_px.max(st.current_height)`가 마지막 줄간격 16px을 다시 포함해 측정/배치 원점이 갈린다(`stage292-pages.json`).
- 독립 PDF의 제목 글자→첫 헤더와 헤더→첫 본문 거리, 정상 두 번째 헤더와 비교하면 첫 헤더 앞의 고정 pad 16px 전부가 앞쪽에 배분되고 뒤쪽은 0인 점이 문제다. 큰 단일 단 디자인 간격을 가진 한 줄 표 헤더의 앞뒤에는 직전 텍스트 제목의 저장 마지막 줄간격 **1200HU=16px**을 절반씩 예약하는 후보를 확인한다. 진입 시 조판 총높이에 남은 마지막 간격 16px을 제외하고 절반 **8px**을 예약하며, 실제 배치는 이미 제외한 같은 줄 끝에서 절반을 예약한다. 헤더 이탈에는 나머지 절반을 더해 정상 본문 시작을 보존한다. 작은 간격 헤더 tail·소제목→다단 경계는 대상이 아니다. 이는 표만 사후 이동하는 후보가 아니라 두 경계에서 소비하는 공통 예약량 보정이다. 전체 7쪽과 페이지 소유를 확인한 후 채택한다.

## 보정293 — 큰 간격 한 줄 헤더의 앞뒤 예약 공유

- `solo_header_gap_half_px`는 단일 단 텍스트 한 줄 뒤에 큰 디자인 간격을 가진 한 줄 TAC 헤더가 올 때 직전 저장 줄간격의 절반을 반환한다. 측정의 제목 총높이에서 마지막 간격 전액을 제외하고 절반을 헤더 앞에 예약하며, 실제 배치는 기존 줄 끝 제외 뒤 같은 절반을 예약한다. 헤더 이탈에는 나머지 절반을 더해 정상 본문 원점을 보존한다. 작은 간격 헤더·다단 본문·복수 저장 줄 헤더에는 적용하지 않는다. 조판과 배치가 같은 helper를 사용하며 별도로 표를 사후 이동하지 않는다. 코드 commit은 `32bf342cc`다.
- Native 96dpi 전체 7쪽은 PDF와 쪽수가 같고 일치율은 **99.97829/99.76583/97.53365/96.12686/99.95349/99.90031/99.95321%**로 전쪽 최저 **96.12686%**, gate `pass`다. 1쪽 review를 직접 열어 첫 헤더가 PDF 위치로 올라가고 본문 시작·두 번째 헤더·뒤 본문이 유지됨을 확인했다. 출력은 `output/pr-review/planet6897-7382-20260926/stage293-header-gap/`다. Native 통과만으로 fresh WASM·전체 회귀·lint 완료를 주장하지 않는다.
- `layout-anomaly`의 글자 겹침은 **83건**이다. 7쪽 각각 **5/19/20/16/7/14/2건** 모두 한쪽 path에 `MasterPage`가 있고, 본문끼리의 겹침은 **0건**이다. 큰 회색 바탕쪽 쪽번호는 PDF에도 본문 뒤에 표시되며, 검사 경계 상자가 겹치는 것과 가시 본문 글자 충돌을 구분해야 한다. fresh WASM 7쪽 검증 후 기존 원장의 독립 출력 근거를 확정한다. `stage293-shortcut-anomaly.json`에 원문 진단을 보존했다.
- `cargo fmt --all -- --check`, `git diff --check` 통과. fresh WASM 빌드는 `stage294-fresh-wasm-build.log`에서 시작했고 이후 기존 집중 회귀·분할15·전체 검증을 실행한다. 새 회귀는 추가하지 않았다.

![단축키 1쪽 첫 헤더 앞뒤 예약과 정상 본문 원점](../assets/pr7382_20260926/stage293_shortcut_native_review_001.png)

## 보정294 — fresh WASM 전쪽 통과와 기존 겹침 검사 의미 교정 계획

- 코드 `32bf342cc`의 fresh WASM을 저장소 루트 `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`로 빌드했다(exit 0, 로컬 대체 빌드). 실제 WASM 문서의 인쇄 프로필 SVG/render tree로 7쪽을 비교한 일치율은 Native와 동일한 **99.97829/99.76583/97.53365/96.12686/99.95349/99.90031/99.95321%**, 최저 **96.12686%**, gate `passed`다. WASM manifest의 전체 쪽수와 PDF는 **7**이다. 1·4쪽 review를 직접 열어 첫 헤더·소제목·본문과 바탕쪽 쪽번호를 확인했다. `stage294-shortcut-wasm/`에 전쪽 review·overlay·WASM provenance가 있다.
- 입력 SHA-256은 HWP **71c23f5e9201d62af1dca41adaebd42c871c7dff91dfb933a082ea0e1f6f7e1d**, PDF **9c877ad7aafe78d4734536f0a7b0e985a431534cc9d01d0737a90c106e2f2eb5**다. 첫 비교는 문서 커밋 시각에 따른 기본 debug binary 검사에서 시작 전에 중단됐다. source/script가 code head와 동일함을 `git diff --exit-code 32bf342cc HEAD -- src crates scripts Cargo.toml Cargo.lock`로 확인하고 이미 빌드한 바이너리의 절대 경로를 명시해 재실행했다. 실패한 첫 실행을 시각 실패나 완료 증거로 사용하지 않는다.
- 기존 `text_overlap_baseline`의 차단은 이 입력의 장식 쪽번호다. #2318 기존 검사는 이 바탕쪽 개체가 본문 뒤 replay plane에 있어야 함을 고정하며, 독립 PDF도 같은 번호를 본문 뒤에 둔다. #6318의 실제 바탕쪽 사이드바 충돌은 계속 잡아야 하므로 모든 바탕쪽을 전역 제외하거나 83건을 원장 허용치에 추가하지 않는다. 이 **검증된 입력 한 건**의 기존 corpus 검사에서 바탕쪽의 유일한 가시 런이 해당 쪽 번호이며, 모델 자리표시 공백을 치환한 런이고 뒤쪽 레이어임을 확인한 후 그 번호와의 경계 상자 교차만 구분하는 후보를 준비한다. 쪽수·번호 소유는 독립 PDF의 7쪽과 1~7을 검사하고, 본문끼리의 겹침·다른 바탕쪽 텍스트는 기존 래칫 판정을 유지한다. 특정 픽셀 좌표·83이라는 사건 수로 회귀를 고정하지 않는다. 새 test 함수는 추가하지 않는다.

![단축키 1쪽 fresh WASM 전체 인쇄 비교](../assets/pr7382_20260926/stage294_shortcut_wasm_review_001.png)

![단축키 4쪽 fresh WASM 소제목 경계 비교](../assets/pr7382_20260926/stage294_shortcut_wasm_review_004.png)

- 기존 #702의 두 검사도 같은 독립 출력에 맞게 교정한다. 현재 첫 검사는 페이지 수를 `≤8`로 느슨하게 검사하고 두 번째는 SVG의 좌표로 글자를 합쳐 `파일/편집`만 확인한다. 이미 검증한 PDF 7쪽에 따라 페이지 수 **7**, 첫 쪽 지우기 항목의 **왼쪽 3개/오른쪽 3개 단 소유**, 둘째 쪽의 **파일/미리보기/편집** 소속을 render tree의 실제 문단 문자열로 판정한다. 기존 test 함수 두 개를 유지하며 새 함수를 test로 등록하지 않고 절대 픽셀 기대도 추가하지 않는다.

## 보정295~296 — 기존 단축키 회귀 의미 교정과 27건 통과

- 보정293 코드의 교정 전 집중 nextest는 **23 PASS/1 FAIL**이고 실패는 `text_overlaps_do_not_grow_partition_15`의 단축키 신규 **83건**뿐이다(`stage295-focused-before-ledger-nextest.log`, exit 100). #702의 기존 두 함수는 같은 이름·수로 유지하면서 독립 PDF의 7쪽 구역 소유와 첫 쪽 지우기 항목의 좌우 단 소유를 검사하게 바꿨다. 둘째 쪽은 파일/미리 보기/편집 세 구역을 render tree 텍스트로 검사한다. SVG 좌표 클러스터링과 `≤8` 기대를 없앴다.
- 기존 corpus 검사에서 `basic/shortcut.hwp`에 한해 바탕쪽의 유일한 가시 런이 자동번호의 모델 공백을 치환한 **각 쪽 1~7**인지, 뒤쪽 바탕쪽 레이어에 속하는지 검사한다. 그 런과 진단 endpoint의 동일성으로 장식 번호와의 교차만 구분한다. 같은 출력에서 노드를 대응시키는 bbox 동일성은 비교에 쓰지만 절대 픽셀 위치나 겹침 건수는 기대값으로 고정하지 않는다. 원본·PDF·기존 원장 수치·검사 대상 목록·전역 진단기는 바꾸지 않았다. 새 test 함수도 추가하지 않았다.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 -E 'test(issue_702) | test(issue_6030) | test(issue_2299) | test(issue_2318) | test(issue_6318) | test(text_overlaps_do_not_grow_partition_15)' --no-fail-fast`는 **27/27 PASS**, 10238개 미선택, exit 0다(`stage296-focused-nextest.log`). 분할15 전체 코퍼스와 #6318 실제 바탕쪽 사이드바 충돌, #2318 뒤쪽 replay, 다른 TAC/편집 대조군도 통과했다. 파생 suite 준비·fmt·diff check 통과. CLI의 원시 진단 83건은 그대로이며 원장 통과를 렌더 변경으로 오해하지 않는다.
- Native와 WASM의 각 쪽 `rhwp_png/rhwp_001..007.png`는 **7/7 바이트 동일**하다. 전체 비교 점수와 실제 렌더도 일치하며, 두 backend의 역할 문자열·표 구역 보존 근거를 공유한다. PR 전체의 최종 검증은 아직 남는다.
- 최신 대상 `upstream/devel=02530b9ed567a44663edb26c65fb565c4a79f00d`를 fetch했고 검토 branch와 **46/400개**로 갈린다. `git merge-tree --write-tree upstream/devel HEAD`는 `mydocs/orders/20260930.md` add/add 한 곳만 보고하며 코드 텍스트 충돌은 없다. 최종 검증 전에 같은 브랜치에서 최신 대상 변경을 반영하고 양쪽 오늘할일을 보존한다.


## 보정297 — 최신 devel 반영 후 작은 문서와 도구 재검증

- 최신 `upstream/devel`의 `02530b9ed567a44663edb26c65fb565c4a79f00d`을 현재 검토 브랜치에 합쳤습니다. 후보는 `1c1d088cb6997ebcbd330c1119d923460aa59bfc`입니다. 충돌은 `mydocs/orders/20260930.md`의 독립 작업 기록뿐이며 양쪽 기록을 보존했습니다. source/test 충돌은 없었습니다.
- 후보에서 Native CLI와 fresh WASM을 각각 새로 빌드했습니다. 단축키 입력의 Native/fresh WASM 전체 7쪽은 독립 PDF와 같은 쪽수이고 최저 **96.12686%**, 양쪽 gate `passed`입니다. 일치율은 **99.97829/99.76583/97.53365/96.12686/99.95349/99.90031/99.95321%**입니다. 두 backend의 결과 PNG 바이트도 **7/7쪽 동일**합니다. 이전 후보의 판독을 최신 전쪽 직접 판독으로 표현하지 않으며, 점수는 2px 관용 비교여서 완전 픽셀 일치를 뜻하지 않습니다. [최신 base 검증](../assets/pr7382_20260926/stage297_current_base_validation.json)에 manifest와 해시를 기록했습니다.
- Visual Sweep Python 도구 검사는 **77 PASS**, 글꼴 규칙 Node 검사는 **22 PASS**, 각각 exit 0입니다. WASM 빌드는 Mac 로컬 대체 빌드이며 Docker 최적화 빌드 통과로 보고하지 않습니다.
- 비교 출력은 `output/pr-review/planet6897-7382-20260926/stage297-native-current-base/shortcut/review/`와 `stage297-wasm-current-base/shortcut/review/`입니다. 원본 HWP와 PDF는 현재 커밋 blob과 같은 바이트임을 확인했습니다. PDF나 허용치를 변경하지 않았습니다. 전체 nextest와 Native Skia·lint·원 PR 최종 전수 시각 검증은 별도 게이트로 유지합니다.


## 보정298 결과 / 보정299 사전 분석 — 전체 회귀의 단일 hit-test 차단

- 전체 nextest는 **10,229개 실행, 10,228 PASS / 1 FAIL / 50 SKIP**, 526.096초, exit **100**입니다. 실패는 `issue_7442_nested_cell_hit_test::nested_text_hit_keeps_same_offset` 한 건입니다. [고정 후보와 실행 증거](../assets/pr7382_20260926/stage298_full_nextest_validation.json). 고정 base의 suite manifest·source-unit 분류와 fmt check는 exit 0입니다. 아직 전체 통과나 PR 준비 완료로 보고하지 않습니다.
- #7442는 `samples/basic/issue1994_behindtext_table_20200830.hwp`의 첫 쪽 중첩 칸 글자에 `(64.5, 406.0)`라는 절대 클릭 좌표를 사용하고 `charOffset == 2`를 요구합니다. 실제 경로 깊이 2와 바깥 칸 10은 맞으며 offset만 1입니다. 같은 문서의 나머지 #7442 검사 6개는 통과했습니다. 고정 좌표가 다른 글자 위치를 가리키게 된 것인지 실제 hit-test 회귀인지 구분해야 합니다.
- 원본은 4쪽입니다. 이미 보존된 독립 PDF와 Native 전체 4쪽 비교를 시작했습니다. 다음으로 해당 중첩 경로의 모델 텍스트·커서 위치와 hit-test 왕복을 확인합니다. 정상적인 문자 경계에서 여전히 offset이 어긋나면 생산 hit-test를 보정하고, 고정 좌표의 기대 오류라면 기존 함수를 의미 검사로 교정합니다. 단순히 기대값 2를 1로 바꾸거나 새 테스트 함수를 추가하지 않습니다.


## 보정300 — 중첩 셀 글자 검사 고정 좌표 교정

- 독립 모델의 해당 셀 텍스트는 `* `이고 실제 셀 폭은 **8.4px**입니다. 글자 run 폭은 **18.5px**라 뒤 공백 커서 offset 2는 칸 밖에 있습니다. 공개 경로 커서 API는 offset 1·2 모두 칸 오른쪽으로 제한하며 `cellOverflowed=true`를 반환합니다. 원 검사 좌표는 별표 경계의 offset 1을 가리켜, offset 2라는 기대가 현재 가시 위치의 계약과 맞지 않습니다. 진단의 offset 0·1 커서→hit-test 왕복은 각 offset과 전체 깊이 2 경로를 보존했습니다.
- 기존 함수 한 개를 원본 `* ` 모델 텍스트 확인, 가시 별표 앞뒤 커서 경계 0·1의 hit-test 왕복과 전체 중첩 경로·부모 문단·section 소유 검사로 교정했습니다. 절대 픽셀 클릭을 없앴으며 기대 2를 단순히 1로 바꾸지 않았습니다. 나머지 #7442 함수 6개와 source는 유지했고 새 테스트 함수는 추가하지 않았습니다. 집중 **11/11 PASS**, exit 0입니다. [실행·시각 한계 기록](../assets/pr7382_20260926/stage300_issue7442_validation.json).
- Native 전체 4쪽을 두 기존 독립 PDF와 비교했습니다. 최신 보존 PDF의 해당 1쪽은 **91.49%**이고 review를 직접 확인했습니다. 3쪽은 최신 PDF **87.04102%**, 원래 #1994 PDF **87.12326%**로 남습니다. 해당 review도 직접 읽었으며 전 문서 시각 승인으로 보고하지 않습니다. 이번 변경은 편집 hit-test의 기대 교정이고 렌더링 배치를 변경하거나 회귀 fixture를 신규 등록하지 않습니다. PDF·허용치·기존 렌더링 검사는 유지합니다.
- 전체 후보를 다시 실행해 전수 통과 여부를 확인한 뒤 Native Skia와 나머지 필수 게이트를 진행합니다. 현재 통합 PR 준비·전체 시각 승인은 계속 보류입니다. 로그와 전쪽 비교는 `output/pr-review/planet6897-7382-20260926/stage299-issue7442-*`, `stage300-issue7442-focused.log`에 보존합니다.


## 보정301 — 기존 회귀 전수 통과

- 후보 `afcb5dcea`의 전체 nextest는 **10,229개 실행 / 10,229 PASS / 0 FAIL / 50 SKIP**, 523.871초, exit **0**입니다. 로그의 최종 summary와 실제 프로세스 exit를 모두 확인했습니다. [고정 후보·명령·로그 해시](../assets/pr7382_20260926/stage301_full_nextest_validation.json).
- #7442 기대 교정 뒤 기존 전체 corpus와 유지 회귀를 다시 실행했습니다. 이번 단계에서 새 회귀를 추가하거나 추가 함수를 skip·#7445 이관하지 않았습니다. 50 SKIP은 유지된 기존 보류 상태이며 해소로 보고하지 않습니다.
- 전체 통과 결과를 별도 커밋하고 Native Skia lib 전체와 공식 통합 두 경로, 필수 lint/빌드 및 최종 시각 증거를 이어서 확인합니다. 원 PR 문서·4쪽 주보의 남은 시각 차이와 PR 생성/merge는 아직 완료가 아닙니다.


## 보정302 — Native Skia 3종 통과와 원격 base 재확인

- 사용자 지정 nextest로 `--features native-skia --lib` 전체를 실행했습니다. 이름 filter 없이 **4,109 PASS / 0 FAIL / 13 SKIP**, exit 0입니다. 공식 통합 경로 `issue_2225_missing_picture_placeholder`는 **2 PASS**, `render_p37_direct_pdf_export`는 **4 PASS**, 각각 exit 0입니다. [명령 범위·로그 해시·검증 한계](../assets/pr7382_20260926/stage302_native_skia_validation.json).
- 첫 placeholder 실행은 기존 생성 파일과 변경된 함수 본문으로 다시 계산한 suite 배치가 달라 **0개 실행 / exit 4**였습니다. 이를 통과로 쓰지 않았습니다. 공식 `--prepare`로 생성 파일을 갱신하고 다시 실행해 위 결과를 확인했습니다. 재배치 전후 동일한 모듈 입력 **1,401개**, 중복 0개입니다. 검사 내용·함수 수·전체 검사 입력을 변경하지 않았으며 generated 파일은 커밋하지 않습니다. 앞선 전체 nextest는 같은 커밋 source의 모든 기존 검사 실행 결과입니다.
- 사용자 지적에 따라 원격 `devel`을 다시 fetch하고 `ls-remote`로 확인했습니다. 최신 SHA는 **02530b9ed**이고 검토 브랜치에는 **1c1d088cb**에서 이미 합쳤습니다. `HEAD..upstream/devel` 미반영 커밋은 **0개**, ancestor 확인도 성공했습니다. 현재 base 추가 갱신을 위한 리베이스는 필요하지 않습니다. 이 조회 시점 이후 새 병합이 있으면 제출 전에 다시 확인합니다.
- source 변경 없이 이 단계를 별도 커밋하고 필수 lint·빌드·정책·최종 시각 증거를 이어갑니다. 통합 PR 준비/전체 시각 승인/병합은 아직 완료가 아닙니다.


## 보정305 — 최신 devel 리베이스 / 보정307 사전 분석

- 사용자 요청에 따라 최신 `upstream/devel` `02530b9ed567a44663edb26c65fb565c4a79f00d` 위로 409개 검토 커밋을 리베이스했습니다. 후보는 `0790cfdc992dee6fb6a992a4e7b4d7730c7e5f5c`이며 이전 후보는 `codex/7382-pre-rebase-20261001`에 보존했습니다. 충돌은 20260930 작업 기록뿐이고 양쪽 내용을 보존했습니다. 전후 tree `6ec057d0430abc8ed0bebbe6c14cbcb98c9358cc`가 동일하여 코드 변경은 없습니다. 보정302의 리베이스 필요 여부 설명은 당시 조회 기록입니다.
- 정책연구 HWPX는 실제 Native CLI/API에서 216쪽, 독립 한컴2024 PDF는 215쪽입니다. 원본 쪽수 검사는 이전 #7445 이관으로 현재 실행되지 않으므로 전체 10,229 PASS가 이 쪽수 복원을 증명하지 않습니다. 기존 전체 통과와 원본 쪽수 차단을 구분합니다.
- 첫 소유 차이는 11쪽 그림6(문단240)의 다음 쪽 이월입니다. 앞 문단237 표와 238·239 글줄의 배치는 이전 215쪽 출력과 같습니다. 저장 그림6 상단은 본문 상대 291.56px인데 흐름 커서는 297px입니다. 기존 사전 이월 판정은 앞쪽의 임의 표 존재만으로 이 역전을 충돌로 봅니다. 실제 앞 표가 예약한 끝 경계와 비교하도록 보완하고 실제 충돌은 유지할 계획입니다. 고정 좌표나 PDF 쪽수를 변경하지 않습니다.
- 로그·진단은 ignored `output/pr-review/planet6897-7382-20260926/stage305-*`, `stage307-*`에 보존했습니다. 필수 빌드 파이프라인은 리베이스를 위해 중단(exit -2)했으며 통과로 기록하지 않습니다.


## 보정308 — 앞 표 점유와 후행 글줄 간격을 구분해 215쪽 복원

- 저장 원점과 현재 흐름 커서의 역전만으로 판단하던 경로를 보완했습니다. 현재 쪽의 앞 표에 확정한 배치 계획이 있으면 그 `occupied_bottom`과 다음 표의 저장 `table_top`을 비교합니다. 앞 표의 실제 점유를 넘어선 원점을 표 충돌로 취급하지 않습니다. 점유에 진입하는 표 또는 배치 계획이 없는 표는 기존 이월 판정을 유지합니다. PDF·저장 줄·허용치를 변경하지 않았습니다.
- 기존 검사 함수 안의 임시 진단으로 원본 11쪽 그림6·캡션 단일 소유·뒤 소제목과 전체215쪽을 확인했습니다. 수정 전 **13 PASS / 1 FAIL**, exit100(그림6 소유 실패), 수정 후 **14 PASS / 0 FAIL**, exit0입니다. 원본 HWPX와 HWP의 실제 CLI `info --json`도 각각 **215쪽**입니다. [전후 검증과 임시 검사 증거](../assets/pr7382_20260926/stage308_page_count_validation.json).
- 원본 전체 피델리티 90% 미만 문서를 새 회귀로 등록하지 않는 지침에 따라 임시 원본 조건은 진단 후 제거했습니다. 기존14개 함수와 파일 바이트는 유지하며 #7445의118개 이관 검사를 일괄 복구하지 않습니다. 쪽수 차단 수정과 전체 시각 피델리티 완료를 구분합니다.
- Native 전체215쪽 비교를 진행 중입니다. 실제 앞뒤 표 충돌을 막는 #5585 대조군·fresh WASM 쪽수와 전체 nextest를 다음 단계에서 재검증합니다. 앞선10,229 PASS를 이번 생산 코드 변경의 최종 통과로 재사용하지 않습니다.


## 보정309 중간 결과 — fresh WASM 215쪽 및 충돌 대조군

- 생산 코드 후보 `016aba9bf`에서 기존 #7379 **14/14 PASS**, #5585 **2/2 PASS**, 각각 exit0입니다. #5585의 독립77·78쪽 앞뒤 표 소유 검사가 유지되어 실제 표 충돌 방지는 검증했습니다. 수정 전 별도 debug 출력기의 HWP도216쪽이었고, 수정 후 HWP/HWPX CLI는 각각215쪽입니다.
- Mac fresh WASM 빌드는3분33초, exit0입니다(로컬 대체 빌드이며 Docker 최적화 통과 아님). WASM 전체 문서 쪽수215, 선택11·12·215쪽은 **99.57078/99.93582/96.77735%**, 선택 gate passed, exit0입니다. Native와 WASM의11·12쪽 PNG는 동일합니다. Native11쪽과 WASM215쪽을 직접 판독해 그림6·뒤 소제목·각주 및 마지막 본문/꼬리말 소유를 확인했습니다.
- 이전215쪽 출력과 현재215개 트리의 본문 문단·표 소유 순서는 **215/215쪽 동일**합니다. 이전 출력은 페이지 소유 대조 자료이며 독립 PDF나 전쪽 시각 통과의 대체 기준으로 쓰지 않습니다. [선택 시각·전쪽 소유 증거](../assets/pr7382_20260926/stage309_wasm_page_count_validation.json), [11쪽](../assets/pr7382_20260926/stage309_wasm_review_011.png), [215쪽](../assets/pr7382_20260926/stage309_wasm_review_215.png).
- Native 전쪽 이미지 비교와 변경 후 전체 nextest는 아직 진행 중입니다. 전체 피델리티90% 미만의 기존 #7445 범위와 쪽수 복원을 구분하며 그 검사를 일괄 복구하지 않습니다.


## 보정310 사전 분석 — 전체 회귀 반례와 사용자가 확인한 잔존 시각 차단

- 보정308 생산 코드 후 전체 nextest는 **10,228 PASS / 1 FAIL / 50 SKIP**,640.890초,exit100입니다. 반례는 `hwpctl_ParameterSetID_Item_v1.2.hwp` 물리70쪽입니다. 앞 표는 끝났지만 본문174의 가시 글줄과 다음 저장 표175가 겹쳤습니다. 보정308이 표 예약 하단만 확인한 범위 오류이며 기준값 변경·검사 제거로 통과시키지 않습니다. 일반 문단을 확정하는 단계에서 뒤 줄간격을 제외한 본문 하단을 기록해 같은 충돌 판정에 포함할 계획입니다. [전체 실패 증거](../assets/pr7382_20260926/stage310_full_nextest_blocker.json).
- 사용자 직접 판독: 정책연구10쪽67.90467%,17쪽64.97015%,28쪽88.24856%,75쪽52.88891%,88쪽89.58124%를 시각 차단으로 확인했습니다. 추가74쪽86.98592%도 자동 목록에 있습니다. 84·85쪽은90% 이상이어도 각주133의 소유가 PDF84쪽에서 rhwp85쪽으로 넘어가 구조 차단입니다. 이미지 점수만으로 승인하지 않습니다.
- 10쪽 그림3은 원본BMP4(1020×529)의 배경RGB224/235/255와 rhwp raster가 같은 반면 독립 기존PDF의 같은 크기JPEG는234/242/255입니다. 크기·DPI를 추측해 renderer 색을 임의 변경하지 않습니다. 사용자가 지정한 비공개 환경의 Hancom2024 client로 원본 전체 기준PDF를 재산출 요청했습니다(1800초). 서버정보·인증값은 기록하지 않았습니다. 결과와 기존PDF를 대조한 뒤 기준 및 렌더러 원인을 결정합니다.
- 17쪽 빈 표295는 앞 쪽 저장host vpos59941과 문단offset3022HU를 갖고 fresh 쪽으로 넘어간 뒤에도 오프셋을 적용합니다. 28쪽 그림32는 두 줄 host384의 기준 줄 선택,75쪽은 본문 첫 줄과 각주100의 이월,88쪽은 본문 수식 폭 차이를 순서대로 검토합니다. 아직 해결이나 전체 시각 승인으로 기록하지 않습니다.


## 보정311 — 저장 표 앞의 가시 본문 하단도 함께 예약

- 전체 일반 문단을 확정할 때 같은 구성 결과의 가시 하단(뒤 줄간격·문단 아래 여백 제외)을 기존 본문 조각 하단 원장에 기록합니다. 현재 쪽의 모든 앞 가시 본문이 다음 저장 표 원점보다 위에 끝났음이 입증될 때만 앞 표 점유 하단에 따른 간격 공유를 허용합니다. 일반 본문 기록이 없으면 기존의 보수적인 이월을 유지합니다. 원본 저장vpos를 실제 새 쪽 글줄 좌표로 오인하지 않습니다.
- 겹침 검사의 전체16개 partition **16/16 PASS**, 기존 #7379 **14/14 PASS**, 각각 exit0입니다. 첫 컴파일의 누락 import를 수정한 뒤 재컴파일·재검증했습니다. 원본 HWP/HWPX CLI는 각각215쪽을 유지합니다. 새 회귀 함수·기준값 변경은 없습니다. [검증 원장](../assets/pr7382_20260926/stage311_text_content_reservation_validation.json).
- 생산 코드 후 전체 nextest는 아직 재실행 전입니다. 사용자가 확인한 그림·각주 잔존을 순서대로 검토하고 마지막 후보로 전수 검증합니다.
- 원본의 fresh Hancom2024 PDF 변환은 성공(215쪽,3,790,026bytes)했습니다. 그림3 JPEG의 배경색도 기존PDF와 같아 재출력만으로10쪽67%가 해소되지 않음을 확인했습니다. 기존PDF는 유지하며 새PDF는 현재 ignored output에서 대조 중입니다.


## 보정312 — PNG 합성 없는 실루엣 TSV 모드

- 사전 분석: 전쪽 sweep에서 raster 이후 compare·overlay·review 합성과 상세 분석이 보조값 확인을 지연합니다. 기존 2px·90% 계산 계약을 유지하고 별도 `--silhouette-only` 모드와 기존 raster 재사용 `--png-pair`를 추가했습니다. 최소 RGB 마스크와 불일치 집계를 동일한 PIL 연산으로 계산해 픽셀별 Python 순회도 줄였습니다.
- 실제 Native 10·11·17·28·75·84·85·88쪽 8개를 **0.891초**에 TSV로 산출했습니다. 기존 overlay 지표와 소수점5자리까지 **8/8 동일**, 새 PNG0개입니다. 90% 미만 쪽이 있어 예상대로 exit1입니다. 원문 선택11쪽 export 경로도 exit0이고 compare·overlay·review 디렉터리를 생성하지 않았습니다.
- 기존 Python 도구 테스트 **77/77 PASS**, exit0입니다. 새 테스트 함수를 추가하지 않고 기존 검사에서 TSV 지표 일치·PNG 미생성·누락 입력 거부·승인 판정 금지를 확인했습니다. 가이드에 명령·입력 provenance·검증 범위를 추가했습니다. [실행 증거](../assets/pr7382_20260926/stage312_silhouette_tsv_validation.json).
- 보조값이 모두90% 이상이어도 이 모드는 `not_evaluated`로 기록합니다. 각주·문단 소속과 전체 쪽수 및 직접 시각 검토는 별도로 필요합니다. 사용자 확인 시각 차단과 최종 전체 renderer 검증은 해결·통과로 기록하지 않습니다.


## 보정313 — 정책연구 문서 전체215쪽 TSV 산출

- `stage308-native/liver7379`의 원본 HWPX/독립 PDF 비교 raster가 양쪽215쪽 모두 완성되어 `--silhouette-only --png-pair`로 전체를 산출했습니다. 원문 재출력·PNG 합성 없이 **23.711초**, 새 PNG0개, 기존 overlay 실루엣 지표와 **215/215 동일**합니다.
- 90% 미만은 **12쪽**, 최저 **38.30328%**이며 예상대로 exit1입니다. 해당 쪽은 10쪽(67.90467%), 17쪽(64.97015%), 28쪽(88.24856%), 74쪽(86.98592%), 75쪽(52.88891%), 88쪽(89.58124%), 107쪽(38.30328%), 169쪽(42.83023%), 199쪽(89.94190%), 207쪽(89.61032%), 208쪽(85.36044%), 211쪽(81.63850%)입니다. 기존에 사용자가 확인한 차단과 함께 다음 개별 보정 대상으로 유지합니다.
- 84·85쪽은 각각93.25790/93.59277%여도 각주133 소유 차이가 있어 구조 보류입니다. 이 TSV는 보정308 raster의 재산출 결과이며 현재 HEAD의 재출력·fresh WASM·최종 전체 검증 완료로 사용하지 않습니다. 원본 실행에 dirty 수정이 있어 기록된 git SHA만으로 생산 코드 전체를 특정하지 않고 binary/source 입력 해시를 함께 보존했습니다.
- [전체 TSV](../assets/pr7382_20260926/stage313_pr7382_silhouette.tsv), [명령·입력 provenance·실행 결과](../assets/pr7382_20260926/stage313_pr7382_silhouette_validation.json). 로컬 경로는 `output/pr-review/planet6897-7382-20260926/stage313-pr7382-tsv/silhouette.tsv`입니다. 로그는 ignored output에만 남겼습니다.


## 보정314 사전 분석 — 각주 표시 쪽과 본문 꼬리 쪽의 소유

- 사용자가 지적한84·85쪽은 각주133이 앞 표시 쪽에서 뒤 꼬리 쪽으로 옮겨졌습니다. 같은 형태로74·75쪽 각주100도 소유를 확인합니다. 90% 이상 점수는 각주 수량 차이를 면제하지 않습니다.
- 본문 각주 등록 `notes/footnotes/body.rs`는 표시를 가진 완료 문단 조각을 찾고 HWPX에서는 실제 본문 하단+추가 각주 높이가 수용되는지 확인합니다. 그 뒤 `whole_note_owned_by_tail_page`가 저장 글줄 하단이 본문 높이90% 이상이라는 별도 heuristic으로 완료 쪽 결정을 덮습니다. 실제 점유 예산 판정과 소유 덮어쓰기의 모순 여부를 진단하고, 기존 설명의131쪽 각주180 반례를 함께 확인합니다. 아직 원인 확정·수정 완료로 기록하지 않습니다.

- 진단 결과 각주133은 해당90% heuristic에 도달하지 않았습니다. 앞 가설을 수정합니다. host916은 각주133·134·135를 함께 가진 문단인데 본문 reset helper가 `controls.len()==1`로 제한하여 유효한 앞 쪽 소유 조회 자체를 생략했습니다. 각주180은 단일 표시 문단이며 실제 완료 쪽 예약이 수용되지 않아 route가 없었습니다. 복수 각주만 든 본문 문단에도 저장 표시 줄·실제 수용 예산 판정을 사용하도록 helper를 확장합니다. 각주100은 다른 본문 컷 원인으로 별도 분석하며 이번 수정으로 해결했다고 기록하지 않습니다.

- Native 보정 결과: 84쪽 각주131·132·133,85쪽134·135·136으로 독립PDF와 일치합니다. 215개 트리의 각주 번호를 전수 비교해 **변경 쪽은84·85뿐**이며,131·132쪽 각주180 대조 소유와 전체215쪽은 유지됩니다. HWP84쪽도131·132·133입니다. 새 TSV84·85쪽은 **97.06154/95.14701%**,131·132쪽은94.77080/99.77961%입니다. 84·85쪽 새 raster와PDF를 각각 직접 판독했습니다.
- 기존 #7379 집중 검사 **14/14 PASS**,exit0입니다. 새 회귀 함수·fixture·기준값 변경은 없습니다. [검증 결과](../assets/pr7382_20260926/stage314_multi_body_note_owner_validation.json), [84쪽](../assets/pr7382_20260926/stage314_native_084.png), [85쪽](../assets/pr7382_20260926/stage314_native_085.png). fresh WASM 재빌드는 진행 중이며 최종 전체nextest는 재실행 전입니다. 각주100과90% 미만12쪽 해결 완료로 확장하지 않습니다.

- 보정314 fresh WASM 빌드는2분45초·exit0(로컬 대체 빌드)입니다. 선택84·85·131·132쪽 TSV 산출exit0이고 Native와4쪽 PNG가 모두 동일합니다. 기존 debug binary 거부를 받은 첫 실행을 기록하고 최신 명시 binary로 재실행했습니다. 최종 전체 검증·다른 차단은 미완료입니다.


## 보정315 사전 분석 — 첫 표 조각 이월 후 문단 오프셋 재적용

- 17쪽 표6은 저장host59941HU·문단offset3022HU를 가진2행 그림/설명 표입니다. 실제 스캔은 첫290.4px 행이 앞 쪽 잔여0px에 들어가지 않아 새 쪽으로 이월합니다. 현재 각주 수량과 원본 그림은 맞지만 표 첫 원점이PDF보다약36.5px 아래로 밀립니다.
- `block/prepare.rs`의 첫 행 이월 → `continuation/fragment/budget.rs` 예산 → fragment 배치 원점으로 추적했습니다. 현재 공통 offset 소진 helper는 앵커+offset+25px가 쪽을 넘는지만 보므로, 첫 행 전체가 못 들어가 실제로 이월한 상황을 보존하지 못합니다. 예산 차감과 출력이 같은 실제 이월 사실을 소비하도록 첫 조각 준비 상태에 전달할 계획입니다. 임의 좌표 clamp·문서번호 분기·비교 지표 변경은 하지 않습니다. 실제 분할 없이 이월한 빈 호스트의 문단 상대 흐름 표와 안정된 저장 줄에 한정하고, 본문/편집/TAC/절대 배치는 기존 경로를 유지합니다.


## 페이지별 TSV 검증 지침 전파 — 2026-10-01

- 사전 분석: TSV 전용 모드는 구현되어 있으나 기여자·에이전트·PR 접수/검증 지침에 전파가 부족하여 전쪽 PNG 합성과 대표 이미지 중심 점수 확인으로 되돌아갈 여지가 있었습니다. 기존 PNG 재사용과 새 head 출력의 출처도 구분할 필요가 있습니다.
- `CONTRIBUTING.md`, `CLAUDE.md`, `AGENTS.md` 및 시각 sweep·거버넌스·fixture 증적·로컬 검증·접수 가이드의 총8개 문서를 보완했습니다. 검증 대상 전쪽 Native/fresh WASM TSV를 먼저 산출해 최저값·90% 미만·누락 쪽을 확인하고, 해당 쪽·구조 차이·대표 경계는 별도 output에서 PNG로 직접 판독합니다. 정본에는 명령·저장 위치·열 의미·재사용 출처·전체 쪽수 대조를 추가했습니다.
- TSV 측정 성공은 승인 완료가 아닙니다. 대표 PR 이미지, 각주 수량·문단 소속·내용 누락/중복·전체 쪽수 검증을 유지합니다. 90% 미만은 `re_review_required`, 모두90% 이상인 TSV 전용 결과는 `not_evaluated`입니다.
- 변경8문서 내부 상대 링크, 매뉴얼5문서 메타데이터, TSV 연결 anchor와 CLI 옵션, `git diff --check` 모두 통과했습니다. 일반 문서 수정으로 Rust/전체 렌더링 검증을 새로 실행한 것으로 기록하지 않습니다. 진행 중인 보정315와 최종 PR 준비 상태는 이 문서 보완으로 완료되지 않습니다.


## 보정315 결과 — 첫 조각 이월 사실과 선언 프레임의 점유 범위

- 실제 첫 행 이월 사실을 준비 상태에 보존해 첫 새 쪽의 예산과 출력 원점이 같은 문단 거리 소진을 사용하도록 보완했습니다. 안정된 저장 줄·빈 문단 호스트·단일 단·문단 상대 흐름 표에 한정합니다.
- 초기 후보는17쪽을 개선했지만24쪽의 선언 프레임243.4px에 셀 전체490.4px를 흐름 점유로 강제하여 뒤 그림을 밀었습니다. 전쪽 검증에서 발견한 자체 회귀를 기록하고, 선언 프레임이 측정 행 점유를 덮는 경우에만 새 쪽 배치를 사용하는 공통 출처 조건을 추가했습니다. 문서 ID·행수 특례나 임의 좌표 보정은 사용하지 않았습니다.
- 최신 Native/fresh WASM 모두215쪽입니다. 17쪽64.97015→99.75662%,24쪽은92.98011%로 복구됐고16·18·23·25쪽도99% 이상입니다. Native 전쪽 raster를 보정308과 대조해 변경은17·84·85쪽뿐이며, 나머지212쪽은 동일합니다. 17쪽의 표·설명·후행 제목·각주를 독립PDF와 직접 판독했습니다. 양 backend215쪽 raster도 모두 동일합니다.
- 기존 #7379 회귀14/14 PASS, nextest exit0(threads8), fmt exit0, 단위 테스트 분류 정책(base02530b9ed) exit0입니다. 새 함수·fixture·기준값은 추가하지 않았습니다. 전체 TSV와 검증 증거는 보정317 기록으로 연결합니다. 잔존11쪽 미달 및 각주100·258 소유와 최종 전체 검증은 미완료입니다.


## 보정317 — 최신 코드 Native/fresh WASM 전체 TSV 재생성

- 사용자 요청에 따라 문서 전파 커밋88cb306a8을 포함한 브랜치에서 원본 정책연구 HWPX와 독립PDF를 새로 출력했습니다. 기존 PNG 재사용이 아닙니다. 실행 당시 보정315 생산 코드가 미커밋 상태여서 파일·patch·binary hash를 보존했으며, 검증 후 cf0eb806f로 커밋된 생산 코드와 파일 해시가 같음을 확인했습니다.
- Native와 fresh WASM 각각215쪽, 기준PDF215쪽, 누락0쪽입니다. 양 backend TSV와PNG가215/215쪽 동일하고 compare·overlay·review PNG는 새로 생성하지 않았습니다. 모두 미달이 있어 exit1/`re_review_required`로 정상적으로 보류됩니다.
- 최저38.30328%,90% 미만11쪽:10(67.90467),28(88.24856),74(86.98592),75(52.88891),88(89.58124),107(38.30328),169(42.83023),199(89.94190),207(89.61032),208(85.36044),211(81.63850).17쪽은99.75662%로 해소됐습니다.84·85쪽은97.06154/95.14701% 및 올바른 각주 번호 소유를 유지합니다. 각주100·258 소유는 잔존합니다.
- [Native TSV](../assets/pr7382_20260926/stage317_pr7382_native_silhouette.tsv), [fresh WASM TSV](../assets/pr7382_20260926/stage317_pr7382_wasm_silhouette.tsv), [명령·출처·해시·종료 결과](../assets/pr7382_20260926/stage317_pr7382_silhouette_validation.json). 전체 저장소 문서의 검증 완료나 최종 전체nextest 통과를 뜻하지 않습니다. 통합 PR은 아직 준비 완료가 아닙니다. 로그와 임시 raster는 ignored output에만 보존합니다.


## 보정318 사전 분석 — 74·75쪽 본문 reset과 각주100

- PDF74쪽 각주는99·100,75쪽은101인데 최신 HWPX는74쪽99,75쪽100·101입니다. host839의 저장 첫 줄58000HU 뒤0HU reset에도 본문 두 줄이74쪽에 남아 각주100 소유 조회가 유효한 경계를 찾지 못합니다.
- 기존 각주 영역의 실제 높이·앞 marker의 추가 높이·저장/흐름 일치·다음 줄의 실제 침범을 확인하는 reset helper가 HWP5에만 열려 있습니다. 미편집 단일 단의 안정된 저장 HWPX에도 같은 물리 계약을 적용하되 각주 측정과 충돌 조건은 유지합니다.74·75쪽, 앞뒤 쪽 및84·85·131·132쪽 소유 반례를 먼저 검증하고 결과에 따라 보정합니다.

- 보정318 결과: Native/fresh WASM74쪽97.66915%,75쪽98.79649%.74쪽각주99·100,75쪽101으로 독립PDF와 일치합니다.73·76·84·85·131·132쪽 대조도90% 이상,8쪽 PNG가 양backend동일하고 전체215쪽은 유지됩니다.75쪽 본문·그림·각주를 독립PDF와 직접 판독했습니다.기존 #7379 회귀14/14 PASS, 각 실행exit0이며 새 회귀함수·fixture·비교 기준 변경은 없습니다. [검증 원장](../assets/pr7382_20260926/stage318_body_note_reset_validation.json), [Native TSV](../assets/pr7382_20260926/stage318_native_silhouette.tsv), [WASM TSV](../assets/pr7382_20260926/stage318_wasm_silhouette.tsv), [75쪽 출력](../assets/pr7382_20260926/stage318_native_075.png).원 미달11쪽 중2쪽을 해결했고 나머지9쪽 및 최종전수검증은 계속합니다.


## 보정319 사전 분석 — 마지막 본문 각주258과 다음 쪽 reset

- 199쪽 독립PDF에는257만 있고258은200쪽에서 시작합니다.최신HWPX는257·258을199쪽에 함께 놓습니다.기존 helper가 마지막 줄의 단일 한줄각주,다음 본문0원점,각주 추가 시 실제 본문 예산 초과를 함께 검사하지만 HWP5에만 열려 있습니다.미편집 단일단 저장HWPX에도 동일조건을 적용하고199·200 및131·132쪽 대조를 확인합니다.단순 쪽번호/좌표 이동이나 점수 면제는 사용하지 않습니다.

- 보정319 결과: Native/fresh WASM199쪽97.36126%,200쪽99.70533%.199쪽257,200쪽258로 독립PDF각주 소유와 일치합니다.198·201·131·132쪽도90% 이상이며6쪽 PNG양backend동일,전체215쪽 유지입니다. 기존#7379 회귀14/14 PASS,각실행exit0이며 새 함수·fixture·기준값을 추가하지 않았습니다. [검증 원장](../assets/pr7382_20260926/stage319_final_marker_note_validation.json),[Native TSV](../assets/pr7382_20260926/stage319_native_silhouette.tsv),[WASM TSV](../assets/pr7382_20260926/stage319_wasm_silhouette.tsv).원미달목록8쪽과 최종전수검증은 잔존합니다.


## 보정320 사전 분석 — 106·107쪽 표29 첫 프레임의 온전한 행 소유

- 원문표1136의 선언첫프레임315.4667px는 온전한 앞3행의 실제paint높이합과 정확히 일치합니다.하지만첫조각4px꼬리말보정이일반sourceframe조회까지금지하고,행scanner가앞4행363.7px를수용하여107쪽의국내이타적기증행이누락됩니다.후행본문의양수vpos되감김은이표가이어받기쪽을갖는추가증거입니다.
- 일반최근접/확장특례는유지하되 안정된미편집저장빈호스트·되감김·무각주표에서선언프레임이실제온전한행끝과정확히같을때만첫scanner의행소유끝을제한합니다.106·107·108쪽과기존17·24쪽프레임반례를검증합니다.원문/PDF·비교식·행내용은변경하지않습니다.

- 보정320 결과: 선언프레임과정확히같은온전한행끝을첫scanner의소유끝으로사용해106쪽앞3행/107쪽나머지5행을복원했습니다.Native/fresh WASM106쪽99.88046%,107쪽98.86043%,105·108·17·24쪽도90% 이상이며6쪽PNG가양backend동일합니다.107쪽행내용·괘선·캡션·후행본문을독립PDF와직접판독했습니다.첫빌드의변수참조누락을수정한재빌드exit0,기존#7379 회귀14/14 PASS,새함수·fixture·기준값은추가하지않았습니다.[검증 원장](../assets/pr7382_20260926/stage320_exact_row_owner_validation.json),[Native TSV](../assets/pr7382_20260926/stage320_native_silhouette.tsv),[WASM TSV](../assets/pr7382_20260926/stage320_wasm_silhouette.tsv),[107쪽 출력](../assets/pr7382_20260926/stage320_native_107.png).원미달목록7쪽과최종전수검증은잔존합니다.


## 보정321 사전 분석 — 168·169쪽 저장 표 프레임의 마지막 줄간격

- 169쪽 표44의 앞 조각에 남아야 할 `이식대상자로부터 설명동의를 구함` 한 줄이 뒤로 넘어가 표와 그림65를 약26.7px 내립니다. source row2는 8000HU 뒤 다음 문단0HU를 기록하며 첫 프레임의 선언329.2667px는 앞 행들과 마지막 가시 줄·패딩을 포함한 높이와 일치합니다.
- 마지막 줄의 그려지지 않는 간격을 제외하는 기존 다행 표 helper가 HWP5에만 열려 있습니다. 원본 HWPX에서도 선언 첫 프레임과 실제 prefix의 가시 점유가 정확히 일치하고 형제 셀이 prefix 안에서 모두 완결되는 경우에 한해 같은 간격 제외를 사용합니다. 문단 로컬 reset만으로는 열지 않으며 편집·개체·rowspan을 제외합니다.168·169·170쪽과 기존 작은 표·후행 본문 대조를 검증합니다.

- 보정321 결과: Native/fresh WASM 169쪽 **99.91753%**, 168쪽 99.94271%, 170쪽 99.95118%입니다. 17·24·106·107쪽도 90% 이상이며 7쪽 PNG가 양 backend에서 동일하고 전체 215쪽을 유지합니다. 169쪽 표 내용·괘선·그림·캡션을 독립 PDF와 직접 판독했습니다. 기존 #7379 회귀 14개와 줄간격 회귀 1개 모두 PASS, 빌드 및 최종 비교 exit 0입니다. 새 테스트 함수나 fixture·허용치는 추가하지 않았습니다. [검증 원장](../assets/pr7382_20260926/stage321_saved_row_gap_validation.json), [Native TSV](../assets/pr7382_20260926/stage321_native_silhouette.tsv), [WASM TSV](../assets/pr7382_20260926/stage321_wasm_silhouette.tsv). 원래 미달 목록에서는 10·28·88·207·208·211쪽 6쪽이 잔존하며 최신 전체 전수 검증은 아직 수행하지 않았습니다.


## 보정322 사전 분석 — 28쪽 그림32의 저장 앵커 줄

- 문단384의 두 저장 줄은 8000·10000HU이며 그림 컨트롤은 본문 마지막 줄 끝에 붙어 있습니다. 문단 원점을 사용하면 그림 top이10650HU가 되어 아직 전체 폭인 마지막 글줄(10000~11000HU)을 침범합니다. 실제 PDF는 마지막 줄 기준12650HU입니다. 다음 문단385의12000~13000HU 줄은 그림의 좌우 경계21740·42702HU에서 나뉘어, 후자가 유효한 그림 프레임임을 독립적으로 확인합니다.
- 컨트롤의 원시 UTF-16 소유 줄과 깨끗한 연속 저장 줄, 뒤 문단의 가로·세로 carve가 같은 그림 프레임을 증명할 때 배치 계획에 앵커를 저장합니다. 본문 원점·캡션·그림 오프셋을 따로 이동하지 않으며 실제 paint와 어울림 예약이 계획을 함께 소비하게 합니다. 편집·합성·되감김·불명확한 carve는 기존 경로를 유지합니다.

- 보정322 결과: Native/fresh WASM 28쪽 **98.43605%**입니다. 27·29·17·24·107·169쪽도 90% 이상이며 7쪽 PNG가 양 backend에서 동일하고 전체 215쪽을 유지합니다. 그림32와 캡션·주변 본문을 PDF와 직접 판독했습니다. 기존 #7379 회귀 14개 및 개체 프레임 회귀 13개 모두 PASS, 최종 빌드·비교 exit 0입니다. 새 테스트 함수나 fixture·허용치는 추가하지 않았습니다. [검증 원장](../assets/pr7382_20260926/stage322_tail_picture_anchor_validation.json), [Native TSV](../assets/pr7382_20260926/stage322_native_silhouette.tsv), [WASM TSV](../assets/pr7382_20260926/stage322_wasm_silhouette.tsv). 원래 미달 목록에서는 10·88·207·208·211쪽이 잔존합니다.


## 보정323 사전 분석 — 88쪽 수식의 한글 대체 글꼴과 가로 점유

- 원문 수식은 HYhwpEQ·baseUnit800·선언 폭15268HU입니다. PDF 수식의 한글 33자는 Haansoft Batang으로 출력되고 전진폭이 약10px인 반면 Native는 수식 전체를203.6px에 압축합니다. 선언 폭만 실제 대체 글꼴의 가시 점유보다 작고 본문 줄바꿈도 함께 틀립니다. 수식은 그림과 달리 스크립트·기본 글자 크기로 재조판되는 개체입니다.
- 한글을 수식 글꼴 밖의 전각 글꼴로 출력하는 경로에서 스크립트 기반 가로 점유를 공통으로 계산하는 후보를 검증합니다. 측정·인라인 조합·표/본문 배치가 같은 폭을 소비해야 하며 SVG만 늘리면 안 됩니다. 대체가 없는 기존 수학식의 저장 폭은 유지합니다. 후보의 최종 폭·줄 소유는 독립 PDF와 판독하며, 불일치하면 적용 근거를 재검토합니다. 새 회귀 함수·fixture·기준값은 추가하지 않습니다.

- 보정323 후보 기각: 공통 intrinsic 폭 후보는 88쪽 89.87177%이며 저장 줄 분할을 유지해 우단 넘침이 생겼습니다. 문단 프레임 재조판 후보는 86.56953%로 더 낮아졌습니다. 가로 선언 폭만 무효화하거나 한글 유무만으로 문단을 다시 나누는 판정은 충분한 저장 유효성 근거가 아닙니다. 두 후보는 해결로 인정하지 않고 소스 변경을 되돌렸으며 분석용 patch·TSV·로그는 ignored output에 보존했습니다. 다음 원인 분석에서 본문 들여쓰기·각주 참조 폭·실제 수식 대체 글꼴의 계약을 함께 확인합니다. 회귀 기준·비교식·원문·독립 PDF는 변경하지 않았습니다.


## 보정324 사전 분석 — 빈 표 호스트 앞뒤 문단 간격의 저장 경계

- 211쪽 표50 호스트 앞 문단의 마지막 줄은49900HU, 줄 높이·간격은 각각1000HU입니다. 앞 문단 뒤 간격300HU와 호스트 앞 간격300HU를 더하면 호스트 저장 좌표52500HU와 정확히 일치합니다. 따라서 이 경계에는 표 높이가 선소비되지 않았습니다. 기존 커서는 빈 TopAndBottom 표라는 이유로 호스트 좌표를 버리고51900HU를 사용하여 두 문단 간격8px를 제거합니다. 독립 PDF 표 상단은 약780px이며 Native는771.2px입니다.
- 공통 HeightCursor에서 앞 문단의 가시 텍스트·연속 저장 줄과 명시적 앞뒤 간격의 정확한 합이 호스트 경계를 증명하는 경우 저장 원점을 사용합니다. 표 예약 높이를 포함한 호스트·되감김·편집·직전 개체는 기존 경로를 유지합니다. 표 높이나 그림을 임의로 늘리거나 최종 좌표를 clamp하지 않습니다. 측정과 실제 배치가 같은 커서를 소비하며 표 뒤 캡션과 쪽 소유도 함께 검증합니다. 새 회귀 함수·fixture·허용치는 추가하지 않습니다.

- 보정324 결과: Native/fresh WASM 211쪽 **99.41819%**, 210·212쪽도99% 이상입니다. 17·24·28·107·169쪽을 포함한 대조8쪽 모두90% 이상이며 PNG가 양 backend에서 동일하고 전체215쪽을 유지합니다. 표·캡션을 독립 PDF와 직접 판독했습니다. 빌드·집중14개 회귀·선택 비교 exit0입니다. 최신 Native 전쪽 TSV도 다시 산출했으며 미달은10·88·207·208쪽4쪽입니다. 새 테스트 함수나 fixture·허용치는 추가하지 않았습니다. [검증 원장](../assets/pr7382_20260926/stage324_paragraph_gap_validation.json), [Native 전쪽 TSV](../assets/pr7382_20260926/stage324_native_all_silhouette.tsv), [WASM 선택 TSV](../assets/pr7382_20260926/stage324_wasm_silhouette.tsv). 최종 전쪽 WASM과 전체 회귀는 계속 필요합니다.


## 보정325 사전 분석 — 같은 글꼴 이름의 TrueType·HFT 출력 구분

- 208쪽 가운뎃점 다음 글자는 PDF보다9.36px 왼쪽입니다. 독립 PDF의 실제 휴먼명조 프로그램은 TrueType이며 설치된 HMKMM.TTF의 가운뎃점·여닫는 따옴표는512/512 전각입니다. 기존 폭307/1024는 Type3/HFT 출력의 호환 폭입니다. 원본의 TTF 선언만으로 양자를 구분할 수 없으므로 전역 폭을 바꾸지 않습니다.
- 기존 명시적 FontEnvironment에 호출자가 확인한 TrueType realization을 선택하는 세션 설정을 추가하는 후보를 검증합니다. 알려진 실제 TrueType 글리프 폭만 측정에 적용하고 같은 선택의 원 글꼴 outline을 출력에 사용합니다. 프로필만으로 파일 설치를 인증하지 않으며 실제 PDF·설치 파일 해시·임베드 결과를 함께 확인합니다. HFT 및 프로필 없는 기존 경로는 유지합니다.207·208쪽과 앞서 보정한 경계를 비교하고 기존 환경 회귀를 실행합니다. 새 회귀 함수·fixture·허용치는 추가하지 않습니다.

- 보정325 중간 반례: 전쪽 Native에서 TrueType 파일의 따옴표 전각을 문단 전진폭으로 그대로 사용한 후보가163쪽89.39386%를 만들었습니다. 실제 한컴 문단은 이 기호를 반각으로 전진시킬 수 있으므로 파일 글리프 폭만으로 확장하지 않습니다. 가운뎃점만 독립 PDF의 전각과 일치하는 범위로 유지하고 따옴표 후보는 제거하여 다시 비교합니다. 중간 전쪽 TSV·후보 결과는 output에 보존하며 통과로 계산하지 않습니다.

- 보정325 결과: 확인된 TrueType 세션의 가운뎃점 전진폭과 원 글꼴 outline 임베드를 함께 적용하여 Native/fresh WASM207쪽 **93.36062%**,208쪽 **96.82326%**입니다. 최종 선택5쪽의 PNG가 양 backend에서 동일하며215쪽을 유지합니다. 기존 폰트 환경 회귀5/5 PASS, 최종 빌드 exit0입니다. 원본·PDF·전역 호환 메트릭 표를 변경하지 않았고 새 테스트 함수·fixture·기준값도 추가하지 않았습니다. [검증 원장](../assets/pr7382_20260926/stage325_true_type_realization_validation.json), [세션 환경](../assets/pr7382_20260926/stage325_font_environment.json), [Native TSV](../assets/pr7382_20260926/stage325_native_r2_silhouette.tsv), [WASM TSV](../assets/pr7382_20260926/stage325_wasm_r2_silhouette.tsv).
- 163쪽은 따옴표 제거 뒤에도89.39386%로 같았습니다. 따옴표가 원인이라는 중간 추정을 철회합니다. 직접 판독에서 표42 첫 조각이 선언 첫 프레임보다1000HU 작아지고 가운데 정렬 본문도 위로 올라가는 기존 줄간격 제거 문제를 확인했습니다. 이는 후속 개별 보정으로 해결합니다. 최초 전쪽 후보 미달은10·88·163쪽이며 최종 R2 전쪽 검증은 미실행입니다. 최초11쪽 중 아직10·88쪽이 잔존하고 통합 PR 승인 판정은 계속 보류합니다.


## 보정326 사전 분석 — 선언 첫 표 프레임에 포함된 마지막 줄간격

- 163쪽 표42는 단일 셀의 저장 문단을 다음 쪽에서0HU로 다시 시작합니다. 첫 조각의 세 줄은0·2000·4000HU이고 각 줄은 높이1000HU·간격1000HU입니다. 마지막 간격까지6000HU와 위아래 padding282HU의 합6282HU는 선언 첫 프레임6277HU와 반올림 오차 안에서 일치합니다. 독립 PDF는 이83.7px 프레임을 사용하지만 기존 helper가 reset이라는 이유로 마지막 간격1000HU를 제거해70.4px로 줄입니다. 가운데 정렬 본문과 아래 괘선도 함께 위로 올라갑니다.
- 공통 `native_saved_reset_cut_trailing_trim`에서 선언 첫 프레임이 온전한 prefix와 padding을 정확히 포함하는 경우 간격을 제거하지 않습니다. 컷 선택·측정·실제 배치가 이 helper의 같은 높이를 사용하므로 paint에서만 표를 늘리지 않습니다. 선언이 가시 끝만 포함하는 기존 #7406 및 앞 보정321은 유지합니다.163·164쪽과17·24·107·169·207·208·211쪽을 비교하며 새 회귀 함수·fixture·기준값은 추가하지 않습니다.


## 보정327 사전 분석 — 실루엣의 색상 경계 이진화 오류

- 사용자 한컴·rhwp·PDF 직접 판독 요청에 따라10쪽을 재집계했습니다. 기존232 경계 실루엣67.47059%의 불일치56190픽셀 중55672픽셀은 지도 영역이고, 지도 밖은99.19832%입니다. 주된 동일 위치 RGB 쌍은 원본(224,235,255)와 PDF(233,241,254)/(234,242,255)입니다. 원본 밝기0·명암0·REAL_PIC와 추가 효과 없음은 HWP5 형식 문서 표32·107·108과 일치하며 원본 색상을 PDF 압축색으로 바꾸지 않습니다.
- 오류는232를 넘은 PDF의 유색 픽셀을 빈 배경으로 분류하는 이진화입니다. 기존 내용 마스크·2px 반경·90% gate를 유지하며, 이진화 불일치 픽셀 중 **동일 위치 양쪽 모두 기존 흰 배경 조건(모든 채널244 이상)에 해당하지 않고 RGB 최대 차이가 기존 엄격 비교 기본값32 이내**인 경우만 양쪽에 가시 내용이 있는 것으로 대조합니다. 실제 흰 배경의 그림 누락·반경 밖 이동·큰 색상 변경은 계속 실패해야 합니다. 원래 실루엣 값과 조정된 픽셀 수를 TSV/overlay에 함께 남겨 점수 변화의 근거를 노출합니다. 임계값 선택·마스킹·문서 ID 예외가 아니며 기존 PNG 재사용은 새 head의 renderer 재출력 증명이 아닙니다.
- 먼저 Python 도구 반례와10쪽 PNG를 검증하고 전체215쪽 저장 PNG의 영향도 재산출합니다. 이 단계는 pending163쪽 source 검증이나 최종 Native/fresh WASM 전체 회귀를 대체하지 않습니다.

- 보정327 결과: `visual_sweep.py`의 색상 경계 대조 후10쪽은 **99.69607%**, 이진화 원값은67.47059%이고55665픽셀의 양쪽 가시 내용을 확인했습니다. 엄격 내용 픽셀75.61250%와 전체 픽셀95.27556%는 그대로입니다. 기존 Python 검사와 누락·이동·큰 색상 변경 반례를 포함해81/81 PASS, `git diff --check` PASS입니다. 도구의 계산 방법·원값·조정 수를 TSV/manifest/review에 병기하고 가이드·거버넌스·CONTRIBUTING·CLAUDE에 전파했습니다.
- **기존 stage325 Native PNG 재산출**215쪽에서29쪽의 보조값만 변했고 미달은88쪽89.99772%,163쪽89.39386%입니다. 후자는 보정326의 새 Native에서90.44522%였으나 아직 해당 correction의 fresh WASM/result 커밋을 완료하지 않았습니다. 이번 단계는 renderer·원문·PDF 변경, 신규 Rust 회귀 추가, 최종 head 전체 검증을 포함하지 않습니다. [수정된10쪽 review](../assets/pr7382_20260926/stage327_native_010_review.png), [기존215쪽 TSV](../assets/pr7382_20260926/stage327_existing_native_all_silhouette.tsv), [검증 원장](../assets/pr7382_20260926/stage327_silhouette_boundary_validation.json). 10쪽의 직접 최종 판독과88쪽 실제 조판 보정·최종 전수 검증은 계속 필요합니다.
- 문서 검사: 변경한 가이드2문서의 메타데이터는PASS입니다. 저장소 전체 검사는 변경 외4문서의 필수 메타데이터 누락16건으로exit1이므로 전체 성공으로 보고하지 않습니다.

- 보정326 결과: 선언 첫 물리 프레임을 보존한 Native/fresh WASM163쪽90.44522%,164쪽96.39290%, 대조9쪽 PNG 동일·전체215쪽 유지, 기존 회귀14/14 PASS, 최종 Native·fresh WASM 빌드 및 선택 비교exit0입니다. 최초 치환 실패 후 실제 후보를 적용한 재빌드를 검증했습니다. 직접 비교에서 표 안의 가운데 정렬은 PDF보다약6.7px 위에 남았으므로 물리 프레임 보정만 커밋하고 **정렬 잔존을 별도 보정328로 처리**합니다. [검증](../assets/pr7382_20260926/stage326_first_fragment_frame_validation.json), [163쪽 비교](../assets/pr7382_20260926/stage326_native_163_compare.png). 최종 승인·전체 검증은 완료되지 않았습니다.


## 보정328 사전 분석 — 부분 셀의 마지막 가시 줄과 세로 정렬

- 보정326은163쪽 표42 첫 물리 프레임을83.7px로 복원했으나 가운데 정렬 본문은PDF보다약6.7px 위입니다. `table_partial`의 `line_ranges → total_content_height → centered_content_height → text_y_start` 경로가 마지막 줄간격 제외 여부를 **전체 원본 셀의 마지막 문단/줄**로 판정합니다. 현재 조각은 다음 문단을164쪽으로 이월하므로 마지막 가시 줄의 간격13.3px를 내용 높이에 포함하고 가운데 여백을6.7px 줄입니다.
- 선택된 줄 소유 범위의 마지막 가시 줄을 정렬 내용의 끝으로 사용합니다. 물리 조각·컷·페이지 흐름은 보정326의 온전한 높이를 유지하며 정렬에서만 그려지지 않는 마지막 줄간격·후행 문단 간격을 제외합니다. 일반 완전 셀은 마지막 가시 줄과 원본 마지막 줄이 같아 무동작이며 위 정렬은 원점이 바뀌지 않습니다. 중첩 개체의 가시 점유는 기존 같은 컷 기반 height를 유지합니다. 끝조각 glyph-em 정렬 계산도 같은 가시 줄 경계를 소비하게 합니다.
-163·164쪽과17·24·107·169·207·208·211쪽 및 기존 부분 셀 정렬 회귀를 검증합니다. 신규 회귀 함수·fixture·픽셀 기대값은 추가하지 않습니다.

- 보정328 1차 후보: 마지막 가시 줄 경계 수정만으로는163쪽90.44522%가 그대로였고 기존 회귀14개는PASS입니다. 단일 셀은 유한한 첫 프레임을 증명해도 `align_saved_opening_frame`의 다행 전용·override 조건에서 빠져 windowed composition/Top 강제로 갑니다. 보정326과 같은 온전한 prefix+padding 선언 프레임 증거를 공통 helper로 공유하여 이 첫 조각의 원래 정렬을 적용하고, 그 경로에서 가시 내용 높이를 계산합니다. 후행 호스트 간격이나 다른 단일 셀 프레임 판정은 변경하지 않습니다.

- 보정328 결과: 공통 온전한 prefix 프레임 증거를 물리 간격 소비와 첫 조각 정렬에 함께 사용하고 마지막 가시 줄을 정렬 끝으로 삼아163쪽Native/fresh WASM **100.00000%**입니다. 대조9쪽PNG 동일·전체215쪽 유지, 기존 부분 셀 정렬/쪽 분할 등23개 회귀PASS, 최종 빌드exit0입니다. 최초 가시 끝 후보는Top 경로 때문에 무동작이었고 수정 후 직접 비교에서표42 본문·외곽이PDF와 맞습니다.164쪽 내용 소유와종전96.39290%는 유지되며 아래 괘선 차이는 잔존합니다. 잘못된 옛 test target을 current suite manifest로 바로잡은 최종23개만 성공 증거로 사용합니다. [검증 원장](../assets/pr7382_20260926/stage328_visible_alignment_validation.json), [163쪽 비교](../assets/pr7382_20260926/stage328_native_163_compare.png), [Native](../assets/pr7382_20260926/stage328_native_silhouette.tsv), [WASM](../assets/pr7382_20260926/stage328_wasm_silhouette.tsv). 새Rust 회귀는 추가하지 않았고88쪽과최종전체검증은 남았습니다.


## 보정329 사전 분석 — 쪽을 이어받는 글머리 문단의 본문 들여쓰기

-88쪽 첫 줄은87쪽 문단940의 마지막 줄입니다. 원본 head는BULLET(idRef2), 글머리 정의는문자`-`·autoIndent1·본문거리50%이며PDF의 본문 시작은107.8px입니다. Native는94.5px입니다. 같은 쪽의 새 글머리 문단은마커 폭13.0px를소비하여107.5px에서 본문을 시작합니다.
- `layout.rs PartialParagraph`는 첫 조각에만`apply_paragraph_numbering`을 적용하고 이어지는 조각에는`numbering_text`를 전달하지 않습니다. `layout_partial_paragraph`도start_line0일 때만 그 폭을계산합니다. 마커를 다시 그리지 않는 것과 본문의 같은 들여쓰기를 유지하는 것을 혼동했습니다.
- 글머리는 번호 카운터를 전진시키지 않으므로 같은 정의의 마커 폭을 이어지는 조각에도 전달합니다. 실제 마커 출력은 여전히start_line0에서만 수행하고 문자열/문서문자offset·문단 순서·페이지는 바꾸지 않습니다. 번호/개요 카운터 경로는 그대로 유지합니다. 모든 조각의 같은 본문 폭·배치가 동일한 마커 폭을 소비하도록 수정하며, 수식 폭 차이는 별도 원인으로 남깁니다. 기존 head/쪽 분할 회귀와87·88·89쪽/정상대조를검증하고 새 회귀 함수는추가하지 않습니다.

- 보정330 사전 분석:88쪽 수식은 저장 폭15268HU(203.57px)에 자연 수식 폭을 가로 압축하며, 독립 한컴 PDF는 한글 분자35자의 기본 글자 크기를 유지합니다. 저장 줄 중 수식 이전8행은PDF와 맞으므로 전체 문단을 재조판했던 보정323 후보(86.57%)를 반복하지 않습니다. 대체 수식 메트릭이 달라지는 최초 객체 소유 행부터 뒤쪽만 재구성하고, 앞행은 저장 프레임 수용을 확인한 뒤 보존하는 후보를 검증합니다. 측정·paint는 같은ComposedParagraph 줄과 수식 폭을 소비해야 합니다. 정상 영문 수식·사용자가 조절한 저장 상자의 적용 범위를 별도 확인하며,88쪽 수식·뒤 본문·각주139와87/89쪽을 비교합니다. 현재90.66%라도 수식 폭 차이가 남아 완료가 아닙니다.

- 보정329 결과: 이어받은 글머리표 문단의 첫 줄은 이미 소비한 표식의 폭을 들여쓰기에 반영하고 표식은 다시 그리지 않습니다. 번호 counter 경로는 유지했습니다.88쪽Native/fresh WASM **90.66487%**,87/88/89쪽PNG 동일, 기존 번호·표예약20개PASS, Native/fresh WASM 빌드·비교exit0입니다. 수식 폭과 뒤 본문 줄바꿈은 남으므로 완료 판정하지 않습니다. [검증 원장](../assets/pr7382_20260926/stage329_bullet_continuation_validation.json), [88쪽 review](../assets/pr7382_20260926/stage329_native_088_review.png). 새Rust 회귀는 추가하지 않았습니다.

- 보정330 결과: 실제 HYHWPEQ.TTF의 한글 cmap이 비어 있고 PDF는 HyhwpEQ와 Haansoft Batang을 사용하는 사실을 확인했습니다. 보호되지 않은 한글 수식의 폭·높이·기준선을 같은 AST에서 계산하고, 수식 소유 행부터만 다시 조판합니다. 앞쪽 저장 줄과 문자 축은 유지합니다.
  - 최초 후보는 첫 채움 행을 문자0으로 초기화하여 본문이 중복됐으므로 기각했습니다. 이어받기 경계를 보존한 폭 보정 후보(91.48576%)의 세로 밀림도 공통 높이·기준선으로 보완했습니다.
  - 최종 88쪽 Native SVG/fresh WASM **95.34594%**(보정329: 90.66487%). 87쪽 99.83337%, 89쪽 98.72103%, 3쪽 PNG 동일, 전체 215/215쪽 유지입니다. 87·88·89쪽 문단별 정규화 본문이 동일하고 87·89쪽 Native PNG도 전후 동일합니다. 각주139는 유지됐습니다.
  - Native Skia feature를 포함한 기존 수식·표 예약 검사 **27개 PASS / 0 FAIL**. 빌드, fresh WASM과 최종 비교 exit0이며 새 Rust 회귀는 추가하지 않았습니다. Canvas/Skia도 한글을 명조 대체 글꼴로 그리고 SVG는 실제 face를 임베딩합니다. Native Skia의 직접 PDF 이미지 비교는 미실행이며 feature 검사를 시각 증거로 보고하지 않습니다.
  - 수식 AST의 추정 글자 폭과 한컴 실제 글리프 폭, 뒤 본문의 일부 줄바꿈 차이는 남습니다. 최신 head 전체 비교·최종 전체 회귀·lint도 남아 PR 준비 완료가 아닙니다.
  - [검증 원장](../assets/pr7382_20260926/stage330_equation_flow_validation.json), [글꼴 출처](../assets/pr7382_20260926/stage330_equation_font_evidence.json), [88쪽 review](../assets/pr7382_20260926/stage330_native_088_review.png), [overlay](../assets/pr7382_20260926/stage330_native_088_overlay.png), [Native TSV](../assets/pr7382_20260926/stage330_native_silhouette.tsv), [WASM TSV](../assets/pr7382_20260926/stage330_wasm_silhouette.tsv). 변경 매뉴얼의 metadata 오류는 0건이며 전체 검사에는 변경하지 않은 4문서의 기존 16오류가 남습니다. 로그·글꼴 binary는 output에만 보존합니다.

- 보정331 사전 분석: 보정330 뒤88쪽 수식 자연 폭393.1px와 뒤 문장의 줄 끝 차이가 남습니다. 동일 한컴2024에서 별도1쪽 대조 HWPX를 출력하여 baseUnit600/800/1000/1200/1600의 한글 글자 전진을 측정했습니다. 약8/10/14/16/22px로, 단순 전각(fs=8/10.67/13.33/16/21.33px)과 다르며48dpi 단위 반올림에 해당합니다. Mac/Windows HBATANG의 hmtx는 모두1024/1024이므로 글꼴 파일의 한글 폭 차이가 아닙니다. 진단 입력은 원본의 수식 문단·header를 복제해 저장 줄을 제거하고 script/baseUnit만 바꾼 수동 대조군이며, 원본 PDF를 대체하지 않습니다.
  - HYhwpEQ 한글 대체 경로에서 이 전진을 계산하고 같은 LayoutBox 폭으로 SVG/Canvas/Skia 글자 원점을 배치합니다. 보호 상자 계약과 일반 영문 수식·OLE 경로는 유지하며 크기나 특정 문서의 맞춤 계수를 사용하지 않습니다. 수식 분자·분모·뒤 문장·각주139와87/89쪽, 기존 수식 검사를 확인합니다. 아직 결과는 미검증입니다.

- 보정331 결과: HYhwpEQ 대체 한글의 크기별 논리 전진폭과, 부분 재조판에서 누락했던 글머리표 본문 폭을 함께 적용했습니다. 수식 뒤 줄 끝은“안전”, 다음 줄 시작은“기준 지침”으로 독립 PDF와 같아졌습니다. Native/fresh WASM88쪽96.43691%,87쪽99.83337%,89쪽98.72103%; 양쪽3개 raster hash 동일,215/215쪽 유지입니다. 기존 수식·번호·글머리표·표예약36개PASS, Native/fresh WASM 빌드·비교exit0입니다. 새Rust 회귀 함수는 추가하지 않았습니다. [한컴 크기 대조](../assets/pr7382_20260926/stage331_font_advance_evidence.json), [검증 원장](../assets/pr7382_20260926/stage331_validation.json), [88쪽 review](../assets/pr7382_20260926/stage331_native_088_review.png).
  - 직접 판독과VPOS trace에서 다음 문단pi947의 순차 높이975.84px가 저장 사다리981.03px로5.19px 되돌아가는 별도 잔차를 확인했습니다. 수식 새 높이26.01px와 저장 높이31.2px 차이와 같습니다. 이 잔차는 보정332로 이어서 해결하며88쪽 완료·PR 제출 가능으로 판정하지 않습니다.

- 보정332 사전 분석: 원본88쪽의 마지막 문단pi947은 앞문단 저장 마지막줄의끝+줄간격과 정확히 연결됩니다(67340HU). 보정331 VPOS trace에서 typeset는892.68→897.87px, paint는975.84→981.03px로 같은5.19px를 옛 저장 높이에 되돌립니다. 실제 수식 재조판으로31.2→26.01px가 된 줄 높이와 일치합니다. 재조판의 실제 순차 높이가 정본인데 저장 절대 vpos가 이를 덮어씁니다.
  - 수식 대체 메트릭의 부분 재조판 소유 경로와, 같은 쪽의 연속 저장 경계가 확인되는 경우에는 페이지네이터·renderer 공유HeightCursor가 실제 순차 높이로 사다리 기준점을 갱신합니다. 저장 단/쪽 리셋과gap은 연속성이 없으므로 이 경로에서 재앵커하지 않습니다. unsupported float·보호 수식·편집 경로는 유지합니다.88쪽 뒤 문단과89쪽 소유·전체 쪽수를 직접 비교하고 기존 커서·수식·표예약 검사를 실행합니다.

- 보정332 결과: 수식 부분 재조판 뒤 연속 저장 경계에서는 실제 순차 높이로 저장 사다리를 재앵커합니다. typeset/paint의 공유HeightCursor가 같은 규칙을 소비하며 옛 수식 높이에 되돌리지 않습니다.88쪽 마지막 문단은981.03→975.84px로 이동하고 PDF와의5.19px 과대 간격을 해소했습니다. Native/fresh WASM88쪽 **98.62448%**(보정33095.34594%, 보정33196.43691%),87쪽99.83337%,89쪽98.72103%;3쪽 PNG hash 동일,215/215쪽입니다.
  - 직접 compare에서 수식 분자·분모, 뒤 본문의“안전”→“기준 지침” 줄 경계, 마지막 문단과각주139를 확인했습니다.87/89쪽 raster는 보정330과동일하고87/88/89 문단별 정규화본문도 같습니다. 수식 기호·글꼴 raster의 미세 잔차는 있으나 수식 뒤 줄바꿈과5.19px 후속 문단 이동은 해결했습니다. 완전 픽셀 동일을 주장하지 않습니다.
  - Native Skia feature를 포함한 기존 커서·번호 lib 및6개 suite의 관련 검사 **124/124 PASS**, Native/fresh WASM 빌드·비교exit0, fmt/diff check통과, 변경 매뉴얼 metadata오류0건입니다. 새Rust 회귀 함수와픽셀 기대값은추가하지 않았습니다. [최신88쪽 review](../assets/pr7382_20260926/stage332_native_088_review.png), [overlay](../assets/pr7382_20260926/stage332_native_088_overlay.png), [검증 원장](../assets/pr7382_20260926/stage332_validation.json), [Native TSV](../assets/pr7382_20260926/stage332_native_silhouette.tsv), [WASM TSV](../assets/pr7382_20260926/stage332_wasm_silhouette.tsv). 최신 head 전체215쪽 시각 검증·전체 회귀/lint·PR 제출 게이트는 별도 남아 있습니다.


## 보정333 최종 검증 시작 / 보정334 사전 분석 — 휴먼명조 TrueType 기대값

- 검증 후보는 `98f2e879c3a0002b119bdcc1ffca29524f827afd`, 최신 base는 `02530b9ed567a44663edb26c65fb565c4a79f00d`입니다. fetch 뒤 뒤처짐0·앞섬454이며 원 PR head `81a402179dc556cce781d844d4b9252be36ba8af`는 같습니다. collaborator_external_pr 통합 경로와 intake/local_validation/visual_fixture_evidence/multi_pr_update_branch/post_merge 절차를 적용합니다.
- 전체 nextest는 threads8·no-fail-fast로 실행 중입니다. 보안 검사에는 base 대비 변경된4개 실제 sample 문서를 전달했습니다. Native/fresh WASM은 같은 입력·기준215쪽·인쇄 프로필·보정325 명시적 TrueType 환경으로 전쪽 TSV를 생성합니다. 전체 완료 전 승인으로 바꾸지 않습니다.
- 첫 실패는 기존 `test_b7_advance_follows_the_font_table_only_when_trusted`입니다. 보정325는 확인된 휴먼명조 TrueType 선택에 U+00B7 전각 전진을 적용했지만 이 검사는 신뢰true에도 옛HFT 호환0.3em을 요구합니다. 실제512/512 글꼴 hmtx와 독립 PDF208쪽·보정325 증거는1.0em입니다. 비신뢰false는 여전히0.3em이고 HY신명조/한양신명조 대조도 유지해야 합니다.
- 생산 코드와 PDF를 바꾸지 않고 기존 함수의 TrueType 기대와 설명만 교정할 예정입니다. 이 검사는 합성 글꼴 계약이며 전체 문서의 위치·색상 검사를 대체하지 않습니다. 전체 원본215쪽 검증과208쪽 증거를 연결하고, 수정 전FAIL/수정 후 개별PASS·전수 재실행을 기록합니다. 새 Rust 검사 함수는 추가하지 않습니다.
- 다른 실패인 #1189는 수식 그룹의 문자열에 명시적 `scale(...,1.0000)`이 있어야 한다고 요구합니다. 현재 출력은 `translate`만 있어 세로 확대가 없습니다. 해당10–12쪽을 독립 PDF와 Native/fresh WASM으로 직접 비교해 실제 세로 변형 여부를 먼저 판정하고 별도 단계로 처리합니다.

- 보정333 최초 전수 결과는10,227 PASS/2 FAIL/50 SKIP,722.750초,exit100입니다. 실패2건만 처리하며 정상 검사를 일괄 이관하지 않습니다. Native 원본215쪽 전수는최저22쪽90.01587%,90%미만0쪽·88쪽98.62448%,exit0입니다. fresh WASM과 최종 Rust 게이트는 진행 중입니다.
- 보정334 결과: 기존 함수에서 신뢰된 휴먼명조 기대만0.3→1.0em으로 교정하고 비신뢰0.3·다른 두 face 대조는 유지했습니다. 생산 코드는 바꾸지 않았습니다. 수정 전 해당FAIL,수정 후 개별1/1 PASS,exit0이며 새함수는 없습니다. [검증 원장](../assets/pr7382_20260926/stage334_font_test_validation.json). 이어서 다른 실패를 별도로 교정한 뒤 전수를 다시 실행합니다.


## 보정335 사전 분석 — 수식의 생략된 단위 배율 검사

- 두 번째 실패는 #1189의 기존10–12쪽 검사에서12쪽 한글 수식 그룹에 문자열 `,1.0000)`을 요구하는 부분입니다. 현재그룹은translate만 적용합니다. SVG에서scale 생략은단위 배율이므로 이것을세로 확대라고 판정하면 형식 검사 오류입니다. 원래 목적은저장 bbox높이로수식 Y축을늘리지 않는 것입니다.
- 원본은 `samples/3-11월_실전_통합_2022.hwp`,21쪽·한컴2022 저장본입니다. 기존 독립 `pdf/3-11월_실전_통합_2022.pdf`12쪽에 대한현재 Native/fresh WASM은각92.02527%,PNG동일이며 수식 기호·분자/분모와후행 본문을직접판독했습니다. 먼저 사용한2020재출력PDF는같은쪽47.46626%이며기존 PDF와글꼴/배치 실현이달랐습니다. 기존원문 대응PDF를보존·대조하였고더낮은재출력 결과도감추지않습니다.
- 같은기존PDF의10/11쪽은82.91577/79.17615%입니다. 이부분은전문서시각완료로선언하지않으며,이번실패한12쪽 SVG배율 표현교정과구분합니다. 정상인10/11쪽소유검사와나머지본문·화살표검사는유지합니다. 전체노후회귀를일괄이관하지않습니다.
- 기존 함수 안에서scale이있는경우숫자로Y배율을해석해1인지검사하고,생략된경우단위배율로처리할예정입니다. 단일인자scale도양축을바꾸므로검사합니다. 생산코드·배치·기준PDF와새검사함수는바꾸지않습니다. 수정전FAIL/수정후개별PASS와전수재실행을기록합니다.

- 보정335 결과: 생략된배율을단위배율로수용하고,명시적scale의세로인자를해석해실제확대/축소를검출합니다. 기존함수의10/11쪽문단·12쪽내용/화살표검사를유지했고새함수·픽셀기대값·생산변경은없습니다. 수정전FAIL→수정후개별1/1 PASS,exit0이며원본PDF12쪽Native/fresh WASM각92.02527%·raster동일입니다. [원장](../assets/pr7382_20260926/stage335_equation_test_validation.json), [12쪽review](../assets/pr7382_20260926/stage335_nov2022_012_review.png), [overlay](../assets/pr7382_20260926/stage335_nov2022_012_overlay.png). 이것을21쪽전체시각완료라고보고하지않습니다. 두실패교정후전체회귀·lint를다시실행합니다.


## 보정337 사전 분석 — 최종 Clippy 표현 교정

- 보정336의전체nextest는10,229 PASS/0 FAIL/50 SKIP,650.795초,exit0입니다. 변경Markdown34개링크도0오류입니다. native Clippy는`obfuscated_if_else`와`manual_contains`2건으로exit101이므로아직PR준비완료가아닙니다.
- `table_layout.rs`의lead수용조건·반환값은그대로두고bool→Option→0선택을if/else로표현합니다. `row_step.rs`의정수end_cut에1이있는지검사는contains(&1)로표현합니다. 수치·소유·컷·반환값을바꾸지않는표현교정이며allow로오류를숨기지않습니다. lint를먼저통과시키고관련경계·최종전체회귀를확인합니다. 기존전쪽시각증거는새head의출력과대조해재사용여부를구분하며최신캡처라고허위표시하지않습니다.

- 보정337 표현교정결과: 동일lead수용조건에서참이면lead·거짓이면0을반환하도록if/else로표현하고,end_cut정수1존재검사는contains로표현했습니다. 수치·컷·소유·반환값은변경하지않았습니다. nativeClippy는47.07초·exit0입니다. fmt와다른필수lint/정책검사를이어실행중이며,전수통과나PR준비완료로승격하지않습니다. 새Rust회귀는추가하지않았습니다.


## 보정338 최종 결과와 통합 PR 후속 계획

최종 생산 코드 `cd85bdf43`의 전체 회귀·Native Skia3·문서 테스트·필수 lint/빌드와 정책 검사 exit0을 확인했습니다. 보정333 전수 raster/TSV와 보정338 전수 SVG/tree 동일성 및6쪽 최신 raster를 연결한 근거는 상단 최종 판정을 따릅니다. Visual Sweep 도구81개와 글꼴 규칙 Node22개도 통과했습니다. 모든 로그는 ignored output에 보존하고 증적 JSON·TSV·review/overlay PNG만 커밋합니다.

통합 PR에는 원 기능8e0f0249·원 증적fcba72b1과 각 메인터너 보정을 구분합니다. 원 upstream merge81a40217은 중복 체리픽하지 않았습니다. 최신 검토 그래프의 체리픽은 기능530f2754f·증적e67b964dc이며 접수 당시 SHA와 리베이스 후 SHA를 혼동하지 않습니다.

### 병합 후 실행할 작업

1. 통합 PR 정확한 head의 필수 CI·mergeability·base 포함을 확인해 병합하고 merge SHA와 duration 갱신 결과를 기록합니다.
2. 원 PR #7382에는 원 기여의 진단·표/각주 예약 개선을 인정하고, 추가 쪽 경계·글꼴·수식·기존 검사 교정이 필요한 이유와 통합 증거를 한국어 존댓말로 설명한 뒤 통합 PR 링크와 함께 close합니다. 기여자 fork branch는 보존합니다.
3. #7379의 실제 종료 상태를 확인하고215쪽·다섯 경계 증거를 연결합니다. #7445와 다른 미해결 피델리티 이슈는 종료하지 않습니다.
4. devel 동기화와 이번 작업의 소유 브랜치·output 정리를 수행합니다. 원본 문서·커밋한 증적·공유 target/pr-review는 보존합니다. 병합 전에는 후속 완료로 기록하지 않습니다.


## 통합 PR #7505 코드 후보 CI 완료

- 코드+증적 head `b8da28d30e4bd6cddc0bccac55d22d53f623c5db`의 [Full CI / Build & Test](https://github.com/edwardkim/rhwp/actions/runs/36900285152)가 성공했습니다. 실제 default-feature Archive A/B/C/D, Native Skia, lint와 frontend package gate를 수행했습니다. 원 contributor CI나 과거 head를 재사용한 판정이 아닙니다.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/36900284576), [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/36900285143), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/36900285134), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/36900285288), Skill router와 CI Impact Policy도 성공했습니다.
- 코드 후보 CI 녹색 이후 이 기록과 오늘할일의 통합 링크만 같은 PR의 trailing 문서 commit으로 보완합니다. source·test·baseline·기준 PDF·시각 증거는 변경하지 않습니다. 새 문서 head의 preflight/집계와 정확한 SHA를 별도로 확인합니다. 병합 후 확정 merge SHA·duration 결과와 실제 원 PR/이슈 후속 상태는 GitHub 후속 comment에 기록하며 별도 번호용 검토 문서나 기록 전용 PR을 만들지 않습니다.
