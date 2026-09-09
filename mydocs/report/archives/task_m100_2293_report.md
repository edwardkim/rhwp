# 최종 결과보고서 — Task M100 #2293: skia 조각 래스터 텍스트 소실 정정

- 이슈: #2293 / 브랜치: `local/task2293` / 작성일: 2026-07-15
- 계획: `plans/task_m100_2293.md` (승인됨, 버그 정정형)
- **작업지시자 시각 판정 통과** (2026-07-15, `output/png/chart_check_text/` 5종)

## 요약

`export-png`(native-skia)의 차트(RawSvg) 텍스트 전량 소실을 정정. 근인은
`svg_fontdb()` 가 ①프로젝트 `ttfs/` 미로딩 ②generic 폴백을 존재 확인
없이 특정 배포판 폰트명에 하드 고정 — 매칭 실패 시 resvg 가 텍스트 드롭
(#2292 검증 중 실험으로 확정: 실존 폰트 지정 시 잉크 0→451).

## 정정 (`src/renderer/skia/image_conv.rs`)

- PDF 경로(`create_fontdb`) 규약 정합: `ttfs/`(재귀)·`ttfs/windows`·
  `ttfs/hwp` + WSL `/mnt/c/Windows/Fonts` 로딩.
- generic 폴백 = **fontdb 실존 확인 체인의 첫 항목** (`first_existing_family`).
- **작업지시자 권고 반영 — 한국어 우선**: serif/mono 도 라틴 전용 최후
  폴백 앞에 한국어 sans 배치 (한글 표시 > 스타일 정합). 저장소
  `ttfs/opensource/NotoSansKR` 로 체크아웃 환경 한국어 폴백 보장.

## 검증

| 게이트 | 결과 |
|--------|------|
| 표적 `issue_2293_chart_png_text` (제목·카테고리 밴드 잉크) | 수정 전 **FAILED 실증** → 통과 |
| #2292 표적 (기하 회귀 가드) | 통과 |
| default / native-skia 전수 | **3,191/0** / **3,242/0** |
| fmt / clippy | 통과 / 0 |
| OVR 5샘플 (`output/poc/task2293/`) | 회귀 0건 |
| **시각 판정** | **통과** — 제목·축 값·카테고리·범례 텍스트 완비 (`chart_check_text/` 5종) |

## 계열 정리

#2292(기하 잘림) + #2293(텍스트 소실) 로 **PNG 차트 렌더 완성** — SVG
경로와 동등한 시각 산출. 파생 규칙: 폴백은 한국어 가용 폰트 우선 +
존재 확인 체인 (메모리 및 코드 주석 기록).
