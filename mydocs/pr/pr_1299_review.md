# PR #1299 리뷰 — 0-length field range fieldBegin/fieldEnd 순서 보정

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | #1299 |
| 제목 | Task #1298 (#1289 후속 작업) - fix: HWPX 시리얼라이저 0-length field range fieldBegin/fieldEnd 인터리빙 수정 |
| 작성자 | Martinel2 |
| 대상 이슈 | #1298 |
| 대상 브랜치 | `devel` |
| 검토 기준 | `local/pr1299-upstream` |

## 2. 변경 범위

변경 파일:

| 파일 | 판단 |
|---|---|
| `src/serializer/hwpx/section.rs` | 실제 기능 수정 및 단위 테스트 |
| `mydocs/plans/task_m100_1298.md` | 기여자 작업 문서 |
| `mydocs/plans/task_m100_1298_impl.md` | 기여자 작업 문서 |
| `mydocs/working/task_m100_1298_stage1.md` | 기여자 작업 문서 |
| `mydocs/report/task_m100_1298_report.md` | 기여자 작업 문서 |

코드 변경은 `render_run_content` 내부의 fieldEnd 방출 위치 보정이다.

## 3. 문제 구조

기존 HWPX serializer는 `fieldEnd`를 `end_char_idx == next_idx` 기준으로 문자 처리 후 방출했다.
이 방식은 일반 필드 범위에서는 자연스럽지만, `start_char_idx == end_char_idx`인 0-length field에서는 두 가지 문제가 생긴다.

| 케이스 | 기존 출력 문제 | 기대 출력 |
|---|---|---|
| `start=0, end=0` | `fieldEnd`가 문단 텍스트 뒤로 밀림 | `fieldBegin` 직후 `fieldEnd` |
| `start=N, end=N` | `fieldEnd`가 `fieldBegin`보다 먼저 방출될 수 있음 | `fieldBegin` 직후 `fieldEnd` |

PR은 slot 방출로 `fieldBegin`을 먼저 내보낸 뒤, 문자 push 전에 0-length field 전용 `fieldEnd`를 즉시 방출하도록 한다. 기존 post-char 검사에는 `start_char_idx < end_char_idx` guard를 추가해 0-length field의 역순 방출을 막는다.

## 4. 검증 결과

PR head(`local/pr1299-upstream`)에서 직접 실행:

```text
cargo fmt --all -- --check
통과

cargo test --lib -- serializer::hwpx::section::tests
14 passed

cargo clippy --lib -- -D warnings
통과

cargo test --lib serializer::hwpx -- --nocapture
81 passed
```

## 5. 검토 의견

수정 방향은 타당하다. `fieldBegin`은 slot 시스템에서 이미 문자 앞에 방출되므로, 0-length field의 `fieldEnd`도 같은 문자 앞에서 즉시 닫아야 한다.

추가된 테스트는 다음 두 회귀를 직접 고정한다.

- 문단 시작 0-length field: `fieldBegin < fieldEnd < text`
- 문단 중간 0-length field: `ABC < fieldBegin < fieldEnd < DE`

다만 PR 본문에도 적혀 있듯이, 빈 문단(`text == ""`)의 0-length field는 아직 범위 밖이다. 현재 루프가 실행되지 않으면 post-loop 처리 순서상 `fieldEnd`가 slot 기반 `fieldBegin`보다 먼저 나올 수 있다. 실사용 빈도는 낮아 보여 이번 PR의 blocker로 두지는 않되, 후속 이슈 후보로 남기는 것이 좋다.

## 6. 권장 처리

권장안: **수용**

- 코드 변경은 좁고 테스트로 근거가 있다.
- HWPX serializer 전체 관련 테스트와 clippy가 통과했다.
- 단, 기여자 `mydocs/*` 작업 문서는 프로젝트 운영 문서 흐름과 충돌할 수 있으므로, 실제 반영은 코드 커밋 중심 cherry-pick을 권장한다.

권장 절차:

1. `local/devel` 기준 통합 브랜치 생성
2. PR의 코드 변경 커밋만 cherry-pick 또는 동일 패치 적용
3. `cargo fmt --all -- --check`
4. `cargo test --lib -- serializer::hwpx::section::tests`
5. `cargo test --lib serializer::hwpx -- --nocapture`
6. `cargo clippy --lib -- -D warnings`
7. 결과 보고서 작성 후 승인 게이트

## 7. PR 코멘트 초안

```markdown
검토했습니다. 0-length field range에서 `fieldBegin`/`fieldEnd`가 역순이 되거나 `fieldEnd`가 텍스트 뒤로 밀리는 문제를 정확히 좁혀 처리한 것으로 확인했습니다.

로컬에서 다음 검증을 통과했습니다.

- `cargo fmt --all -- --check`
- `cargo test --lib -- serializer::hwpx::section::tests` (14 passed)
- `cargo test --lib serializer::hwpx -- --nocapture` (81 passed)
- `cargo clippy --lib -- -D warnings`

빈 문단의 0-length field는 아직 후속 과제로 남길 수 있어 보이지만, 이번 PR 범위에서는 blocker로 보지 않겠습니다. 감사합니다.
```
