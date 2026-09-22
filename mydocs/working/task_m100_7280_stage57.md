# Task #7280 Stage 57 — 최신 devel 통합과 분리 모듈 반영

- Issue: #7280. 이전: [Stage56](task_m100_7280_stage56.md).
- 승인: PR 생성 전에 devel을 merge하고 baseline 이후 typeset 변경을 흡수한다.
- 시작 head: `933bdb6915bd4652600af20ee0853f476c6693b8`.
- 통합 base: `1966af77fa8046c844d654b157b5168baad8a30e` (`upstream/devel`).
- 원래 구조 비교 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`는 보존한다.
- 상태: 충돌 해결 및 로컬 merge 기록 후 정확한 통합 SHA로 검증 진행.
  원격 push·PR·댓글은 수행하지 않는다.

## 통합 범위와 실제 연결

시작 head 대비 원격 전용 47개 commit을 통합한다. 원래 baseline 이후 원격의
typeset 영역 변경은 root `typeset.rs` 4개 hunk(78행 추가/2행 삭제)다.
Git 충돌은 해당 root 한 파일에서 발생했다. 기존 추출 구조를 보존하고 다음처럼 반영했다.

| 원격 변경 | 통합 위치와 소비 경로 |
| --- | --- |
| #6761 저장 되감김의 쪽 상단/직전 흐름 위치 일치 helper 2개 | root의 기존 저장 위치 helper 옆, 원격 본문 그대로 |
| #6761 채움률 또는 위치 일치 판정 | `paragraph/whole_fit.rs::inspect`; 기존 점유 높이와 평가 순서를 유지 |
| #6761 분할 진입도 동일 되감김 판정 소비 | `WholeFitEvidence` → `WholeFitDecision` → `paragraph/flow.rs` → `place_after_failed_fit` → `split_entry::should_advance` |
| #6656 pagination 예약 높이 유지 설명 | `paragraph/format.rs`; 계산 변경 없이 원격 주석 반영 |

되감김 결과는 Hangul2024 override를 적용한 기존 시점의 bool을 전달하며 분할 진입에서
재계산하지 않는다. overflow 판정은 원격과 같이 `current_height`를, 위치 일치 판정은
`visible_float_exclusions`를 포함한 점유 높이를 사용한다. 신규 상수·예외·공개 API는 없다.
원격의 다른 layout/height_measurer/parser/serializer/Studio/CI/테스트·기준값 변경은 그대로
통합하며 이 작업에서 통과를 목적으로 baseline이나 ignore를 추가 변경하지 않는다.

## 검증 기록

검증 결과는 아래에 보완한다. 이전 제품 `7947ee45f`의 R5 결과는 통합 head의 결과가 아니다.
주 작업트리의 과거 파생 suite에 삭제된 test 참조가 남아 있어 최초 `cargo fmt --all -- --check`가
완주하지 못했다. 파생 파일을 source 변경으로 커밋하지 않고 별도 review worktree에서
현행 `--prepare` 후 다시 검증한다.

전체 제출 gate와 Native/fresh WASM 시각 대조는 통합 후 정확한 head에 대해 별도로 확인해야 한다.
이번 merge만으로 제출 준비 완료나 시각 동등성을 선언하지 않는다.
