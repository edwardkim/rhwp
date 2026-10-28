# Task M100 #2424 Stage G 완료보고서 — IME 조합 anchor 좌표 재조회 제거

## 1. 계측 범위 정정

Stage F의 `71.1~75.3ms`는 WASM mutation만이 아니라 IME `input` handler 전체 시간이었다.
반면 리뷰의 일반 문자 `0.1ms`는 WASM mutation 호출만 측정한 값이므로 직접 비교할 수 없다.

같은 HWP 115쪽 fixture와 Chrome에서 범위를 맞춰 다시 측정했다.

| 경로 | 수정 전 |
|---|---:|
| 영문 stable operation | p50 46.0ms, p95 47.6ms |
| 영문 keyboard 전체 | p50 66.2ms, p95 67.7ms |
| 영문 cursor query | p50 45.3ms, p95 46.9ms |
| IME `ㅎ→하→한` handler | 73.9 / 73.8 / 72.9ms |

일반 입력과 IME 모두 `cursor.moveTo()`의 `getCursorRectByPathNear()`가 지배 항이다. IME는 여기에
조합용 검은 caret의 시작점을 그리기 위해 같은 anchor를 `getCursorRectInCell()`로 다시 찾았다.
첫 자모도 느렸으므로 delete+insert 자체가 아니라 이 두 번째 exact cursor lookup이 IME 고유
추가 비용의 주원인이었다.

## 2. 구현

- `compositionstart`에서 logical cursor와 anchor가 정확히 같을 때 현재 `CursorRect`를 복사한다.
- IME 조합 caret은 저장한 `compositionAnchorRect`를 사용하고, 좌표를 확보하지 못한 특수 경로만
  기존 exact lookup으로 fallback한다.
- fallback으로 얻은 exact 좌표도 이후 조합 갱신에서 재사용한다.
- shadow pagination commit이나 동기 pagination flush가 공개 레이아웃을 교체하면 캐시를 폐기한다.
  조합이 계속 중이면 다음 caret 갱신에서 exact 좌표를 한 번 다시 구해 새 레이아웃 기준으로 보존한다.
- composition end, deactivate, dispose와 form-mode 거부 경로에서 캐시를 초기화한다.

## 3. 실브라우저 결과

동일한 macOS headless Chrome, HWP/HWPX 115쪽 fixture에서 검증했다.

| 형식 | 수정 전 IME | 수정 후 IME | 조합 중 `getCursorRectInCell` |
|---|---:|---:|---:|
| HWP | 73.9 / 73.8 / 72.9ms | 48.2 / 45.5 / 46.9ms | 0회 |
| HWPX | 74.0 / 71.3 / 72.8ms | 49.7 / 46.9 / 47.4ms | 0회 |

HWP 기준 수정 후 영문 stable operation은 p50 45.4ms, p95 46.5ms이고 IME는 45.5~48.2ms다.
따라서 IME 고유 추가 비용을 약 34~36% 제거해 영문 operation과 사실상 같은 수준으로 맞췄다.

기존 계약도 유지됐다.

- deferred insert 3회, deferred delete 2회
- immediate/path delete와 동기 pagination flush 0회
- HWP/HWPX Backspace/Delete, 저장 barrier 통과
- HWP 115페이지 인쇄 barrier 통과
- HWP focused 경계: begin 1회, step 115회, 동기 flush 0회
- 조합을 유지한 채 shadow pagination commit을 기다리는 경계 smoke에서 HWP/HWPX 모두 기존
  anchor를 폐기하고 direct cell rect를 정확히 1회 재취득했다. 다음 조합 갱신은 새 캐시를
  재사용했고 동기 flush는 0회였다. 경계 갱신은 각각 HWP 79.0/80.2ms, HWPX 80.3/81.4ms였다.

## 4. 검증

| 게이트 | 결과 |
|---|---|
| Studio `npm test` | 512 passed, 0 failed |
| Studio `npm run build` | 통과 |
| HWP focused + review + IME commit smoke | 통과 |
| HWPX review + IME commit smoke | 통과 |

## 5. 후속 추적 판단

- 상세 근거의 canonical 기록은 이 Stage G 문서와 최종 보고서로 둔다.
- PR #3125 본문과 리뷰 후속 코멘트에는 삭제·IME 누락 보정, B/C barrier,
  IME `73ms→46~48ms`, 전체 게이트를 요약해 reviewer가 최초 지적의 해소 여부를 바로
  확인할 수 있게 한다.
- #2424 이슈에는 구현 중간 코멘트를 중복해서 남기지 않는다. PR이 merge될 때 최종 결과와 잔여
  후속 이슈 링크를 한 번 남긴 뒤 close하는 흐름이 적절하다.
- #2193과 #2021은 이미 종료됐으므로 재개하지 않는다. 남은 일반 입력 병목인
  `getCursorRectByPathNear()` p50 약 45ms는 [#3137](https://github.com/edwardkim/rhwp/issues/3137)로
  분리하고 #2021, #2424, #2193을 관련 이슈로 연결했다.

후속 이슈:

- [#3137 perf: 거대 표 셀 입력의 cursor rect lookup을 frame budget 안으로 축소](https://github.com/edwardkim/rhwp/issues/3137)
- 기준선: HWP cursor query p50 44.8ms, p95 45.6ms; stable operation p95 46.5ms
- 후보: deferred mutation 응답의 cursor rect 동봉, focused-cell line layout cache, exact lookup의
  다음 frame 이전
- 수용 기준: 영문·IME stable operation p95 16ms 근접, cursor/caret exact 정합과 #2214/#2424
  저장·pagination 계약 유지
