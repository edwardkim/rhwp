# PR #3399 검토 기록 — bug-hunter 방법론과 fidelity 비교 하네스 보완

| 항목 | 내용 |
| --- | --- |
| PR | [#3399](https://github.com/edwardkim/rhwp/pull/3399) — `docs: 실사례 여정 기반 탑다운 버그 헌팅` |
| 작성자·처리자 | `@kevin9327` (external contributor) · `@jangster77` (collaborator) |
| base / head | `devel` / `pr/task-3398-bug-hunter-playbook` (fork: `kevin9327/rhwp`) |
| source head | 보정 시작 시점 참고값 `2b8531da40efa2d406d214f57488ffa3d19a6dc9` |
| collaborator 보정 | `fef78b9e` — fidelity 비교 하네스 검증 계약 보강 |
| 관련 이슈 | [#3398](https://github.com/edwardkim/rhwp/issues/3398) |
| 원 PR 규모 | 작성 시점 참고값 14 files, +702 / -7 |
| 보정 규모 | 6 files, +748 / -154 |

## 라우팅

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md, visual_fixture_evidence.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, docs_and_git_workflow.md, dev_environment_guide.md
current head: 2b8531da40efa2d406d214f57488ffa3d19a6dc9 (보정 시작 시점 참고값)
```

PR 작성자는 외부 contributor이고 `maintainerCanModify=true`다. contributor 원 commit은 rewrite하지
않고, source head 위에 collaborator 보정 commit을 별도로 추가한다. code·test 보정이 있으므로
review-only fast-pass 대상이 아니며 push 뒤 최신 head의 full CI가 필요하다.

## 메인터너 후속 제안 5건 재검토

기준은 [메인터너 코멘트](https://github.com/edwardkim/rhwp/pull/3399#issuecomment-5103259389)다.

| 요청 | 반영과 검증 | 판정 |
| --- | --- | --- |
| PDF 텍스트층 ↔ SVG 문자 멀티셋 대조를 playbook 보조 기계 판정으로 추가 | playbook·Claude agent·Codex Skill 계약을 갱신하고 하네스가 `text-report.tsv`를 생성한다. NFC 정규화 뒤 공백·순서를 제외한 문자 멀티셋의 기준본 전용/렌더 전용 문자를 기록한다. | 충족 |
| `tools/fidelity_compare` 크로스 플랫폼 기본값과 Linux 예시 | `os.name`·`sys.platform`·`shutil.which`로 Windows/macOS/Linux의 `rhwp`와 Chrome/Chromium을 탐색하고 `RHWP_BIN`/`CHROME_BIN` override를 유지했다. README에 PATH 사용과 override를 모두 포함한 Linux 예시를 추가했다. | 충족 |
| `--out-dir` | 지정한 디렉터리 자체를 산출 루트로 쓰며 생략 시 기존 `output/fidelity/<키>`를 유지한다. 저장소 밖 임시 경로 smoke로 실제 산출을 확인했다. | 충족 |
| REG가 선택하는 기준 PDF 등급 명시 | plan/manual/korexam/math/eng는 `pdf/`의 버전 접미사 기준 PDF로 변경하고, `samples/` 동반본만 있는 bunjang은 참고 PDF로 명시했다. 선택 경로·등급은 README와 `provenance.tsv`에 남는다. | 충족 |
| Chrome 캡처 1회 재시도와 stderr 표면화 | SVG PNG와 비교 시트가 공용 capture helper를 사용한다. 첫 실패 stderr를 출력한 뒤 한 번 재시도하고 두 번 실패하면 false를 반환하는 회귀 test를 추가했다. | 충족 |

최초 재검토에서는 README의 명시적 Linux 실행 예시가 빠진 것을 발견했다. 보정 commit을 원격에
올리기 전에 예시를 추가하고 링크 검사를 다시 통과했으므로 최종 판정은 5건 모두 충족이다.

## 텍스트층 방법 실증

메인터너가 계약 시행에 사용한 것과 같은 입력·기준을 사용했다.

- 원본: `samples/2022년 국립국어원 업무계획.hwp`
  (`SHA-256 ab59c95dde8cd42e490f7b9a3deb13a9142969706e784053237bf9dc625150e9`).
- 기준 PDF: `pdf/2022년 국립국어원 업무계획-2022.pdf`, 한컴 2022, A4, 35 pages
  (`SHA-256 94695a77c2e68dedaa9e0658c1d490a1a89b561ab6419aa39cd99bc9d4a1a153`).
- 범위: 1–10쪽. 산출은 저장소 밖 임시 `--out-dir`에 만들고 검증 뒤 휴지통으로 이동했다.

| 쪽 | 기준 PDF에만 있음 | SVG에만 있음 | 후보 |
| --- | ---: | ---: | --- |
| 2 | `U+00B7` 589자 | 0 | 탭 채움점 소실 |
| 3 | `-` 2자, `1` 1자 | 0 | 쪽번호 소실 |
| 6 | PUA 2자 | `U+300A`·`U+300B` 각 1자 | PUA 책괄호 치환 |
| 7 | 0 | 한글 15자 | 그림자/숨김 대상 과잉 출력 |
| 1·4·5·8·9·10 | 0 | 0 | 문자 멀티셋 무격차 |

픽셀 diff는 1쪽 2.22%, 나머지 후보군 9.24–19.57%로 폰트·자간 잡음이 섞였지만 문자 보고서는
메인터너가 기록한 소실·과잉·치환 네 축을 페이지별로 분리했다. 문자 멀티셋은 path 글리프와 PDF
텍스트 매핑 차이를 오탐할 수 있으므로 후보 검출 근거이며 최종 시각 판정을 대신하지 않는다.

## 로컬 검증

- `python3 -m unittest discover -s scripts/tests -p 'test_*.py'`: **15 passed**
  (기존 capability 4건 + fidelity 신규 11건).
- `python3 -m py_compile tools/fidelity_compare/fidelity_compare.py scripts/tests/test_fidelity_compare.py`:
  통과.
- `ruff check`·`ruff format --check`: 통과.
- `python3 scripts/check_markdown_links.py --changed-from 2b8531da... --forbid-redirect-references`:
  430개 문서, 내부 상대 링크 이상 없음.
- `git diff --check`: 통과.
- macOS 1쪽·10쪽 실제 smoke: Chrome 캡처, 픽셀 report, text report, provenance report 생성 성공.
- Linux·Windows 탐색 분기는 mock 기반 회귀로 통과했고, 두 지정 교차 호스트에서도 실제
  end-to-end 실행을 통과했다. 두 실행 모두 `RHWP_BIN`·`CHROME_BIN`을 비우고 플랫폼 기본 탐색을
  사용했으며 입력 HWP와 기준 PDF의 SHA-256이 위 실증 입력과 일치함을 먼저 확인했다.
  - Linux: Ubuntu 24.04.3 LTS, SSH 기본 `bash`, Python 3.12.3, Cargo/Rust 1.93.1. 격리 worktree와
    `CARGO_INCREMENTAL=0` 전용 target에서 `release-test`를 빌드하고 Chromium
    150.0.7871.128 snap 및 격리 venv의 `pypdfium2`·Pillow를 설치했다. PATH에서 전용 `rhwp`,
    `/snap/bin/chromium`을 탐색했다. 1쪽 pixel diff 2.3%, text multiset 기준 전용/렌더 전용 0/0이며
    SVG·양쪽 PNG·비교 시트·세 보고서를 모두 생성했다.
  - Windows: Windows 10 Pro 19045, PowerShell 5.1, Python 3.12.10, Cargo/Rust 1.93.1. 사용자 파일
    5개가 있는 기존 dirty worktree를 건드리지 않고 별도 worktree와 `CARGO_INCREMENTAL=0` 전용
    target에서 `release-test`를 빌드했다. PATH의 전용 `rhwp.exe`와 Program Files의 기존 Chrome
    150.0.7871.187을 기본 탐색했다. 1쪽 pixel diff 2.16%, text multiset 0/0이며 같은 산출물 세트를
    모두 생성했다.
- 확인 시점 `upstream/devel` `39ed2f65`는 source head의 ancestor이며 source가 29 commits 앞서 있어
  merge simulation은 충돌 없이 already-up-to-date 조건이다.
- 변경 파일 9개는 모두 LFS 비대상이다. 정상 push dry-run은 contributor fork의 LFS lock 확인 권한만
  실패했고, 가이드가 허용한 `GIT_LFS_SKIP_PUSH=1` dry-run은 source head 위 두 local commit의
  fast-forward 성공이다.

Rust, renderer, sample, golden, baseline은 변경하지 않는다. 따라서 Cargo·Native Skia·WASM과 별도
시각 asset 검증은 생략했다. 다만 Python code와 test가 추가되므로 원격 반영 뒤 최신 PR head의 full
CI와 CodeQL 결과를 최종 조건으로 둔다.

## 최종 권고

메인터너 후속 제안 5건은 collaborator 보정 `fef78b9e`에서 모두 충족됐다. 작업지시자 승인으로
두 local commit을 contributor source branch에 push하고, 최신 head가 `MERGEABLE`이며
full CI·CodeQL·Render Diff가 모두 통과하는지 재확인한 후 merge 여부를 결정한다.
