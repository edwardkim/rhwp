# PR #7348 — #7280 조판 책임 분리 제출 검토

## 접수와 범위

이 문서는 내부 타스크 작성자의 제출/self-review 기록이며 독립 reviewer의 승인 기록이 아니다.
reviewer assign 또는 GitHub approve는 수행하지 않았다.

- base route: `docs_and_git_workflow.md`의 Internal Task PR Approval.
- 번호 확정 후 기록 경로: `collaborator_self_merge.md`의 self PR 기록 절차를 준용한다.
  실행 계정은 maintainer `edwardkim`이며 admin merge 예외는 사용하지 않는다.
- modifiers / loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
  `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `rework_and_exceptions.md`(대형 PR), `collaborator_self_merge.md`.

| 항목 | 2026-09-23 생성 직후 참고값 |
| --- | --- |
| PR / 작성자 | [#7348](https://github.com/edwardkim/rhwp/pull/7348) / edwardkim |
| base / head branch | devel / task_m100_7280 |
| 첫 제출 head | `b77a0d692718e105c0ef1a1b380eacd7b556125a` |
| base SHA | `1966af77fa8046c844d654b157b5168baad8a30e` |
| 규모 | 182파일, +41,351 / -27,026, base 이후 125 commit |
| 원격 상태 | OPEN, Draft 아님, mergeable=true / blocked, CI 진행 중 |

이 기록의 후속 commit이 최종 CI 대상 head가 된다. 위 mergeability와 CI는 작성 시점 값이며
merge 전 최신 head의 required check·독립 검토·작업지시자 승인을 다시 확인한다.
대형 구조 변경이므로 즉시 admin merge하지 않는다.

[Issue #7280](https://github.com/edwardkim/rhwp/issues/7280)의 목적은 기여자의 조판 코드
추가·수정·삭제 경계와 관리 체계다. 개별 피델리티 버그 수정이나 #7195 미병합 변경 이식이 아니다.
125개 commit은 단계별 구현·보고와 devel 통합 이력이며, 현재 base 기준 제품 diff는 typeset 영역
109파일이다. Cargo/tests/CI 변경은 없고 테스트·baseline·ignore 완화도 없다.

## 실행 검증과 실제 경로 확인

[최종 결과보고](../../report/task_m100_7280_report.md),
[Stage57 통합](../../working/task_m100_7280_stage57.md),
[Stage58 전체 검증](../../working/task_m100_7280_stage58.md)를 재사용한다.
제품 내용은 실행 SHA `29130d539…`와 첫 제출 head가 동일함을 `git diff --exit-code`로 확인했다.
전체 회귀 10,137 PASS / 0 FAIL / 기존 제외 50, Skia lib 4,112 PASS 및 focused 2/4 PASS,
Native 772쪽·WASM 748쪽 데이터와 선택 23쪽 PNG 전후 동일이다.

제출 직전 별도 clean review worktree를 `b77a0d692…`로 고정해 다음을 순차 재실행했다:
prepare → fmt 적용/검사 → Native Clippy(55.51초) → WASM Clippy(26.24초) → workspace build
(1분19초) → workspace all-target Clippy(40.53초) → 고정 base manifest/unit-tier 검사. 모두 PASS.
실제 인자와 로그는 `output/7280/stage60-submit/lint.sh`, `*.log`다.
manifest 1,399 sources / 48 targets, unit-tier 4,205 tests / 298 modules / cfg support 28이다.
fmt 후 제품/테스트 변경이 없고 review worktree도 tracked clean임을 확인했다.

`git merge-tree --write-tree <base> <첫 제출 head>`는 exit 0,
tree `9d05662102f23f07d83c6a6df002d5cd781eda0c`로 head tree와 같았다.
`git diff --check`와 `python3 scripts/check_markdown_links.py --changed-from upstream/devel`도
통과했다(검사 문서 681개, 변경 파일 182개). 제출 직전 원격 devel SHA도 동일했다.

이번에 코드에서 재확인한 대표 연결:

- `paragraph/whole_fit.rs`는 저장 rewind를 occupied height로 판단하고 overflow는 current height로
  구별한다. 기존 override 뒤 확정 결과가 `paragraph/flow.rs`의 failed-fit 호출과
  `paragraph/split_entry.rs::should_advance`로 전달되는 Stage57 경로를 유지한다.
- `state.rs::TypesetState`의 `data`는 private이며 불변 Deref만 있다. 새 mutable data 접근자를
  추가하지 않았다. 나머지 책임·입출력·소비 경로는 [구조 정본](../../tech/typesetting_architecture.md)의
  의미 ID 표와 단계별 검증 증거에 연결한다.

## 조판 원칙 적용과 주장 경계

이 PR은 typeset 경로를 이동하므로 렌더 영향 검증 대상이다. 기존 규칙의 사양 타당성 전체와
이동의 동작 보존은 다른 주장이다. 아래 충족은 명시한 구조 보존 범위에만 적용한다.

| 항목 | 근거·남은 범위 | 판정 |
| --- | --- | --- |
| 구현 근거·일반성 | 신규 조판 예외/상수 수정이 아닌 승인된 책임 분리. 기존 호환 예외는 재승인하지 않음 | 구조 범위 충족; 기존 예외의 사양 재입증 미검증 |
| 측정·배치 일관성 | 기존 결과·평가 순서 이동과 Native/WASM 전체 render tree/SVG 전후 대조. 기존 모든 규칙의 정확성 주장은 아님 | 검사 범위 충족 |
| 분할·이어받기 | table/scan → continuation/fragment → state → layout 경로와 Native/WASM 재개 수명은 R3–R5 기록 및 기존 계약으로 보호. 컷·소유 데이터 전후 동일 | 보존 범위 충족; 모든 합성 경계 전수는 미검증 |
| 줄 소속·점유 높이 | paragraph/line_queries, whole_fit/flow/split_entry의 기존 계약 보존. 새 저장 정보 수용 규칙 없음 | 보존 범위 충족 |
| 사례·독립성 | 별도 빌드한 devel을 동작 기준으로 비교; 한컴 PDF와 내부 계약은 구별. 신규 결함 수정 전 FAIL/후 PASS 주장은 없음 | 충족 |
| 기준값 변경 | 테스트·golden·baseline·ignore 변경 없음 | 비해당 |
| 주장·검증 범위 | 정확한 실행 SHA, 명령, 직접 판독 범위와 기존 PDF 차이를 보고. CPU/메모리 성능은 미측정 | 기록 충족; 독립적인 전체 코드 review는 대기 |

## 입력 커밋과 시각 증적

입력 커밋 확인은 **충족**이다. [asset 목록](../assets/issue_7280_typeset_refactor/README.md)에
14개 원본·11개 PDF의 경로와 역할을 보존했다. 아래 해시는 첫 제출 head의 Git blob/LFS oid와
실행한 파일을 대조한 값이다. 파일로 만든 신규 합성 fixture나 신규 PDF는 없다.

| asset 목록의 key | 역할 | SHA-256 |
| --- | --- | --- |
| square-host | 원본 | `ed9a0589d9223c2750f4fb8240548551d4aa70fd35d6243f405183d525ff1f1f` |
| square-host | PDF | `6be7d47ef90d0af026705e690bbf1aedd4b1254f17668c7d92e8120d4bcc5ec5` |
| square-body | 원본 | `88fb25749003426331b0d055c2bff62b3cb9a182138d0c6f0aadcb80b233f850` |
| square-body | PDF | `df2a1b9bf16b498527603e8bb1118cb3cd9a1d4aa03257b3538693a966dd94e1` |
| square-table | 원본 | `d6f4d431b9a4d934b3b4e4330546ef61768c953c2e1328010d2f75440fefa070` |
| square-table | PDF | `60b4d14e7305d148a913f281c6629b531a3fedcc7e6f76042c0994169001ccfc` |
| night-guard | 원본 | `932152d5f97ef07dcf5f5a4890b123cf31b7073ac80dafadacfa4edb1e01dc86` |
| night-guard | PDF | `3af3cb24be68b9058b64156eb7ae52b88bc1553fc258b557a1b6c2273b5f1c5d` |
| deferred-picture | 원본 | `50094a3db2b2003b293c5cbf43014d001aa97929acb488cef0cb7ea0e16b3113` |
| deferred-picture | PDF | `7879ffee6313575132187c44c0090cd2e62c32c12c29b7eabd989181acf27b3a` |
| tac-order | 원본 | `15e37d4e8139f8cb494f882f6d7423fe8d7ae991b51e238a23f5388699e8ef8a` |
| tac-tail | 원본 | `276c9ac502983b6e438d8dbf766965363da50783ab1f4d2402241bb7adf9d490` |
| square-tac | 원본 | `1b99b763aac36a14a9f463e35ee894a23eb1083780040eab5e0f02a481c694b8` |
| endnote-2022-09 | 원본 | `d451ffbc72e8c84433f380c0cbcc8472f23286d53dce4b793a4bbb82f8715108` |
| endnote-2022-09 | PDF | `da3cd549f5925fe6565ff25349da0f58e9eaf427ecaf05cbf64c2d27056a12a3` |
| endnote-between20 | 원본 | `d306a1025776e56766227e1c2f53d0deda9f1b7ec1ff314dbdfdd323bb800d3e` |
| endnote-between20 | PDF | `4aa311d6c3923d2c7cbea7e7ec1448c2e7b51d15cc376bad754d3a8cfcc78aa2` |
| endnote-zero | 원본 | `0c53a6b07d896f63cab4f27a165600bfda4ec6ef1a38e74fb6b3f5c546f03ffc` |
| endnote-zero | PDF | `33eb1ad732512d10ba8e386124c88d306d640610001f1aa5f63962695438e20c` |
| endnote-no-separator | 원본 | `58edc88128ce8703d64c9dd21f996e1eb270e1da597f891f02988eb58f693bee` |
| endnote-no-separator | PDF | `366470d077efa914df98d71c8165dcd7c940106949e068b552f54f94126ed255` |
| market-rewind | 원본 | `84d297375c48e1e903247a389e9ec3907e3cdc57a4a7e29a819a01c2a1e3a341` |
| market-rewind | PDF | `89ff2583598e1bb8ab9d95b754fcc0cf42e3409547aec4dbe01ac0f4207b312e` |
| chemical-rewind | 원본 | `398d03a5d5e4d6e857086be532d6d9ed0cec9c8ad06f95c17bbb7f83056ae860` |
| chemical-rewind | PDF | `f8e5c0408e221080ede9a9a67b153d02d792d22961c738e46749641f32a32e79` |

대표 Native square-host 1쪽은 pixel match 89.60167%, proxy 9.77316%, 자동 후보 0/1쪽이다.
fresh WASM chemical 13·14쪽은 후보 0/2쪽이며, 대표 14쪽 pixel match 93.79743%, proxy 21.71057%다.
후보 0은 완전 일치가 아니다. 직접 판독에서 전자의 글꼴 굵기·자형, 후자의 표 세로 위치 차이가 남는다.
devel/refactor 동일 출력이므로 신규 회귀와 구별할 뿐 한컴 피델리티 성공으로 승인하지 않는다.
market 5쪽과 endnote 22쪽 잔여 차이도 Stage58의 기록을 유지한다.

Native/fresh WASM 대표 review·overlay 4개는 `mydocs/pr/assets/issue_7280_typeset_refactor/`에 있다.
원시 compare/overlay/review 경로와 SHA는 위 asset 문서에 연결했다. 생성 직후 실제 PR 화면에서
4개 이미지의 로딩/디코드와 고정 head URL을 확인했다. 후속 기록 head로 본문 URL도 갱신한다.
WASM review 상단의 긴 도구 라벨 일부 잘림은 asset 안내에 공개했다. 문서 그림 자체 누락과는
구별하며 게시용 라벨 보완 또는 별도 증적 도구 이슈 처리는 merge 전 검토 항목으로 남긴다.

## Merge 후 contributor PR comment 계획

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라
위 대표 2쪽·후보 수·pixel/proxy 값·직접 판독과 한계를 재사용한다. asset 문서에 열거한 4개 PNG를
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7280_typeset_refactor/<PNG>`로
고정해 Markdown image로 표시한다. asset이 실제 devel merge commit에 존재하고 별도 댓글 승인을
받은 뒤에만 UTF-8 `--body-file`로 게시하고 API로 본문을 확인한다. 이번에는 댓글을 게시하지 않는다.

## 최종 판정

**머지 보류** — 제출 및 작성자 로컬 검증은 완료했으나 최신 원격 CI는 진행 중이고 대형 PR의
별도 검토/작업지시자 merge 판단은 미실행이다. 새 실행 회귀가 확인되어 보류한 것은 아니다.

해제 조건은 최신 head의 required check 성공, 대형 구조 변경 검토와 남은 증적 라벨 처리 확인,
작업지시자의 merge 승인이다. 제품 수정이 추가되면 해당 head를 다시 검증한다.
이 기록은 GitHub approve·admin bypass·merge·이슈 close를 수행하지 않는다.
