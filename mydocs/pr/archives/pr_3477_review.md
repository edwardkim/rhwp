---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-27
---

# PR #3477 리뷰 — Studio HWP5 암호 문서 열기

- PR: [#3477](https://github.com/edwardkim/rhwp/pull/3477)
- Issue: [#3474](https://github.com/edwardkim/rhwp/issues/3474)
- 역할: `jangster77` collaborator self-review

## 라우팅과 작성 시점

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md, visual_fixture_evidence.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md
current head: 00c1f361ab5b893e955961c367fc66822a4fb598 (문서 작성 전 최초 PR head 참고값)
```

이 PR은 collaborator 자신의 변경이므로 GitHub self-review 요청으로 독립 승인을 대체하지 않는다. 최종
merge 판단은 이 문서와 filename 보정 commit을 포함한 **최신 head**의 required check, mergeable 상태,
메인터너 검토 및 작업지시자 승인을 다시 확인한 뒤에 한다.

## PR metadata (작성 시점 참고값)

| 항목 | 값 |
| --- | --- |
| 작성자·검토 기록 작성자 | `jangster77` (collaborator self PR) |
| base → head | `devel` → `task/3474-hwp5-password-dialog` |
| 최초 PR head | `00c1f361ab5b893e955961c367fc66822a4fb598` |
| 최초 규모 | 10 files, +475 / -24 |
| mergeable / merge state | `MERGEABLE` / `BLOCKED` (CI 진행 중인 참고값) |
| 관련 이슈 | `Closes #3474` |

## 변경 범위 판정

1. `WasmBridge`가 일반 열기와 암호 열기 모두에서 다음 문서의 파싱·편집 준비·파일명 설정을 성공시킨 뒤에만
   이전 문서를 해제하도록 원자화했다. 따라서 암호 필요·오답·손상에서는 기존 문서가 유지된다.
2. Studio는 HWP5 `EncryptVersion 4`의 명시적 암호 필요 신호에서만 암호 입력 대화상자를 열고,
   확인·취소·Enter·label·modal ARIA·오답 재입력을 제공한다. 비밀번호는 열기 호출의 지역 변수로만 쓰며
   URL·최근 문서·local/session storage·문서 메타데이터에 전달하지 않는다.
3. 실제 HWP5 암호 fixture로 취소·오답·성공·storage 비보존을 headless Chrome에서 검증하는 E2E와
   manifest 항목을 추가했다.
4. 기존 HWP5와 새 HWP3 암호 fixture의 이름을 `-password-123456.hwp` 형식으로 통일했다. HWP3 fixture는
   향후 `src/parser/hwp3/` 복호화 회귀용 보관물일 뿐, 이번 PR은 HWP3 복호화·열기 지원을 주장하지 않는다.
5. `CHANGELOG.md`, README, parser HWP3 구현, 공개 API, CI workflow, golden/baseline TSV는 바꾸지 않았다.

## 요구사항 대조

| #3474 수용 기준 | 검토 결과 |
| --- | --- |
| 암호 HWP5 선택 시 대화상자 | 명시적 HWP5 암호 필요 오류만 `showHwpPasswordDialog()`으로 전환하도록 확인 |
| 정상 입력 시 실제 열기·렌더 | 실제 fixture 브라우저 E2E가 HWP source·64쪽·canvas 준비까지 확인 |
| 오답·취소 시 부분 교체·노출 없음 | 원자적 bridge 교체, 취소·오답 E2E, storage 비보존 검증으로 확인 |
| 비암호 열기 회귀 없음 | 일반 열기를 먼저 시도하는 기존 경로를 보존하고 Studio 단위 계약·전체 npm test·CI frontend gate로 확인 |
| 미지원 암호 방식·DRM | 암호 필요 신호 외 오류는 대화상자로 숨기지 않고 기존 명시 오류를 유지하도록 확인 |

## 검증 기록

| 검증 | 결과 | 판정 |
| --- | --- | --- |
| `node --test tests/hwp-password-open.test.ts` | 4 passed | 암호 UI 전환·원자성·입력 비보존 정적 계약 |
| `npm test` | 674 passed, 0 failed | Studio 전체 회귀 |
| `npm run build` | passed | Studio production build |
| `VITE_URL=http://127.0.0.1:7714 npm run e2e:hwp-password-open` | passed | 실제 HWP5 fixture의 취소·오답·Enter 성공·storage 비보존 |
| `npm run e2e:manifest-check` | 78 tracked / 78 manifest, 이상 없음 | E2E 등록 완결성 |
| `CARGO_TARGET_DIR=target/review-3477-hwp5-password-dialog CARGO_INCREMENTAL=0 cargo test --profile release-test --test hwp5_password_fixture` | 2 passed | 이름 변경 뒤 실제 HWP5 fixture·CLI 계약 |
| `RHWP_IR_SWEEP_DUMP=/tmp/ir_field_sweep_3477_renamed_20260727.tsv CARGO_TARGET_DIR=target/review-3477-hwp5-password-dialog CARGO_INCREMENTAL=0 cargo test --profile release-test --test ir_field_sweep_baseline -- --nocapture` | 2 passed; 803 samples (3 skipped), 671 paths, 110345 records | 신규·이름 변경 HWP fixture baseline 규칙 |
| `diff -u tests/fixtures/ir_field_sweep_baseline.tsv /tmp/ir_field_sweep_3477_renamed_20260727.tsv` | no output | 새 비영 왕복 발산 없음; baseline TSV 갱신 불필요 |

Cargo 검증은 다른 작업과 공유하지 않도록 `target/review-3477-hwp5-password-dialog`과
`CARGO_INCREMENTAL=0`을 사용해 순차 실행했다. PR head의 GitHub CI는 문서 작성 시점에 build archive,
Native Skia, CodeQL Rust가 진행 중이므로, 위 로컬 성공으로 최신 head CI를 대체하지 않는다.

## 시각·fixture 판정

Studio의 열기 UI와 bridge 상태 전이 변경이며 renderer/layout/paint·페이지 geometry를 변경하지 않는다.
실제 HWP5 fixture E2E는 canvas가 준비되고 64쪽이 열리는 기능 검증에 사용했지만, 기준 PDF 대비
wrap·clipping·margin·pagination의 시각적 개선을 주장하지 않는다. 따라서 visual sweep·기준 PDF·대표 PNG는
이 PR의 merge 판단 근거가 아니며 생성하지 않는다.

새 HWP3 fixture는 복호화 지원 전의 향후 회귀용 보관물로 렌더 결과가 없다. 이에 대해 visual evidence를
꾸미지 않고, 신규 fixture 필수 절차인 IR field-sweep baseline 비교로 등록 영향을 확인했다.

## 위험과 후속 범위

- JavaScript 문자열은 언어 차원에서 확정적인 zeroize가 불가능하다. 다만 암호 참조를 시도 직후 비우고,
  대화상자를 닫을 때 DOM input도 비우며, 영속·로그 경로로 전달하지 않는다. 이 PR의 E2E는 browser storage
  비보존을 고정한다.
- HWP3 암호 문서는 아직 지원하지 않는다. 복호화 명세가 확보된 별도 작업에서 parser HWP3 경로와 fixture
  회귀를 추가해야 한다.

## 최종 권고

**조건부 merge 권고.** 코드·fixture 이름 변경·로컬 검증에서 blocker는 발견하지 못했다. 다만 최종 merge
조건은 review 기록을 포함한 최신 head에서 GitHub Actions가 모두 성공하고, 메인터너 검토와 작업지시자 승인을
받는 것이다. 그 전에는 draft/merge 가능 여부 및 최초 head의 CI 상태를 최종 사실로 사용하지 않는다.
