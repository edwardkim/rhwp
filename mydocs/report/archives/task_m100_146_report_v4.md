# 최종 결과보고서 v4: text-align.hwp SVG ↔ 한컴 PDF 일치 (v1~v4 종결)

- **타스크**: [#146](https://github.com/edwardkim/rhwp/issues/146)
- **마일스톤**: M100
- **브랜치**: `local/task146`
- **작성일**: 2026-04-23
- **이전 결과보고서**: `task_m100_146_report.md` (v2 기록, Geometric Shapes)

## 1. 전체 여정 요약

작업지시자가 제공한 `text-align.hwp` 1페이지 문서를 rhwp SVG 로 내보낸 결과가 한컴 오피스 PDF 출력과 시각적으로 달라, 이를 4단계(= 4번의 재스코핑)에 걸쳐 수렴시켰다.

### 단계별 스코프와 수정 요지

| 버전 | 원인/증상 | 수정 | 파일 수 | 주 단계 |
|------|----------|------|---------|---------|
| v1 (초안) | 3가지 가설(Justify·Hanging·공백) | — | (문서만) | 조사/계획 |
| v2 | **제목 □ 이 반각으로 측정** | `is_fullwidth_symbol` 에 Geometric Shapes(U+25A0-U+25FF) 범위 추가 | 소스 1+테스트 1+golden 1 | 단계1~3 |
| v3 | **TAC 표가 body_left 에 붙음** | `layout_table_item` 에 TAC 선행 텍스트 폭 직접 계산 + `compute_tac_leading_width` 헬퍼 | 소스 1+테스트 1 | 단계4 |
| v4 | **HY헤드라인M 제목이 regular 로 렌더** | `is_heavy_display_face` + `TextStyle::is_visually_bold` + svg.rs 4곳 분기 치환 | 소스 3+테스트 1 | 단계5 |

### 초기 가설 중 기각된 것들 (PDF 좌표 정밀 비교로 확정)

- **Justify SVG 미반영**: 실제로는 정상 동작 (본문 line 1 끝 SVG 538.27pt ≈ PDF 538.15pt)
- **Hanging indent 어긋남**: 실제로는 정상 동작 (line 2 "조" SVG 86.03pt ≈ PDF 85.99pt)
- **자간(-8%) × 공백 상호작용**: 실제 원인 아님 (진짜 원인은 □ 글자 폭 측정)

## 2. 최종 좌표·시각 수렴 결과 (text-align.hwp)

| 대상 | 초기 (v1 전) | v2 후 | v3 후 | v4 후 | PDF 환산 | 최종 오차 |
|------|-------------|-------|-------|-------|---------|-----------|
| 제목 "국" x | 94.40 | **105.40** | 105.40 | 105.40 | 105.39 | **0.01 px** |
| 표 첫 셀 x | 75.59 | 75.59 | **109.59** | 109.59 | ≈ 112.0 | **2.41 px** |
| 제목 font-weight | 없음 | 없음 | 없음 | **"bold"** | (HY헤드라인M heavy) | 시각 근사 |
| 본문 line1 "범" 끝 | 717.69 | 717.69 | 717.69 | 717.69 | 717.53 | 0.16 px |
| 본문 line2 "조" 시작 | 114.70 | 114.70 | 114.70 | 114.70 | 114.65 | 0.04 px |

## 3. 자동 테스트 영향

| 시점 | lib passed | lib failed | svg_snapshot |
|------|-----------|------------|--------------|
| 시작 (devel) | 927 | 14 | 3 |
| v2 후 | 929 | 14 (기존) | 3 (form-002 golden 갱신) |
| v3 후 | 931 | 14 (기존) | 3 (변화 없음) |
| v4 후 | **933** | 14 (기존) | **3** (변화 없음) |

- 본 타스크로 신규 통과 테스트 6건 추가
- 기존 14건 실패는 `serializer::cfb_writer::tests` / `wasm_api::tests` roundtrip 계열 — 본 PR 과 무관 (스태시 후 재측정으로 사전 확인)
- `cargo clippy --lib -- -D warnings`: 모든 단계에서 clean

## 4. 스모크 스위프 (v4 완료 후)

heavy face 사용 가능성이 큰 샘플에 대해 150dpi 렌더 회귀 확인:

| 샘플 | 페이지 | 결과 |
|------|--------|------|
| `samples/exam_kor.hwp` | 25페이지 | **정상** — 제목/헤더 영역이 굵게 렌더되어 PDF 계열 시각과 유사 |
| `samples/biz_plan.hwp` | 6페이지 | **정상** — 목차 페이지 번호/항목 굵기 유지 |
| `samples/text-align.hwp` | 1페이지 | **일치** — 제목/표 위치/bold 모두 PDF 시각 근사 |

## 5. 커밋 이력 (devel 대비 총 12커밋)

```
b38a126 Task #146 단계3: svg_snapshot golden + 최종 결과보고서
8642159 docs: Task #146 v2 계획서 + 단계2 보고서
ffaf488 Task #146: Geometric Shapes(U+25A0-U+25FF) 를 전각 심볼로 처리
bca5599 Task #146 단계1: 샘플 편입 + 비교 파이프라인 고정
10c36e2 Task #146: TAC 표 선행 텍스트 폭을 inline x 좌표에 반영
(v3 docs)
6f9fc85 Task #146: Heavy display face 를 visual bold 로 렌더
(v4 docs)
(본 커밋) 단계6 v4 결과보고서 + orders
```

## 6. 잔여 범위 (본 타스크 외)

- **한컴·한양 실제 폰트 번들**: HY헤드라인M 등이 설치된 환경에서는 fallback 없이도 heavy 로 렌더되나, 라이선스상 재배포 불가. 폰트 폴백 전략 `mydocs/tech/font_fallback_strategy.md` 범위.
- **WASM Canvas / HTML / PDF 렌더러의 heavy bold 적용**: 본 타스크는 SVG 한정. 필요 시 각 렌더러에서 `TextStyle::is_visually_bold()` 호출하도록 별도 이슈로 확장 가능.
- **다른 유니코드 심볼 블록(화살표, 수학 연산자 등) 전각 처리**: 실사례 발견 시 `is_fullwidth_symbol` 동일 방식으로 확장.

## 7. 종결 승인 요청

- 모든 단계 완료 (단계1~6)
- 자동 테스트·clippy 통과
- 좌표·시각 모두 PDF 기준 수렴
- 회귀 관찰 없음

승인 시 `local/task146` → `local/devel` → `devel` 머지 및 이슈 `#146` 종결 (`gh issue close 146`, gh 인증 필요 시 작업지시자가 직접 수행).
