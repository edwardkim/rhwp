# PR #1419 리뷰 - 누름틀 양식 모드와 경계 편집 구현

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | #1419 |
| 제목 | task 258: 누름틀 양식 모드와 경계 편집 구현 |
| 작성자 | jangster77 |
| 관련 이슈 | #258 한글 누름틀 + 양식 모드 구현 요청 |
| base | `devel` |
| head | `jangster77:task_m100_258` |
| draft | false |
| mergeable | `MERGEABLE` |
| 초기 merge state | `BLOCKED` - GitHub Actions 진행 중 |
| 초기 head | `a063ad49` |
| 문서 커밋 | 현재 PR head의 리뷰 문서 커밋 |
| 변경량 | 60 files, +5143 / -185 |

PR 본문에 `Closes #258`가 포함되어 있으므로 merge 시 이슈 자동 close 대상이다.

## 2. 변경 범위

핵심 변경:

- `src/document_core/queries/field_query.rs`
  - ClickHere 필드의 값/범위/editable 속성 조회와 갱신 계약 보강
- `src/document_core/commands/text_editing.rs`, `clipboard.rs`
  - 누름틀 경계 입력, 삭제, 복사/붙여넣기, field range 재정규화 처리
- `rhwp-studio/src/engine/*`
  - 양식 모드 입력 제한, 누름틀 내부/외부 경계 이동, 선택 표시, Home/End 보정
- `rhwp-studio/src/ui/*`
  - 누름틀 삽입/편집 대화상자 구현과 modal 바깥 클릭 닫힘 방지
- `src/serializer/hwpx/section.rs`, `src/wasm_api.rs`
  - HWPX/HWP5 editable 보존과 WASM API 노출
- `tests/issue_258_clickhere_form_mode.rs`
  - 누름틀 삽입, 저장 라운드트립, 입력/삭제/선택/복사/붙여넣기 회귀 테스트 12건 추가
- `samples/누름틀-2024.*`, `pdf/누름틀-2024.pdf`
  - 한컴 동작 대조용 샘플 추가

## 3. 시각 검증 반영 사항

작업지시자가 다음 동작을 실제 브라우저/한컴 기준으로 확인했다.

- 누름틀 삽입 후 guide 표시와 첫 입력 렌더링
- 누름틀 삭제 확인창과 확인 후 실제 삭제
- 누름틀 내부/외부 방향키 경계 이동
- 인접 누름틀 2개 동시 선택
- 선택 highlight 색상 회귀 보정
- `[123][123]` 복사/붙여넣기 후 두 누름틀 모두 표시
- `Home`/`End`로 누름틀 바깥 줄 시작/끝 이동

## 4. 로컬 검증

PR 준비 단계 Stage30에서 전체 로컬 검증을 완료했다. 작업지시자 지시에 따라 PR 문서 커밋 시점에는 로컬 테스트를 재실행하지 않았다.

| 명령 | 결과 |
|---|---|
| `git diff --check` | 통과 |
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | 통과, 1824 passed / 6 ignored |
| `cargo test --profile release-test --tests` | 통과 |
| `cargo fmt --check` | 통과 |
| `wasm-pack build --target web --out-dir pkg` | 통과 |
| `cd rhwp-studio && npm run build` | 통과, Vite chunk size 경고만 발생 |

## 5. GitHub Actions

PR 생성 직후 상태:

| 체크 | 상태 |
|---|---|
| Build & Test | pending |
| Canvas visual diff | pending |
| Analyze (rust) | pending |
| Analyze (javascript-typescript) | pending |
| Analyze (python) | pending |
| WASM Build | skipping |

문서 커밋 push 후 GitHub Actions가 다시 실행되므로, 최종 merge 판단은 재실행된 checks 통과 후 진행한다.

## 6. 리스크

| 항목 | 평가 |
|---|---|
| 변경량 | 큼. 다만 단계별 커밋과 회귀 테스트가 함께 포함됨 |
| 편집 경계 회귀 | 중간. 누름틀 내부/외부 caret 상태가 복잡하므로 Stage20~29에서 집중 보정 |
| 일반 본문 편집 영향 | 낮음. 양식 모드 OFF 기존 동작과 일반 선택 회귀를 별도 확인 |
| HWP/HWPX 저장 호환성 | 낮음. editable 속성 보존 테스트와 샘플 라운드트립 포함 |
| 후속 범위 | 사용자 정보/문서 요약/날짜/파일명 탭, 양식 개체 전체 상호작용은 후속 분리 |

## 7. 최종 권고

GitHub Actions 재실행 통과 후 merge 가능으로 판단한다.

권고 순서:

1. PR 리뷰 문서와 오늘할일 문서 커밋을 PR head에 포함
2. PR diff에 `mydocs/pr/pr_1419_review.md`, `mydocs/pr/pr_1419_review_impl.md`, `mydocs/orders/20260616.md` 포함 확인
3. GitHub Actions 완료 대기
4. 모든 required check 통과 시 merge
5. #258 close 여부 확인
6. `local/devel`/`upstream/devel` 동기화
7. PR 문서를 `mydocs/pr/archives/`로 이동
