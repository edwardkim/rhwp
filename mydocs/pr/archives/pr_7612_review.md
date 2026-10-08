# PR #7612 리뷰 — 떠 있는 개체·자동 번호 뒤 문단 분할의 캐럿 축

## 최종 판정

**승인 — 작업지시자가 승인한 #7612 시각 증적 예외 적용.** 이 PR은 캐럿 자리에서 문단을 나누고 클립보드 범위를 자르는 편집 계약 수정이다. 글자 점유와 개체 이동을 분리한 구현, 현재 head의 신규 10개·기존 3개 focused 검사 통과, 수정 제거 시 9개 실패, source Full CI 및 최신 head CI 재사용 근거를 수용한다. 2026-10-09 작업지시자 지시에 따라 일반 한컴 PDF/Native·fresh WASM 시각 증적 조건을 이 PR의 수용 요건에서 제외한다.

작업지시자의 “merge 를 진행하세요” 승인 후 정확한 head·완료한 CI·최신 devel merge simulation을 재확인하여 병합했다. 병합 커밋은 `83131ecd186bd7483671b7886d659ecb4fd380ca`이며, 실제 devel 포함과 기본 작업공간 fast-forward를 확인했다. 시각 증적 예외는 유지하고 미실행 검사를 통과로 보고하지 않는다.

## 이번 PR의 시각 증적 예외

2026-10-09 작업지시자 지시: “이 PR 의 경우 편집에 대한 것이기 때문에 일반적으로 요구되는 시각적 증적 조건의 예외로 진행해야 합니다.”

이 명시 지시가 일반 시각 gate보다 우선한다. 적용 범위는 #7612의 캐럿 축·문단 분할·클립보드 편집 계약이며, 검토 head는 `34ceeb0d067549ee0a8689b3b24231bfb1f23428`이다. 동일 편집 저장본의 한컴 Print PDF, Native/fresh WASM Sweep, 90% 일치율·TSV·대표 review/overlay PNG를 추가 제출·수용 조건으로 요구하지 않는다. 프로젝트 전체 정책을 변경하거나 다른 PR에 예외를 일반화하지 않는다.

기존의 시각 증적 부족에 따른 보류는 이 지시로 해제한다. 실제 시각 비교와 WASM/Studio E2E는 미실행 상태로 유지하며, 한컴 출력 일치 또는 브라우저 실행을 확인했다고 보고하지 않는다. 코드의 문단 분할·소속/편집 구조 검증과 시각 출력 검증을 구분한다. 예외 지시와 적용 범위는 ignored `output/pr-review/pr7612-20261009/visual-exception-authorization.json`에도 기록했다.

## 접수 정보와 경로

| 항목 | 값 |
| --- | --- |
| PR | [#7612](https://github.com/edwardkim/rhwp/pull/7612), semanticist21, MERGED, base devel |
| 원·검토 head | `34ceeb0d067549ee0a8689b3b24231bfb1f23428` |
| 수정 code candidate | `baca828e5c0502f57d0ecaf3376f9c9a20b20de8` |
| tests-only 선행 commit | `bfe6ba974c60f7cdd1f478ce7c825e357a760099` |
| head에 병합된 devel / API base snapshot | `b3c3047db575d146dff9d39098e6de98c4630b3e` |
| 실제 최신 devel | `5168e3256ad58eda425d84fdd4d67e9b47bb74b1` — fetch, ls-remote, GitHub ref 재조회 일치 |
| 변경 | 3 commits, 3 files, +693 / −6; 생산 코드 2개, 신규 검사 1개 |
| reviewer | 기존 요청 `jangster77` 유지; 추가 원격 assign 없음 |
| 관련 이슈 | PR 본문에 연결·종료 이슈 없음. #7444는 기존 캐럿·복사 계약의 대조군이며 종료 대상으로 해석하지 않음 |
| 작업공간 | `/tmp/rhwp-pr7612-review-20261009`, `review/pr7612-20261009` |

라우팅: 기본 `maintainer_general`; 보조 `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `rework_and_exceptions`. 모 workflow·선택표·해당 자식·review template·문서/Git workflow·개발 환경·시각 거버넌스를 확인했다. fetch 전 기본 경로를 보고하고 ignored output의 `routing.json`에 기록했다. 최신 devel과 head는 61/3개 독자 commit이 있으나 PR 고유 diff는 위 세 파일이며, 억지 merge/rebase나 체리픽을 하지 않았다.

## 변경과 원인 검토

기존 `split_logical_control_positions`와 `split_text_pos_for_logical_offset`은 `is_split_movable_control`인 떠 있는 도형·그림·표·자동 번호도 각각 한 칸으로 셌다. 탐색 길이·캐럿 계약은 TAC 개체와 각주·미주, 글자겹침을 한 칸으로 세고, 떠 있는 개체는 칸을 갖지 않는다. 자동 번호는 텍스트의 자리표가 이미 한 칸이다. 따라서 기존 분할은 캐럿 이전의 해당 컨트롤 수만큼 글자를 덜 남겼다.

`Paragraph::occupies_split_slot`은 기존 `Control::is_logical_inline`과 `CharOverlap`의 합으로 점유를 정한다. 이동 여부는 기존 `is_split_movable_control`로 유지한다. `clipboard::text_to_split_logical_offset`도 같은 점유 helper를 사용한다. 특정 문서 ID·높이·좌표·화면 점수를 위한 예외와 clamp, baseline 변경은 없다.

### 실제 생산·소비 경로

| 경로 | 생산 → 소비 → 최종 결과 |
| --- | --- |
| 공통 모델 | `src/model/paragraph.rs:670` 점유 분류 → `split_logical_control_positions` → `split_text_pos_for_logical_offset` → `split_at:1490`의 텍스트 컷·UTF-16/서식 분할과 `1723`의 개체 소유 결정 |
| 복사 | `clipboard.rs:177` 글자/개체 뒤 경계를 같은 점유 축으로 변환 → `clip_paragraph_caret_range_for_clipboard:217`의 앞뒤 컷. 이후 `keep_control`·field remap으로 선택에 포함되는 개체를 별도 결정함도 확인 |
| 본문 Enter·쪽/단 나누기 | `text_editing.rs:3729`, `3844`, `4038`의 `split_at` → 문단 삽입·양쪽 reflow → vpos 재계산·recompose·pagination. helper 이후 글자/개체 소속은 실제 출력 구성에 전달됨 |
| 셀 | `text_editing.rs:4562`의 `split_at` → 셀/글상자 문단 삽입 → 셀 reflow·재구성 |
| 머리말 | `header_footer_ops.rs`의 split → `reflow_hf_paragraph` → 재구성·pagination |
| 각주·미주 | `footnote_ops.rs:617/626`의 split → 양쪽 `reflow_footnote_paragraph` → 재구성·pagination |
| 여러 문단 붙여넣기 | `clipboard.rs:811/908/1228`의 split → clip 문단 삽입·오른쪽 절반 병합 → 영향 문단 reflow |
| WASM·Studio | `wasm_api.rs:2461/1667/1912/4931`와 Studio `core/wasm-bridge.ts`가 같은 Native 메서드를 호출. 소스 배선 확인이며 실제 WASM/브라우저 실행 증거는 아님 |

글자겹침의 한 칸은 기존 `document_core/helpers.rs:104`의 탐색 길이, `queries/cursor_rect.rs:27` 및 layout의 겹침 advance와 대조했다. 줄 메트릭·paint 알고리즘 자체는 바꾸지 않으나 **편집 후 조판 입력과 소속이 달라지므로** Visual Sweep의 영향 경로는 인정하되, 이번 PR에는 위 작업지시자의 수용 조건 예외를 적용한다. 표 행/rowspan의 물리 분할 알고리즘 변경은 없으므로 그 전수 경계를 이 PR의 추가 요건으로 요구하지 않는다.

## 실행 검증과 CI

공용 target은 `/home/edward/mygithub/rhwp/target/pr-review` 한 곳이며 Cargo 실행 전 활성 작업을 확인하고 순차 실행했다. ignored 증적 루트는 기본 저장소의 `output/pr-review/pr7612-20261009/`다.

| 검사 | 실제 결과·증적 |
| --- | --- |
| `git diff --check`, `cargo fmt --all -- --check` | diff/포맷 PASS; 생산 source/test 보정 없음 |
| current-base `git merge-tree --write-tree upstream/devel upstream/pr7612-head` | exit 0, tree `2b88f56b830c8302d411c63ccf142be41387e7be`; `merge-tree.txt`. 텍스트 통합 검증이며 최신 통합 tree의 전체 실행 성공으로 승격하지 않음 |
| `node scripts/rust-test-suite-manifest.mjs --prepare` | review worktree에서만 준비; `logs/manifest-prepare.log` |
| manifest `--check --base-ref b3c3047…` 및 `--base-ref 5168e325…` | 모두 PASS; 1,487 sources, 48 integration targets; 각 로그 보존 |
| `node scripts/run-rust-test.mjs split_caret_axis -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --no-fail-fast` | **10 PASS / 0 FAIL**, final head 재컴파일 2m17s; `logs/focused-split-caret-axis.log` |
| 같은 runner의 `issue_7444_caret_axis_after_inline_control` | **3 PASS / 0 FAIL**; 각주 직후 입력·복사와 문단 끝 떠 있는 도형 복사 대조; `logs/focused-7444.log` |
| 최종 head 재컴파일·13개 재확인 | **13 PASS / 0 FAIL**, exit 0; `logs/final-focused-13.log`, `final-focused-result.json`. 음성 대조 뒤 원 head source 바이트 확인·재컴파일 후 두 suite의 같은 13개 확인 |
| 수정 제거 음성 대조 | 현재 head의 별도 소스 사본에서 생산 수정만 반전해 재컴파일: **9 FAIL / 1 PASS**(글자겹침 대조군), exit 100. `logs/negative-split-caret-axis.log`, `negative-control/result.json` |
| CI code candidate | [Full CI #37496782588](https://github.com/edwardkim/rhwp/actions/runs/37496782588): head `baca828e5…`, success. Lint·Native Skia·Archive A/B/C/D 실제 success 확인 |
| 최신 PR head CI | [CI #37637816284](https://github.com/edwardkim/rhwp/actions/runs/37637816284): head `34ceeb0d…`, success. preflight는 `direct-source-build-and-test-green:success`, source parent `baca828e5…`, `current-base-merge-tree-match`를 확인하고 heavy lane 재사용. 단순 미실행 CI를 Full 실행으로 보고하지 않음 |
| 전체 로컬 회귀·lint | 이 접수에서는 반복 실행하지 않음. 위 code candidate의 GitHub Full CI와 merge-tree 재사용 경위를 보존했으며, maintainer source/test 보정 commit은 없음. 최신 devel을 포함한 새로운 통합 code head 검증 완료라는 뜻은 아님 |
| Docker WASM·Studio E2E·Visual Sweep | **미실행/미검증**, 일반 시각 증적 수용 조건은 작업지시자 예외 적용. WASM/Studio 실행 통과 또는 한컴 출력 일치로 보고하지 않음 |

nextest 설치 버전 `0.9.137`에서 권장 `0.9.140` 경고와 비기능적 JUnit skipped-report key 경고가 있었다. 실제 검사 exit 0·성공 수를 확인했으며 전체 회귀 수로 외삽하지 않았다. 필터 밖 185/225 skipped는 미실행 검사다.

음성 대조는 기본 작업트리·review head를 수정하지 않고 ignored 별도 source 사본에서 수행했다. 첫 두 시도는 manifest symlink가 원 package를 선택하거나 Git patch가 하위 경로에서 적용되지 않은 준비 오류로 무효 처리했다. Cargo manifest 실제 경로·수정 제거 후 source를 확인하고 `patch -p1 -R`로 정확히 적용한 세 번째 실행에서 해당 사본의 재컴파일과 9개 실패를 확인했다. 준비 오류의 10 PASS는 음성 대조 근거로 사용하지 않는다. 음성 대조 후 첫 최종 head 실행은 공용 산출물의 음성 대조 바이너리를 재사용한 것을 확인해 무효 처리했다. 원 head의 두 생산 파일 바이트가 Git blob과 일치함을 확인한 뒤, 이 review worktree 파일의 mtime만 갱신해 정상 head를 명시적으로 재컴파일하고 최종 13개가 모두 통과함을 다시 확인했다. 환경/캐시 혼입을 제품 회귀로 분류하지 않는다. 원래 검사·shared target cache·다른 worktree는 삭제하지 않았다.

신규 검사는 실제 공개 편집 API, 메모리 내 HWP/HWPX 저장·재열기, 저장 HWPX XML 재배치를 사용한다. 정상 한컴 저장본과 수동 합성 입력을 구분한다. `check_split`은 앞 문단의 글자·개체 위치와 뒤 문단의 글자·개체 **수**를 검사한다. 뒤 문단 모든 개체의 ID/상대 위치와 최종 페이지 소속까지 검사했다고 확대하지 않는다. `split_after_anchors_survives_save_and_reopen`은 특정 컷에서 기존 개체 모두가 앞 문단에 남는 재저장 경계를 확인한다.

## 검증 입력 커밋 확인

실제 focused 검사가 읽은 아래 파일은 `34ceeb0d…` Git blob과 실행 파일의 바이트가 모두 일치했다. `input-provenance.json`에 경로·해시·크기·commit을 기록했다. 네 개의 서로 다른 기존 문서를 사용하며, 교육 통합 문서는 두 문단을 검사한다.

| 기존 경로 | SHA-256 |
| --- | --- |
| `samples/143E433F503322BD33.hwp` | `1bcdc7a2de38680cdb715556f40c8477f8242131d28de949b09e945022bf35fa` |
| `samples/3-09월_교육_통합_2023.hwp` | `47a503ea0e92a63ee58b552e661fbde27f8a611afffd67e822be5928319e3c87` |
| `samples/eq-002.hwp` | `e3306c23cf1d991debb3fdf113e5f27c5a28e614e6c95c246afb18ce8a549ea9` |
| `samples/issue1880_takeplace_oracle_p13.hwpx` | `0725f1b5c3424ad094b1241d0b49c7f40ceb04fcfb732da80b12cf67d6b4fdaf` |
| `assets/logo/logo-16.png` | `ea7d417dc9de2f04a3074a63690c99363d8738b9d614e96ccb289ea7b6573390` |

기존 입력의 커밋 확인은 **충족**이다. 코드에서 생성·소비하는 메모리 합성 입력에는 별도 파일 의무를 부여하지 않았다. 실제 **편집 저장본·대응 PDF 대조는 미검증**이며 이번 PR의 시각 수용 조건 예외 대상이다. 기존 원본의 `pdf/143E433F503322BD33-hwp-2020.pdf`(한컴 2020, 1쪽), `pdf/3-09월_교육_통합_2023.pdf`(한컴 2024, 20쪽)의 존재/metadata만 확인했다. 동일하게 편집한 문서의 기준 PDF로 대신하지 않았다.

## 조판 원칙과 증거 판정

| 검토 항목 | 판정·근거 |
| --- | --- |
| 구현 근거와 일반성 | **충족** — 점유 규칙과 이동 규칙 분리; 기존 캐럿 계약 대조, 문서 전용 분기 없음 |
| 측정·배치 일관성 | **미검증 / 시각 수용 조건 예외** — 분할 결과의 reflow·출력 전달은 추적했으나 실제 Native/fresh WASM 배치·한컴 Print 출력 비교 없음 |
| 분할·이어받기 계약 | 문단 소유/텍스트 경계는 focused **충족**. 최종 출력 소속·누락/중복은 **미검증 / 시각 수용 조건 예외**. 표 행의 물리 컷·rowspan 이어받기 알고리즘 변경은 **비해당** |
| 줄 소속·점유 높이 | 줄 계산식 직접 변경은 **비해당**. 편집 후 달라진 문단/개체 소속을 소비하는 실제 줄·쪽 배치는 **미검증 / 시각 수용 조건 예외** |
| 사례·증거 독립성 | 기존 캐럿/탐색 계약과 원본 입력은 확인. XML 합성 입력을 정상 한컴 출력 근거로 승격하지 않음; 대응 Print 기준·시각 비교 **미검증 / 시각 수용 조건 예외** |
| 기준값 변경 | **비해당** — baseline/golden/래칫 변경 없음 |
| 주장·검증 범위 | Native 구조 검사 13개와 source CI는 확인. 실제 WASM/Studio·편집 출력의 Native/fresh WASM 시각 증적은 **미검증 / 시각 수용 조건 예외** |

## 수용·원격 조치 계획

현재 head는 **승인**이다. 기존 보류 사유인 일반 시각 증적 부족은 명시 예외로 해제했고, 코드·Native 편집 계약 검증은 충족한다. 기여자에게 이전의 PDF·시각 gate 보완 요청 초안을 게시하지 않는다.

예외 적용 후 GitHub 조회에서 head `34ceeb0d…`, Open/Ready, `mergeable=true`, `clean`, 최신 `Build & Test=success`, 실패·진행 중 check 없음이 확인됐다. 최신 devel `5168e325…`와 merge simulation은 clean이며 tree `2b88f56b830c8302d411c63ccf142be41387e7be`이다. 재조회 증적은 `exception-pr-snapshot.json`, `exception-checks-snapshot.json`, `exception-merge-tree.txt`에 보존했다. 문서·판정만 변경했으므로 완료한 같은 source head의 Rust 검사를 반복하지 않았다.

한국어 존댓말 댓글 초안 `output/pr-review/pr7612-20261009/review-comment-ko.md`은 편집 계약 수용·시각 증적 예외를 설명하는 내용으로 갱신했다. 원격 댓글/GitHub review와 merge는 해당 승인 범위에 따라 로컬 `gh`로 수행하고 REST API로 재확인한다. 병합 단계에서 `post_merge.md`를 추가로 로딩했다.

## 병합 및 후속 운영 기록

- 사용자 승인: 2026-10-09 “merge 를 진행하세요”. 검토 기본 경로에 `post_merge.md`를 추가로 로딩했다.
- 실행: `gh pr merge 7612 --repo edwardkim/rhwp --merge --admin --match-head-commit 34ceeb0d067549ee0a8689b3b24231bfb1f23428`. exact-head guard를 사용했다.
- GitHub API: `merged=true`, CLOSED, merge SHA `83131ecd186bd7483671b7886d659ecb4fd380ca`, `merged_at=2026-10-08T21:00:28Z`(한국시간 2026-10-09 06:00:28).
- source merge 뒤 `git fetch upstream devel`, `git merge --ff-only upstream/devel`로 기본 작업공간 devel을 동기화했고, merge SHA의 upstream/devel 조상 포함을 확인했다.
- `closingIssuesReferences=[]`이다. PR 본문에 관련 종료 이슈가 없고 #7444는 기존 회귀 대조군이므로 별도 이슈를 닫지 않는다.
- 후속 문서 경로: **maintainer 직접 반영**. 이 archive review와 `mydocs/orders/20261009.md`만 운영 기록 commit으로 반영한다. source/test/workflow/golden/sample을 추가로 변경하지 않는다.
- 병합 직후 자동 실행은 duration metadata refresh 및 issue metadata close workflow이며, CI·CodeQL·Adapter·Proptest·Oracle 검증은 재실행하지 않았다.
- contributor 안내 계획: 아래 문안의 병합 사실, 편집 계약 검증 13 PASS, 음성 대조 9 FAIL/1 PASS, 원 candidate Full CI/최신 head fast-pass, 시각 증적 예외, 실제 시각/WASM·Studio 미실행을 한국어 존댓말로 전달한다. 실제 시각 자료가 없으므로 이미지 일치율이나 overlay를 만들어 수용 근거로 게시하지 않는다. 게시 시 `gh --body-file` 및 REST UTF-8 본문 재조회를 사용한다.
- 종료 정리 대상은 이번 검토의 `/tmp/rhwp-pr7612-review-20261009`, `review/pr7612-20261009`, 로컬 fetch ref `upstream/pr7612-head`, ignored `output/pr-review/pr7612-20261009/`다. 영구 요약을 이 문서와 오늘할일에 보존한 뒤 clean/미사용/소유를 확인해 정리한다. contributor fork `semanticist21/rhwp`의 `fix/split-caret-axis`와 다른 worktree·공용 `target/pr-review`는 삭제하지 않는다.
- 위 로그·JSON·임시 source 경로는 실행 시점의 진단 위치다. 검토 종료 후 임시 output을 정리하며, 실제 결과·SHA·명령·수용 범위·예외·CI URL은 이 archive 문서에 남긴다.

### 병합 후 자동 실행 확인

[Refresh nextest target duration data #37843669946](https://github.com/edwardkim/rhwp/actions/runs/37843669946)는 success다. 수집 결과는 `ready:false`, `reason:no-verified-pr-duration-measurements`로 실측 정책 갱신 보류다. 이는 검증 CI 실패가 아니며 CI를 추가 실행하지 않는다. [Close Issues on devel Push #37843669943](https://github.com/edwardkim/rhwp/actions/runs/37843669943)도 success다. source merge SHA의 run 목록에서 CI·CodeQL·Adapter·Proptest·Oracle의 병합 후 재실행은 없었다.
