# PR #1194 검토 — task 1189: task 1139 후속 미주/수식 정합 보정

- **작성일**: 2026-06-01
- **PR**: #1194 → MERGED (devel `6f42d200`)
- **컨트리뷰터**: @jangster77 (task 1139/PR #1178 후속)
- **연결 이슈**: #1189 (`Closes #1189`) — 시각 판정 통과 후 클로즈 예정
- **base/head**: `devel` ← `task_m100_1189` (`44e18961`)
- **규모**: 19 파일, +937/-53 (소스 9, test 1, docs/stage/orders 9)
- **CI**: 이전 head 기준 전부 SUCCESS (Canvas visual diff 포함)

## 1. 변경 내용 (PR 본문 + diff 검토)

미주/수식 흐름 정합 후속 보정 (시험지 샘플 3-09월 2023, 3-11월 2022 등):

### 소스 (9 파일, 미주/수식 조판 영역 국한)
- **`text_measurement.rs` (+40)**: `U+FFFC`(인라인 객체 placeholder) 텍스트 폭 0 처리.
  기존 `U+2007`/`U+F081C` 처리와 동일 패턴. **5개 측정 경로(Embedded ×2, Wasm ×2, estimate) 전부 일관 적용** + 회귀 테스트.
- **`svg.rs`/`web_canvas.rs` (+3/-6, +3/-8)**: 수식 `scale_y` 1.0 고정. bbox 높이는 줄높이+여백
  포함 영역이라 세로 스케일 시 글자 찌그러짐. **두 렌더러 경로 일관** (feedback_image_renderer_paths_separate).
- **`paragraph_layout.rs` (+2/-12)**: 위 변경에 맞춰 `eq_y` baseline 계산에서 scale 분기 제거(단순화).
- **`height_cursor.rs` (+115/-11)**: compact 미주 흐름 질문 제목 간격(직전 하단+10px, page-base 는 7mm 유지),
  빈 문단 뒤 완충 제거(`prev_was_blank_para`), 큰 display 수식 줄 되감기 helper. 신규 필드 생성자/reset 일관.
- **`symbols.rs` (+7)**: HWP 대문자 대각 화살표 토큰(`NEARROW`/`SEARROW` 등) → `↗`/`↘` 매핑 + 테스트.
- **`cursor_nav.rs` (+52/-2)**: 미주 드래그 선택 시 TextRun 끝점 없으면 같은 TextLine 우측 끝 fallback (body line 한정, cell 제외).
- **`composer.rs` (+22)**: 수식-only 미주 문단 marker 합성 조건 보정.

### 테스트
- `tests/issue_1139_inline_picture_duplicate.rs (+230)`: #1189 회귀 케이스 추가 (2023 19쪽, 2022 10~12/14/17쪽, 드래그 선택, U+FFFC 등).

## 2. 충돌 처리

- **소스(.rs) 충돌 0건** — 9개 소스 파일 전부 auto-merge 성공.
- **충돌은 orders 작업일지 2개뿐** (`20260531.md` UU, `20260601.md` AA): PR 이 메인테이너 일일
  작업일지에 Task #1189 기록을 추가했는데, 같은 날짜 파일을 devel 에서 #1192/#1193 처리로 수정해 발생.
- **작업지시자 결정**: 메인테이너가 양쪽 기록 보존으로 직접 해소 (rebase 요청 대신).
  - `20260531.md`: devel 전체(#1192/#1193 + M100 테이블) + PR 의 `## Task #1189` 섹션 append.
  - `20260601.md`: 헤더 1회 + devel PR #1193 섹션 + PR Task #1189 Stage6/7 섹션.

## 3. 위험 평가

- **낮음.** 변경이 미주/수식 흐름 + 텍스트 측정에 국한. additive 성격(placeholder 폭 0, 화살표 토큰,
  scale_y 고정). U+FFFC 폭 0 은 5경로 일관이라 측정 불일치 위험 낮음.
- **단, 시각 회귀 영역**(미주/수식 배치) → 작업지시자 직접 시각 판정 게이트
  (feedback_visual_regression_grows). PR 본문이 PDF 비교 산출물 명시.

## 4. 검증 결과 (devel 머지 후)

| 단계 | 결과 |
|------|------|
| merge | ✅ 소스 CLEAN, orders 2개 수동 해소(마커 0) |
| fmt | ✅ clean |
| build | ✅ Finished |
| 전체 테스트 | ✅ **1923 passed, 0 failed** |
| WASM | ✅ pkg 빌드 |
| 머지 검증 | ✅ PR head 조상 확인 (PR_ANCESTOR=YES) |

## 5. 판단

소스 무충돌 + 검증 통과 → **머지 완료**. 시각 판정(미주/수식 정합)은 작업지시자 직접 진행.
시각 판정 통과 후 이슈 #1189 클로즈. 결과는 `pr_1194_report.md`.
