# PR #2074 검토 — 표 셀 앵커 그림 셀 vertical_align 반영 (lpaiu-cs)

- 이슈: #2071 / base: devel ← fix/cell-anchor-picture-valign / 작성일: 2026-07-09
- 컨트리뷰터 사이클: **재기여자** (#2039 찾아바꾸기 undo, #2040 BinData storage id — 둘 다 머지).
  동일 저자 열린 PR 3건(#2074/#2076/#2078) — 시간순 처리, 상호 supersede 없음(축 상이).

## 1. PR 정보

- CI: **전 항목 pass** (Analyze rust/js/py, Build & Test, default-feature).
- mergeable: **CONFLICTING** — `table_layout.rs`. **원인은 우리 R11(#2089)**: 셀 그림 배치
  코드가 `layout_horizontal_cell_paragraphs` 로 통이동되어 hunk 컨텍스트 위치가 이동.
  본문은 원본 무변경 이동이라 **hunk 를 신규 메서드 내부에 그대로 적용 가능**.
- 변경: `table_layout.rs`(+34) / `table_partial.rs`(+26, 짝 경로 동일 보정 —
  [[feedback_fix_scope_check_two_paths]] 정합) / 표적 테스트 3건 + 픽스처 2건.

## 2. 내용 검토

- 주장: 셀 앵커 floating 그림(restrict-ON, TopAndBottom+Para)은 한컴이 **셀 valign 으로만**
  배치, 그림 pos vert_align 무시. 근거 = **한글 2024 편집기 COM 자동화 오라클**
  (PyMuPDF+pdfplumber 2종 교차 <0.01px, 경쟁가설 기각 절차 포함) — 편집기 직접 계측이라
  변환본 PDF 등급 문제(#1936 사례)와 다른 축이나, **2024 편집기**는 정답지 등급 목록
  (2010/2020/2022) 밖 → **시각 판정 게이트 필요** 판단.
- 수정 위치가 `compute_object_position` 직후 pic_y 재정의 — 가드 4중
  (top_and_bottom_para/flow_with_text/!unrestricted/!detached)로 케이스별 명시 가드
  ([[feedback_hancom_compat_specific_over_general]] 정합). 분할 셀 Top 강제는 범위 외 명시.

## 3. 처리안

1. **충돌은 메인테이너 해소** (우리 리팩토링이 원인): devel 기준 병합 + hunk 를
   `layout_horizontal_cell_paragraphs` 내부 동일 지점에 적용, 저자 커밋 보존
   (merge commit 방식).
2. 로컬 CI급 검증: fmt/clippy/`--tests`(신규 3건 포함)/OVR 5샘플.
3. **작업지시자 시각 판정**: 첨부 비교 이미지(수정 전 154px vs 수정 후 362px vs
   한컴 362px) + 필요 시 WASM 빌드. 2024 편집기 오라클의 정답지 등급 미달분 보완.
4. 판정 통과 시 approve + merge, 처리 코멘트(결과+검증+감사).

## 4. 리스크

- 낮음: 가드가 좁고 표적 테스트가 pos vert_align 무시를 직접 단언. R11 이동과의
  상호작용은 병합 검증에서 확인.
