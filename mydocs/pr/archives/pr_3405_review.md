---
kind: pr_review
status: active
canonical: mydocs/pr/archives/pr_3405_review.md
last_verified: 2026-07-27
---

# PR #3405 검토 — HWP5 비밀번호 암호 문서 복호화

Issue: #3474 (후속 UI 열기 경로)

base route: collaborator_external_pr
modifiers: intake_and_review, local_validation, visual_fixture_evidence, rework_and_exceptions

## 1. Metadata (작성 시점 참고값, merge 전 재확인)

| 항목 | 값 |
|---|---|
| PR | #3405 — `feat(crypto): HWP5 비밀번호 암호 문서 복호화 지원` |
| 작성자 | `scari` (external contributor) |
| base / 원 head | `devel` `7995786bf` / `5a41caff10` |
| 규모 | 원 PR +1,679 / -149, 15 파일, 기능 commit 1개 + devel merge 2개 |
| mergeable / 상태 | 작성 시점 `MERGEABLE` / `UNSTABLE`; merge 직전 재확인 필요 |
| 관련 이슈 | #1946은 이미 closed이며, 본 PR은 이를 닫지 않는다고 명시한다. UI 후속은 #3474 |
| reviewer | `jangster77` assign 완료 |
| collaborator 보정 | `ecdfd9ca4` (원 head 위 별도 commit, 아직 remote push 안 함) |

`maintainerCanModify=true`를 source head, `ls-remote`, local fetch의 동일 SHA로 확인한 뒤
`review/pr3405-maintainer`에서만 보정을 준비했다. 원 contributor commit은 rewrite하지 않았다.

## 2. 변경 범위와 보정

원 변경은 HWP5 `EncryptVersion 4`의 키 파생·CFB 복호화, DocInfo/BodyText/BinData/Scripts 처리,
Rust·WASM·CLI 공개 경로, 평문 HWP 저장, 라이선스 고지를 추가한다.

collaborator 보정 `ecdfd9ca4`은 다음만 추가·정리한다.

- 작업지시자 지시에 따라 maintainer 전용 최상위 `CHANGELOG.md`, `README.md`, `README_EN.md`의
  PR 추가 내용을 제거했다. 상세 사용법은 변경 범위의 component 문서에 남긴다.
- 실제 HWP5 암호 fixture `samples/hwp3-sample16-hwp5-2024-password-123456.hwp`를 유지하고,
  정답·오답·미입력, Rust 공개 API, CLI stdin, 일반 HWP 저장 후 재파싱을 고정하는
  `tests/hwp5_password_fixture.rs`를 추가했다. fixture SHA-256은
  `59d4bed335b9552fe78fa68d2a56f7cfa3d586bcdeaaba839af80df13f3e08dc`다.
- 일반 HWP5 roundtrip gate는 비밀번호 입력을 받지 않으므로 암호 fixture를 자동 제외하고, 전용
  fixture test가 책임진다는 범위와 근거를 `tests/hwp5_roundtrip_baseline.rs` 및 가이드에 기록했다.
  IR field-sweep TSV는 암호 fixture가 일반 no-password sweep에서 제외되어 신규 baseline 행이 없다.

## 3. 시각·fixture 판정

새 HWP fixture와 WASM 공개 API가 있어 fixture 경로를 확인했다. 다만 renderer/layout/paint,
페이지 배치 또는 시각 충실도는 변경하지 않고 그런 주장을 하지 않는다. 실제 fixture를 CLI로 SVG
64페이지까지 내보낸 결과는 parser 경로 smoke evidence일 뿐, 기준 PDF와의 visual sweep 또는 merge
판정 근거로 사용하지 않았다. 따라서 대표 PNG·기준 PDF를 새로 만들지 않는다.

## 4. 검증

모든 Cargo 검증은 `CARGO_TARGET_DIR=target/review-scari-hwp5-password-20260727`,
`CARGO_INCREMENTAL=0`에서 수행했다. 공유 `target` 경로는 건드리지 않았다.

| 검증 | 결과 |
|---|---|
| 최신 `upstream/devel` 위 merge simulation | `--no-commit --no-ff` 충돌 0, abort 후 clean |
| `cargo fmt --check`, `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |
| 실제 fixture 회귀 (`hwp5_password_fixture`) | 2 passed — 열기·오답·CLI exit contract·평문 저장 재열기 |
| HWP5 roundtrip baseline | 3 passed |
| IR field sweep baseline | 2 passed, baseline diff 0 |
| 압축 해제 상한 단위 회귀 | compressed 확장·uncompressed 초과 각각 1건 통과 |
| `cargo test --profile release-test --tests` | 최종 `2c8dbfaf6` 기준 exit 0 |
| 원 PR GitHub CI | CI 모든 실행 check 성공. CodeQL Action의 언어 분석도 성공했으나 아래 security check는 실패 상태 |

## 5. 발견한 차단 사유

### 5.1 CodeQL security alert 57건 — 근거 확인 뒤 dismiss 완료

원 head의 CodeQL Action 언어 분석은 성공했지만, 2026-07-27의 aggregate `CodeQL` check는 새 alert
57건 때문에 failure였다. annotation과 source를 대조하고 실제 CLI binary로 두 입력 경로를 실행한 뒤,
alert를 다음 사유로 dismiss했다.

| rule / 수 | 판단 | dismiss 사유 |
|---|---|---|
| `rust/cleartext-logging` 41 | `extract_global_password()` 반환 tuple 전체가 taint돼, 비밀번호 토큰을 제거한 뒤 쓰는 `args` 오류 출력까지 모두 sink로 분류한 path-insensitive 과탐지 | `false positive` |
| `rust/hard-coded-cryptographic-value` 16 | `src/parser/crypto.rs`, `src/parser/mod.rs`의 `#[cfg(test)]` 고정 알고리즘 벡터·합성 fixture 입력이며 release binary와 사용자 암호에 포함되지 않음 | `used in tests` |

동적 확인은 `--password`와 `--password-stdin` 각각에 고유 sentinel을 주고, CodeQL이 가리킨
`test-caption`의 옵션 오류를 발생시켜 수행했다. 두 경우 모두 stdout/stderr에 sentinel이 없었다.
`raw_args`는 pre-scan 외에 사용되지 않고, 오류 출력은 비밀번호 토큰을 제거한 `args` 또는 공개 오류
분류만 사용한다. 따라서 이 57건은 실제 비밀번호 유출 경로를 입증하지 않는다.

다만 `--password <값>` 자체는 OS 프로세스 목록에 노출될 수 있으므로 CLI가 고지한 대로
`--password-stdin`을 권장한다. 또한 단발 CLI의 thread-local `String`은 zeroization하지 않으므로,
장기 실행 UI에서는 #3474의 비보존·수명 제한 요구사항을 따른다. dismiss는 과탐지의 처리이지 이 두
운영상 주의사항을 없애는 조치가 아니다.

기존 failure check는 과거 run의 불변 결과다. source push 뒤 새 CodeQL run에서 새 alert가 없는지와
required check 성공을 다시 확인한다.

### 5.2 압축 해제 제한 주장과 구현 범위 불일치 — 메인터너 보정 완료

`2c8dbfaf6`에서 `MAX_PASSWORD_DECOMPRESSED_STREAM_BYTES`(512 MiB)를 도입하고
`decrypt_password_protected_limited()`로 strict·lenient `DocInfo`, strict·lenient `BodyText`,
즉시 materialize BinData, lazy BinData 기본 resolve를 모두 연결했다. bounded lazy resolve도 같은
함수에 위임한다. 압축 스트림의 확장과 비압축 복호화 결과가 작은 회귀 상한을 초과하는 경우를 각각
전용 오류로 고정했다.

상한은 **스트림별 복호화 후 결과**에 적용된다. CFB 원본 암호문을 읽는 크기는 파일 자체 크기에
좌우되며, 이 보정은 deflate 확장으로 인한 추가 메모리 폭주를 제한한다. 따라서 기능 범위를 축소하는
대안 (b)는 선택하지 않았다.

## 6. 최종 권고

**수용 후보.** 실제 fixture, API·CLI·저장 경로, 전 스트림 압축 해제 상한과 Rust 전체 회귀를
확인했다. CodeQL 57건은 근거 확인 뒤 dismiss했다.

merge 전 조건:

1. source push 뒤 새 CodeQL run에서 새 alert가 없고 required check가 성공함을 확인
2. source head가 다시 바뀌지 않았음을 확인한 뒤 collaborator 보정 commit·review 기록을 push하고,
   최신 head full CI와 작업지시자 승인을 재확인
