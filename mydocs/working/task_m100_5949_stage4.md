# Task M100 #5949 Stage O3-4 — Linux AArch64 원격 실행 검증

- **Issue**: [#5949](https://github.com/edwardkim/rhwp/issues/5949)
- **PR**: [#6573](https://github.com/edwardkim/rhwp/pull/6573)
- **검증 head**: `ea20afc4bc0c14825a176d54d052ca8a58696025`
- **Release Binary run**:
  [33510934562](https://github.com/edwardkim/rhwp/actions/runs/33510934562)
- **수행일**: 2026-09-01 KST
- **Stage 판정**: 통과 — native Linux AArch64 build·실행·artifact 확인

## 1. 선행 PR CI

trailing self-review가 포함된 exact head `ea20afc4b`에서 workflow 변경 PR의 Full CI가 실행됐다.

- [CI run 33509308330](https://github.com/edwardkim/rhwp/actions/runs/33509308330): 성공
- required `Build & Test`: 성공
- Lint, Native Skia, Frontend package와 test archive A/B/C/D: 성공
- CodeQL JavaScript/TypeScript, Python, Rust: 성공
- PR 상태: `MERGEABLE` / `CLEAN`

review-only tail이지만 이 PR은 workflow 자체를 바꾸므로 controller가 fast-pass 재사용을 적용하지 않고
Full CI로 전환했다. 이는 변경된 실행 정책을 PR head 스스로 신뢰하지 않는 보호 불변식과 일치한다.

## 2. dry-run dispatch와 배포 경계

다음 입력으로 exact branch에서 수동 실행했다.

```text
workflow: .github/workflows/release-binary.yml
ref: task_m100_5949
tag: test
head: ea20afc4bc0c14825a176d54d052ca8a58696025
event: workflow_dispatch
```

run은 2026-09-01 13:01:21 UTC에 시작해 13:19:35 UTC에 성공했다. `tag=test`가 `v`로
시작하지 않아 `Attach to GitHub Release` job은 의도대로 skip됐다. API에서 `test` tag의
GitHub Release가 존재하지 않음도 확인했다. npm·extension·Pages publish는 실행하지 않았다.

## 3. 5플랫폼 matrix 결과

| job | runner label | 결과 | wall time |
| --- | --- | --- | --- |
| `Build aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` | 성공 | 9분 57초 |
| `Build x86_64-unknown-linux-gnu` | `ubuntu-latest` | 성공 | 9분 45초 |
| `Build aarch64-apple-darwin` | `macos-14` | 성공 | 8분 54초 |
| `Build x86_64-apple-darwin` | `macos-14` | 성공 | 10분 46초 |
| `Build x86_64-pc-windows-msvc` | `windows-latest` | 성공 | 18분 7초 |

기존 네 target도 모두 성공해 Linux AArch64 행 추가가 기존 release matrix를 깨뜨리지 않았다.
run 전체 wall time은 약 18분 14초이며 가장 긴 Windows cold release build가 임계 경로였다.

## 4. Linux AArch64 native 실행

Linux AArch64 job은 실제 GitHub-hosted runner `ubuntu-24.04-arm`에서 실행됐다.

1. checkout 성공
2. stable Rust toolchain 설치 성공
3. `aarch64-unknown-linux-gnu` target 설치 성공
4. native release build 성공 — `Finished release profile` 8분 14초
5. `target/aarch64-unknown-linux-gnu/release/rhwp --version` 성공
6. 출력: `rhwp v0.8.4`
7. tar.gz 패키징과 artifact upload 성공

현재 source version이 0.8.4이므로 dry-run 출력도 0.8.4다. #5949는 v0.8.5 버전 bump를 포함하지
않으며, 실제 v0.8.5 release 전 버전 정합 검증은 release-prep 작업에서 별도로 수행한다.

## 5. artifact API 결과

run은 정확히 다섯 artifact를 생성했다.

| artifact | ID | Actions 저장 크기 |
| --- | ---: | ---: |
| `rhwp-test-linux-aarch64.tar.gz` | 9801951353 | 9,268,226 bytes |
| `rhwp-test-linux-x86_64.tar.gz` | 9801943261 | 10,059,712 bytes |
| `rhwp-test-macos-aarch64.tar.gz` | 9801910714 | 8,937,117 bytes |
| `rhwp-test-macos-x86_64.tar.gz` | 9801982017 | 9,739,581 bytes |
| `rhwp-test-windows-x86_64.zip` | 9802267044 | 10,233,395 bytes |

Actions artifact는 7일 보존이며 이번 run의 만료일은 2026-09-08 UTC다. 표의 크기는
`upload-artifact` 저장 envelope 크기이고, Linux AArch64 job이 만든 원본 tar.gz는 로그상
9,317,965 bytes다.

## 6. Linux AArch64 archive 독립 검증

artifact ID 9801951353을 별도 임시 디렉터리에 내려받아 내부 tar.gz를 검사했다.

```text
archive: rhwp-test-linux-aarch64.tar.gz
archive SHA-256: ba55608c2ea67ebdd9b2aff46334d33824d9ecea878a86d4b3ee7d520126fe8c

rhwp/
rhwp/LICENSE
rhwp/README.md
rhwp/README_EN.md
rhwp/rhwp
```

`file`과 `readelf -h` 판독 결과는 다음과 같다.

```text
ELF 64-bit LSB pie executable, ARM aarch64
Class: ELF64
Data: 2's complement, little endian
Type: DYN (Position-Independent Executable file)
Machine: AArch64
interpreter: /lib/ld-linux-aarch64.so.1
mode: -rwxr-xr-x
```

업로드된 Actions artifact envelope의 로그상 SHA-256은
`d8fb27a83a6bc2af7367a0404ff92487a1e77b22d4b262c432ea0104f4beb366`이다. 이 값은
내부 tar.gz SHA-256과 대상이 다르므로 서로 바꿔 쓰지 않는다. 검증용 임시 파일은 검사 뒤 모두
삭제했으며 repository 작업 트리에 artifact를 남기지 않았다.

## 7. 보호 불변식 판정

- 기존 네 target: 모두 성공
- 신규 Linux AArch64: native runner build·실행·패키징·upload 성공
- target·suffix 유일성: 로컬 workflow 계약과 실제 다섯 artifact로 확인
- `tag=test` 비게시 계약: release job skip, `test` GitHub Release 없음
- 권한·event·action pin: 변경 없음
- 제품 source·feature·lockfile: 변경 없음
- private sample·font·credential: artifact에 포함되지 않음

## 8. 잔여 단계

Stage O3-4의 원격 실행 목표는 완료됐다. 이 문서가 추가된 trailing head는 workflow 내용이 같은
문서-only tail이므로 Release Binary dry-run을 반복하지 않는다. 다만 push 뒤 최신 PR head의 required
checks와 `MERGEABLE/CLEAN`은 다시 확인한다.

그 뒤 작업지시자의 별도 merge 승인을 받아 정상 2-parent merge commit 방식으로 `devel`에 반영한다.
#5949는 구현 merge 뒤에도 열어 두고, 실제 v0.8.5 GitHub Release의
`rhwp-v0.8.5-linux-aarch64.tar.gz`와 `SHA256SUMS.txt`를 검증한 뒤 close한다.
