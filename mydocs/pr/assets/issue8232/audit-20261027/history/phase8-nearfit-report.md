---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase8-nearfit-report.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase8-nearfit-report.md` · 원문 SHA-256: `37db4e27822a7c43bbeb1525eb8050a2021ff70905a9c1effd7c4a7bc4bbd31e`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

## #8232 추가 조사 — 잘못된 출력 자체를 정답으로 고정하는 검사

조사 head는 `238de6de41a208648f4ade6b494fe433a81c1fce`입니다. 전수 조사는 미완료이며 제품·테스트 변경, 이름 변경, 커밋·PR 제출은 아직 없습니다.

### 보정 목적 정정

기존 검사 형태나 기대값을 무조건 보존하지 않습니다. 기존 기대값과 검출 목적부터 검증하고, 현행 지침·독립 근거에 맞지 않는 단언은 교체하거나 제거합니다. 유효한 결함 검출 목표는 독립 근거에 따른 내용·소속·관계 검사로 보호합니다. 실제 보정한 파일·함수 및 호출·등록 경로만 `issue_8232` 기준으로 갱신하고, 이전 이름의 wrapper나 중복 등록은 만들지 않습니다.

### 추가 함수와 helper 조사

integration 200그룹의 단언·호출과 필요한 원본/helper 문맥을 추가 대조했습니다. 국소 수치 의미상 원본 단위·상대 계약인 113그룹, 독립 근거를 더 연결해야 하는 83그룹, 보정이 필요한 4그룹을 구분했습니다. 이 수는 전체 호출 경로의 적합 판정 수가 아닙니다.

- `issue_7059_nearfit_shrink_stored_ladder::recipe_still_disagrees_with_its_oracle`은 한컴과 다른 좁은 표 폭을 의도적으로 유지하도록 단언합니다. 현재 PASS가 올바른 회귀 계약의 증거가 되지 않는 실제 사례입니다.
- `issue_7150_tac_line_owner_anchor`의 세 검사는 같은 입력의 1쪽 Native raw가 77.19425%인데 y·높이를 ±0.2/0.3px로 고정합니다. 같은 쪽 WASM 100%로 Native 미달이나 px 자격을 상쇄하지 않습니다.
- 숫자 단언이 없는 기대값 생산 helper 2개도 원장에 추가했습니다. `widths`의 `declared = drawn.max(body)`는 원본 선언을 읽지 않습니다. 단, 현재 세 caller는 이 두 번째 반환값을 버리므로 이것이 기존 PASS를 직접 만들었다고 주장하지 않습니다. `max_table_width` 역시 원본 표 소유 대신 모든 Table의 최대 폭을 고릅니다.
- 글꼴 trace의 `baseAdvanceHwpunit`와 허용폭 2/4는 HU입니다. CSS px 2/4와 혼동해 정상 프로그램 단위 계약을 완화하지 않습니다.

통합 후보 원장은 9,666함수/helper 그룹·2,103파일입니다. 이는 위반 수가 아니며, 조건부·간접 호출과 미분류 항목은 계속 조사합니다.

### Book/Movie/Recipe의 새 독립 Print와 현재 양쪽 출력

세 입력의 저장 제품은 한컴 2010입니다. 2020 engine·Print method 0·one-up·timeout 1800으로 새 독립 PDF를 생성하고, 현재 canonical PDFium 5.13.0·96 CSS dpi·2px raw로 전쪽을 비교했습니다.

| 원본 | 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| `samples/basic/BlogForm_BookReview.hwp` | 1 | 100.00000% | 100.00000% |
| `samples/basic/BlogForm_MovieReview.hwp` | 1 | 97.58688% | 97.58688% |
| `samples/basic/BlogForm_Recipe.hwp` | 1 | 90.09535% | 90.05374% |

Recipe Native review와 overlay를 직접 대조했습니다. 주 표의 오른쪽 끝이 좁고 본문·제목에도 미세 차이가 남습니다. 세 조건이 90% 이상이라는 결과를 Movie/Recipe의 절대 px 검사 자격으로 해석하지 않습니다.

RHWP 모델을 거치지 않은 CFB·원시 레코드 대조에서 대상은 최상위 문단 1의 표이며, 세 입력 모두 선언 폭 35,719HU(476.253333 CSS px), 본문 폭 35,149HU(468.653333 CSS px)입니다. 현재 출력의 Book/Movie는 약 476.3px, Recipe는 약 468.7px입니다. compact tree 수치는 소수 첫째 자리로 반올림되므로 정밀 좌표 정답으로 쓰지 않습니다.

독립 PDF의 실제 stroke path는 세 입력 모두 주 가로선 폭 475.79068 CSS px입니다. 선언 frame과 stroke 범위는 서로 다른 의미이므로 두 값을 정확히 같다고 단언하지 않습니다. 기존 Recipe 검사의 `abs(drawn-body) <= 1.5`는 잘못된 본문 폭 축소를 수용하고, 선언 폭을 보존하는 출력을 약 7.6px 차이라는 이유로 거부합니다. 이는 단언의 수학적 반례이며 과거/보정 engine 실행 결과로 대체하지 않습니다.

### 실제 기존 검사 실행 및 원 해결 경로

현재 source에서 `run-rust-test.mjs issue_7059_nearfit_shrink_stored_ladder`를 release-test·threads 8로 실행했습니다. 실제 suite는 `regression_suite_020`이며 3 PASS, 271 filtered, 실행 0.528초, exit 0입니다. run id는 `cc9deae4-35d0-48f1-9936-12bb212a2745`입니다. 전체 회귀 또는 수정 후 통과가 아닙니다.

원 제안 PR #7120은 CLOSED·미병합이며, 실제 반영은 통합 PR #7138의 `63547ddf1525b9c2ce35e580a32f72905c2cda49`입니다. 현재 HEAD의 ancestor임을 확인했습니다. 원 PR와 로컬 검토 기록도 Recipe 잔여를 명시합니다. `merge_commit_sha` 필드만 보고 원 PR가 병합됐다고 판단하지 않습니다.

제품 출력의 원인을 보정하고 독립 기대 계약을 정한 뒤 검사를 재작성해야 합니다. 잘못된 Recipe 상태를 유지하는 검사만 새 이름으로 복사하거나, 공차만 넓혀 통과시키지 않습니다. 전수 판정·필요 보정·이름 변경·등록/검출력 재검증은 미완료이며 PR은 나중에 진행합니다.

### 직접 소수 공차 후보의 추가 대조

이후 integration 270그룹의 직접 단언 의미를 더 분류했습니다. 원본 HU·글꼴 em·실제 요소 사이의 상대/동등 계약인 146그룹과 독립 원본·출력·caller 대조가 더 필요한 124그룹입니다. 이번 두 묶음에서는 위반을 추가 확정하거나 검사·구현을 변경하지 않았습니다.

`tests/`의 미분류 함수 중 직접 소수와 abs/approx/assert_near 등이 나타나는 검색 집합에는 남은 항목이 0입니다. 이는 그 검색 조건에 한정한 목록 대조이며, 전체 전수 조사 완료가 아닙니다. 상수·정수·생성 기대값·숫자 없는 selector·다른 파일 helper·동적 순회·조건부/E2E 경로와 이미 보류한 독립 증거 대조는 남아 있습니다.

추가로 표식 번호를 고정 y270/558±8 영역에서 찾는 검사, cellzone 선을 고정 x/y 범위에서 고르는 검사도 확인했습니다. 최종 위치뿐 아니라 대상 선택 자체가 잠정 좌표에 묶였는지 조사합니다. 반면 caret와 실제 glyph 원점의 동등, source HU 여백 편집 차이, 독립 글꼴 em, actual 셀 내부 포함은 소수 공차가 작다는 이유만으로 지침 위반으로 처리하지 않습니다.

source 내부 80그룹도 추가 분류했습니다. HWPX→HWP의 bit/byte·원본 문자/자원 인덱스·CSV·보안 nonce·데이터 추출·캐럿 탐색 성능 카운터 등이며, 직접 단언 수치는 실물 페이지의 절대 px 핀이 아니었습니다. 예를 들어 0.01/924.09는 직접 작성한 shape의 original/current 크기 비율을 저장 affine에 정확히 기록하는 검사입니다. 소수라는 이유로 이 정상 단위/저장 계약을 완화하지 않습니다. 전체 호출 경로와 독립 시각 판정을 마쳤다는 뜻은 아닙니다.
