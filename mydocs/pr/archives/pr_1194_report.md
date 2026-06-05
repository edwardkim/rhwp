# PR #1194 처리 보고서 — task 1189: task 1139 후속 미주/수식 정합 보정

- **작성일**: 2026-06-01
- **PR**: #1194 → **MERGED** (devel, 머지커밋 `6f42d200`)
- **컨트리뷰터**: @jangster77 (task 1139 / PR #1178 후속)
- **연결 이슈**: #1189 → **CLOSED** (`Closes #1189`, cross-repo `--no-ff` 라 수동 클로즈)
- **판단**: **머지** ✅ (작업지시자 시각 판정 통과)

## 결정 사유

task 1139 후속으로 시험지 샘플(3-09월 2023, 3-11월 2022)의 미주/수식 흐름 정합을 보정.
소스(.rs) 충돌 0건, 변경이 미주/수식 + 텍스트 측정에 국한, additive 성격. 시각 회귀 영역이라
작업지시자 직접 시각 판정을 게이트로 두었고 **통과**.

## 변경 요약 (19 파일, +937/-53 — 소스 9, test 1, docs/stage/orders 9)

| 파일 | 변경 |
|------|------|
| `text_measurement.rs` (+40) | `U+FFFC` 인라인 객체 placeholder 텍스트 폭 0 (Embedded ×2 / Wasm ×2 / estimate = 5경로 일관) + 회귀 테스트 |
| `svg.rs`/`web_canvas.rs` | 수식 `scale_y` 1.0 고정 (bbox 높이는 줄높이+여백 영역 → 세로 스케일 시 글자 찌그러짐). 두 렌더러 일관 |
| `paragraph_layout.rs` (+2/-12) | 위 변경에 맞춰 `eq_y` baseline scale 분기 제거(단순화) |
| `height_cursor.rs` (+115/-11) | compact 미주 질문 제목 간격(직전 하단+10px, page-base 7mm 유지), 빈 문단 뒤 완충 제거, 큰 display 수식 줄 되감기. 회귀 테스트 3건 |
| `symbols.rs` (+7) | 대문자 대각 화살표 토큰(`NEARROW`/`SEARROW` 등) → `↗`/`↘` 매핑 + 테스트 |
| `cursor_nav.rs` (+52/-2) | 미주 드래그 선택 시 TextRun 끝점 없으면 같은 TextLine 우측 끝 fallback (body line 한정, cell 제외) |
| `composer.rs` (+22) | 수식-only 미주 문단 marker 합성 조건 보정 |
| `tests/issue_1139_inline_picture_duplicate.rs` (+230) | #1189 회귀 케이스 |

## 충돌 처리 (소스 무충돌, orders만 충돌)

- **소스(.rs) 9파일 + test 전부 auto-merge CLEAN.**
- 충돌은 메인테이너 작업일지 2개(`20260531.md` UU, `20260601.md` AA)뿐 — PR 이 같은 날짜
  파일에 Task #1189 기록을 추가했는데 devel 에서 #1192/#1193 처리로 동시 수정해 발생.
- **작업지시자 결정**: 메인테이너가 양쪽 기록 보존으로 직접 해소 (rebase 요청 대신).
  - `20260531.md`: devel(#1192/#1193 + M100 테이블) + PR `## Task #1189` 섹션.
  - `20260601.md`: 헤더 1회 + devel PR #1193 섹션 + PR Task #1189 Stage6/7.

## 검증 결과

| 단계 | 결과 |
|------|------|
| merge | ✅ 소스 CLEAN / orders 2개 수동 해소(마커 0) |
| fmt | ✅ clean |
| build | ✅ Finished |
| 전체 테스트 | ✅ **1893 passed, 0 failed** |
| WASM | ✅ pkg 빌드 (`pkg/rhwp_bg.wasm`) |
| 머지 검증 | ✅ PR head(`44e18961`) 조상 확인 (PR_ANCESTOR=YES) |
| **시각 판정** | ✅ **통과** (작업지시자 직접 — 미주/수식 정합) |

## 부수 작업 — 폰트 fallback 추적 (작업지시자 요청)

`Haansoft Dotum` (3-11월 2022 샘플) fallback 3계층 추적:
- **치환**(`resolve_ttf_font`): 매핑 없음 → None → 이름 유지(치환 안 됨).
- **글리프**(`generic_fallback`): sans-serif 분기 → `Malgun Gothic`→`Apple SD Gothic Neo`→`Noto Sans KR`→… .
- **폭 메트릭**(`find_metric`): `font_metrics_data.rs:41121` 자체 엔트리(FONT_7) 직접 매칭, 폴백 없음.
- **발견**: 한글명 `한컴돋움`/`함초롬돋움`은 `함초롬돋움`/`HCR Dotum` 으로 정규화되나 영문
  `Haansoft Dotum` 은 독립 처리 → 글리프(Malgun) / 메트릭(FONT_7) 경로가 한글명과 불일치.
  `feedback_font_alias_sync` 2계층 동기화 관점의 잠재 결함 후보(별도 이슈 판단 사안, 본 PR 범위 외).

## 처리 절차

1. PR 정보 확인 — CONFLICTING/DIRTY. 충돌 진단: 소스 0건, orders 2개뿐.
2. 작업지시자 결정 → 메인테이너 직접 해소(양쪽 보존).
3. 소스 검토(미주/수식, 양호) + 머지 + orders 충돌 수동 해소 + 검증(test 1893) + push + WASM.
4. 머지커밋 해시 정정(추정값 → 실제 `6f42d200`).
5. **작업지시자 시각 판정 통과** + 폰트 fallback 추적 보고.
6. 이슈 #1189 클로즈 + 보고서 + orders 갱신.

## 비고

- cross-repo `--no-ff` 머지라 GitHub 자동 MERGED 는 됐으나 이슈 자동 클로즈는 수동 처리.
- @jangster77 의 task 1139/1189 미주·수식 조판 누적 기여.
