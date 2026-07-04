# Task 1293 구현 계획서

## 원칙

증상별 y/gap 보정 추가를 중단하고, 한컴 공식 `미주 모양` 의미를 먼저 IR과 렌더 계산식에
반영한다. 기존 바이너리 슬롯은 라운드트립 보존을 위해 유지하되, 렌더러는 정규화된 의미
접근자를 사용한다.

## 1단계: 정규화 모델 도입

- `FootnoteShape`에 공식 UI 의미를 반환하는 접근자를 추가한다.
  - `separator_above_margin_hu()`
  - `separator_below_margin_hu()`
  - `between_notes_margin_hu()`
- 기존 필드명과 실제 의미가 다른 부분은 주석을 정정한다.
- HWP5/HWPX 원본 슬롯 보존 필드는 유지한다.

## 2단계: 파서 의미 검증

- HWP5 샘플에서 한컴 UI 값과 내부 정규화 값이 일치하는지 테스트한다.
- HWPX `<hp:noteSpacing>`의 `betweenNotes`, `belowLine`, `aboveLine`이 정규화 접근자에서
  공식 의미대로 노출되는지 테스트한다.
- `구분선 위` 20mm 샘플을 기준 테스트에 추가한다.

## 3단계: 타입셋/렌더 계산식 통일

- `typeset.rs`의 미주 separator 생성과 between-notes 계산을 정규화 접근자로 교체한다.
- `layout.rs`의 separator 렌더도 같은 정규화 값을 사용한다.
- `height_cursor.rs`의 `endnote_between_notes_hu` 전달 구조가 공식 `미주 사이` 값만 받도록 정리한다.
- 각주 전용 경로(`picture_footnote.rs`, `height_measurer.rs`)도 같은 접근자를 사용하되,
  각주/미주의 배치 차이는 별도 분기로 남긴다.

## 4단계: visual sweep 보강

- sweep 결과에 미주 모양 정규화 값이 함께 기록되도록 한다.
- `구분선 위`, `구분선 아래`, `미주 사이` 샘플을 별도 target으로 비교한다.
- overlap 후보뿐 아니라 separator line과 첫 미주 내용 사이의 실제 거리도 측정한다.

## 5단계: 기존 보정 재평가

- PR #1292의 stage별 보정 커밋을 새 모델 기준으로 다시 검토한다.
- 공식 모델로 사라지는 보정은 제거한다.
- 여전히 필요한 보정은 원본 LINE_SEG 재구성 문제인지, 미주 모양 설정 문제인지 분리한다.

## 검증 계획

- focused:
  - `cargo test --test issue_1139_inline_picture_duplicate`
  - `cargo test --test issue_1050_footnote_serialize`
  - `cargo test --lib compact_endnote`
- visual:
  - `python3 scripts/task1274_visual_sweep.py --target all`
  - 구분선 위/아래/미주 사이 샘플별 PDF/PNG 직접 비교
- PR CI 전체 테스트 - 작업지시자 명시 승인 후에만:
  - `cargo fmt --all -- --check`
  - `cargo build --verbose`
  - `cargo test --verbose`
  - `cargo clippy -- -D warnings`

전체 CI 성격의 PR 전 검증은 시간이 길고 작업지시자의 검토 타이밍과 맞춰야 하므로,
visual sweep과 focused test가 완료된 뒤 작업지시자 승인을 받은 경우에만 실행한다.
작업 전체에 대한 자동 승인 또는 `/Goal` 자동 진행 지시가 있어도 PR CI 전체 테스트 승인을
대체하지 않으며, PR CI는 별도 명시 승인이 필요하다.
따라서 PR 준비 단계에서 위 네 명령을 실행하기 전에는 반드시 작업지시자에게 현재 focused
검증과 시각 sweep 결과를 공유하고, "PR CI 실행 승인"을 별도로 확인한다.

## 승인 대기

이 계획은 아직 구현 승인 전이다. 소스 수정은 작업지시자 승인 후 진행한다.
