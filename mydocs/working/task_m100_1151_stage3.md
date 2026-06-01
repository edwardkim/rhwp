# Task #1151 Stage 3 (C) 완료 보고서 — 시각 검증 + 회귀 검증

수행계획서: [task_m100_1151.md](../plans/task_m100_1151.md) · 구현계획서: [task_m100_1151_impl.md](../plans/task_m100_1151_impl.md) · Stage 1: [task_m100_1151_stage1.md](task_m100_1151_stage1.md) · Stage 2: [task_m100_1151_stage2.md](task_m100_1151_stage2.md)

## 1. 사용자 시각 검증

dev server (http://localhost:7700) 에서 사용자가 직접 시나리오 확인:

1. 신규 문서 → 표 (3×3) 생성
2. 셀 안 커서 → 이미지 삽입
3. 결과: 이미지가 셀 영역에 floating 으로 정상 배치
4. **삽입 직후 셀 클릭 → cursor 이동 정상** (#1151 핵심 결함 해소)

사용자 확인: "이게 정답이였음. 잘 된다"

## 2. 자동 회귀 (Stage 1 시점 + 본 단계 재확인)

- `cargo test --lib`: 1425 passed, 0 failed, 6 ignored
- `cargo test --tests`: 통합 테스트 그룹 모두 통과
- `cargo clippy --lib -- -D warnings`: 무경고
- `cargo fmt --all -- --check`: clean
- WASM 빌드 (docker compose): success
- rhwp-studio `npx tsc --noEmit`: 본 작업 관련 에러 0 (사전 canvaskit 에러만 잔존, 무관)

## 3. v1 대비 v2 (현재) 차이

v1 (cell_path 로 inline 셀 내부 삽입) 은 한컴 패턴과 불일치 + 셀 클릭 무반응 발생. v2 는 한컴 incellpicture.hwp 와 정합:

| 항목 | v1 (실패) | v2 (현재) |
|---|---|---|
| 셀 안 picture 구조 | inline 셀 내부 (tac=true) | floating sibling (tac=false) |
| 위치 결정 | 셀 paragraph 내부 | 표 같은 paragraph 의 sibling, horz/vert_rel_to=Page + offset |
| 셀 자체 상태 | picture 가 들어감 | 비어있음 (한컴 정합) |
| 셀 클릭 | 무반응 (picture 선택 모드) | cursor 진입 정상 ✓ |

## 4. 후속 이슈로 분리 보류 사항

본 PR scope 외 (별도 이슈 후보):

- 한컴 Automation `InsertPicture` 의 `sizeoption`/`reverse`/`watermark`/`effect` 옵션 — #194 에서 제안된 사양
- Inline 셀 picture (tac=true, samples/pic-in-table-01.hwp page 22) 의 "클릭 시 cursor 진입 안 됨" 본래 동작 — UX 개선 여지 있으나 본 task 결함과 별개
- 중첩 표 (N-depth) 셀 안 picture 삽입 — 현재는 1-level cell_path 우선

## 5. PR 준비

- 모든 단계 보고서 + 최종 보고서 작성 완료
- 빌드 / 테스트 / clippy / fmt 통과
- 한컴 정합 + 사용자 시각 검증 통과

→ push + PR 생성 가능.
