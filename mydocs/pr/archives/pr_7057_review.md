# PR #7057 self-review — 조판 지침과 공통 PR 준수 검토

- PR: [#7057](https://github.com/edwardkim/rhwp/pull/7057), Issue: [#7056](https://github.com/edwardkim/rhwp/issues/7056)
- 검토일: 2026-09-12 KST. 작성자 `jangster77`의 self-review이며 외부 reviewer는 지정하지 않았다.
- 사용자 승인: 이슈 등록·지침 개선·모든 PR 경로의 공통 준수 항목 추가, 이후 remote push·PR 생성.
- base route: `collaborator_self_merge`; modifiers: `intake_and_review`, `local_validation`, `review_only_fast_pass`.
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 경로와 `visual_fixture_evidence.md`.
  시각 증적 문서는 규칙 연결 정합성 검토에 사용했으며 실물 렌더 검증을 수행한 것은 아니다.

## 대상과 범위

| 항목 | 확인 결과 |
| --- | --- |
| 문서 candidate | `cac22f52efc48717e6b873bbef5fc0e5675d5920` |
| 기준 `upstream/devel` | `8a06c99b91916b6c01756ff1bfdebe4d40dca0b1` |
| base / head branch | `devel` / `docs/7056-typesetting-review-guidance` |
| 원격 상태 | 생성 뒤 조회 시 OPEN, non-Draft, MERGEABLE / BLOCKED; CI 진행 중인 작성 시점 참고값 |
| 규모 | 기록 추가 전 11파일, +297/-12, 3 commits; 이번 trailing은 review·오늘할일·보고서 기록만 추가 |
| 참조 | PR 본문의 `Closes #7056`, [수행 계획](../../plans/task_m100_7056.md), [결과보고서](../../report/task_m100_7056_report.md) |

제품 source·test·fixture·baseline·Cargo·CI workflow는 바꾸지 않았다. 공통 지침 2개와 review 절차
5개를 수정했고 계획·결과 문서 4개를 추가했다. 이 기록이 추가되는 head에서도 같은 문서 범위다.
`AGENTS.md`·`CLAUDE.md`는 review-only 허용 목록의 `mydocs/**` 밖이므로, 문서 PR이라는 이유만으로
GitHub fast-pass나 heavy job skip을 예단하지 않는다. 실제 최신 head의 required checks를 확인한다.

## 공통 조판 원칙 준수 검토

[공통 접수 2.7](../../manual/pr_review/intake_and_review.md#27-조판-원칙-준수-검토)에 따라 실제 diff와
주장 범위를 확인했다. 제품 조판·줄 구성·baseline 변경의 실물 검증은 **비해당**이다. 지침 문서의
공통 적용과 기존 계약 보존은 아래처럼 직접 확인했다.

| 검토 항목 | 근거와 범위 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | [원 검토 댓글](https://github.com/edwardkim/rhwp/pull/7040#issuecomment-5637631535)의 규칙·증거 요구를 문서화하고 샘플 전용 예외와 구별했다. #7040 구현을 재심사하지 않았다. | 충족 |
| 측정·배치 일관성 | 제품 측정·배치 코드는 변경하지 않았다. 공통 결과의 요구는 AGENTS가 소유하고 reviewer는 코드 위치·자료 흐름을 확인하도록 연결했다. | 비해당 |
| 줄 소속과 점유 높이 | 제품 계산은 변경하지 않았다. 저장 LineSeg/재조판 구분, 기준선·여백·줄간격, 중첩 표에 한정한 네 사례의 전달을 확인했다. | 비해당 |
| 사례와 증거의 독립성 | 합성 계약/정상 한컴 출력/미검증 구분과 실행 검출 회귀/코드 우려 구분을 공통 절차에 반영했다. 실물 자료 추가·검증은 없다. | 충족 |
| 기준값 변경 | baseline·golden·허용치 데이터 변경 없음. 정상 변화의 독립 근거 요구를 로컬 검증과 시각 증적 절차에 연결했다. | 비해당 |
| 주장과 검증 범위 | source SHA·명령·기존 메타데이터 오류·미실행 항목을 기록하고 문서 수정이 실제 에이전트 준수를 보장한다는 주장을 하지 않았다. | 충족 |

공통 계약과 선택표가 모두 접수 2.7을 연결하며 maintainer 일반·collaborator self·collaborator 매개
외부 PR에 동일하게 적용된다. 특정 역할 전용 문서에 규칙을 두지 않았다. 개별 항목 상태와 PR 최종
판정 세 종류를 구분하고, 비해당 이유·필수 증거 누락·보류 해제 조건이 기록되도록 했다.

## 로컬 검증 결과

candidate에서 다음을 실제 실행했다. 상세와 기존 오류 목록은 [Stage 1](../../working/task_m100_7056_stage1.md)에 있다.

- `git diff --check upstream/devel...HEAD`, `git diff --check`: 종료 코드 0.
- `python3 scripts/check_markdown_links.py --changed-from upstream/devel --forbid-redirect-references`:
  종료 코드 0, 618개 문서·변경 11개·redirect 30개, 내부 링크 오류 없음.
- 추가한 조판 anchor 12개와 실제 제목 대조, `@AGENTS.md` import 및 기존 Rust lint·suite·증빙 절 보존 확인.
- `python3 scripts/check_document_metadata.py`: 전후 모두 기존 16건으로 종료 코드 1. 신규 오류 0건.
  변경 장기 문서 5개를 같은 `validate_file`로 확인한 결과 오류 0건.
- `git merge-tree --write-tree <base> <candidate>`: 종료 코드 0,
  tree `c324560bf2e589dac65f136022e59cb421688c62`가 검증 candidate tree와 같고 공백 검사 통과.

문서-only 변경이므로 Cargo·WASM·한컴 변환·실물 visual sweep·제품 성능 계측은 수행하지 않았다.
상수·정책·호출 경로의 제품 동작 변경도 없어 로컬 검증 4.3.0.0의 제품 음성 대조는 비해당이다.
시각 판정을 승인 근거로 사용하지 않아 대표 이미지와 contributor 시각 comment 계획은 비해당이다.
GitHub CI는 생성 뒤 진행 중이며 이전 head의 결과를 trailing head 통과로 쓰지 않는다.

## 최종 판정과 후속 조건

- 최종 판정: **승인** — 문서 개선 범위의 self-review 판정이다.
- 근거: 공통 적용 경로·조판 계약·증거 구분·기존 검증 및 권한 경계를 확인했고 로컬 문서 검증에 신규 오류가 없다.
- 잔여 제한: 기존 메타데이터 누락 16건, 실제 에이전트 준수·제품 시각 품질은 미검증.
- trailing push 전: 정확한 최신 base/head로 병합 시뮬레이션, merge tree 공백·링크 검사와
  기존 오늘할일 기록 보존을 확인하고 원격 SHA를 재조회한다. 실제 실행 결과는 PR 본문에 기록한다.
- merge 전: review 기록이 포함된 최신 head의 required checks·mergeability 재확인과 사용자 병합 승인.
- 현재 승인된 원격 작업은 PR 생성·필요한 기록 push까지다. GitHub approve·comment·merge·issue close는 수행하지 않는다.
- merge와 후속 처리가 승인되면 `post_merge.md`의 해당 절차를 적용한다. 현재 전용 branch/worktree를 보존한다.
