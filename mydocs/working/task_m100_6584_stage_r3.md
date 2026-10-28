# Task M100 #6584 Stage R3 — 버전·CHANGELOG·릴리스 노트 결과

- 작업 기준: `task_m100_6584` / release base `063041a2ced54085b5cf94c2e646ac7aa0e1960d`
- 대상 버전: `0.8.6`
- 기록일: 2026-09-02 KST
- 상태: 로컬 구현·Stage R3 검증·결과 검토 완료, commit 승인

## 1. 결과

공식 사용자 표면의 정본 버전을 0.8.6으로 맞췄다. root Cargo package, npm editor,
Studio, VS Code, Chrome/Edge, Firefox, Safari와 각 package-lock의 root entry가 하나의 버전을
사용한다. `pkg/package.json`은 파생 산출물이므로 직접 편집하지 않았다.

root `CHANGELOG.md`·`CHANGELOG_EN.md`와 GitHub Release 게시용 초안에 v0.8.6 변경,
호환성, 알려진 후속 게이트와 기여자를 기록했다. VS Code 변경 기록과 한·영
README, third-party license 기준 버전도 새 정본에 맞췄다.

## 2. 버전 표면 대사

| 표면 | 이전 | 결과 |
|---|---:|---:|
| `Cargo.toml`, root `Cargo.lock` package | 0.8.4 | 0.8.6 |
| `npm/editor/package.json` | 0.8.5 | 0.8.6 |
| Studio package + lock root | 0.8.4 | 0.8.6 |
| VS Code package + lock root | 0.8.4 | 0.8.6 |
| Chrome package/manifest + lock root | 0.8.4 | 0.8.6 |
| Firefox package/manifest + lock root | 0.8.4 | 0.8.6 |
| Safari manifest | 0.8.4 | 0.8.6 |

package-lock 내 외부 의존성 `modern-tar` 0.8.4는 자체 릴리스 버전이 아니므로
그대로 유지했다. 과거 fixture·보고서의 `toolVersion` 같은 역사 증적도 변경하지 않았다.

## 3. 기여자 불변식

세 공개 기록의 `release-contributors` 블록을 기계적으로 읽어 ledger와 순서·
대소문자까지 일치하는지 검사했다.

- ledger: `mydocs/tech/investigations/issue-6584/release_contributor_ledger.json`
- ledger SHA-256: `934a96927831ec87d2b19db296ea1976111b1f1e815c66291369e2a0c1929c28`
- 사람 credit-key 목록 SHA-256: `169db39bb034abca43b16bae1e6a9d65f127af0e29785e4dcee855e1bed3a2bf`
- 공개 사람: 20명
- bot: `dependabot[bot]` 1개, 세 공개 사람 명단에서 제외
- GitHub 계정 미확인: `dkh0324`를 다른 사람과 병합하지 않고 Git author credit으로 보존

`scripts/tests/test_release_record_contributors.py`가 두 CHANGELOG와
`mydocs/working/task_m100_6584_release_notes.md`의 집합을 독립 검사한다.

## 4. 추가 릴리스 문서 판정

- `README.md`·`README_EN.md`: 현재 개발 버전이 0.8.4로 표시돼 갱신 필요.
- `THIRD_PARTY_LICENSES.md`: root Rust package와 자체 npm 배포 package 버전이 0.8.4로
  남아 갱신 필요. 외부 의존성 판정은 변경 없음.
- `rhwp-vscode/CHANGELOG.md`: 새 0.8.6 절을 추가하고 과거 0.8.4의 `미출시`를
  실제 날짜 2026-08-12로 정정.
- browser store 문서: 확장 재제출과 `.xlsx` 다운로드 판정·미리보기 압축 방어
  변경이 있어 Chrome 한·영, Edge, Firefox AMO의 0.8.6 버전별 심사 문서를 추가.
  v0.8.4 대비 manifest 권한·host permission·content-script 선언은 변경 없음.

## 5. 검증 결과

| 검증 | 결과 |
|---|---|
| release channel + contributor audit + public record tests | 19 tests, PASS |
| package/manifest/lock JSON parse | PASS |
| `cargo metadata --locked --no-deps --format-version 1` | PASS |
| 변경 Markdown 13개 상대 링크 검사 | PASS |
| `git diff --check` | PASS |
| package-lock root 0.8.6 / `modern-tar` 0.8.4 구분 | PASS |

Stage R3은 Rust source나 런타임 구현을 바꾸지 않았다. full Rust lint·nextest, Docker
WASM, Studio/CDP, 확장 build, 5-platform dry-run은 마스터 수행계획에 따라 Stage R4·R5의
장시간 게이트로 분리한다.

## 6. 판정과 다음 게이트

Stage R3 종료 게이트인 공식 정본 0.8.6 일치와 세 공개 기록의 사람 20명
ledger 일치를 충족했고, 메인테이너가 결과와 local commit을 승인했다. 다음은
Stage R4 예상 소요시간과 디스크 예산을 먼저 계측해 보고하고, 별도 승인 후 장시간
release candidate 검증을 실행하는 절차다.
