# PR #1276 처리 보고서

## 1. 대상

- PR: https://github.com/edwardkim/rhwp/pull/1276
- 제목: `Task #1271: HWPX 글뒤로 표 분할로 인한 바탕쪽 홀짝 밀림 수정`
- 작성자: `postmelee`
- 처리일: 2026-06-04

## 2. 처리 내용

PR head 커밋을 현재 `devel` 위에 cherry-pick 했다.

- 원본 PR head: `e2b84d796cbaf5caf3cf80183c85b0e4a8ad9f78`
- 로컬 통합 커밋: `f2292883 fix: HWPX 글뒤로 표 페이지 밀림 보정`

충돌은 문서 1개에서만 발생했다.

- `mydocs/orders/20260603.md`

해결 방식:

- 기존 `devel`의 #1205, #1251 행 유지
- PR의 #1271 행 추가

## 3. 코드 변경 요약

### 3.1 HWPX paper-anchored 글뒤로/글앞으로 표 분할 억제

`src/renderer/typeset.rs` 에서 종이 기준 절대좌표 표가 본문 흐름을 차지하는 `PartialTable` 로 분할되지 않도록 보정했다.

핵심 조건:

- `treat_as_char=false`
- `vert_rel_to=Paper`
- `horz_rel_to=Paper`
- `text_wrap=BehindText` 또는 `InFrontOfText`

이 조건의 표는 페이지 배경/전경 성격으로 유지되어 앞쪽 페이지가 밀리지 않는다.

### 3.2 바탕쪽 선택 시점 보정

`src/document_core/queries/rendering.rs` 에서 바탕쪽 odd/even 선택을 구역 간 쪽번호 carry 이후 수행하도록 변경했다.

이로써 이전 구역 쪽번호 누적 때문에 다음 구역의 실제 쪽번호 홀짝이 바뀌는 경우에도 올바른 바탕쪽이 적용된다.

### 3.3 HWPX 바탕쪽 글상자 current size 음수 보정

`src/renderer/layout/shape_layout.rs` 에서 HWPX 바탕쪽 글상자의 `curSz` 높이가 음수 HWPUNIT 값으로 래핑된 경우를 보정했다.

문제 샘플의 바탕쪽 하단 글상자는 `curSz height=4294965455` 형태로 들어오며, 이를 unsigned 높이로 그대로 해석하면 글상자 내부 폰트 축소 비율이 극단적으로 작아져 텍스트가 사실상 보이지 않았다.

보정 방식:

- current size 값을 signed `i32` 로 해석
- 양수인 경우에만 original size 대비 scale ratio 계산
- 0 이하 또는 음수 래핑 값은 scale ratio 1.0 으로 처리

### 3.4 바탕쪽 AutoNumber placeholder 위치 보정

`src/renderer/layout.rs` 에서 쪽번호 AutoNumber placeholder 탐색 조건을 강화했다.

문제 샘플의 홀수 바탕쪽 글상자 원본 순서는 다음과 같다.

- `독서 · 문학  `
- `AutoNumber(Page)`

하지만 기존 fallback 탐색이 일반 `fwSpace` 를 AutoNumber placeholder 로 오인해, 웹/SVG 출력에서 `독서 5 · 문학` 순서가 되었다.

보정 방식:

- 일반 공백만으로는 AutoNumber placeholder 로 인정하지 않음
- `char_offsets` 기준 8 UTF-16 단위 폭을 가진 placeholder 위치만 우선 인정
- `char_offsets` 가 없는 legacy fallback 에서만 기존 공백 탐색 허용

수정 후 홀수 바탕쪽 하단 텍스트는 `독서 · 문학  5` 순서로 출력된다.

### 3.5 회귀 테스트 추가

`tests/issue_1271_hwpx_behind_text_table.rs` 추가.

재현 샘플:

- `samples/hwpx/[2027] 온새미로 1 본교재.hwpx`

검증 내용:

- page 2: MEMO 쪽 유지
- page 2: `PartialTable   pi=3 ci=0` 없음
- page 3: 1주차 표지 도형 유지
- page 4: section 1 본문이 `page_num=4` 로 시작
- page 4: 바탕쪽 하단 글상자가 microscopic font 로 축소되지 않음
- page 5: 홀수 바탕쪽 쪽번호가 `독서 · 문학` 뒤에 위치

## 4. 검증

| command | result |
|---|---|
| `cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture` | pass |
| `cargo test master_page_selection_uses_final_carried_page_number_parity -- --nocapture` | pass |
| `cargo test --test issue_1058_textbox_list_header issue_1058_new_footnote_inner_para_contract -- --nocapture` | pass |
| `cargo fmt --check` | pass |
| `docker compose --env-file .env.docker run --rm wasm` | pass |

WASM 산출물:

- `rhwp-studio/public/rhwp_bg.wasm`
- size: 5.3M
- sha256: `7a904b43fdf4dcfd00c4110ba42e0357febdf9748a38f4669d14aee299144a1c`

시각 판정:

| 대상 | 판정 | 비고 |
|---|---|---|
| SVG page 5 홀수 바탕쪽 | 통과 | `독서 · 문학  5` 순서 확인 |
| rhwp-studio 웹 캔버스 | 통과 | 메인테이너 직접 판정 |

## 5. 남은 절차

- 메인테이너 최종 승인
- `mydocs/pr/pr_1276_review.md`, `mydocs/pr/pr_1276_report.md` 문서 커밋
- `origin/devel` push
- PR #1276 종료 처리

## 6. 판정

코드 통합, 포커스 테스트, WASM 빌드, SVG/웹 시각 판정 기준 통과.

현재 상태에서는 수용 가능하다.
