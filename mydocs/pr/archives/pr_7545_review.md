---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7545 기여자 변경 검토

## 최종 판정

**메인터너 보정 후 수용 가능** — 셀 폭 격자·120~121쪽 보정·실패한 기존 검사 정리의 개별 수용 범위와 최신 base 통합 후보의 최종 로컬 검증을 완료했습니다. 원 PR을 직접 병합하지 않고 기록한 체리픽 통합 후보를 대상으로 합니다. #7445 문서 전체의 피델리티 승인은 아닙니다. code candidate CI는 완료됐으며 trailing head의 CI·보호 요건 통과 후 병합합니다.

## 접수·범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/7545
- 기여자: planet6897. 제목: fix(layout): 셀 내용 폭을 한/글 저장 격자(4 HWPUNIT)로 내려 줄을 나눈다 (#7412)
- 원 head: `044027a9030f6893b8369be0e64efbb0ec410d57`. base: devel.
- 검토 branch: `review/planet6897-20261004`. 통합 base: `732047aa253e92772fd7625e4a5d7f6a9897fd73`. 적용 단계 head: `caa9d67d3f5171dae027798a8248218343ef5696`.
- 원 PR reviewer: jangster77 지정
- 경로: collaborator_external_pr 9.1.1; intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch, post_merge.
- 작성 시점 CI/mergeability는 참고값이며 수용·push 전에 최신 source head를 다시 확인합니다.

## 원 기여와 체리픽 출처

| 원 commit | 통합 commit/처리 | 비고 |
| --- | --- | --- |
| `044027a9030f6893b8369be0e64efbb0ec410d57` | `35d655c3d81988ba480a28e242e90c805a437a99` | layout_frame::origin_is_authoritative 필드 보존; rows 주석은 셀 폭 격자 근거로 갱신 |

## 조판 원칙과 검증 현황

| 항목 | 현재 근거와 범위 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 셀 폭 격자와 120–121쪽 보정; 전체 문서 잔여는 #7445 | 개별 범위 확인 |
| 측정·배치 일관성 | 아래 저장 사양·내용 소유 및 직접 결과의 범위로 확인 | 개별 범위 확인 |
| 분할·이어받기 계약 | 아래 개별 구조 검사·시각 결과에 한정; 전체 corpus 수용 주장 없음 | 개별 범위 확인 |
| 줄 소속과 점유 높이 | 아래 저장 줄·개체 소유 및 정상 대조 검증 범위에 한정 | 개별 범위 확인 |
| 사례와 증거의 독립성 | 아래 통합 직접 실행·독립 한컴 PDF 또는 사양 및 입력 hash 참조 | 확인 |
| 기준값 변경 | 아래 기존 회귀 보정·유지 이유 참조; 출력 좌표를 새 정답으로 고정하지 않음 | 확인 |
| 주장과 검증 범위 | 개별 범위와 최종 로컬 검증 통과; 최신 head 통합 CI 완료 전 병합 불가 | 통합 CI 대기 |

## 검증 입력 커밋 확인

아래 개별 단계의 실제 입력·독립 PDF·실행 head와 증적 JSON/manifest에 내용 hash를 기록했습니다. 기존 #7445 이관 자료는 해당 단계에 적은 범위만 유지하며 전체 피델리티 승인을 주장하지 않습니다. 최종 전체 검증 결과는 별도 완료 후 기록합니다.

## 시각 검증 계획

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라 Native/fresh WASM의 전체 대상 페이지 TSV를 먼저 산출합니다. 영향 페이지·미달 페이지·대표 경계는 review/overlay PNG로 직접 확인하며 쪽수·각주·문단 소유·누락/중복을 별도 판단합니다. 새 회귀는 실제 검증 범위 최저 90% 이상만 수용합니다.

## 다음 단계

원 PR 단위로 실패 원인과 증적을 먼저 분석하고, 필요한 보정은 코드 수정·결과 보고·커밋을 완료한 뒤 다음 보정으로 진행합니다. 통합 code candidate의 최종 검증 뒤 수용 판정·contributor 후속 comment 계획을 확정합니다. 아직 원 PR 또는 통합 PR을 병합한 것으로 표시하지 않습니다.

## Stage 16 메인터너 보정 — HWPX 구판 여백 단위와 첫 조각 원점

**계속 보류**입니다. 사용자 요구에 따라 HWP 121쪽은 원 PR 정본 93.56018% 이상, HWPX 121쪽은 90% 이상을 해제 조건으로 유지합니다. HWPX를 #7445로 이관하거나 회귀를 삭제하지 않습니다.

- 동일 HWPX를 한컴 engine 2020으로 PDF와 HWP로 각각 변환했습니다. 새 PDF도 156쪽이며, HWP 재저장의 문단 내어쓰기 값은 xmlVersion 1.2 case 값과 같습니다. 현재 파서가 여기에 최신 판본의 2배 정규화를 적용하던 단위 오류를 보정했습니다. 직렬화도 같은 패키지 판본의 역단위를 적용합니다.
- 저장 줄 없는 표 전용 호스트의 첫 조각은 예산에 예약된 앞 여백·오프셋을 확정 배치 원점으로 전달합니다. 픽셀 보정 상수를 추가하지 않았습니다.
- 기존 여백 왕복 검사 2개와 #7412 줄 소속 검사 2개: 4 PASS / 0 FAIL. 검사 수는 늘리지 않았습니다. HWPX의 규제 개요는 독립 PDF 120쪽에 대응하므로 기존의 잘못된 121쪽 전제를 바로잡았습니다.
- 새 WASM은 로컬 대체 빌드입니다. Native/fresh WASM HWP 121쪽은 91.73087%, HWPX 121쪽은 86.09215%입니다. 두 값 모두 이번 사용자 해제 조건을 충족하지 못했습니다.
- HWPX 출력과 PDF는 이제 모두 156쪽이지만, 전체 156쪽 TSV에는 4쪽 조문 대비표의 줄바꿈에서 시작하는 내용 대응 차이가 남아 있습니다. 쪽수 일치만으로 정상 페이지 소유를 주장하지 않습니다.
- 코드·바이너리·WASM 해시와 전체 TSV, 새 121쪽 PNG: [Stage 16 증적](../assets/planet6897_20261004/7545-stage16/results.json). 로그와 진단 변환본은 ignored output에만 보존합니다.

다음 보정은 4쪽 저장 줄의 너비·내어쓰기와 120~121쪽 표 이어받기의 내용 소유 및 글줄 폭을 독립 PDF와 대조합니다. HWP 121쪽의 원 PR보다 낮은 일치율을 허용하거나 Visual Sweep 수치를 인위적으로 변경하지 않습니다.

## Stage 17 메인터너 보정 — 자연 공백·숫자 폭과 물리 프레임 경계

### 원인과 구현

- `한양중고딕` 공백의 기존 `550/1024em`은 양쪽 정렬에서 늘어난 간격을 자연 폭으로 사용했다. 독립 한컴 2024 HWP PDF 122쪽과 재산출한 한컴 2020 HWPX PDF 121쪽의 같은 말미 문장은 자연 공백이 약 `0.5em`임을 확인한다.
- 두 PDF의 Type3 `T6` `/Widths`는 숫자 0~9를 모두 `500/1000em`으로 선언한다. 기존 COM 측정 표의 `509/1024em`을 `512/1024em`으로 보정했다. 공백과 숫자 근거는 [독립 PDF 숫자 폭](../assets/planet6897_20261004/7545-stage17/pdf-glyph-width-evidence.json)과 [말미 줄 공백](../assets/planet6897_20261004/7545-stage17/pdf-natural-space-evidence.json)에 기록했다. 빈 공백 글리프의 선언 `/Widths`와 실제 줄의 이동량은 구분한다.
- 공백만 보정한 중간 후보는 수치는 높아졌지만 `시ㆍ도조 / 례로` 줄 소속 검사를 실패했다. 숫자 폭을 보정한 뒤에도 물리 프레임에 추정용 초과 여유를 더해 4 HU 격자 밖 글자를 허용했다. 확정된 물리 구간의 채움·온전한 낱말 회피·말미 재생은 같은 경계 판정을 소비하도록 수정했다. 추정 폭을 쓰는 기존 scalar 경로와 유효한 저장 줄 재사용은 별도 계약을 유지한다.
- 생산 결과는 `layout_paragraph_in_frame_impl`의 실제 구간 채움으로 만들어지는 줄이다. 셀 측정과 실제 배치가 모두 `recompose_cell_lines_in_frame`을 통해 같은 결과를 소비한다. 문서 ID·페이지 번호·임의 좌표를 구현 조건으로 사용하지 않았다.
- 기존 `#3820` 검사에서 실패한 고정 픽셀 위치는 원본 용지의 본문 하단, 표 안 문단 포함, 앞쪽 소유 문단의 중복 방지, 빈 이어받기 칸 뒤 다음 행의 배치 관계로 교체했다. 새 검사 함수·새 픽스처는 추가하지 않았다.

### 검증 결과

| 대상 | 보정 전 | Native | 새 WASM | 판정 |
| --- | ---: | ---: | ---: | --- |
| HWP 121쪽 | 91.73087% | 96.53486% | 96.53486% | 원 PR PNG 93.56018%, exact source 재빌드 93.77918%보다 높음 |
| HWPX 121쪽 | 86.09215% | 93.93579% | 93.93579% | 요청한 90% 이상 충족 |
| HWPX 120쪽 | 91.69543% | 95.71513% | 95.71513% | 기존 줄 소속 회귀와 시각 기준 충족 |
| 대조군 `76076` 35쪽 | Stage 16 전 92.95975% | 94.21576% | 이 단계에서는 미실행 | 기존 주요내용 줄 소속 유지 |

- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --lib --test regression_suite_014 --test regression_suite_023 --test-threads 8 --no-fail-fast`의 관련 이름 필터로 **96 PASS / 0 FAIL**. 공백·숫자 폭, `#7412`의 HWP/HWPX 줄 소속, `#3820` 소유 관계, 기존 줄 채움·condense 검사를 함께 실행했다.
- `cargo fmt --all -- --check`, `git diff --check` 통과. 새 WASM은 로컬 `wasm-pack --no-opt` 대체 빌드이며 Docker 최적화 빌드 통과로 보고하지 않는다.
- 독립 재산출한 156쪽 `pdf/issue1891/80168_regulatory_analysis-hwpx-2020.pdf`가 기존 PDF를 대체한다. Creator `Hwp 2020`, 변환 job `15cd9992-51c1-4364-870d-c6e77a2de755`. 원 HWP/HWPX는 보존했다.
- [Native·WASM TSV와 hash](../assets/planet6897_20261004/7545-stage17/results.json), [HWP review PNG](../assets/planet6897_20261004/7545-stage17/grid-hwp-p121-native-review.png), [HWPX review PNG](../assets/planet6897_20261004/7545-stage17/grid-hwpx-p121-native-review.png)를 커밋 증적으로 보존한다. 로그는 ignored `output/pr-review/planet6897-20261004/stage17-*`에만 있다.

### 남은 검토와 승인 범위

이번 **121쪽 보정은 검증 완료**다. 전체 PR 승인으로 확대하지 않는다. HWP 120쪽은 기존과 같은 85.38611%이고, 대조군 36쪽도 74.51284%로 남는다(Stage 16 전 후보에서도 74.71294%였다). 기존 구조 회귀의 행·문단 소유 통과를 이 페이지의 시각 정확도 통과로 보고하지 않는다. HWPX 쪽수는 기준과 같은 156쪽이지만 전체 내용 대응을 보장하지 않으며, Stage 17 전체 페이지 TSV와 통합 전체 nextest는 아직 완료하지 않았다. 이 HWPX 문서나 회귀 검사를 `#7445`로 이관·제거하지 않는다.

## Stage 18 전체 TSV와 잔여 범위 이관

사용자의 추가 지시에 따라 157쪽 HWP·156쪽 HWPX의 **문서 전체 피델리티**를 [#7445 추가 기록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5981105691)으로 이관했습니다. 앞 절의 전체 문서 보류는 이 추가 지시로 검토 범위를 분리한 과거 상태입니다. #7545의 4 HU 셀 폭·120~121쪽 줄 소속 보정과 해당 회귀 검사는 유지합니다. 원본·PDF를 삭제하지 않았습니다. 전쪽 통과·전체 내용 대응이 정상이라고 주장하지 않습니다.

- 검증 code head: `072048a8ae2d19b1140c6b675ebfc234961b84a1`. Native 전쪽: HWP 56/157쪽 미달·최저 37.73664%, HWPX 126/156쪽 미달·최저 1.43762%. fresh WASM도 HWP 56/157쪽·최저 37.73664%, HWPX 126/156쪽·최저 1.43762%로 Native와 같습니다. 두 backend의 전체 TSV 산출을 완료했으며 전체 시각 gate는 미달로 남습니다.
- HWP 120쪽은 exact 원 PR 재빌드에서도 85.17792%이며 현재 85.38611%입니다. 원 PR의 121쪽 93.77918%보다 현재 96.53486%가 높습니다. HWPX 원 PR 121쪽 42.38348%보다 현재 93.93579%가 높습니다. 전체 미달을 모두 이번 변경의 새 회귀로 분류하지 않습니다.
- HWPX 6쪽 review PNG를 직접 확인했습니다. rhwp의 규제 필요성 본문과 PDF의 앞 문단 말미가 서로 다른 쪽에 있습니다. 원본 저장 줄 13개와 한컴 재저장의 12개 차이도 확인했으며, 무조건 저장 줄을 재조판한 진단은 155쪽으로 줄어 적용하지 않았습니다.
- [전쪽 TSV·provenance·대표 PNG](../assets/issue7445/80168-20261004/results.json)를 커밋 증적으로 보존합니다. 6글자 ngram 인접 쪽 후보는 진단 보조값이며 내용 소유·완전성 통과 판정의 대용으로 쓰지 않습니다.
- 원 PR의 셀 내용 폭 격자는 `ParagraphBox → LayoutFrame`에서 만들어지고 셀 측정·실제 배치 모두 같은 프레임 줄 구성을 소비합니다. Stage 17의 96 PASS와 독립 PDF 줄 경계·대표 PNG를 근거로 이 제한된 보정 범위는 수용 후보입니다. 전체 통합 nextest·Rust lint·미완료 다른 PR 검증·최신 source head/CI 확인을 완료하기 전 PR 준비 완료로 표시하지 않습니다.

## Stage22 — 전체 회귀에서 확인한 표 폭 소유 검사 보정

최종 전체 nextest(`4f151f870`)는 10,313 PASS/11 FAIL/50 skip로 완료했습니다. 실패 중 eager/on-demand 표 폭 검사 두 개는 목적이 마지막 칸의 프레임 소유와 안 여백 계약인데, 편집 경로에 적용되지 않는 4 HWPUNIT 격자를 정답으로 추가해 5000을 고정하고 있었습니다. 원본 합성 표 선언 폭에서 앞 열 폭을 뺀 독립 관계를 읽도록 바꾸고, HWPUNIT↔실수 정수 절삭의 1 HU만 허용했습니다. 원시 칸 폭만 쓰거나 칸 안 여백을 다시 빼는 구현은 계속 실패합니다. 생산 코드·새 test 함수 변경 없이 기존 두 검사 2/2 PASS입니다.

[보정 결과](../assets/planet6897_20261004/7545-stage22/owner_width_results.json). 합성 저장 폭 계약은 화면 좌표 회귀가 아니며, 실물 120–121쪽의 독립 PDF/90% 증적은 stage17/18에 유지합니다. 다른 9개 차단을 해결하기 전 최종 전체 통과로 보고하지 않습니다.

### Stage22 — 이관한 80168 전체 쪽수 gate의 잔존 입력 제거

기존 HWP5-origin 왕복 목록과 쪽수 원장에서 80168 HWP/HWPX 두 입력만 제외했습니다. 전체 피델리티를 #7445로 분리한 뒤에도 낡은157쪽 pin이 남아 있던 차단이며,156쪽을 새 정답으로 바꾸지 않았습니다. 다른 입력·원장 partition·개별120–121쪽 검사는 유지했습니다. 관련 기존 두 검사2/2 PASS(run `6c88d340-8a1e-4b66-817c-6a75af35d8f4`), [제외 결과](../assets/planet6897_20261004/7545-stage22/80168_gate_exclusion.json). [기존 이관 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5981105691)에 결과를 갱신했습니다.

### Stage23 — 정확한 76076 HWP 기준 재출력 후 차단 범위 이관

실패한 #2308 검사의 입력은 기존 이관 HWPX와 다른 `samples/76076_regulatory_analysis.hwp`였습니다. 해당 입력을 한컴2024로 다시 출력한 82쪽 PDF를 커밋에 포함합니다. Native 전체 82쪽 중37쪽이 90% 미만(최저16.58182%,33쪽87.69301%·34쪽58.82931%)이며 PNG에서 줄바꿈과 표 높이 차이를 확인했습니다. 다른 입력용 PDF로 진행한 이전 비교는 판정에서 제외했습니다.

[#7445 이관](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5981636304) 후 고정 픽셀 위치·높이를 검사하던 함수1개만 제거했습니다. 원본·PDF·다른 구조 검사는 유지하며 남은 기존4개는4/4 PASS(run `b73ec59a-4cac-4810-8193-96f51af56f16`)입니다. [전쪽 TSV·정확한 입력/PDF hash·대표 PNG](../assets/issue7445/76076-original-hwp-20261005/results.json). 전체 피델리티 통과나 fresh WASM 확인을 주장하지 않습니다.

### Stage24 — 78494의 전체 쪽 구성 불일치 차단 분리

사용자 지시에 따라 [기존 #7445의 78494 이관 범위](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5981655880)에 이번 전체 검증 차단5개를 연결했습니다. Native75쪽/PDF74쪽으로 전체 sweep gate가 실패하며, 동일 번호74쌍의 진단 TSV도71쌍이90% 미만입니다. 이것을 전쪽 비교 성공으로 표현하지 않았으며75쪽 누락 대응을 명시했습니다. 8·37쪽 PNG에서 줄바꿈과 본문 소속 차이를 직접 확인했습니다.

쪽·줄 경계 및 고정 그림 위치를 단정하던 실패5개만 제거했습니다. 원본·독립 PDF·다른 검사와 다른 문서 범위는 유지했습니다. 남은 관련3모듈의 기존8개 검사8/8 PASS입니다. [동일번호 진단 TSV·입력/PDF hash·대표 PNG와 실제 제외 목록](../assets/issue7445/78494-20261005/results.json). 이관 문서의 전체 피델리티가 정상이라는 판정이 아닙니다.

### Stage25 — 정상 clipping 진단 수를 쪽 소속 계약으로 분리

`hy_ladder3`은 독립 PDF와 Native/fresh WASM 모두2쪽이며93.11280%·100%입니다. 첫 쪽 하단 clipping은 PDF에서도 동일하고 뒤 문단은 둘째 쪽에만 있습니다. 원시 `LAYOUT_OVERFLOW_CELL`12→13줄 증가를 소실 회귀로 단정한 고정 개수 검사에서 해당 입력만 분리했습니다. 수집·partition·관측 dump는 유지하고 기존 #7418의2쪽 표 소속·후속 문단 누락/중복 검사를 계속 적용합니다. 새 테스트를 추가하거나 이 문서를 #7445로 이관하지 않았습니다.

원장 partition3은1/1 PASS(run `1a55c4bc-f769-46d9-a0ee-fb109986fbff`), 기존 소속 검사는 직전 stage24의 host 모듈5/5 PASS에 포함됩니다. [두 출력의 전쪽 TSV·PNG·판정](../assets/planet6897_20261004/7545-stage25/results.json). 이제 첫 전체 실행의11개 실패를 모두 개별 처리했으며 최종 전체 재실행은 별도 확인합니다.

### Stage26 — 최종 all-target Clippy의 테스트 모듈 순서 보정

전체 nextest10,318 PASS 후 all-target Clippy에서 HWPX 판본 helper가 테스트 모듈 뒤에 놓인 `items_after_test_module` 오류를 확인했습니다. 기존 `cfg(test)` 모듈을 파일 끝에 배치했습니다. 테스트 모듈 본문과 이를 제외한 생산 코드 본문이 전후 동일함을 확인했으며 동작·회귀 기대값은 바꾸지 않았습니다. HWPX 파서 기존4개는4/4 PASS, workspace all-target Clippy도 `-D warnings`로 통과했습니다. [본문 동일성 hash·실행 head와 결과](../assets/planet6897_20261004/7545-stage26/results.json). 전체10,318 실행 head와 이 순서 정리 후 검증 head를 최종 증적에서 구분합니다.

## 리베이스 전 통합 로컬 검증 — 2026-10-05

전체 회귀 head `aa10d4093f60c4e0c12aa186f2464861acbf89ea`, 최종 lint/build·fresh WASM·Skia head `42ef60c96`(테스트 모듈 순서만 정리), base `731de9e1b4bb946d76f35108ed7e186ebe4ebecb`입니다. 전체 nextest는 **10,318 PASS/0 FAIL/50 skip**,563.885초, threads=8로 완료했습니다. Format·Native/WASM32 Clippy·workspace build·workspace all-target Clippy·manifest/unit tier 정책을 통과했습니다. fresh WASM은 Mac 로컬 `--no-opt` 대체 빌드이며 Docker 최적화 검증으로 보고하지 않습니다. Skia lib·그림 placeholder·직접 PDF 출력 범위도 실제 실행했습니다. Studio 전체1,815 PASS/0 FAIL/2 skip와 TypeScript/Vite build도 완료했습니다.

[정확한 head·명령·결과·원 source CI·개별 시각 범위](../assets/planet6897_20261004/final_validation_before_7574.json)를 참고합니다. 시각 자료는 각 단계의 생산 code head와 입력/PDF hash로 고정되어 있으며, 이후 변경은 테스트·문서 범위입니다. 최종 fresh WASM hash와 기존 단계의 hash, 생산 본문 동일성 증적을 함께 기록합니다. 대용량 미달 문서는 #7445의 기존·추가 이관 기록으로 추적하고 그 문서 전체의 정확성을 승인하지 않습니다. 위의 “통합 검증 미완료/대기”는 해당 단계 작성 시점의 기록이며 최종 로컬 판정은 이 절을 따릅니다. 통합 CI는 아직 미실행입니다.

## Merge 후 contributor PR comment 계획

기여자의 셀 폭 격자·120~121쪽 보정·실패한 기존 검사 정리 결과와 이 문서에 적은 추가 보정의 원인·범위를 한국어 존댓말로 구분해 설명합니다. 실제 통합 merge SHA·CI URL·Visual Sweep 정본과 이관 잔여 범위를 안내하고, 통합 대체 병합으로 원 PR을 close합니다. UTF-8 `--body-file`로 게시 후 API로 본문·close 상태를 재확인하며 중복 댓글은 게시하지 않습니다. [대표 PNG](../assets/planet6897_20261004/7545-stage17/grid-hwpx-p121-native-review.png)는 실제 merge SHA에 고정한 raw URL로 표시합니다.

### 최종 출력 provenance 재확인

최종 Native CLI(`native-skia`)와 실제 Chrome/fresh WASM으로 수용 범위20개 target·104쪽씩을 다시 내보냈습니다. **두 경로 최저90.80846%,90% 미만0쪽**입니다. SVG가 byte-identical인 쪽의 raster만 재사용하고, 달라진 SVG·글꼴 정책 블록은 새 raster로 비교했습니다. [전쪽 수용 범위 TSV·입력/PDF hash·현재 SVG hash와 재사용/재출력 구분](../assets/planet6897_20261004/final-visual/results.json). #7445 이관 문서의 전체 수용 주장은 아닙니다. 자산/속성 기여는 원래의 제한된 검토 범위를 유지하며 Studio crop은 동일한 최종 WASM의 실제 Canvas2D/CanvasKit 증적을 유지합니다.

## 최신 base 동기화 — #7574

base `732047aa253e92772fd7625e4a5d7f6a9897fd73` 위로69개 commit을 충돌 없이 리베이스했습니다. 후보 `2fae901137a3d20662ce203fc4f5fb1ea7cd68ec`입니다. 이전 문서에 기록한 실행 head와 결과는 리베이스 전 실제 실행 기록이며 최신 후보의 통과로 재사용하지 않습니다. [이전 검증 보존](../assets/planet6897_20261004/final_validation_before_7574.json), [현재 재검증 상태](../assets/planet6897_20261004/final_validation.json). 원 contributor head/history는 변경하지 않았습니다.

## #7574 리베이스 후 전체 회귀 완료

- 최신 base `732047aa253e92772fd7625e4a5d7f6a9897fd73`, 실행 head `9e5f0c3fda484f756c8aca819c87d0578b428141`에서 전체 nextest **10,321 PASS / 0 FAIL / 50 skip**, 495.993초(threads=8, slow8)를 확인했습니다. upstream의 기존3개 검사가 포함되었으며 이번 보정으로 새 검사를 추가하지 않았습니다.
- 전체 회귀 완료 시점에는 나머지 검사가 진행 중이었습니다. 아래 최종 검증 절에 실제 완료 결과를 기록합니다.

## #7574 리베이스 후 최종 검증

- 최신 base `732047aa253e92772fd7625e4a5d7f6a9897fd73`, 전체 회귀 head `9e5f0c3fda484f756c8aca819c87d0578b428141`에서 **10,321 PASS / 0 FAIL / 50 skip**, 495.993초(threads=8)입니다. 필수 Format·Native/WASM32/전체 target Clippy·workspace build·manifest·unit tier·fresh WASM·Skia 검사를 순차 실행하여 통과했습니다. Skia lib **4,109 PASS / 0 FAIL / 13 skip**, placeholder2·직접 PDF4 PASS입니다.
- 현재 후보의 Native/fresh WASM 수용 범위 **104쪽씩, 최저 90.80846% / 90.80846%, 90% 미만 0쪽**입니다. 실제 Chrome에서 현재 pkg를 로드했으며 모든 대상 SVG를 재출력했습니다. byte-identical SVG의 기존 raster만 재사용하고 달라진 쪽은 다시 raster했습니다. #7445 이관 문서의 전체 수용을 주장하지 않습니다.
- Studio 그림 자르기도 현재 fresh WASM의 실제 Canvas2D/CanvasKit에서 3쪽씩 재캡처했습니다. backend fallback 없이 각각 최저 **98.90923% / 95.73803%**이며 기관 로고·사진·대조군을 직접 확인했습니다. Studio 소스와 테스트는 리베이스 전 전체1,815 PASS/0 FAIL/2 skip 및 TypeScript/Vite/PWA build 이후 변경이 없습니다.
- [최신 검증 head·명령·결과](../assets/planet6897_20261004/final_validation.json), [전체 수용 범위 TSV·source/PDF/SVG hash](../assets/planet6897_20261004/final-visual-rebased/results.json), [Studio 재캡처 증적](../assets/planet6897_20261004/7529-final-rebased/results.json). 이전 base의 실제 실행 기록은 보존했고 현재 판정은 이 절을 따릅니다. 최신 통합 PR CI·보호 요건·merge는 아직 완료 전입니다.

최신 [대표 review PNG](../assets/planet6897_20261004/final-visual-rebased/7545b/native/review_121.png)와 [overlay PNG](../assets/planet6897_20261004/final-visual-rebased/7545b/native/overlay_121.png)는 merge 후 실제 merge SHA에 고정한 raw URL로 contributor comment에 표시합니다.

## 통합 PR #7575 코드 후보 CI 완료 — 2026-10-05

- [PR #7575](https://github.com/edwardkim/rhwp/pull/7575)의 code candidate `86a6fe85087e4371ac929df2036461ace7a0e986`, base `732047aa253e92772fd7625e4a5d7f6a9897fd73`에서 Full 검증이 완료됐습니다. preflight는 `no-green-build-candidate`로 Full을 선택했습니다. trusted job 성공을 재사용으로 해석하지 않았습니다.
- [CI 37221230992](https://github.com/edwardkim/rhwp/actions/runs/37221230992) · [CodeQL 37221231003](https://github.com/edwardkim/rhwp/actions/runs/37221231003) · [Render Diff 37221230707](https://github.com/edwardkim/rhwp/actions/runs/37221230707) · [Adapter 37221230904](https://github.com/edwardkim/rhwp/actions/runs/37221230904) · [Proptest 37221230929](https://github.com/edwardkim/rhwp/actions/runs/37221230929)는 같은 repository·source branch·PR event·candidate SHA에서 모두 success입니다. CI Impact Policy를 포함한 최신34개 check는 success 또는 정당한 skipped/neutral로 완료됐습니다.
- 실제 CI tested merge `b0c5f9c3758fb35eb37121f15fb8566062ca2d0c`, tree `568fcbc3e837d7f61abda92043ebaf34d341868c`의 parent는 위 base/code candidate와 일치합니다. [run·job·artifact·head 증적](../assets/planet6897_20261004/ci_candidate_7575.json).
- 개별 수용 범위·잔여 판정과 로컬10,321 PASS/0 FAIL/50 skip를 유지합니다. 이 기록은 source/test/fixture/workflow를 변경하지 않는 single-parent trailing 문서 commit입니다. 새 head의 required checks·실제 fast-pass provenance·mergeability·보호 요건을 확인한 뒤 정상 경로로 병합합니다. 아직 merge 완료를 주장하지 않습니다. 병합 후 실제 merge SHA와 보정 이유를 원 contributor PR에 설명하고 통합 대체 병합으로 close합니다.
