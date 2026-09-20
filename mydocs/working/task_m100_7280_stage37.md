# Task #7280 Stage 37 — R2ah 표 옆 문단 흡수 조회·소급 기록 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2ag](task_m100_7280_stage36.md), 시작 head `597e35ddc`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 완료, 고정 제품 SHA 집중 검증 예정. R2 전체/PR 준비 완료가 아니다.

## 1. 책임 분리와 보존 범위

`controls/wrap_absorption.rs`는 매칭된 표 anchor 옆 문단의 전체 흡수 후보와,
이미 fit을 확인한 접두 줄 기록을 반환한다. `WrapAbsorption`은 기존 `WrapAroundPara`와
저장 좌표의 상대 끝점만 담는다. Query에는 원본 문단 참조와 `WrapBand`·DPI만 전달하며
엔진·가변 상태·IR 수정 권한은 주지 않는다.

- 전체 문단: 마지막 저장 줄 cs/sw 판정 → 빈 문단 여부 → 원본 저장 끝점 계산 순서를 유지한다.
  빈 문단은 기록하지만 저장 끝점 확장은 하지 않는다. synthetic 줄 제외, saturating_add,
  최대 끝점, anchor 누락 및 음수/0 차이의 기존 Option 처리를 보존한다.
- 접두 줄: 기존 포맷·1:1 줄 수·전폭 꼬리·가용 높이 검사를 통과한 시점에만 조회한다.
  `take(wrap_prefix_len)`와 배타 end_line의 의미를 유지한다.
- `state::commit_wrap_absorption`: 저장 끝점으로 밴드 확장 → 소급 기록 순서만 연결한다.
  기존 `extend_square_band_to_source_bottom`과 `record_wrap_around_para` 본체는 state로 이동한다.
- 소급 기록은 현재 단의 첫 표 조각을 우선 확인하고, 없으면 확정 페이지/단을 기존 정방향으로
  찾아 최초 일치 단에 기록한다. 찾지 못하면 현재 단에 기록한다. continuation 판별도 그대로다.

실제 소비 경로는 `typeset_wrap_around_paragraph` → Query → commit →
`square_band_bottom` 확장/`wrap_around_paras` 기록이다. 이후 기존 `close_square_band`가
흐름 높이에 반영하고, 조정자는 변경된 높이로 쪽 전환을 재판정한다. 이 후속 순서와
PartialParagraph 발행·전진량·dirty 설정은 변경하지 않는다. 전체 흡수 true/호출자 continue도 유지한다.
기록 값 생성이 확장보다 먼저이지만 값은 상태를 변경하지 않는 scalar/struct이며 확장이 해당 필드를
수정하지 않는다. 기존 `WrapBand` 관측 이후 이 조회 전까지 st 변경은 없다.

조판 규칙 수정이 아닌 동작 보존 리팩토링이다. 새로운 fit 조건·상수·IR/API·테스트/assertion·
baseline/golden/ignore·CI 변경은 없다. prefix/tail 조정과 밴드 종료·쪽 전환의 책임 분리는 후속이다.

## 2. 검증 계획과 한계

정적 대조로 두 Query 계산식/기록 필드·이동한 상태 메서드·확장-before-record를 확인하고,
부모 호출을 원래 블록으로 펼쳐 나머지 경로 불변을 대조한다. 정적 대조는 실행 커버리지나
직접 시각 판정이 아니다.

고정 review worktree에서 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → Stage36과 동일한 323건 focused nextest를 순차 실행한다.
테스트 이름도 baseline 전수 로그의 동일 선택과 대조한다.

주 작업 checkout의 기존 파생 suite가 없는 `issue_7090_sibling_table_occupancy.rs`를 참조해
최초 `cargo fmt --all`이 중단되었다. 제품 결함/검증 통과로 분류하지 않으며 파생 파일을
주 checkout에서 고치지 않고 review worktree의 `--prepare` 뒤 전체 fmt를 재검증한다.

전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM·직접 시각 대조는
구현계획 §7의 책임 묶음/제출 전 통합 게이트에 남긴다. 현재/소급 단의 모든 caption·미주·다단
분기 실행을 주장하지 않는다. 원격 push·PR·댓글은 범위 밖이다.

## 3. 후속

전폭 꼬리의 fit·배치와 매칭 실패의 밴드 종료/쪽 전환 조정 경계를 분리한다.
밴드 종료 전후의 높이를 동일 snapshot으로 고정하지 않는다.
