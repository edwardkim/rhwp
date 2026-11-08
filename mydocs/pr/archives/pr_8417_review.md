# PR #8417 검토 — Windows CFB 테스트 경로 정규화

## 최종 판정

**승인 — 원 기여의 테스트 도우미 수정은 수용 가능하며 독립 집중 회귀 3개가 통과했습니다.** 작업지시자는 2026-11-01 02:22:48 UTC에 승인 리뷰 제출과 devel 정상 병합 및 그 범위의 필수 기록을 승인했습니다. 실제 병합은 최신 문서 head CI·정책·병합 상태가 통과한 뒤 수행합니다. 보호 정책 우회·관리자 강제 병합·CI 재실행·제품 소스 변경·브랜치 삭제는 승인 범위에 없습니다.

## 접수와 변경 범위

- [PR #8417](https://github.com/edwardkim/rhwp/pull/8417), 작성자 `Steveyang8282`, 첫 기여, non-Draft, base `devel`입니다.
- 상세 조사 전에 `requested_reviewers` API의 `users`에서 `edwardkim`을 확인했습니다.
- 검증 code head: `1feecd87284777a1ddcfe4eeafd745abe9cc1894`.
- 검증 base: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- fork source: `Steveyang8282/rhwp`, `fix/issue-8225-windows-cfb-paths`입니다. 기여자 원 commit은 수정하지 않았습니다.
- 변경은 `tests/cases/issue_8225_group_child_ctrl_data.rs` 두 helper의 경로 정규화 두 줄입니다. 제품 parser·serializer·조판·renderer·fixture·assertion 변경은 없습니다.

## 선행 분석과 코드 판단

[이슈 #8225](https://github.com/edwardkim/rhwp/issues/8225)의 그룹 자식 CTRL_DATA 저장 손실은 [PR #8233](https://github.com/edwardkim/rhwp/pull/8233)에서 다뤘습니다. 원 head `d9c8a71b642fad1ee8221f4036817a23dfe15f10`, 원 merge `86f4879934d1fc903b8497d011fdd7db9dede38f`, 실제 [통합 PR #8395](https://github.com/edwardkim/rhwp/pull/8395)의 merge `21ef5020f763e474ff652df5c2c9adf94efededc`와 [기존 독립 검토](pr_8233_review.md)를 확인했습니다. 후속 `b8c0ef2e4ba1b827db13b9b86bf355f7e27f5e4f`는 테스트 근거 주석을 보완했습니다.

두 helper는 `BodyText/Section`을 필터링합니다. Windows 역슬래시는 이 조건과 일치하지 않아 빈 결과를 만들며, 이번 정규화는 저장소의 `issue_3570_record_padding`, `issue_3565_container_child_ctrl_id` 등 CFB 검사와 같은 방식입니다. `cfb` 0.15.0의 `open_stream`은 Rust `Path::components()`로 이름 체인을 만듭니다. CFB 이름 자체에는 `/`·`\`를 허용하지 않아 이름 내용을 바꾸는 치환과 충돌하지 않습니다.

조판 원칙·Visual Sweep·fresh WASM 시각 검증은 비해당입니다. 이 변경은 비시각 저장 회귀의 입력 경로만 고칩니다. snapshot·golden·baseline도 변경하지 않습니다. #8225의 최상위 CTRL_DATA 중복 표현 차이는 별도 잔여 범위이므로 이 PR을 이슈 전체 해결로 판단하지 않습니다.

## 독립 실행 결과

- WSL Ubuntu 24.04 / Linux opx001 x86_64, Rust·Cargo 1.93.1, 기존 Node v24.15.0을 사용했습니다. 새 소프트웨어 설치나 보안 설정 변경은 없습니다.
- Windows 26300 네이티브 Rust 1.93.1의 std 경로 probe에서 실제 `/BodyText\Section0`은 기존 필터에 실패하고 `/BodyText/Section0`은 통과했습니다. 정규화 전후 `Path::components()`도 동일했습니다. 이는 실제 Windows fixture 3개를 독립 실행한 결과와 구분합니다.
- 정확한 code head의 분리 review worktree에서 `cargo fmt --all -- --check`가 통과했습니다(종료 0, 24.49초).
- `node scripts/rust-test-suite-manifest.mjs --prepare` 및 `--check --base-ref bc9dfdd057258de59afebeb5121c4bee7398e07f`가 통과했습니다. 파생 suite·manifest는 커밋하지 않았습니다.
- `node scripts/run-rust-test.mjs issue_8225_group_child_ctrl_data -- --offline --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review`가 통과했습니다. 내부 실행은 `cargo nextest run --locked --test regression_suite_025`와 해당 case 필터입니다. 빌드 포함 321.88초, 검사 실행 0.089초입니다.
- nextest run `96a59bff-7d8e-42a6-8c85-e49659d7de96`: **3 tests run / 3 passed**, 같은 binary의 다른 290개는 focused 필터로 제외했습니다.
- 통과 검사: `child_only_ctrl_data_is_not_written_twice`, `group_child_ctrl_data_survives_in_multi_section_document`, `group_child_ctrl_data_survives_section_rewrite`입니다. 실제 CFB 경로 재열기·자식 payload 바이트와 순서 보존·중복 방지를 실행했습니다.
- nextest 0.9.137은 저장소 권장 0.9.140보다 낮다는 경고와 관측 profile의 JUnit 설정 키 무시 경고가 있었습니다. 기본 profile에서 요청한 3개 검사가 모두 실행됐으며 버전 검사 우회 옵션은 사용하지 않았습니다.
- 기존 #8413의 공유 Cargo 작업과 예약 driver가 종료된 뒤에만 실행했습니다. 해당 작업을 취소하거나 별도 target·강제 잠금 해제를 사용하지 않았습니다. 기본 checkout의 기존 미커밋 변경을 보존했습니다.
- 테스트 후 code head와 원격 head 일치, `git diff --exit-code`·`git diff --check`·clean 상태를 확인했습니다.

## 입력과 CI 근거

실행한 세 입력의 `git hash-object`가 검토 HEAD blob과 일치했습니다.

| 입력 | Git blob |
| --- | --- |
| `samples/basic/sungeo.hwp` | `4b960a5621729bd0196231788d00fd50d04c83a5` |
| `samples/hwp-multi-001.hwp` | `fc8ab9e3704d25212a1deb9ba9a10dfc0f4462f0` |
| `samples/issue6542/156678235_mid_para_vpos_rewind.hwp` | `73ca700aebee303548522f538d9fe934b149e8fa` |

[code head의 CI](https://github.com/edwardkim/rhwp/actions/runs/41220043386)는 정확한 `head_sha`에서 성공했습니다. 실제 PR merge ref `0d00a5919879113de253904205bc542d054e4da2`와 code head의 tree는 모두 `3a7783984f20ca2d72552bd47ad5f4ce733f449c`입니다. local merge-tree도 충돌 없이 동일 tree를 얻었습니다.

preflight는 `classified:rust`·`rust_required=true`였으며, lint 로그에서 fmt·Native/WASM32 Clippy·workspace build/all-target Clippy·WASM check의 실제 실행을 확인했습니다. CI workspace 명령은 `llm-verifier-verdict-protocol`을 제외했습니다. A–E 아카이브 빌드·실행과 Build & Test 집계, CodeQL 4언어 분석·Adapter·Proptest·CI Impact Policy 성공을 확인했습니다. CodeQL 집계 한 건의 neutral과 범위 밖 skipped는 실패로 바꾸어 기록하지 않습니다.

Clippy·workspace build·전체 회귀는 새 source/test 보정이 없는 동일 code head의 위 CI 근거를 확인했으며 로컬에서 중복 실행하지 않았습니다. 작성자의 Windows 수정 전 3 failed/수정 후 3 passed 보고는 기존 증거입니다. 이를 이번 Linux 독립 실행과 혼동하지 않습니다.

## 원격 처리와 후속 조건

이 trailing commit은 이 검토 기록과 [오늘할일](../../orders/20261101.md)만 추가합니다. 제품 소스·테스트·fixture·workflow·보안 설정은 변경하지 않습니다. [승인 리뷰](https://github.com/edwardkim/rhwp/pull/8417#pullrequestreview-5726035825)를 원 code head에 제출했고 API에서 `edwardkim`·`APPROVED`·정확한 commit을 확인했습니다. 문서 기록의 fork push는 추가 승인 대기이며 실제 원격 push는 아직 수행하지 않았습니다. 기록 반영 뒤 최신 trailing head의 CI preflight와 aggregate·정책·충돌 상태를 확인하고 정상 병합을 수행합니다. 강제 병합이나 보호 설정 우회는 사용하지 않습니다.

보호 규칙 조회는 연결 도구의 선행 접근 제한 때문에 미확인 상태이며 다른 경로로 우회하지 않았습니다. 실제 정상 merge API의 정책 판정을 따릅니다. 병합 뒤 merge SHA와 devel 반영, #8225 상태를 재조회합니다. 첫 기여에 대한 감사와 검증 결과는 승인 리뷰에 한국어 존댓말로 남깁니다. #8225는 이 PR로 종료하지 않습니다. 브랜치 삭제는 승인 범위 밖입니다.
