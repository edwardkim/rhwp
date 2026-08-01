---
kind: pr-review
status: active
pr: 3690
issue: 3604
last_verified: 2026-08-01
---

# PR #3690 리뷰: HWP3·HWP5·HWPX 암호 저장과 열기

## 라우팅

```text
base route: collaborator self-merge
modifiers: intake_and_review, local_validation, rework_and_exceptions(대형 PR)
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md, local_validation.md,
  rework_and_exceptions.md
current head: 069c66b21 (문서 갱신 시점 참고값)
```

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#3690](https://github.com/edwardkim/rhwp/pull/3690) |
| Issue | [#3604](https://github.com/edwardkim/rhwp/issues/3604) |
| 작성자 | `jangster77` |
| base / head | `devel` / `task_m100_3604` |
| 규모 | 46 files, +3071/-1876 (PR 생성 시점) |
| reviewer 요청 | GitHub requested-reviewers API가 빈 목록을 반환함 |

PR 생성 뒤 `devel`이 앞선 상태여서 최신 `upstream/devel` 위로 7개 task commit을 충돌 없이
rebase했다. 그 사이 remote head가 `devel` merge commit을 추가했으므로 중복 patch 7개는 Git이
자동으로 건너뛰고, 이 review 기록을 그 원격 head 위에 재적용했다. 위 head와 CI 상태는 merge
직전에 다시 확인해야 한다.

## 변경 범위

- HWP3/HWP5/HWPX의 키 유도·암호화·복호화를 `src/password_crypto.rs`로 통합한다.
- Rust CLI, MCP 무상태/세션 도구, WASM, Studio가 보호 문서 입력과 HWP/HWPX 보호 저장을 지원한다.
- Studio save-as dialog에 암호 설정을 통합하고, 암호 HWPX drag/drop의 Chromium renderer 종료를
  피하려고 드롭 경로의 File System Access handle capture를 제거한다.
- 암호 계약·저장 round-trip test와 사용 문서, Studio lockfile을 갱신한다.

## 알고리즘 대조

[kordoc #59](https://github.com/chrisryugj/kordoc/issues/59)의 후속 구현과 대조했다.

- HWPX는 PBKDF2-HMAC-SHA1을 먼저, HMAC-SHA256을 다음으로 시도하고 AES-256-CBC `NoPadding`,
  raw-DEFLATE, SHA-256 1KiB checksum으로 판정한다.
- HWP5는 복호 뒤 DocInfo의 `DOCUMENT_PROPERTIES`와 `ID_MAPPINGS`를 확인해 틀린 암호가
  빈 문서·경고로 성공처럼 통과하지 않게 한다.
- HWP3는 UTF-16LE 기반 DES-ECB 키 유도, 256-byte 암호 확인 영역 제거, raw-DEFLATE/CRC32/ISIZE
  검증을 사용한다. 아래아는 옛한글 자모열로 보존한다.

따라서 kordoc 이슈에서 지적한 암호 알고리즘 보완은 이 PR에 추가로 필요하지 않다. Node/OpenSSL의
단일 DES provider 제약은 Rust `des` crate를 사용하는 이 구현에 해당하지 않는다.

## 검증

| 검증 | 결과 |
| --- | --- |
| 암호 계약 3개 integration test | 7 passed, 0 failed |
| `npm --prefix rhwp-studio test` | 719 passed, 0 failed (Node.js v24.15.0) |
| `npx --yes tsx --test` 암호 관련 Studio test | 26 passed, 0 failed |
| `npm --prefix rhwp-studio run build` | 통과 |
| `npm --prefix rhwp-studio exec tsc -- --noEmit` | 통과 |
| npm lockfile install/dry-run | 통과 |
| `git diff --check` | 통과 |

Studio renderer/layout 또는 tracked HWP/HWPX fixture를 변경하지 않으므로 visual fixture evidence
절차는 적용하지 않는다. native Finder drag/drop은 이 호스트에 실행 가능한 Chrome automation이 없어
수동 확인을 대기한다. source 회귀 계약은 드롭 경로가 File System Access IPC를 호출하지 않고 파일
메뉴와 같은 password dialog 경로로 진입함을 고정한다.

## 위험과 권고

- 이 PR은 1,000줄을 넘는 대형 변경이므로 즉시 merge하지 않는다.
- 최신 rebased head의 GitHub Actions, native Finder drag/drop 수동 확인, 작업지시자 merge 승인을
  모두 충족해야 한다.
- 현재 권고: **CI 및 수동 확인 대기**.
