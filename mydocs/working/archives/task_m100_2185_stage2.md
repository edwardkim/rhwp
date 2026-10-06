# Task M100 #2185 Stage 2 완료보고서 — 실제 편집·저장 통합 회귀 핀

## 목표

공개 #1949 거대 셀 샘플의 재현 문단에 `1` 하나를 입력한 뒤에도 한컴 저장
`LINE_SEG` 경계와 후속 문단 위치가 유지되는지 HWP/HWPX 양쪽에서 검증한다. Studio의
지연 페이지네이션 입력 경로, 전체 pagination, 원본 형식 저장과 재로드까지 하나의 통합
회귀 테스트로 고정한다.

## 구현 결과

`tests/issue_2185_korean_break_unit.rs`를 추가했다. 두 거대 문서를 병렬로 올리지 않도록
하나의 테스트에서 HWP와 HWPX를 순차 검증한다.

- 픽스처
  - `samples/issue1949_giant_cell_nested_tables_perf.hwp`
  - `samples/issue1949_giant_cell_nested_tables_perf.hwpx`
- 대상 경로
  - `section=0, parent=0, control=2, cell=2, cell_para=5`
- 입력
  - Rust 문자 인덱스 130에 `1` 삽입
  - 대상은 BMP 문자만 포함해 UTF-16 위치도 130임을 별도 단언
- 입력 경로
  - `insert_text_in_cell_native_deferred_pagination`
  - 편집 직후 문단 로컬 reflow 결과 확인
  - `flush_deferred_pagination`으로 전체 pagination 실행

## 고정한 계약

| 검증 시점 | 계약 |
|-----------|------|
| 원본 로드 | `attr1 bit7=1`, 줄 시작점 `[0, 44, 84, 122]`, 4줄 |
| 원본 로드 | 다음 셀 문단 첫 `vpos=17160`, 전체 115쪽 |
| 지연 입력 직후 | 반환 커서 131, 원문 끝에 `1`만 추가 |
| 지연 입력 직후 | 줄 시작점과 다음 문단 `vpos` 불변 |
| 전체 pagination 후 | 줄 시작점, 다음 문단 `vpos`, 115쪽 불변 |
| 저장 후 | HWP는 HWP, HWPX는 HWPX 형식으로 저장 |
| 재로드 후 | bit7, 편집 텍스트, 4개 줄 경계, 다음 `vpos`, 115쪽 보존 |

## 검증 결과

- `cargo test --profile release-test --test issue_2185_korean_break_unit -- --nocapture`
  - 통과, 1 passed
  - HWP: load 1.159s, edit 0.162ms, flush 1.168s, save 8.596ms, reload 1.192s
  - HWPX: load 1.186s, edit 0.096ms, flush 1.185s, save 16.873ms, reload 1.211s
- `cargo test --profile release-test --test issue_1949_giant_cell_render_perf -- --nocapture`
  - 통과, 1 passed, 115쪽 전체 렌더 4.50s
  - 기존 `LAYOUT_OVERFLOW` 진단 로그는 출력됐으나 테스트 실패나 쪽수 변화는 없음
- `cargo test --profile release-test --test issue_2164_cell_enter_overlap`
  - 통과, 3 passed
- `cargo fmt --check`
  - 통과
- `git diff --check`
  - 통과

시간은 현재 로컬 release-test 단일 실행 관측값이며 성능 기준값으로 사용하지 않는다.

## 입력 지연과의 관계

동일 재현에서 문단 로컬 입력·reflow는 HWP 약 0.162ms, HWPX 약 0.096ms였지만 전체
pagination flush는 각각 약 1.17~1.19초였다. 따라서 사용자가 느끼는 큰 입력 지연은 이번에
정정한 한글 어절/글자 분기 계산 자체가 아니라, 같은 편집이 유발하는 115쪽 거대 셀 전체
pagination 비용에서 발생한다는 분리가 다시 확인됐다.

두 현상은 같은 입력 경로에서 연속해서 나타나지만 직접 원인은 다르다.

- 문단 구조 변경: `korean_break_unit` 소비 의미 반전
- 입력 지연: 거대 셀 문서의 전체 pagination 및 후속 화면 갱신

증분 pagination이나 Canvas/page-tree 최적화는 #2185 정확성 수정 범위에 포함하지 않는다.

## 상태

Stage 2 구현과 HWP/HWPX 편집·저장·재로드 검증 완료. 작업지시자 승인 전에는 Stage 3를
진행하지 않는다.
