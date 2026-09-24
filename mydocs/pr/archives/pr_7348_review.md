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
merge 전 최신 head의 required check·검토 결과·작업지시자 승인을 다시 확인한다.
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
| 주장·검증 범위 | 정확한 실행 SHA, 명령, 직접 판독 범위와 기존 PDF 차이를 보고. CPU/메모리 성능은 미측정 | 기록 충족; 아래 별도 self-review 완료. 외부 독립 승인 아님 |

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
구별하며 후속 절차에서 별도 증적 도구 이슈 [#7349](https://github.com/edwardkim/rhwp/issues/7349)로 분리했다.

## Merge 후 contributor PR comment 계획

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라
위 대표 2쪽·후보 수·pixel/proxy 값·직접 판독과 한계를 재사용한다. asset 문서에 열거한 4개 PNG를
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7280_typeset_refactor/<PNG>`로
고정해 Markdown image로 표시한다. asset이 실제 devel merge commit에 존재하고 별도 댓글 승인을
받은 뒤에만 UTF-8 `--body-file`로 게시하고 API로 본문을 확인한다. 이번에는 댓글을 게시하지 않는다.

## CI 성공 후 self-review — 2026-09-23

작업지시자의 “CI 성공하면 self-review 절차를 진행” 승인에 따라 제출 이후 별도 검토 주기를
실행했다. 대상은 `c08b4fc3bce4b06f207af80bd7c3d33ada75f547`, base는
`1966af77fa8046c844d654b157b5168baad8a30e`다. 작성자 self-review이며 외부 독립 승인은 아니다.

### 원격 CI와 통합

- [CI](https://github.com/edwardkim/rhwp/actions/runs/35771920071): success.
  Archive A–D, Lint(fmt/Native·WASM·workspace Clippy), Native Skia, 최종 Build & Test 통과.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35771919652): success.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35771920125): success, Rust 분석 포함.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35773934089): pass.
  WASM Build/Frontend/prop roundtrip/adapter inter-diff의 정책 skip은 실행 성공으로 세지 않는다.
- fetch 후 base/head 불변, OPEN / mergeable=true / clean. `merge-tree` exit 0이며 결과
  `533d9c93beb4615cc072ae788b60f9ac3c675102`는 head tree와 동일하다.
- 실행 source `29130d539…`부터 검토 head까지 `src`, `tests`, `crates`, Cargo 입력과
  Visual Sweep/WASM export 스크립트 diff가 없다. 기존 필수 lint·전체 회귀·Docker WASM 증거를
  재사용하며, 이번 CI 성공을 이전 head에 소급하지 않는다.
- 추가 focused nextest를 시작했으나 같은 제품에 대한 중복 검증이라는 작업지시자 지적에 따라
  빌드 중 중단했다. `self-review-focused.log`는 통과/실패 증거로 세지 않는다.
  이후 추가 빌드·회귀·시각 재실행 없이 기존 Stage57/58 검증과 위 CI 결과를 사용한다.

### 실제 호출 경로 재검토

| 검토 경계 | 대조한 생산·소비·최종 상태 경로 | 결과와 범위 |
| --- | --- | --- |
| 문단 판단 순서 | `paragraph/flow.rs::place`의 예산→흡수/다단→강제 경계→whole-fit→overflow→failed-fit; `whole_fit.rs::inspect`→`split_entry.rs` | rewind의 occupied/current height 구별, override 이후 반환 플래그, available-height 지연 조회 유지. 검토한 경로에서 신규 순서 역전 없음 |
| 표 컷·높이·방출 | `continuation/fragment.rs` 시작 컷 snapshot→`fragment/budget.rs`→`fragment/scan.rs`→`scan/runner.rs`의 block/row step→`fragment/emit.rs`→PageItem | 원본/row-geometry 소유, 시작 행 물리 tail, 누적 consumed, caption/footnote와 최종 컷 종료를 구별. paint 경로는 변경 없음. 기존 호환 예외의 조판 타당성은 재승인하지 않음 |
| Native와 재개 job 수명 | `continuation/step.rs` 동기 drain / `continuation/job.rs` begin·step·finish→`document_core/queries/rendering.rs:4740` step와 완료 후 pagination 교체 | 동일 fragment 경로를 소비하고 Native는 drain 뒤 state 복귀, job은 완료 전 공개 pagination 유지. 호출자는 base 대비 변경 없음 |
| 문단·구역 최종화 | `section.rs` 문단 flow→anchor 마무리→controls→다음 문단; deferred picture→endnotes→deferred table→flush→finalize | 미주·보류 항목 순서와 종료 후처리 유지. `TypesetState`는 private data와 불변 Deref만 제공 |
| 어울림과 미주 Query/Command | `controls/wrap_flow.rs::place`의 classify→흡수 또는 suffix 판단→밴드 종료→방출; `notes/endnotes/measure.rs`의 로컬 문단/컷 재색인과 scratch layout→fit→`emit.rs` | 밴드 종료 전후 관측 시점과 조건부 available-height 조회 유지. 측정용 노드와 확정 페이지 상태를 분리 |

devel 반영 누락 검토에서는 Stage57의 `check-absorption.mjs`와 기존 `absorption.json`을
읽고, 실제 `whole_fit.rs:160`의 위치 일치 판정→override 이후 결과→`flow.rs:141`→
`split_entry.rs:199` 소비를 대조했다. 원격 변경 4개 hunk(#6761 helper/whole-fit/split,
#6656 예약 높이 설명)의 누락은 발견하지 않았다. 이미 통과한 집중 403건과 최신 CI를
증거로 재사용하며 해당 검사나 빌드를 재실행하지 않았다.

기여자 관리 관점에서도 문단 순회는 section, 읽기 판정은 도메인 Query, 확정 상태 변경은
state Command로 구분되고 구조 정본에 변경 위치·소비 경로·관련 테스트가 연결되어 있다.
넓은 불변 `StateView`, `TypesetEngine`의 내부 가변 profile, layout/document_core와의 기존
의존은 남아 있다. 완전한 CQRS 격리나 전체 엔진 재설계가 끝났다는 주장은 하지 않으며,
이번 승인 범위인 기존 동작을 보존한 책임 분리의 제품 차단 사유로는 판단하지 않는다.

R3–R5 단계별 이동 증명과 최신 통합 증거를 함께 검토했다. 이번 별도 코드 검토는 위 핵심
분기와 호출 연결 중심이며 모든 기존 규칙의 사양 적합성 또는 성능 개선을 입증하지 않는다.

### 시각 증거 재검토

아래 재실행은 중복 작업 지적 **이전**의 이력이다. 지적 이후에는 추가 실행하지 않았다.
기존 `compare-native.mjs`, `compare-visual.mjs native`, `compare-visual.mjs wasm`을 재실행했다.
Native 14문서 772쪽의 pagination/render tree/SVG, WASM 11문서 748쪽의 render tree/SVG,
선택 23쪽 PNG의 전후 동일 및 Native/WASM 동일을 다시 확인했다. 결과는
`output/7280/stage60-submit/self-review-*-comparison.json`에 있다.

대표 review/standalone overlay 4개를 직접 열었다. 제품 source와 일치하는 기존 fresh Docker
WASM(`e28071e8…`)을 사용해 chemical 14쪽 Chrome Visual Sweep도 새 output에서 재실행했다:

```bash
VISUAL_SWEEP_CHROME=/home/edward/.cache/puppeteer/chrome/linux-146.0.7680.31/chrome-linux64/chrome \
venv/bin/python scripts/visual_sweep.py --key chemical-review \
  --hwp samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp \
  --pdf pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf \
  --pages 14 --dpi 96 \
  --rhwp-bin output/7280/stage58-integration/bin/head \
  --wasm-pkg output/7280/stage58-integration/pkg/head \
  --out output/7280/stage60-submit/self-review-wasm
```

완료 1/1쪽, 자동 후보 0, pixel match 93.79743%, proxy 21.71057%.
104쪽 SVG/render tree export 완료, 14쪽 raster PNG는 이전 fresh WASM 결과와 바이트 동일하다.
새 review 직접 판독에서도 표 외곽·본문의 기존 PDF 대비 세로 위치 차이를 확인했다.
이는 구조 리팩토링의 신규 회귀가 아니라 보존된 차이이며 한컴 일치 통과를 뜻하지 않는다.

### 보완 사항

제품 코드에서 이번 변경으로 도입된 결함은 검토 범위에서 발견하지 않았다.
다만 `wasm-chemical-rewind-p014-review.png`와 standalone overlay 상단의 긴 진단 라벨이
우측에서 잘리는 증적 문제가 재현된다. 문서 본문 잘림과는 다르다.
`visual_fixture_evidence.md`의 게시 증적 요건에 따라 별도 도구 이슈로 추적한다.
초기 self-review에서는 원격 이슈/댓글·approve·merge를 실행하지 않았다.
최종 기록 정리에서는 제품·테스트·기준값을 수정하지 않았고, 빌드/회귀/시각 검증도 추가 실행하지 않았다.
초기 정리는 review와 오늘할일의 로컬 갱신까지만 수행했다.

### 승인된 후속 처리

작업지시자의 “후속 절차를 진행하세요” 승인으로 라벨 문제를
[#7349](https://github.com/edwardkim/rhwp/issues/7349)에 등록했다. 동일 증상 검색 뒤 #6016의
해결된 한글 tofu 문제와 구별했으며 원래 PNG와 metric·본문을 변경하지 않았다.
이 항목은 도구의 잔여 문제이지 #7280 제품 결함이나 검증 실패가 아니다.

검토 기록·오늘할일·asset 안내만 single-parent review-only 후속 commit으로 같은 PR에 반영한다.
제품 코드·테스트·기준값 변경, devel 재병합, 빌드·회귀 재실행은 하지 않는다.
녹색 candidate는 `c08b4fc3…`와 위 CI run이며 후속 head의 실제 preflight/aggregate 결과는
push 후 확인한다. 기존 head 성공을 새 head 성공으로 미리 기록하지 않는다.

## 최종 판정

**승인** — #7280 구조 리팩토링의 self-review 범위에서 새 제품 결함이나 devel 반영 누락을
발견하지 않았다. 검증된 제품과 동일하며 CI 성공·기존 집중/전체 검증·시각 증거를 재사용했다.
증적 라벨 한계는 #7349로 분리해 공개했다. 외부 독립 승인이나 한컴 피델리티 전체 승인은 아니다.

남은 merge 조건은 review 기록 반영 후 최신 head의 required check 확인과 작업지시자의
merge 승인이다. 제품 수정이 추가되면 해당 head를 다시 검증한다.
이 기록은 GitHub approve·admin bypass·merge·이슈 close를 수행하지 않는다.
