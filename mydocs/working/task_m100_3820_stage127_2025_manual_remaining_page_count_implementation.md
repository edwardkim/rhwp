# Stage 127 - 2025 행정업무운영 편람 잔여 페이지 수 분리

## 목표

Stage 126 커밋 `de5705d27`을 기준으로 남은 HWP 393쪽, HWPX 386쪽을 Hancom PDF 기준 383쪽으로 수렴시킨다.

## 확정 기준선

- PDF: 383쪽
- native HWP: 393쪽, 잔여 +10쪽
- HWPX: 386쪽, 잔여 +3쪽
- HWP section 차이: section 7 +1, section 10 +9
- HWPX section 차이: section 3 -1, section 10 +6, section 11 -2

## Stage 126 이후 보존 계약

- HWP5-origin HWPX의 1x1 object-height advance는 계속 보존한다.
- native HWP 1x1 `RowBreak` 표는 실제 fragment flow를 사용하며, section 10 `pi=4`는 3 fragment를 유지한다.
- source format, 문단 index, fixture 이름으로 분기하지 않는다.

## 구현 순서

1. section 7 `pi=239`의 3x7 rowspan fragment와 PDF/old renderer의 cut 소유를 비교한다.
2. 원인이 확인되면 본 작업 트리의 RowBreak row/block cut 규칙을 최소 범위로 수정한다.
3. HWP와 HWPX의 전체 쪽수, 해당 표의 fragment cut, focused 회귀 결과를 이 문서에 기록한다.
4. 코드·테스트·결과 문서를 한 커밋으로 고정한 뒤에만 다음 Stage를 시작한다.

## 수용 기준

1. HWP와 HWPX 모두 383쪽이다.
2. 수정된 RowBreak 규칙은 source format별 상수가 아니라 저장 row/block 계약을 근거로 한다.
3. focused 회귀 테스트와 production build가 통과한다.
4. 남은 시각 차이가 있으면 PDF 대조 근거와 함께 결과 절에 명시한다.

## 구현

section 7 `pi=235`의 23x3 native HWP `RowBreak` 표는 첫 fragment에 남은 598.9px보다 다음 저장 행이 1.3px만 커서, 현재 renderer가 row 13 앞에서 조기 이월하고 있었다. old renderer와 Hancom 저장 행 계약은 그 미세 HU 반올림 초과를 현재 fragment에 유지한다.

`typeset.rs`의 일반 행 fit에 native HWP5 전용 2px rounding 허용을 추가했다. 이 규칙은 다음 조건을 모두 요구한다.

- native HWP5 profile
- `RowBreak` 표
- 첫 행이 아닌 완전 행
- 시작 cut 없음
- strict painted-bottom fit 미진입

HWPX의 64px drift 허용 및 중첩/부분 행의 물리 cut 정책은 적용 대상에서 제외했다.

## 결과

- `pi=235` 첫 fragment가 rows `0..13`에서 `0..14`로 복원됐고 continuation은 `14..23`이 됐다.
- HWP section 7은 53쪽에서 PDF/old renderer와 같은 52쪽으로 복원됐다.
- HWP 전체는 393쪽에서 392쪽으로 감소했다. section 10의 +9쪽이 남아 PDF 383쪽보다 +9쪽이다.
- HWPX는 386쪽으로 불변이며 PDF 대비 +3쪽이다.
- `CARGO_TARGET_DIR=target/stage124-3820 cargo build --profile release-test --quiet`가 통과했다.
- `CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test --test issue_1891 --quiet`가 4 passed로 통과했다.

전체 383쪽 수용 기준은 아직 충족하지 못했다. 다음 Stage는 HWP section 10의 +9쪽과 HWPX section 3/10/11의 순변화 +3쪽을 별도로 처리한다.
