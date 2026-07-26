# task_m100_3391 처리결과 보고서 — set-cell 검정 글씨 기본 + 정부 실공고 E2E 시연

- **이슈**: [#3391](https://github.com/edwardkim/rhwp/issues/3391)
- **브랜치**: `pr/task-edit-set-cell` (#3345→#3374 위 적층, set-cell PR #3384 에 통합)
- **범위**: `src/main.rs`(edit_set_cell — `--keep-style` + `recolor_cell_text_black` +
  help/capabilities), `tests/edit_set_cell_contract.rs`(검정 글씨·keepStyle 테스트),
  `mydocs/manual/cli_commands.md`, E2E 증거 3종
- **분류**: 버그 수정 + 실증 (제출 요건 정합)

## 1. 배경 — 실물 공고 E2E 시연에서 발견

지금 모집 중인 K-Startup 실공고(2026 방산 특화 창업중심대학, pbancSn=178662, 마감 8/12)의
공식 사업계획서 양식(hwp)을 CLI 로 채우는 시연에서: 원본 양식의 기입 칸 안내문이 **파란
이탤릭**이라, set-cell 로 넣은 값도 그 글자모양을 상속해 파랗게 표기됐다. 그런데 공고 지침이
**"파란색 안내 문구는 삭제하고 검정 글씨로 작성하여 제출"** — 제출 요건과 정면 충돌.

## 2. 설계 결정

- **기본을 검정으로** — set-cell 은 "안내문을 지우고 실값을 쓰는" 용도이므로, 실무 기본이
  검정·비이탤릭·비진하게여야 한다. `--keep-style` 로 셀 스타일 상속을 옵트인한다.
- **글자모양 재사용/생성** — `recolor_cell_text_black` 이 검정·비이탤릭·비진하게·밑줄
  없음·취소선 없음인 글자모양을 doc_info 에서 찾고, 없으면 char_shape 0 을 복제해
  검정화한 뒤 추가한다(raw_data 폐기로 변경 필드가 직렬화되게). 셀 문단 0 의 char_shapes 를
  그 id 하나로 덮는다. 좌표 해석 실패 시 경고만 내고 상속 스타일로 진행(비파괴).
- 봉투에 `keepStyle` 필드 추가(스키마 추가-전용), capabilities recordFields 등재.

## 3. 검증

- **계약 테스트 추가**: 기본 keepStyle=false / `--keep-style` 시 true (기존 4종 + 신규 1종)
- **시각 확인**: 실공고 양식에 "(주)시연용가상기업" 기록 → SVG 렌더에서 **검정·비이탤릭**
  (주변 안내문은 파란 이탤릭 유지) — 색만 바뀌고 배치 불변
- 무회귀: cli_json_contract(22) green, fmt clean, clippy `-D warnings` 0건
- **정부 실공고 E2E 실증** (완전 가상 데이터·전 과정 CLI):
  발굴(pbancSn=178662) → 분석(info/fields=0/tables=39 → set-cell 대상 판정) →
  채움(set-cell 10칸) → **재독 10/10 일치** → 제출용 PDF/HWP 산출.
  증거: `assets/task_m100_3391/`(E2E 스텝 카드 + 원본↔작성본 비교 + 로그).
  **실제 접수는 하지 않음** — 가상 데이터로 실존 프로그램에 접수하면 허위 신청이 되고,
  로그인·실명인증도 자동화 범위 밖. "제출 직전 완성 파일"까지가 정당한 자동화 경계.

## 4. 남긴 것

- set-cell 이 셀의 나머지 서식(정렬·크기)은 안내문에서 상속한다 — 색/이탤릭만 정규화.
  칸별 정렬 세밀 제어는 후속.
- E2E 파이프라인은 tools 화 후보(공고 URL·데이터 JSON → 작성본) — #3370 예제집 연계.
