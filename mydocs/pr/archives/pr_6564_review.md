---
kind: pr_review
status: completed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-01
---

# PR #6564 검토 — 셀 선택 전 표 경계 hover·resize 복원

## 최종 판정

**메인터너 보정 후 수용 가능.** contributor 원 head
`0987770ef0174f0a4e4a0dbb39281707affd9e18`은 #4117의 사용자 여정을 복원하고 60회
mousemove에서 엔진 호출을 1회로 제한했지만, 페이지별 실패 memory와 다른 입력 경로의
성공 cache가 일관되지 않았다. 분할 표에서는 hint가 다른 페이지로 이동할 때 bbox 배열을
매번 선형 검색하는 비용도 남았다.

메인터너 보정 commit `b4df21457419650e97f4fa6d27cbcac5681d40fd`은 이 세 결함만
고쳤다. 최신 `devel` 통합, 필수 Rust lint 묶음, Docker WASM build, Studio 전체 unit,
TypeScript, #4117 headless Chrome 왕복과 사람이 직접 연 전·후 screenshot이 통과했다.

보정이 포함된 source head `257d81c3ec6cb3762463e946c04d5a98a2213a12`의 Full CI와
보조 workflow가 모두 통과했다. review-only 최종 head
`4a2be5541c6ef3c82c41304416ade830500f03ae`의 Fast Pass와 메인터너 self-review를 확인한
뒤, 작업지시자 승인으로 정상 2-parent merge commit
`07e1dd7ef6e51bb063b4b4bf10e5694d8eec94c5`를 `devel`에 반영했다.

## 라우팅

- 기본 경로: `maintainer_general.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`,
  `visual_fixture_evidence.md`, `multi_pr_update_branch.md`, `post_merge.md`
- 작성자는 기존 기여자이므로 `first_time_contributor.md`는 적용하지 않았다.
- 같은 작성자의 #6562를 먼저 정상 merge하고 후속 기록까지 반영한 뒤, 최신
  `devel`을 #6564에 통합했다.

## 메타데이터와 검토 대상

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6564](https://github.com/edwardkim/rhwp/pull/6564) / @jeong-sik |
| 관련 이슈 | [#4117](https://github.com/edwardkim/rhwp/issues/4117) |
| base / draft | `devel` / 아님 |
| contributor commits | `a9bc097d7782a012dab05d1194a4d3dd587a564a`, `0987770ef0174f0a4e4a0dbb39281707affd9e18` |
| 검토 기준 `devel` | `0d1540931d59a8712c27f339fcbb71e1c00fd4b1` |
| current-base merge | `c919bc209e8ae22f9918fde1a0af204e18b8d6c0` |
| 메인터너 보정 code candidate | `b4df21457419650e97f4fa6d27cbcac5681d40fd` |
| 검증 완료 원격 head | `257d81c3ec6cb3762463e946c04d5a98a2213a12` |
| review-only 최종 head | `4a2be5541c6ef3c82c41304416ade830500f03ae` |
| self-review | [COMMENTED review #5077158890](https://github.com/edwardkim/rhwp/pull/6564#pullrequestreview-5077158890) |
| merge commit | `07e1dd7ef6e51bb063b4b4bf10e5694d8eec94c5` |
| 원 PR 규모 | 10 files, `+452/-66` |
| 원격 최종 상태 | head `4a2be5541`, `MERGED`; #4117 자동 종료, 2026-09-01 확인 |
| reviewer | `edwardkim` |

원 head의 GitHub CI·CodeQL·Proptest·Render Diff·Adapter 성공은 보정 candidate의
녹색 CI로 재사용하지 않았다. 보정과 review 증적을 포함한 원격 head `257d81c3e`에서 Full
CI run `33497972187`, CodeQL run `33497972198`, Proptest run `33497972307`, Adapter run
`33497972262`, Render Diff run `33497971898`을 새로 확인했다. 최종 check 집계는 성공 28건,
정책상 neutral 1건과 skip 5건, 실패 0건, 대기 0건이다.

로컬 검증 기준 `devel`은 위 표의 `0d1540931d`다. CI 종료 뒤 원격 `devel`이
`b9d408f0d698de84d4a0c5f1bf4cc12e35ef2f16`까지 전진했으나, 2026-09-01 재조회에서
현재 PR head는 계속 `MERGEABLE/CLEAN`이었다. 이 값은 merge 직전 다시 조회한다.

review-only 최종 head의 Fast Pass는 성공 11건, 정책상 skip 20건, 실패·대기 0건으로
종료됐다. merge 직전에도 head SHA, required checks, `MERGEABLE/CLEAN`과 최신 `devel`의
무충돌 merge simulation을 다시 확인했다.

## 원 변경과 current-base 통합

원 변경은 다음을 수행한다.

- 셀 선택 클릭이 없어도 hover가 `ensureTableCellBboxCache`를 통해 표 bbox cache를
  최초 1회 채운다.
- 현재 페이지를 `getTableCellBboxes` hint로 전달한다.
- Rust core가 cached page tree를 clone하지 않고 참조로 읽도록 바꾼다.
- mousemove storm의 엔진 호출 budget과 실제 hover→mousedown→drag 여정을 E2E로 고정한다.

최신 `devel` 병합에서 충돌은 `rhwp-studio/package.json` 한 파일이었다. #4117 E2E script와
`devel`의 #6557 merged-column·merged-row E2E script를 모두 보존해 해결했다. contributor의
두 commit은 amend·rebase·force-push하지 않았다.

## 발견한 결함과 메인터너 보정

### R1 — 단일 실패 record의 페이지 덮어쓰기

원 구현은 마지막 `tableBboxFetchFailure` 한 건만 저장했다. page 0 실패 → page 1 실패 →
page 0 재진입 시 호출열이 `[0, 1, 0]`이 되어 “문서 변경 전 `(표, 페이지)`당 1회”
계약을 어겼다. 실패 memory를 `(sec, ppi, ci, pageIdx)` key의 `Set<string>`으로 바꿨다.

### R2 — 직접 성공 뒤 남는 stale failure

hover 실패 뒤 셀 선택 mousedown의 직접 조회가 성공해도 실패 record는 남았다. 이후 bbox
cache가 비워지면 같은 페이지 재조회가 영구 차단됐다. 모든 성공 경로가
`cacheTableCellBboxes`를 사용해 bbox, page membership, 대응 실패 해제를 한 번에 수행하도록
통합했다. 빈 결과는 성공으로 취급하지 않는다.

### R3 — 분할 표 mousemove의 O(cells) membership 검색

원 구현은 hint가 다른 페이지에서 `cachedCellBboxes.some(...)`을 매번 실행했다. 성공 시
페이지 `Set`을 한 번 만들고 hover는 `Set.has(pageIdx)`로 판정하도록 바꿨다. 문서 snapshot
변경 시 bbox cache와 실패 `Set`을 함께 비워 새 layout을 다시 조회한다.

## 로컬 검증 결과

### 회귀와 Studio

| 검증 | 결과 |
| --- | --- |
| 보정 전 R1·R2·R3 회귀 | 예상 실패 3건 재현 |
| focused cache·pageHint·mouse tests | 29/29 통과 |
| `npx tsc -p tsconfig.ci-unit.json --noEmit` | 통과 |
| `npm test` | 1,353 pass / 1 skip / 0 fail |
| `npm run e2e:manifest-check` | tracked 125 / manifest 125, 이상 없음 |

fresh review worktree에서 `npm ci` 전 전체 test를 먼저 실행했을 때 18건이 TypeScript 실행기와
`@noble/hashes`를 찾지 못해 실패했다. 이는 source 실패가 아니라 `node_modules`가 없는 환경
조건이었다. lockfile대로 `npm ci`한 뒤 전체 suite가 위 결과로 통과했다. 설치 과정은 기존
dependency에서 vulnerabilities 3건(낮음 1, 높음 2)을 보고했으며, 이 PR은 dependency나
lockfile을 바꾸지 않는다.

### Rust 필수 lint 묶음과 WASM

아래 명령을 순차 실행해 모두 통과했다.

```text
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
```

manifest 검사는 1,106 sources, 4,771 static test attrs, 28 suites와 20 exceptions,
48/48 integration targets를 확인했다. 파생 suite·manifest 변경은 남지 않았다.

표준 Docker `wasm` service로 candidate를 fresh release build했고 7분 3초에 `pkg` 생성을
완료했다. review worktree에 없는 로컬 `.env.docker`는 기준 checkout의 기존 파일을 임시
symbolic link로 참조했으며 build 직후 link를 제거했다. env 내용은 기록하거나 변경하지 않았다.

## 브라우저 왕복과 시각 판정

`npm run e2e:issue-4117-border-hover`를 headless Chrome에서 실행해 모두 통과했다.

- 시작 시 bbox cache가 비어 있어 셀 선택 선행 조건이 없음을 확인했다.
- 표 경계 hover에서 `col-resize` cursor와 파란 경계 marker가 표시됐다.
- mousemove 60회 뒤에도 cursor를 유지했고 엔진 호출은 1회였다(허용 상한 2회).
- mousedown으로 resize drag가 시작됐고 첫 열 너비가 39.8px 증가했다.
- 첫 열의 세 행은 같은 수직 경계로 이동해 행별 폭이 어긋나지 않았다.

이 검증은 synthetic 1쪽 문서의 interaction 전·후 상태를 대조한 것이며 HWP/HWPX와 한컴
oracle PDF를 비교하는 visual sweep은 아니다. 따라서 flagged page, `pixel_match`,
`visual_accuracy_proxy_percent`는 적용 대상이 아니다. 사람이 두 PNG를 직접 열어 hover
marker, drag 전·후 열 경계, 세 행 정렬, 도구 UI의 깨짐 여부를 확인했다.

| 역할 | 안정 경로 | SHA-256 |
| --- | --- | --- |
| hover marker | `mydocs/pr/assets/pr_6564_table_border_hover_marker.png` | `dda8ce88e910df31d69d963646258274f7a82913c1d45ca34b57d20d86b17561` |
| drag 후 geometry | `mydocs/pr/assets/pr_6564_table_border_after_drag.png` | `10463efb3ecb6f179bca04c43dde6e5e0933d520a0e79916089ec94367001323` |

임시 HTML 보고서는 `output/e2e/table-border-hover-resize-issue4117-report.html`에 생성됐고
SHA-256은 `c585c42add86b8903c6b663b06bd3576e8ad93b0bc017134b43f3b36ccb7faba`다.
output은 source 제출 대상이 아니며 위 두 PNG만 장기 review 증적으로 보존한다.

## 잔여 위험과 완료 상태

- 실패 `Set`은 문서 snapshot 변경 시 비워지며 그 전에는 실제로 hover한 실패 page key만
  보관한다. 문서 전체 표·페이지를 사전 열거하지는 않지만, 극단적으로 많은 표를 모두 hover한
  세션의 memory 상한을 별도 계측하지는 않았다.
- E2E는 3×3 synthetic 표의 happy path와 이동 budget을 증명한다. 원 이슈가 수치를 확보하지
  못한 중첩 표와 장시간 multi-page stress는 이번 PR의 merge blocker로 확대하지 않는다.
- current-base merge, 보정 commit, review 증적은 source branch에 반영됐고 정확한 원격 head의
  Full CI와 Fast Pass를 확인했다.
- 메인터너 self-review에서 blocking finding이 없음을 확인하고 별도 merge 승인을 받은 뒤 정상
  merge commit 방식으로 병합했다. #4117은 closing keyword에 의해 자동 종료됐다.

## Merge 후 contributor PR comment 계획

- 방법 링크는 [Visual Sweep GitHub merge comment 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 사용하되,
  이번 증적이 pixel visual sweep이 아니라 synthetic 1쪽 interaction E2E임을 명시한다.
- 실제 수치는 mousemove 60회, engine 1회, 첫 열 `+39.8px`, 세 행 동일 이동으로 고정하고
  적용 불가한 pixel metric을 추정하지 않는다.
- merge commit에 두 asset이 존재한 뒤 다음 SHA 고정 raw URL 형식으로 표시한다.
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6564_table_border_hover_marker.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6564_table_border_after_drag.png`
- source merge SHA와 archive 기록 commit을 확정한 뒤에만 UTF-8 without BOM body file로
  contributor PR comment를 게시한다. 게시 후 API로 한글, 선두 BOM, `??` 치환, Markdown
  image URL을 재조회한다.

## 원격 조치 상태

승인에 따라 current-base merge, 메인터너 보정과 review 증적을 contributor source branch에
fast-forward push했다. 메인터너 self-review는 `COMMENTED`로 남겼고 PR #6564는 정상 merge
commit으로 병합됐다. #4117은 자동 종료됐으며, maintainer issue comment와 merge 결과 contributor
comment는 후속 게시 대상으로 남긴다.
