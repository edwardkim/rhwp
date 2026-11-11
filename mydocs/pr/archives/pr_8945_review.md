# PR #8945 검토

## 최종 판정

**승인** — 2026-11-11 작업지시자가 “PR 승인, 머지, 후속 처리까지 진행하세요”라고 지시했다.
직전 승인 요청에 명시한 과거 보호값 미재현 911행의 **이번 PR 한정 수용**과 원격 처리 범위를
승인한 결정으로 기록한다. 현재 base/후보 PNG 3,542쌍은 모두 같고 새 회귀는 없다.
보호 TSV·90% 기준·branch protection을 변경하지 않는다. 새 회귀·결함·미검증은 예외에 포함하지 않는다.
목표 8쪽 Native/fresh WASM 각각 100.00%, 최소 글꼴 집중 14건, 필수 편람 gate와 원 head CI를 확인했다.

원 head·devel은 검증 당시와 같고 38개 check 및 CI Impact Policy가 모두 정상이다.
fresh merge-tree exit 0을 재확인했다. 원 기여 source/test/fixture 보정 없이 원 PR 승인·merge와
운영 기록·댓글·동기화·종료 정리를 진행한다. #7620은 부분 해결 이슈로 OPEN을 유지한다.

## 대상과 기존 해결 이력

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8945](https://github.com/edwardkim/rhwp/pull/8945), sacru2red |
| 원 head | `06f47c8a518f542a80d38e5a398ed73fed7dcc64` |
| 원 head의 base | `e118de8abadd2dffb2798036b9ffd381b96ed7d5` |
| 현재 검증 기준 devel | `99b9d7ec5ca36884ea8fd981ac815c1edc34c653` |
| 실제 검증한 로컬 병합 source | `e98d884fc35295c8b612ceb0ac86437fb8108fee` |
| 검토 branch | `review/pr-8945-rowbreak-margin-budget` |
| 재현·producer 연결 | [원장](../assets/pr8945/source-association.json) |

#7620은 #7644를 #7681의 merge `4c7092397e6fb6d1d16e5521bb9049ef25814d5a`로
수용한 뒤에도 바닥 예산·footer·CellBreak 범위가 남은 이슈다.
[당시 후속 설명](https://github.com/edwardkim/rhwp/issues/7620#issuecomment-6059366404)과
[기존 검토](pr_7644_review.md)를 먼저 확인했다. #8149의 빈 host 첫 조각 예약과
#7147의 선언 높이/첫 조각 계약도 구현과 기존 검사에서 추적했다.

이번 PR은 **바닥 예산의 부분 개선**이다. #7620을 닫지 않고 `Refs` 관계를 유지한다.
e118 이후 현재 devel의 #8913 선택 UI·#8915 Chrome DOM 변경은 이 표의 측정·분할·paint를
수정하지 않는다. 실제 최신 devel과 후보를 같은 입력·PDF·글꼴로 비교했다.

## 조판 원칙과 실제 소비 경로

위반된 규칙은 첫 조각에 실을 행이 바깥 아래 여백과 함께 들어가는지를 측정과 배치가
다르게 판단한 것이다. 생성 대조군 b에서 최신 devel은 두 행 표를 나누고 뒤 본문도
2쪽에 배치하지만, 같은 입력의 독립 한컴독스 Print는 표 전체만 2쪽으로 옮기고 뒤
본문을 1쪽에 남긴다. 문서 ID·목표 쪽수·좌표 상수로 보정하지 않는다.

| 경로 | 실제 연결과 확인 |
| --- | --- |
| 공통 예약 생산 | `typeset.rs::empty_host_float_first_fragment_reserve_px`가 빈 host·문단 기준 자리 차지·아래 여백/캡션 계약을 생산한다. |
| 첫 조각 수용 | `table/block/prepare.rs:753`의 `first_fragment_bottom_reserve`에 조각 아래 여백과 빈 host 예약을 함께 담는다. 저장 원점을 소비하는 `remaining_with`도 같은 예약을 뺀다. 선언 높이가 첫 조각만 덮는 경우와 저장 첫 프레임 예외는 유지한다. |
| 행 스캐너 | `table/continuation/fragment/budget.rs:540`의 빈 host 첫 조각 예약·footer guard·각주 분기를 대조했다. 수용한 행/내용 컷을 기존 분할 경로가 소비한다. |
| 첫 유닛도 들어가지 않을 때 | `table/block/deferred_float.rs:39`에서 측정된 첫 줄/패딩과 위·아래 여백으로 판정한다. 전체 표가 새 쪽에 들어가는 경우에 기존 deferred frame 경로를 사용한다. host 글줄은 현재 쪽에서 점유하고 표만 이월한다. |
| 새 쪽 요구 높이·배치 | `state/commands.rs::defer_stored_frame` → `state/transition.rs:1044`의 `occupied_bottom` 예약·`paragraph_float_placements` → 같은 파일 `:695`의 `PageItem::Table` 출력으로 이어진다. 뒤 본문도 이 점유 끝을 소비한다. |
| 적용 경계 | 단일 단·문단 기준 TopAndBottom·Top 정렬·오프셋 0·적합한 host에 한정된다. 새 deferred 분기는 캡션·표 각주를 받지 않으며 기존 다단/캡션/각주 경로를 확장했다고 주장하지 않는다. None 대조군과 실제로 한 행 이상 들어가는 분할을 함께 검증했다. |

새 판정은 이미 측정한 셀 목록을 두 번 순회한다. 재측정·글꼴 탐색·I/O를 추가하지 않는다.
독립 성능 벤치마크를 추가 실행한 결과로 과장하지 않는다.

## 입력·독립 기준·전쪽 시각 검증

b·c·d는 `samples/issue7620/generate-budget.mjs`가 Document API로 생성한 HWP다.
한컴 저장본으로 표시하지 않는다. 기준 PDF는 같은 입력의 한컴독스 서버 Print이며,
새 자료의 manifest는 build `20261102065324`, 2026-11-10을 기록한다. 과거 a/None의
Print provenance와 새 자료를 구분했다. 원본·PDF는 원 PR commit에 포함돼 있다.

96 DPI·print·compat 2022·pypdfium2 5.13.0·색상 경계 보정 전 2px 실루엣으로 측정했다.
Native는 실제 Skia PNG, fresh WASM은 release 빌드와 실제 `wasm-opt -O`를 완료한 패키지의
Chromium 152 / revision 1660786 Canvas 출력이다. 각 producer의 source·도구·SHA는 원장에 있다.

| 문서 | 쪽 | devel Native 최저 | devel WASM 최저 | 후보 Native / WASM 최저 |
| --- | ---: | ---: | ---: | ---: |
| b: 첫 행+아래 여백 부족 | 2 | 59.20% | 58.31% | 100.00% / 100.00% |
| c: 표와 뒤 본문 이월 | 2 | 100.00% | 100.00% | 100.00% / 100.00% |
| d: 40행의 행 경계 분할 | 2 | 100.00% | 100.00% | 100.00% / 100.00% |
| a: 기존 RowBreak | 1 | 100.00% | 100.00% | 100.00% / 100.00% |
| None 대조군(issue7621) | 1 | 100.00% | 100.00% | 100.00% / 100.00% |

[전쪽 TSV·요약](../assets/pr8945/focus/summary.json),
[Native 직접 비교](../assets/pr8945/focus/native-review/),
[fresh WASM 직접 비교](../assets/pr8945/focus/wasm-review/)를 보존했다.
100.00%는 위 실루엣 지표의 표시값이며 전체 RGB 픽셀이 PDF와 같다는 뜻이 아니다.
독립 Print와 같은 영역을 확대해 b의 뒤 본문/표 소속, c의 다음 쪽 점유와 뒤 본문,
d의 1–21행/22–40행 분할과 종료 뒤 문단을 확인했다. 누락·중복·표 밖 본문 겹침이나
불필요한 빈 쪽을 발견하지 않았다. a·None 정상 대조도 유지됐다.

## 실행 검증과 CI 재사용

공개 최소 글꼴 5종만 노출한 환경에서 실행했다. Windows·사설 글꼴 경로를 bind하지 않고,
시스템 글꼴과 Native 웹폰트 cache를 가렸다. 실제 `fc-list` 결과와 명령을 보존했다.
해당 HwpDocument 검사 경로는 CLI Visual Sweep의 글꼴 cache를 자동 탐색하지 않는다.

| 집중 검사 | 실제 결과 |
| --- | ---: |
| 새 첫 조각/이월/40행 내용 보존 | 3 PASS |
| 기존 #7620 외부 여백 | 1 PASS |
| #8149 빈 host·caption·선언 높이 경계 | 9 PASS |
| #8382 HWP5-origin HWPX split budget | 1 PASS |
| 합계 | **14 PASS / 0 FAIL** |

[실행 결과](../assets/pr8945/focused-results.json),
[실제 노출 글꼴](../assets/pr8945/exposed-minimal-fonts.txt).
현재 nextest 0.9.137은 권고 0.9.140과 달라 JUnit skip 보고 설정에 경고를 냈지만,
집중 선택과 위 실행 개수·성공 결과는 실제로 확인했다.

필수 `--handbook-gate`도 별도로 실행해 **HWPX Native 383쪽·정본 대비 하락 0쪽**으로
통과했다. [gate 원본](../assets/pr8945/handbook-gate/required-handbook-8153-gate.json),
[전쪽 TSV](../assets/pr8945/handbook-gate/silhouette.tsv).

[원 head CI](https://github.com/edwardkim/rhwp/actions/runs/42991534935),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42991534852),
CodeQL·Adapter·Proptest·정책 감사가 성공했다. 현재 base 병합은 충돌이 없고 source/test/fixture
보정을 하지 않았으므로 정본 3.2.2에 따라 전체 Rust/Native Skia 회귀는 성공한 exact-head CI를
재사용했다. 로컬에서 전체 회귀를 새로 실행했다고 기록하지 않는다.
`svg_snapshot`은 그 release-test 전체에 포함된다. 실패한 golden을 좁히거나 재생성해야 할
변경이 없어 별도 재생성·golden 갱신은 하지 않았다.

## 정본 보호 검증

기준은 `visual-protected-baseline/final.tsv`, SHA-256
`ee41455cf7f05585487d92961ccb9fd8160d5d1e15c3567019ea89e2b7bf2aeb`의 18개 문서·36개
Native/WASM dataset·3,542행이다. 기본 90%뿐 아니라 **공식 두 자리 표시값 0.00%p 하락**
기준도 별도로 적용한다. 일반 contributor에게 사설 글꼴 확보나 이 특별 기준 재현을 요구하지 않는다.

로컬 HFT 7종은 기존 수용 자료의 프로그램 ID와 모두 일치했다. 이 사실이 다른 TTF·fallback의
완전한 일치를 뜻하지는 않는다. 편람은 고정 KoPub/HCR 공개 글꼴을 공급했다. 개요 번호와 법령
문서는 입력의 글꼴 종류·독립 PDF에 기록된 Batang/Dotum과 실제 프로그램을 대조해 선택을
명시했다. 보호 TSV·입력·PDF·측정 허용치를 바꾸지 않았다. 나머지 과거 정본 차이의 원인을
모두 글꼴이나 OS로 확정하지 않는다.

| 검증 | Native | fresh WASM |
| --- | ---: | ---: |
| 기준/후보 전쪽 | 각각 1,771쪽 | 각각 1,771쪽 |
| 현재 후보 raw 최저 | 97.29% | 94.65% |
| 현재 base 대비 raw 하락·PNG 차이 | 0쪽 / 0쪽 | 0쪽 / 0쪽 |
| 과거 보호 공식 값 미달 | 862행 | 49행 |
| 과거 보호값 대비 최대 하락 | 1.80%p | 0.52%p |
| 90% 미만·쪽수 불일치·누락 | 0 | 0 |

[전쪽 대조·문서별 요약](../assets/pr8945/protection/README.md),
[3,542행 비교와 모든 PNG SHA-256](../assets/pr8945/protection/comparison.tsv),
[72 dataset·7,084쪽의 실제 산출물 검증](../assets/pr8945/protection/saved-results-verification.json)을 보존했다.
본문 글꼴·간격, sample10 제목 차례의 괘선 등 기존 직접 판독 차이도 그대로 남긴다.
점수가 90% 이상이라는 이유로 이 차이를 면제하거나 이번 PR이 해결했다고 표시하지 않는다.
모두 같은 현재 base/후보에서 같은 출력이다. 수용 요청은 **이 기존 차이에만 한정**하며
새 회귀·새 결함·측정 누락이나 보호 TSV 변경을 포함하지 않는다.

## 재개와 원격 범위

검토 중 장시간 실행 세션 3개가 SIGTERM으로 종료됐다. 완료한 문서의 입력/PDF/producer/
글꼴·PNG·부분 산출물 해시를 재검증하고, 미완료 쪽 묶음은 정본 재개 기능으로 실행했다.
여러 단독 명령이 같은 출력 루트를 사용해 생긴 target-plan 재개 거부는 하네스 운용 오류로
분리한다. 이를 제품 검사 실패나 성공으로 바꾸지 않으며 완료 manifest의 독립 검증 결과를 연결한다.
원시 로그는 ignored `output/`에만 두고 정식 자료에 commit하지 않는다. 글꼴 프로그램도 배포하지 않는다.

WASM sample10의 첫 실행에는 연결 그림 3개 공급을 누락했다. 이를 PR 결함으로 판정하거나
해당 쪽을 제외하지 않았다. Native와 같은 원본 세 그림의 Git blob·SHA-256을 확인해
`--linked-image`로 공급하고, 기준/후보 모두 새 경로에서 전 763쪽을 다시 실행했다.
첫 쪽 raw는 양쪽 모두 31.60% → 99.96%로 복원됐다. 이는 검증 환경 보정이며 원 PR
코드 보정이 아니다. [그림 공급·직접 비교](../assets/pr8945/protection/README.md)에 연결했다.

최종 원격 조회에서 원 head와 devel은 그대로이며 38개 check가 모두 성공 또는 정상 skip이다.
mergeability는 clean이고 fresh merge-tree도 exit 0이다. 실제 로컬 source와 병합 simulation의
차이는 검토 계획 문서 한 파일뿐이다. [확인 원본](../assets/pr8945/final-remote-state.json),
[원격 처리안](../assets/pr8945/remote-action-plan.md)을 준비했다. 원격 처리는 별도 승인 뒤 수행하며
#7620은 부분 해결 이슈로 OPEN을 유지한다.

## Merge 후 contributor PR comment 계획

- 원 PR b 사례 1·2쪽의 같은 입력·PDF에 대응하는 Native/fresh WASM review·overlay를 표시한다.
  승인 리뷰에서 쓰는 이미지는 원 head에 이미 공개된 **원 기여자 제출 증적**임을 표시하고,
  독립 메인테이너 실행의 source·수치와 구분한다. 원격 PNG 바이트·SHA를 대조했다.
- 병합 뒤 원격에 보존한 메인테이너 증적은 b/c/d 각 1·2쪽과 a/None 대조군의 실제 Native/WASM
  PNG에서 생성한 review·overlay를 사용한다. 렌더를 다시 실행하거나 점수를 바꾸지 않는다.
- merge SHA·운영 기록 SHA·CI·집중 14건·목표 8쪽씩 raw 100.00%와 자동 보조값의 한계를 적는다.
  과거 보호값의 기존 차이는 이 PR 한정 결정으로 구분하고 새 개선으로 주장하지 않는다.
- asset이 devel에 존재한 뒤 고정 commit raw URL의 이미지 형식·SHA를 확인하고 댓글을 게시한다.
  API 본문과 실제 화면의 이미지 표시를 확인한다. #7620에는 바닥 예산의 부분 개선과 잔여 범위를
  설명하며 OPEN을 유지한다. fork branch는 보존한다.

## Merge·후속 기록

- 원 PR은 2026-11-11에 merge commit `4ab860651b857bfe6d28b22b4565b2855bebb042`로 병합했다.
  [승인 리뷰](https://github.com/edwardkim/rhwp/pull/8945#pullrequestreview-5858273982)의 exact head·본문·UTF-8 일치를 재조회했다.
- 실제 검증한 로컬 source와 원격 merge의 runtime·fixture 파일은 전부 같다. 차이는 로컬 검토 계획
  문서 한 파일뿐이며 source 원장에 남겼다.
- [병합 후 duration workflow](https://github.com/edwardkim/rhwp/actions/runs/43015665339)가 성공했고, 실제 worker 측정 4개를 수집해
  메타데이터 commit `8f761d8b7903705dcdcdda42ce8dd2f95c3ab731`로 게시했다. 제품 CI·CodeQL을 재실행하지 않았다.
- 기록은 maintainer 직접 반영으로 선택했다. archive review·assets·오늘할일만 한 운영 기록 commit으로
  devel에 반영하고 최종 sync 뒤 아래 확정 본문으로 댓글을 게시한다.
- #7620의 OPEN 유지와 fork branch 보존을 확인한다. 게시/API/화면 확인 및 종료 정리 결과는
  저장소 밖 `~/.local/state/rhwp/cleanup/pr8945-20261111.json`에 기록한다.
- [머지 댓글 확정 template](../assets/pr8945/merge-comment-template.md),
  [이슈 진행 댓글 확정 template](../assets/pr8945/issue-comment-template.md),
  [병합·duration machine receipt](../assets/pr8945/post-merge-record.json).
