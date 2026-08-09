---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 93 — 전체 회귀의 증분 페이지네이션 실패 분리

## 시작 근거

Stage 92에서 #2020 접수증 `㊞` 위치를 14pt 전용 한양신명조 공백 측정으로 보정했다.
focused #2430은 2/2, #2020은 4/4 통과했다. 그러나 전체
`CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
의 마지막 lib 단계가 exit 101로 끝났고, 아래 두 HWPX incremental contract가 실패했다.

| test | 실제 실패 내용 |
| --- | --- |
| `issue2214_scoped_cache_coherence_preserves_transient_pagination` | HWPX input 56의 `flow signal`이 기대 `false` 대신 `true` |
| `issue2424_resumable_delete_commits_only_after_final_fragment` | 삭제 뒤 fifth line가 사라져야 하나 cut 배열이 삭제 전과 동일 |

## 분리 원칙

두 테스트는 #2020 receipt table이 아닌 deferred/incremental pagination cache를 검사한다.
따라서 test expectation을 완화하거나 Stage 92의 성공을 근거로 무시하지 않는다. 먼저 단독 실행이
재현되는지, 이전 committed baseline에서도 재현되는지, 14pt/10pt space 범위화가 실제 입력의
line-break를 바꾸는지 순서대로 판정한다.

## 재현·원인 분석

두 failure는 각각 `--lib ... --exact --nocapture` 단독 실행에서도 재현됐다. 따라서 suite 순서나
cache warm 순서의 우연이 아니다.

두 test가 공통으로 편집하는 HWPX input은
`issue1949_giant_cell_nested_tables_perf.hwpx`의 `(section=0, parent=0, control=2,
cell=2, para=5)`다. 원본 `Contents/header.xml`의 `charPrIDRef=48`은
`한양신명조`, `height=1200HWPUNIT` 즉 **12pt = 16px**이며, 저장 line start는
`[0, 44, 84, 122]`다.

Stage 92의 임시 “정확히 14pt만 411/1024em” 제한은 이 12pt standard-body 문단을 일반
0.5em 공백으로 바꿨다. 그 결과 HWPX의 한컴 adapter boundary가 기존 61번째 입력에서
56번째로 앞당겨져 #2214 `cellFlowChanged`와 #2424의 5번째 줄 삭제 contract가 동시에
깨졌다. HWP fixture는 별도 stored LINE_SEG route라 기존 56회를 유지해 한쪽만 실패한
관측과도 일치한다.

따라서 원래의 font-name 전역 적용도, 14pt 단일 크기 적용도 모두 너무 넓거나 좁다.
확정된 계약은 다음과 같다.

- 10pt 접수증 날짜 문단: 한컴 PDF대로 일반 반각 0.5em.
- 12pt HWPX standard-body 및 14pt p81 표: 기존 한컴 line-decision대로 411/1024em.

수정은 `한양신명조`의 **12pt 이상**에만 411/1024em을 적용한다. 이는 확인된 font size와
line-boundary 입력을 기준으로 한 minimum threshold이며, receipt 전용 좌표 shift나 test baseline
변경이 아니다.

## 계획

1. 두 test를 각각 release-test lib filter로 재실행해 order-dependent failure인지 확인한다. — 완료
2. 실패 input의 글꼴/크기와 수정 전후 page cut을 확인한다. — 완료
3. `한양신명조` 411/1024em 조건을 12pt 이상으로 조정하고, unit contract에 12pt를 추가한다.
4. 수정 뒤 #2430을 먼저, 이어 focused #2214/#2424와 #2020을 통과시킨 뒤 전체 release-test를
   다시 실행한다.

## 수정 뒤 focused 결과

`한양신명조` 411/1024em의 minimum을 12pt로 조정하고 다음 순서로 검증했다.

| gate | 결과 |
| --- | --- |
| `issue_2430_cell_rewrap_threshold` | `2 passed; 0 failed` |
| `text_measurement::tests::issue_3820_hanyang_shinmyeongjo_space_is_standard_body_only` | `1 passed; 0 failed` |
| `issue2214_scoped_cache_coherence_preserves_transient_pagination` | `1 passed; 0 failed` — HWPX 61번째 boundary와 115 fragments 복원 |
| `issue2424_resumable_delete_commits_only_after_final_fragment` | `1 passed; 0 failed` |
| `issue_2020` | `4 passed; 0 failed` — 10pt receipt PDF contract 유지 |

즉 cache 구현을 바꾸지 않고 metric 적용 범위만 바로잡아, 두 incremental failure와 receipt
regression을 함께 해소했다. 다음은 전체 release-test 재실행이다.

## 전체 회귀 결과와 이월

전체 재실행은 #2214, #2424 및 #2020을 통과했지만
`tests/issue_2185_korean_break_unit.rs`에서 exit `101`로 중단됐다.

```text
HWPX 편집 직후: 앞선 LINE_SEG 경계를 보존해야 함
expected: [0, 44, 84, 122]
actual:   [0, 45, 87, 125]
```

이는 #2214의 ASCII 입력 boundary와 다른, **한글 한 글자 편집 뒤 저장된 HWPX LineSeg를
불필요하게 재래핑하지 않아야 하는** 보존 계약이다. #2214/#2424의 12pt metric 회복을
되돌리지 않고 Stage 94에서 reflow 필요성 판정으로 분리한다.

## 완료 기준

- 두 incremental HWPX contract가 단독과 전체 suite 모두에서 통과한다.
- #2020의 PDF 기반 도장 위치와 #2430 cell rewrap 계약이 유지된다.
- 전체 release-test의 최종 summary가 `0 failed`다.
