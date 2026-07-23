# Task M100 #2424 최종 보고서 — 거대 표 pagination을 browser task 사이에서 재개

- 이슈: [#2424](https://github.com/edwardkim/rhwp/issues/2424)
- 최종 기준: `upstream/devel@cbddc1cd8`
- 작업 브랜치: `issue-2424-resumable-pagination-v3`
- 작성일: 2026-07-23

## 결론

#2424는 최신 devel에도 유효한 미구현 작업이었다. 거대 셀 입력이 4줄에서 5줄로 넘어가는 56번째
입력에서 `flushDeferredPagination()`이 section 전체를 동기 조판했고, HWP/HWPX에서 약
1.06~1.11초 동안 main thread를 점유했다.

table continuation을 borrow-free shadow job으로 분리하고 WASM begin/step/cancel/barrier API와 Studio
macrotask runner를 연결했다. 실제 브라우저에서는 경계 입력이 동기 flush 없이 약 76~81ms에 반환되고,
115개 fragment가 task 사이에서 하나씩 처리된다. 공개 pagination은 완료 전까지 그대로이며 마지막
fragment에서만 정확한 115쪽 결과로 원자 교체된다.

PR 리뷰 후 insert 전용이던 fast path를 flat 셀 삭제에도 연결했다. HWP/HWPX 실브라우저에서
Backspace/Delete WASM 호출은 1.5~2.5ms, `ㅎ→하→한` 조합 input handler는 71.1~75.3ms였고,
기존 약 917ms 동기 삭제 프리즈와 자모별 full pagination은 사라졌다. Enter는 구조 변경이므로
후속 범위로 분리했다.

추가 계측에서 일반 문자 0.1ms는 WASM mutation만, IME 71~75ms는 handler 전체를 잰 값임을
정정했다. 같은 범위에서 영문 stable operation은 p50 46.0ms, p95 47.6ms였고, IME가 조합 anchor
좌표를 한 번 더 조회해 약 25~28ms를 추가로 썼다. 조합 시작 좌표를 보존하고 pagination 공개
레이아웃 교체 시 폐기하도록 보정한 뒤 HWP IME는 45.5~48.2ms, HWPX는 46.9~49.7ms로 줄어
영문 operation과 사실상 같은 수준이 됐다.

## 원인과 구현

Stage A 계측에서 전체 incremental flush의 98.7~98.8%가 `TypesetEngine`이었고, 하나의 대형
RowBreak 표 continuation이 114개 후속 페이지를 만들었다. measurement나 cache invalidation만 줄여서는
long task를 해소할 수 없어 다음 계층으로 구현했다.

1. deferred edit descriptor에 revision, target 좌표, 첫 page와 재귀 table structure fingerprint를 보존했다.
2. table continuation의 row/cut 진행을 owned cursor/context로 옮기고 fragment budget step을 만들었다.
3. `DocumentCore`가 shadow pagination job을 호출 사이에 보존하고 stale/unsupported 상태를 구분한다.
4. WASM에 begin/step/cancel과 저장·인쇄용 동기 barrier를 노출했다.
5. Studio runner가 한 macrotask에 fragment budget 1만 처리하고 새 입력에서 이전 job을 교체한다.
6. 마지막 step에서만 pagination, measurement, dirty state와 render cache를 함께 commit한다.

초기 fast path는 단일 section·paragraph·column의 마지막 non-TAC RowBreak 표로 제한한다. 구조가
불명확하거나 revision/fingerprint가 달라지면 기존 full pagination으로 안전하게 fallback한다.

## 검증 결과

- HWP/HWPX exact oracle: 115 steps, 115 fragments, 115 pages, 113 changed cuts.
- atomic commit: 114 pending steps 동안 공개 cut chain 불변, 115번째 step에서만 교체.
- release fragment timing:
  - HWP p50 10.156ms, p95 10.733ms, max 21.662ms.
  - HWPX p50 10.178ms, p95 11.002ms, max 22.292ms.
- 브라우저 HWP/HWPX 각 3회:
  - 동기 flush 0회, begin 1회, step 115회.
  - 경계 operation 75.9~81.3ms, begin 32.0~34.0ms.
  - model/tree/layout/cursor/caret와 page 0 crop exact 유지.
- IME/iOS stable·boundary smoke 모두 통과.
- Rust 전체 library: 2537 passed, 0 failed, 7 ignored.
- Studio: 512 passed, production build 통과.
- wasm32 check, wasm-pack web build, fmt, clippy, #2214 focused 회귀 통과.
- PR #3125 CI 후속으로 #2724 패스스루 분류 가드에 pagination API 4개의 `SessionState` 근거를
  등록했고, 해당 integration guard 5건을 통과했다.
- 실제 1쪽→2쪽 RowBreak 합성 문서에서 pending page count hold와 final-only publish를 확인했다.
- 5줄→4줄 deferred 삭제도 115개 pending step 동안 공개 cut chain을 유지하고, final commit에서만
  55자 full-pagination oracle과 일치함을 HWP/HWPX 양쪽에서 확인했다.
- pending 경계 입력 직후 실제 저장을 실행해 `flush → HWP/HWPX export`, 저장본 최신 텍스트·115쪽
  재오픈을 확인했고, 실제 인쇄는 `flush → 첫 SVG render` 뒤 115페이지를 생성했다.
- IME 조합 caret의 direct `getCursorRectInCell` 재조회는 HWP/HWPX 모두 0회이고, 조합 anchor
  캐시는 pagination commit/flush와 입력 세션 종료에서 폐기된다.
- 조합 중 shadow pagination이 실제 commit되는 경계에서는 anchor rect를 정확히 1회 갱신하고
  다음 자모가 새 캐시를 재사용함을 HWP/HWPX 브라우저 smoke로 확인했다.

세부 수치와 단계별 계약은 다음 문서에 있다.

- `mydocs/working/task_m100_2424_stage1.md`
- `mydocs/working/task_m100_2424_stage2.md`
- `mydocs/working/task_m100_2424_stage3.md`
- `mydocs/working/task_m100_2424_stage4.md`
- `mydocs/working/task_m100_2424_stage5.md`
- `mydocs/working/task_m100_2424_stage6.md`
- `mydocs/working/task_m100_2424_stage7.md`

## 잔여 위험과 후속 후보

- begin의 약 32ms에는 정규화와 selective measurement가 포함된다. continuation step과 별도로 더
  잘게 나눌 여지가 있다.
- 범용 다중 section/paragraph/column, footnote/endnote, pagination 종속 floating object는 아직
  resumable fast path 대상이 아니다.
- fragment 총 계산량은 기존 full pagination과 비슷하다. 이번 변경의 목적은 총량 제거보다 main-thread
  blocking slice 분할과 정확한 atomic commit이다.
- Enter는 `splitParagraphInCell` 구조 편집이므로 삭제·IME 보정에 포함하지 않았다. 구조 fingerprint와
  undo/selection 계약을 별도로 설계한 뒤 후속 최적화로 다룬다.
- 일반 영문과 IME에 공통으로 남은 `getCursorRectByPathNear()`는 HWP p50 44.8ms, p95 45.6ms로
  stable operation의 지배 항이다. 종료된 #2021/#2193을 다시 열기보다 별도 후속 성능 이슈로
  분리해 deferred mutation cursor 결과나 focused-cell layout cache를 검토한다.

현재 fast path 대상인 이슈 fixture의 완료 조건은 모두 만족한다. 원격 push, PR 생성과 이슈 상태 변경은
작업지시자 승인 전에는 수행하지 않는다.
