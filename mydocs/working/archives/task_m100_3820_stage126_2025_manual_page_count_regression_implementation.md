# Stage 126 - 2025 행정업무운영 편람 페이지 수 회귀 복원

## 목표

동일한 HWP/HWPX fixture에서 현재 renderer의 `HWP 393쪽`, `HWPX 386쪽`을 모두 `383쪽`으로 복원한다. 기준은 Hancom Office 2020 PDF와 `07555d200` renderer binary의 교차 실행 결과다.

## 입력 및 기준선

- HWP: `samples/2025 행정업무운영 편람(최종).hwp`
- HWPX: `samples/2025 행정업무운영 편람(최종).hwpx`
- 기준 PDF: `pdf/2025 행정업무운영 편람(최종)-2024.pdf`, 383쪽
- 현재 입력 SHA-256은 `07555d200` worktree의 동명 fixture와 일치한다.
- `07555d200` binary는 현재 두 입력 모두 383쪽을 출력한다.

## 구현 전 분석 절차

1. section별 페이지 수와 103행 x 2열 `RowBreak` 표의 row cut을 old/current에서 추출한다.
2. 직접 HWP와 HWPX 모두에 공통으로 작동하는 page ownership 또는 row split 정책 차이를 찾는다.
3. source format이나 특정 문단 index가 아니라 저장된 표 계약과 fragment 경계만을 조건으로 하는 최소 규칙을 구현한다.

## 수용 검증

1. focused HWPX/HWP 저장 layout 회귀 테스트가 통과한다.
2. HWP와 HWPX의 `dump-pages` 결과가 각각 383이다.
3. 103행 규정표의 fragment 수와 row cut이 383쪽 기준선에 수렴한다.
4. 수정 결과와 남은 시각 차이는 Stage 126 결과 절에 기록하고, 문서와 코드 및 테스트를 한 커밋으로 고정한다.

## 후보 A - RowBreak visible-tail grace의 저장 계약 복원

- `b40d535dd` 직전 renderer는 현재 HWP/HWPX fixture에서 모두 383쪽이다.
- `b40d535dd`는 `RowBreak` 셀 단위 cut에서 overflow 가시 줄 뒤에 spacer가 하나라도 있는지 검사하던 `any(empty_spacer)`를, 뒤 전체가 spacer인지 검사하는 `all(empty_spacer)`로 변경했다.
- 현재 구현은 spacer run 뒤의 가시 본문·중첩 조각까지 추가 판별하는 세 번째 helper를 사용한다. 현재 fixture에서는 HWP 393, HWPX 386으로 과다 분할한다.
- 후보 A는 소스 형식, 문단 index, fixture 이름을 보지 않고 저장 RowBreak cell unit의 `empty_spacer` 계약만 사용한다. 먼저 격리 worktree에서 단순 `any(empty_spacer)`를 실행해 두 입력의 page count와 기존 focused 회귀를 확인한다.
- 결과가 383/383이 아니면 이 후보는 폐기하고, 그 수치와 section별 증감을 Stage 126 결과에 남긴다.

## 후보 A 결과 - 폐기

- 최신 renderer와 동일한 분리 worktree에서 `grace_visible_tail_before_spacer`를 historical `any(empty_spacer)`로 바꿔 실행했다.
- 결과는 HWP 393쪽, HWPX 386쪽으로 완전히 불변이었다.
- 따라서 현재 fixture의 초과 분할은 visible-tail grace 경로가 아니라 그 이전의 저장 vpos/frame 경계 또는 row-cut 소유 경로에서 발생한다. 후보 A는 본 작업 트리에 적용하지 않는다.

## 실행 이력으로 고정한 회귀 경계

| renderer 상태 | HWP | HWPX | 판정 |
| --- | ---: | ---: | --- |
| `07555d200` | 383 | 383 | 기준선 |
| `b40d535dd^` | 383 | 383 | 기준선 |
| `b40d535dd` | 383 | 381 | visible-tail 정책 단독 변화 |
| `0f155ea8f` | 383 | 383 | #1686 통합 전 기준선 |
| `1048383e2` | 383 | 383 | co-anchored RowBreak 보정 단독 상태 |
| `445130197` | 383 | 383 | HWP RowBreak p5 보정 단독 상태 |
| `3a531a38f` | 384 | 382 | #1722 merge 결합 뒤 첫 변화 |
| 현재 | 393 | 386 | 후속 회귀 누적 |

#1722 merge는 두 부모의 RowBreak 정책을 결합하면서 처음 `384 / 382`로 이동했다. 이후의 개별 feature를 일괄 되돌리지 않고, 현재 row-cut에서 저장 frame reset이 행/문단 경계로 승격되는 조건과 fragment tail 소유 조건을 분리 계측한 뒤 구현한다.

## 후보 B - 저장 프레임 강제 경계 해제

- `#4069`가 추가한 `strict_saved_frame_break`를 두 RowBreak 컷 경로에서 모두 비활성화한 임시 renderer를 전용 target으로 빌드했다.
- `dump-pages`의 첫 줄과 `=== 페이지` 헤더를 기준으로 HWP 393쪽, HWPX 386쪽을 확인했다.
- 이 규칙은 1x1 중첩 표의 authoritative 저장 프레임만 엄격하게 보존하며, 이번 직접 표의 초과 페이지에는 영향이 없다.
- 규칙을 제거하면 기존 중첩 표 계약만 약화하므로 후보 B는 폐기한다. 본 작업 트리에는 적용하지 않는다.

## 다음 분해 대상

최초 변화가 발생한 `3a531a38f` 병합(`#1722`)의 양 부모와 merge base를 같은 입력으로 각각 계측한다. 이후 직접 RowBreak 셀에서 cut 원장이 달라지는 경로만 다음 구현 후보로 제한한다.

## 후보 C - co-anchored RowBreak 표 지연 배치 해제

- `#1686`이 도입한 지연 큐만 환경 gate로 해제한 임시 renderer를 실행했다.
- 결과는 HWP 393쪽, HWPX 386쪽으로 불변이었다.
- `#1686` 병합은 최초의 `384 / 382` 변화와 시간상 연결되지만, 현재 fixture의 초과 페이지에는 직접 기여하지 않는다. 후보 C는 폐기한다.

## 후보 D - RowBreak 셀 local vpos origin의 과거 조건 복원

- 현재 `cell_has_local_vpos_origin`이 허용하는 `0..=500` 시작 vpos를 제거하고, 과거처럼 정확히 `0`일 때만 local origin으로 인정한 임시 renderer를 실행했다.
- 결과는 HWP 393쪽, HWPX 386쪽으로 불변이었다.
- 다만 section 10의 `pi=4` 1x1 표 cut은 `27/60/94`에서 `33/65`로 바뀌어 fragment가 4개에서 3개로 줄었다. 그 대신 표 시작이 한 쪽 늦어져 총쪽수는 상쇄됐다.
- 이 조건은 단독 복원으로는 수용 기준을 만족하지 못한다. 후보 D는 구현안으로 채택하지 않는다.

## 다음 분해 대상

HWP가 `390 -> 393`으로 증가한 `193cd714d`를 다음 회귀 경계로 고정한다. 이 커밋의 row-cut 변경을 현재 fixture의 section 7 및 section 10 초과와 대조한다.

## 후보 E - 1x1 RowBreak object-height advance 해제

- `193cd714d`가 추가한 단일 행 object의 declared-height advance를 환경 gate로 해제했다.
- 총쪽수는 HWP 393쪽, HWPX 386쪽으로 불변이었다. 그러나 section 10 `pi=4`는 `27/60/94`에서 `29/62`로 바뀌어 4개 fragment가 3개로 줄었다.
- 이 후보도 표의 시작 page를 한 쪽 늦추는 상쇄가 있어 총쪽수 기준으로 채택할 수 없다. object height와 실제 fragment flow를 분리해 다시 설계해야 하며, 단순 해제는 폐기한다.

## 다음 분해 대상

`2edd0bd03`에서 HWP는 `384 -> 390`, HWPX는 `382 -> 387`로 함께 증가했다. 두 형식에 공통인 이 6쪽/5쪽 회귀 묶음의 RowBreak·stored-frame 변경을 다음 구현 후보로 분해한다.

## 구현

`src/renderer/typeset.rs`에서 1x1 `RowBreak` object-height advance를 `HWP5-origin HWPX`로 한정했다. 이 advance는 #1891의 HWPX 저장 object-height 보정이지만, native HWP에까지 적용하면 실제 fragment 소비량 대신 남은 페이지 전체를 flow로 예약한다.

native HWP는 기존 RowBreak fragment의 실제 소비량으로 다시 조판한다. HWP5-origin HWPX의 저장 object-height 계약은 `profile.hwp5_origin_hwpx()` gate 안에 남겨 기존 보정 범위를 보존한다.

## 결과

- native HWP section 10 `pi=4` 1x1 표는 `27/60/94`의 4 fragment에서 `29/62`의 3 fragment로 줄었다.
- HWP 전체는 아직 393쪽, HWPX 전체는 386쪽이다. PDF 기준 383쪽과 비교하면 각각 +10쪽, +3쪽이 남아 있다.
- `CARGO_TARGET_DIR=target/stage124-3820 cargo build --profile release-test --quiet`가 통과했다.
- `CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test --test issue_1891 --quiet`가 4 passed로 통과했다.

이번 Stage는 native HWP에 잘못 확장된 HWPX 전용 advance를 제거해 p4 fragment 계약을 개선했지만, 전체 페이지 수 수용 기준은 아직 충족하지 못했다. 다음 Stage는 이 커밋을 기준으로 남은 section 7의 1쪽과 section 10의 9쪽, HWPX의 3쪽을 별도 원인으로 분리한다.
