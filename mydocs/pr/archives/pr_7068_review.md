---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-13
---

# PR #7068 — #5819 표 생성 CLI/MCP 옵션 self-review

**최종 판정: 승인.** 아래 본문 표 생성 옵션의 계약과 로컬 회귀를 충족했다. GitHub merge를
뜻하지 않으며, 병합 전 최신 head의 required CI와 작업지시자 승인이 필요하다.

## 대상과 경로

| 항목 | 작성 시점 확인 내용 |
| --- | --- |
| PR | [#7068](https://github.com/edwardkim/rhwp/pull/7068) |
| 관련 이슈 | [#5819](https://github.com/edwardkim/rhwp/issues/5819), Closes로 연결; devel 병합 후 종료 확인 |
| 작성자 / 검토자 | jangster77 / jangster77 self-review; reviewer 미지정 |
| base / branch | devel / fix/5819-insert-table-options |
| 최초 제출 후보 | `728452ca673d02dadc3be831d8e26eeeab0ab2f2` |
| 전체 회귀 검증 head | `be3581023549ff7a1db81c600f5c81f71f9af38f`; 제출 후보까지 계획·보고서만 변경 |
| 최초 제출 규모 | 17개 파일, +929 / -142; 본 기록은 문서-only trailing commit |
| 생성 직후 상태 | Open, non-draft, MERGEABLE / BLOCKED; CI 진행 중인 시점의 참고값 |
| base route | collaborator_self_merge |
| modifiers | intake_and_review, local_validation, visual_fixture_evidence, review_only_fast_pass, rework_and_exceptions |

`pr_review_workflow.md`, 선택표와 위 기본·보조 문서, 편집 Command 체크리스트를 적용했다.
1,000줄 초과 규모이므로 코드 검토·병합 시뮬레이션·해당 검증을 구분해 수행했다. 별도 구현
계획서는 생략하고 [#5819 계획](../../plans/task_m100_5819.md)과 본 기록의 처리 순서를 사용한다.

## 변경 계약과 코드 검토

기준 `f537df5ea328ba7d8c0c8e85c0199897c0047049`의 실제 CLI에서 네 새 옵션은 각각
exit 2로 거부됐다. CLI와 MCP를 공통 생성 API에 연결해 다음 동작을 추가했다.

- 양수 HWPUNIT 절대 너비 또는 합계 100% 비율, 열 수와 같은 left/center/right 정렬.
- 유일한 본문 필드 문단 바로 뒤의 독립 표 삽입과 기존 필드 ID·이름·명령·내용 보존.
- CLI/MCP의 반복 머리행 기본 true와 명시적 false. 기존 코어 API는 기존 기본 옵션으로 위임한다.
- 잘못된 옵션과 dry-run은 입력·기존 목적 파일에 쓰지 않는다. MCP의 값 소비 bool false와
  dryRun 같은 presence flag false를 구분한다.

한컴 OLE 실측은 작업지시자 제공 계약으로 사용했다. OLE를 직접 호출하지 않으며 WidthType=2에
대응하는 비등분 셀 너비를 직접 저장한다. HWPX XML의 `pageBreak="CELL"`, `repeatHeader="1"`
기본값과 명시적 `0`을 직접 검사했다. 내부 enum 명칭만으로 판단하지 않았다.

초기 후보의 CellBreak는 XML TABLE로 출력돼 보정했다. 현행 공통 매핑에 맞게 RowBreak와
HWP5 raw TABLE 속성을 함께 설정했다. 생성 래퍼의 패스스루 위임 원장 누락도 보정했고,
실제 무효화 대상 도달 검사까지 통과했다. 공통 serializer 매핑·guard 상한·golden은 바꾸지 않았다.

편집 Command 체크리스트는 CLI/native 문서 mutation 범위로 적용했다. Studio 라우터·Undo/Redo·
selection/caret 경로는 변경하지 않았다. 기존 표 생성의 캐시 무효화와 재조판 경로를 위임한다.

## 완료한 검증

| 검증 | 실제 결과와 head |
| --- | --- |
| #5819 실제 CLI/MCP/XML 집중 회귀 | 5 PASS; 전체 회귀에서도 재통과 |
| 패스스루 분류·위임 보호 회귀 | 5 PASS; 보정 후 전체 회귀에서도 재통과 |
| 전체 release-test | `be3581023`: 9,538 PASS / 46 SKIP / 0 FAIL, exit 0, 실행 334.750초 |
| 제출 후보 별도 worktree | `/Users/tsjang/rhwp-5819-rust-review`, detached `728452ca6`; tracked diff 없음 |
| 제출 직전 Rust lint | 위 worktree에서 prepare, fmt 적용·검사, native/WASM32/all-target Clippy, workspace build, manifest·unit-tier check 9단계 exit 0 |
| 저장 XML 별도 확인 | 2행의 셀 너비 각각 3000/6000/9000, CELL, repeatHeader 1/0; 양쪽 verify diffCount 0 |
| 입력·코드 동일성 | 아래 두 fixture의 실제 파일과 제출 후보 blob 일치; 전체 회귀 head 이후 코드·test 변경 없음 |
| 최초 제출 병합 simulation | base `b116c11d0736fb1c105ed64bfc694b43dcd48b7d`, head `728452ca6`, tree `e196c2124727fd0bb6619897a19a44765f54ec16`; 충돌·공백 오류 없음, 변경 문서 상대 링크 130개 오류 없음 |

전체 회귀는 원 작업 checkout에서 완료했고, 제출 전 별도 detached worktree에서 정확한 제출
후보의 lint·suite 정책 검사를 다시 실행했다. Cargo는 `/Users/tsjang/rhwp/target/pr-review`를
순차 사용했다. 공유 target은 삭제하지 않는다. 파생 suite·manifest는 ignored 검증 산출물이며
PR에 포함하지 않았다. 로그는 `/tmp/rhwp-5819/validation-final2/`와 `validation-worktree/`에 있다.

실제 진입점, 경계·반례와 음성 대조를 검사했다. 절대/비율 너비, 열 정렬, 양 포맷 저장·재파싱,
헤더 on/off, 필드 보존, 잘못된 입력 16종의 실제/preview 요청, 정상 dry-run, 실제 stdio MCP
호출을 확인했다. 한컴 기준 PDF를 사용한 시각 회귀로 승격하지 않는다.

## 공통 조판 원칙과 시각 검증

| 항목 | 판정과 근거 |
| --- | --- |
| 구현 근거와 일반성 | 충족 — CLI 생성 속성 계약과 제공된 OLE XML 실측을 대조; 샘플명 조건·clamp 없음 |
| 측정·배치 일관성 | 비해당 — 측정·배치 알고리즘, 좌표계·backend 분기 변경 없음 |
| 줄 소속과 점유 높이 | 비해당 — LineSeg·줄 구성·점유 높이 규칙 변경 없음 |
| 사례와 증거 독립성 | 충족 — 실제 CLI/MCP 실행과 XML 속성 직접 검사; 자체 IR roundtrip과 외부 XML 계약 구분 |
| 기준값 변경 | 비해당 — baseline/golden/허용치 변경 없음; guard는 위임 분류만 보완 |
| 주장과 검증 범위 | 충족 — 저장 속성·옵션 동작에 한정; PDF 화면 일치·특정 쪽 overflow 해결은 미검증 |

표 생성 속성은 출력에 영향을 주지만 renderer/layout/typeset/paint 알고리즘은 바꾸지 않는다.
이슈의 실제 문서·기준 PDF 첨부가 없고 특정 페이지나 시각 개선을 주장하지 않는다. 이번 옵션·저장
계약 수용에 visual sweep을 추가하지 않았다. Native Skia/WASM 화면 parity/Studio 검증도
미실행이며, 해당 경로의 시각 검증 완료를 의미하지 않는다.

## 검증 입력 커밋 확인

**충족.** 기존 tracked 입력을 재사용했고 실제 실행 파일과
`728452ca673d02dadc3be831d8e26eeeab0ab2f2`의 blob을 바이트·SHA-256으로 대조했다.
`korea_downloads`나 외부 첨부에만 있는 입력을 사용하지 않았다. 테스트가 실행 중 생성하는
HWP/HWPX는 파생 출력이며 원본 테스트 코드와 입력 fixture를 PR commit으로 재현할 수 있다.

| 저장소 입력 / 역할 | SHA-256 |
| --- | --- |
| [samples/field-01.hwp](../../../samples/field-01.hwp) — 본문·중복 필드와 CLI/MCP | `518cb939079e6e0640a5f813597f744e2528a17ca52ee418929f1c8f4b5380c0` |
| [samples/issue5162_field_wraps_table.hwpx](../../../samples/issue5162_field_wraps_table.hwpx) — HWPX 필드·재파싱·XML | `c8f07209f7d60d60c32c7918800545c1b3079ef4e0178eaf44ca2bfe23cfc0c8` |

## 남은 범위와 처리 순서

- 표 셀·글상자 내부 필드와 필드 문단 내부의 문자 위치 삽입은 지원하지 않는다.
- 비공개 235쪽 템플릿의 이미지 보존·성능·198/199/204쪽 LAYOUT_OVERFLOW는 미검증이다.
  이슈 작성자는 overflow를 별도 관찰 대상으로 명시했다. 이는 #5819의 완료 조건이 아니다.
  요청한 표 생성 옵션은 구현·검증했으므로 PR은 `Closes #5819`로 연결한다. 관련 #3608/#4994/#4995의
  전체 범위나 위 미검증 시나리오까지 해결했다고 주장하지 않는다.
- 현재 승인된 원격 작업은 PR 생성과 같은 PR의 검토 기록이다. 본 기록·오늘할일·보고서 갱신을
  문서-only commit으로 추가하며 push 전 최신 base 병합 tree·링크·기존 오늘할일 보존을 검사한다.
- 최종 head의 CI·mergeability를 확인한다. 병합은 최신 required CI와 별도 작업지시자 승인 후이며,
  후속 처리는 해당 시점에 post_merge 절차를 적용한다. 기본 branch가 main이므로 devel 병합 후
  #5819 상태를 확인하고, OPEN이면 후속 처리에서 종료한다.

상세 명령·정정 이력은 [결과 보고서](../../report/task_m100_5819_report.md)를 참조한다.
