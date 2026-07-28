---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-28
---

# PR #3483 리뷰 — HWP3 비밀번호 암호 문서 복호화

- PR: [#3483](https://github.com/edwardkim/rhwp/pull/3483)
- Issue: [#3481](https://github.com/edwardkim/rhwp/issues/3481)
- 역할: `jangster77` collaborator self-review

## 라우팅과 작성 시점

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md, visual_fixture_evidence.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md
current head: 54888876356635a996891423cb4a9c7ed6702a69
  (review 기록 추가 전 최초 draft PR head 참고값)
```

이 PR은 collaborator 자신의 변경이다. self-review 기록은 독립 승인이나 최신 head의 CI를 대체하지
않는다. 최종 merge 조건은 review·오늘할일·증적을 포함한 최신 PR head의 required check 성공,
`MERGEABLE` 상태 재확인, 메인터너 검토 및 작업지시자 승인이다.

## PR metadata (작성 시점 참고값)

| 항목 | 값 |
| --- | --- |
| 작성자·검토 기록 작성자 | `jangster77` (collaborator self PR) |
| base → head | `devel` → `task_m100_3481` |
| 최초 PR head | `54888876356635a996891423cb4a9c7ed6702a69` |
| 최초 규모 | 18 files, +746 / -92 |
| mergeable / merge state | `MERGEABLE` / `BLOCKED` (draft·CI 전 참고값) |
| 관련 이슈 | `Closes #3481` |

## 변경 범위와 수용 판단

1. `src/parser/hwp3/crypto.rs`에 HWP3 전용 UTF-16LE 비밀번호→DES 키 유도, DES-ECB 복호화,
   raw DEFLATE·CRC32/ISIZE trailer 검증, 복호화 결과 512 MiB 상한을 구현했다. 오입력과 손상 입력은
   같은 일반 오류로 처리한다.
2. HWP3 암호 본문의 256바이트 확인 prefix는 검증 뒤 parser에 넘기지 않는다. 기존 HWP3 parser가
   읽는 실제 본문만 전달하고, 공용 IR에는 원본이 암호·압축 문서였다는 메타데이터를 보존한다. HWP
   저장은 기존 정책대로 평문 HWP가 된다.
3. 공용 parser·CLI·WASM `openWithPassword`·Studio의 기존 암호 대화상자 경로가 HWP3에도 연결된다.
   암호는 호출 범위에서만 쓰며 local/session storage·최근 문서·로그에 저장하지 않는다.
4. 실제 fixture `samples/HWP3-password-123456.hwp`의 무입력·오입력·성공·CLI stdin·평문 저장 재열기를
   Rust 회귀로 고정했고, Studio E2E는 기존 HWP5와 HWP3를 같은 취소·오입력·Enter·storage 비보존
   계약으로 검증한다.
5. fixture와 문서의 이전 형식 표기를 `HWP3`로 통일했다. asset 안의 "한글 97"은 원본 문서
   본문·로고의 내용이며 형식 표기가 아니다.

수용 가능한 범위다. 단, **압축된 HWP3 암호 본문만** 지원하며, 비압축 HWP3 암호 payload는 명시적으로
지원하지 않는 방식으로 거부한다. HWP5 복호화 경로와 renderer/layout/typeset 동작은 바꾸지 않았다.

## 검증 기록

모든 Cargo 실행은 `CARGO_TARGET_DIR=target/task_3481_hwp3_password`와
`CARGO_INCREMENTAL=0`에서 순차 실행했다. 공유 target은 건드리지 않았다.

| 검증 | 결과 | 판정 |
| --- | --- | --- |
| `cargo fmt --check`, `git diff --check` | passed | 형식·공백 오류 없음 |
| `cargo clippy --all-targets -- -D warnings` | passed | Rust lint 경고 없음 |
| HWP3 crypto focused test | 4 passed | 키 유도·정상·오입력·상한 |
| 실제 `hwp3_password_fixture` | 2 passed | parser·공개 API·CLI·평문 저장 재열기 |
| `cargo test --profile release-test --tests` | exit 0 | 전체 Rust 회귀 |
| IR field sweep | 803 samples, 3 skipped, 671 paths, 110345 records; 2 passed | fixture 등록 영향 확인 |
| baseline TSV diff | no output | 비영 왕복 발산 없음, TSV 갱신 불필요 |
| `wasm-pack build --target web --out-dir pkg` | passed | 새 WASM 생성 |
| Studio contract test | 4 passed | 암호 UI 전환·원자성·비보존 |
| Studio headless Chrome E2E | HWP3·HWP5 모두 passed | 취소·오입력·Enter 성공·storage 비보존 |
| Studio node test / production build | 674 passed / passed | frontend 회귀·배포 산출물 |
| Chrome·Firefox extension build | 각각 passed | 새 WASM을 각 `dist/wasm/`에 포함 |

Native Skia 3종은 실행하지 않았다. renderer/layout/typeset/paint 구현은 바뀌지 않았고,
`src/wasm_api.rs` 변경은 HWP3 지원 범위를 설명하는 주석뿐이므로 이 PR의 선택 검증 대상이 아니다.

## 시각·fixture 증적

- 원본 fixture: `samples/HWP3-password-123456.hwp`
  - SHA-256: `db743d084efc9e08e839a5b4d978b16b8676434011776e090e4cda43e57304be`
  - 역할: 실제 HWP3 비밀번호 복호화·열기 회귀 fixture
- 실제 Studio 열기 asset:
  [`pr_3483_hwp3_password_open_review.png`](../assets/pr_3483_hwp3_password_open_review.png)
  - SHA-256: `fc1567cb153ef15f86270a61341d9de5347da3903d93fd277bd9ef2f460f4fbf`
  - fresh WASM과 headless Chrome에서 비밀번호 입력 뒤 24쪽 상태·canvas 2개를 확인하고, 로컬 글꼴
    안내 모달을 닫은 뒤 보존한 1쪽 화면이다.

HWP3 parser/open 경로의 구조 보존 기능이며 Hancom 기준 PDF와의 wrap·clipping·margin·pagination
일치를 주장하지 않는다. 따라서 기준 PDF·pixel match·visual sweep은 생성하지 않았고, 이 PNG는
빈 화면이 아닌 실제 문서 열기와 24쪽 결과를 보이는 기능 증적이다. 이미지 확인 결과, 첫 페이지의
제목·본문·인라인 개체·하단 차례가 화면에 나타난다.

## 위험과 후속 범위

- JavaScript 문자열은 언어 차원에서 확정적인 zeroize를 보장하지 않는다. 다만 암호 input은 대화상자가
  닫힐 때 비우고, 호출 이후 참조를 보관하거나 영속 저장소·로그로 전달하지 않는다.
- 새 HWP3 경로의 상한은 **복호화 뒤 raw DEFLATE 결과**에 적용된다. 파일 크기 자체의 운영상 제한은
  기존 업로드·로컬 파일 선택 정책을 따른다.
- HWP3 비압축 암호 payload와 다른 HWP3 암호 방식은 이 PR의 수용 범위 밖이다.

## 최종 권고

**조건부 merge 권고.** 실제 HWP3 fixture, parser·CLI·WASM·Studio, 전체 Rust·frontend·확장 build
검증에서 blocker를 발견하지 못했다. 최종 merge 전에는 review 기록을 포함한 최신 head의 GitHub Actions,
mergeable 상태, 메인터너 검토와 작업지시자 승인을 다시 확인한다.
