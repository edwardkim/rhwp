# PR #1458 리뷰 기록 — 그림 삽입과 인라인 커서 정합 개선

- PR: https://github.com/edwardkim/rhwp/pull/1458
- 작성일: 2026-06-21
- 작성자: collaborator self-merge 후보 경로
- 작성 시점 head: `ce442a0069ea24db807d25a5341e76022fa342e2`
- base: `devel`
- head: `task_m100_1452`

## 1. PR 메타

| 항목 | 확인 내용 |
|------|-----------|
| 작성자 | `jangster77` |
| PR 상태 | open, draft 아님 |
| mergeable | 작성 시점 `MERGEABLE` |
| 관련 이슈 | `Closes #1452` |
| 규모 | 작성 시점 70 files, +3827 / -234 |
| 커밋 수 | 13개 + 본 self-merge review 문서 커밋 예정 |

`mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 이 문서는 작성 시점 값을 참고로만 기록한다.
최종 merge 판단은 merge 직전 최신 PR head 기준으로 다시 확인한다.

## 2. 변경 범위

### 2.1 그림 삽입과 저장

- 외부 파일 드롭 시 한컴처럼 글자처럼 취급되는 인라인 그림으로 삽입한다.
- 드롭 삽입은 원본 크기를 기본으로 사용하되 페이지 폭을 넘지 않도록 자동 축소한다.
- 입력 메뉴의 그림 삽입 동작은 기존처럼 사용자가 배치/드래그하는 흐름을 유지한다.
- 그림 wrap/treat-as-char/위치 속성 변경 시 모델, HWP 저장 attr, HWPX 직렬화가 일관되게 반영되도록 보강했다.

### 2.2 투명도와 렌더링

- PNG 픽셀 알파 보존과 개체 전체 투명도 값을 분리해 모델과 입출력 경로에 연결했다.
- HWP/HWPX alpha 매핑, 렌더 트리, SVG/HTML/Skia/Web Canvas/JSON paint 경로의 그림 투명도 반영을 보강했다.
- 투명도 0/50 샘플을 추가하고, 중복 렌더링 및 문단부호 가시성 문제를 함께 정리했다.

### 2.3 인라인 그림 커서와 편집 동작

- 글자처럼 취급되는 TAC 그림을 문자처럼 다루도록 cursor rect, navigation, hit test, paragraph split 경로를 보강했다.
- 두 그림 앞/사이/뒤에서 Enter, Home/End, 좌우/상하 이동, 조판부호 클릭이 시각 줄 기준으로 동작하도록 보정했다.
- 저장된 커서 위치 복원과 문단부호 렌더링 위치를 한컴 기준에 맞춰 조정했다.
- soft-wrap 경계 offset의 line affinity를 유지해 같은 offset이 다음 줄 시작으로 재해석되는 회귀를 막았다.

### 2.4 Studio UX와 문서

- 개체 속성 창의 탭별 크기 차이를 줄이도록 대화상자 크기를 고정했다.
- 파일 열기 다이얼로그에서 Esc 취소 후 fallback input이 다시 열리는 문제를 수정했다.
- Shift+Tab 내어쓰기 기준을 첫 줄 기준 기대 위치에 맞게 보강했다.
- 계획서와 Stage 1~13 작업 기록을 PR diff에 포함했다.

## 3. 리스크

| 리스크 | 판단 |
|--------|------|
| 커서/navigation 공통 경로 변경 | `issue_1452_saved_caret`, `issue_1139_inline_picture_duplicate`, `issue_1198_nested_cell_paste` 등 회귀 테스트로 핵심 경로를 보강했다. |
| 그림 투명도 renderer별 차이 | 모델/파서/직렬화/렌더러별 반영 지점을 나누고 샘플 fixture로 왕복 및 표시를 확인했다. |
| HWP/HWPX 저장 호환 | attr bit와 HWPX alpha 직렬화 테스트를 추가해 저장 후 속성 보존을 검증했다. |
| 대형 PR 규모 | 작업은 #1452 단일 이슈 범위 안에서 단계별 커밋으로 쪼개져 있으며, 각 stage 문서에 배경과 검증 결과를 남겼다. |

## 4. 검증

PR head `ce442a00` 기준 로컬 검증:

```bash
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo fmt --check
git diff --check
cargo clippy --all-targets -- -D warnings
```

추가 focused 검증과 Studio/WASM 검증은 각 stage 커밋 메시지와 작업 기록에 포함되어 있다.

작업지시자 시각 검증:

- 투명도 0/50 샘플의 한컴 대비 표시
- 외부 이미지 드롭 시 인라인 그림 삽입/자동 축소
- 그림 앞/뒤/사이 커서와 문단부호 위치
- Enter, Home/End, 좌우/상하 이동, 조판부호 클릭 흐름
- 파일 열기 취소 동작

GitHub Actions 작성 시점 확인:

- Build & Test: in progress
- Canvas visual diff: in progress
- CodeQL: in progress
- WASM Build: skipped

본 review 문서 커밋 push 후 GitHub Actions가 다시 실행되므로, merge 전 최신 head 기준으로 위 상태를 재확인한다.

## 5. 판단

작성 시점 기준으로 #1452의 주요 피드백인 그림 투명도, 글자처럼 취급되는 그림 삽입, 인라인 커서/문단부호/줄 이동, 파일 열기 UX, Shift+Tab 내어쓰기 개선이 모두 PR 범위에 포함되어 있다.

최종 조건:

1. 본 review 문서 2건이 PR head에 포함된다.
2. push 후 최신 PR head 기준 GitHub Actions가 통과한다.
3. 작업지시자 승인 상태가 유지된다.

위 조건 충족 시 collaborator self-merge 후보로 merge 수용한다.
