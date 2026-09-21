# PR #7309 리뷰 — Dependabot 의존성 8건 통합

## 접수 정보

2026-09-21에 확인한 code candidate의 참고값이다. 이 검토 기록을 포함한 최신 head의
상태와 CI는 merge 직전에 다시 확인한다.

| 항목 | 값 |
| --- | --- |
| PR | [#7309](https://github.com/edwardkim/rhwp/pull/7309) |
| 작성자·경로 | `jangster77`, collaborator self-review 통합 PR |
| 관련 이슈 | 없음; Dependabot 자동 의존성 갱신 8건 |
| base | `devel` / `a4b8731ca0e989105e17b5dbca7b6b679d7ac50c` |
| 검토한 code candidate | `6f709298ca6f1e23df5e9a8f7886d8774070f42c` |
| 최초 제출 규모 | 9 files, +68 / -77, provenance 보존 8 commits |
| GitHub 상태 | code candidate에서 Open, non-draft, MERGEABLE / CLEAN 및 required check 성공. 최종 head는 다시 확인 필요 |
| reviewer | self-review 통합 PR이므로 지정하지 않음 |

라우팅: `collaborator_self_merge` + `intake_and_review` + `local_validation` +
`multi_pr_update_branch` + `rework_and_exceptions`의 Dependabot 예외.
원 PR은 봇 자동 갱신이므로 사용자 지시에 따라 개별 review 문서를 만들지 않고 이 통합 PR만 검토한다.

## 변경과 provenance

| 원 PR | 원 head | 통합 적용 commit | 변경 |
| --- | --- | --- | --- |
| [#7301](https://github.com/edwardkim/rhwp/pull/7301) | `3440487daaf00b4d191e544e111a7bcc0614dc31` | `8ba0f213424341787fda85b75256325eca6affdc` | `rhwp-chrome`의 `puppeteer` 25.10.0 → 25.11.0 |
| [#7302](https://github.com/edwardkim/rhwp/pull/7302) | `0417986651b7801e21f7c43c5799bee58074e28a` | `256e32c67597778e78d351ddda67b7023d4d4101` | `crc32fast` 1.5.1 → 1.5.2 |
| [#7303](https://github.com/edwardkim/rhwp/pull/7303) | `fe9138ebc9f3f7001cc4fc7c9307ddc0a880e233` | `2bf3b2e820dc3ad3d558d5b130427338a138c08d` | `clap` 4.6.6 → 4.6.7 |
| [#7304](https://github.com/edwardkim/rhwp/pull/7304) | `2ae647201fc36dd53836e14890eda0bf49b0f4e6` | `9e67a1a7f5f098899e4c9347d53552a921c3b1de` | `codepage` 0.1.2 → 0.1.3 |
| [#7305](https://github.com/edwardkim/rhwp/pull/7305) | `e1f72bc1607e0c88669b415ee4055ddce0fbb489` | `18c4d069af1c6e5992cc483b82e7381eb073e79d` | `rhwp-vscode`의 `webpack` 5.110.3 → 5.111.0 |
| [#7306](https://github.com/edwardkim/rhwp/pull/7306) | `8f89514ddaa185ff8fab57dcab6a68b73781122d` | `9ae8f39eee0b5821226c2b8e5bb341e7ed267fc5` | `rhwp-studio`의 `@types/chrome` 0.2.9 → 0.3.0 |
| [#7307](https://github.com/edwardkim/rhwp/pull/7307) | `3d4c0b9d7d6b4e543a0451195e40b3fce2ca46f3` | `5b09dfdcaafd27d45feaa9a54bae64481c5d5c0f` | `rhwp-studio`의 `puppeteer-core` 25.10.0 → 25.11.0 |
| [#7308](https://github.com/edwardkim/rhwp/pull/7308) | `c8cac33b758bab81251acdb67d93851cd986c692` | `6f709298ca6f1e23df5e9a8f7886d8774070f42c` | `taiki-e/install-action` 2.87.11 → 2.87.15 |

#7306과 #7307은 같은 `rhwp-studio/package.json` 및 lockfile의 서로 다른 dev dependency를
변경해 체리픽 충돌이 났다. 통합 결과는 두 갱신을 함께 보존한다.

- `@types/chrome`: `^0.3.0`, lockfile package version `0.3.0`
- `puppeteer-core`: `^25.11.0`, lockfile package version `25.11.0`
- `puppeteer-core`의 `devtools-protocol`: `0.0.1680125`

workflow 변경은 기존 `nextest` 설치 호출·입력은 그대로 두고,
`taiki-e/install-action`의 SHA를 `9534c84618278caac52cb373bb164ed464dbd8af`
(v2.87.11)에서 `4076c08d76dba979c11a7285295b0716c1d67908` (v2.87.15)로만 갱신한다.

## 조판 원칙과 입력 확인

렌더러, layout, pagination, paint, Canvas/WASM API, HWP/HWPX fixture 및 기준 PDF는 바뀌지 않았다.
Render Diff CI 성공은 package 영향에 대한 CI 결과이며 시각 일치 주장으로 사용하지 않았다.

| 검토 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 비해당 | 제품 기능 분기나 renderer 구현을 바꾸지 않고 Cargo/npm dependency와 기존 CI action pin만 갱신한다. |
| 측정·배치 일관성 | 비해당 | 줄 측정·좌표·backend 경로에 변경이 없다. |
| 분할·이어받기 계약 | 비해당 | pagination, table fragment, 예약 높이 경로를 바꾸지 않는다. |
| 줄 소속과 점유 높이 | 비해당 | LineSeg·문단·셀 점유 경로를 바꾸지 않는다. |
| 사례와 증거의 독립성 | 비해당 | 문서 조판 사례를 추가·변경하거나 이를 개선한다고 주장하지 않는다. |
| 기준값 변경 | 비해당 | baseline, golden, 허용치에 변경이 없다. |
| 주장과 검증 범위 | 충족 | npm package별 설치·실행, Windows locked Rust workspace 검사, immutable action pin 정적 검사 및 code candidate Full CI를 실제 수행했다. |

직접 수용 근거에 사용한 HWP/HWPX/PDF 파일은 없다. 변경은 package manifest·lockfile 및
GitHub Actions YAML에 한정되며, HWP 문서나 한컴 PDF를 열어 조판·시각 판정을 내리지 않았다.
따라서 Visual Sweep과 검증 입력 커밋 확인은 **비해당**이다. 새 문서 fixture를 추가하지 않았으므로
개인 다운로드 자료나 미추적 PDF를 사용하지 않았다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `git diff --check upstream/devel...HEAD` 및 merge-tree | 통과; 기준선 `a4b8731ca` 위의 통합 tree `107ffcbc23e3c650cc3b00d754b641bebc8aeed3` 확인 |
| npm lockfile 설치 | `npm ci --ignore-scripts`를 `rhwp-chrome`, `rhwp-studio`, `rhwp-vscode`에서 통과 |
| Chrome 확장 | `npm --prefix rhwp-chrome run test:e2e:smoke` 통과; packaged viewer/options/print/service worker/content script 확인 |
| Chrome download | `npm --prefix rhwp-chrome run test:e2e:download` 통과; XLSX 2건 탭 0, HWP 8건 각각 viewer 탭 1 확인 |
| Studio | `npm --prefix rhwp-studio run build` 및 `npm --prefix rhwp-studio test` 통과: 1,761 passed, 2 skipped |
| VS Code | `npm --prefix rhwp-vscode run compile` 통과 |
| workflow 정적 검사 | `actionlint .github/workflows/build-nextest-archives.yml .github/workflows/run-nextest-archives.yml` 통과 |
| Rust locked workspace | Windows PowerShell 격리 worktree에서 `CARGO_BUILD_JOBS=8 cargo check --workspace --locked` 통과 |
| GitHub Full CI | [CI run 35564126524](https://github.com/edwardkim/rhwp/actions/runs/35564126524): lint, frontend package gates, Native Skia, archive A/B/C/D build 및 default-feature shard, Build & Test 모두 통과 |
| 보조 GitHub CI | CodeQL Rust/JS/Python, Render Diff Canvas visual diff, Proptest roundtrip, Adapter inter-diff, CI impact policy 모두 통과 |

macOS의 별도 `cargo check --workspace --locked` 시도는 소스·lockfile 오류가 아니라 로컬 Xcode
license 미동의로 linker가 시작 전에 exit 69를 반환해 수용 근거에 세지 않았다. 같은 immutable
통합 tree의 Windows locked workspace 검사와 GitHub Full CI가 이를 대체한다.

## 최종 판정

- 판정: **승인**
- 근거: 8개 원 PR의 exact head를 최신 `upstream/devel` 위에 provenance를 남겨 적용했다. 유일한
  통합 충돌은 두 Studio dependency 갱신을 모두 보존한 lockfile로 해소했고, 각 package의 설치·빌드·행동
  경계와 Windows Rust lockfile 해석을 확인했다. code candidate `6f709298c`의 Full CI도 성공했다.
- merge 전 조건: 이 review/implementation trailing commit을 포함한 최신 PR head의 required CI 성공,
  `MERGEABLE`/`CLEAN` 재확인, 작업지시자의 merge 승인.
- 이 문서는 self-review 기록이며 GitHub approve, 원 PR close/comment 또는 merge를 수행하지 않는다.
