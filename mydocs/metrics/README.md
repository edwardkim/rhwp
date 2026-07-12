# 코드 품질 대시보드 스냅샷 보관소

`scripts/metrics.sh` 의 **최신 대시보드는 이 폴더에 발행**된다 — `dashboard.html` 을
브라우저로 열면 현재 수치·델타·시계열(총량 추이 포함)을 바로 볼 수 있다 (tracked 라
클론만 받아도 열람 가능). raw 실행 이력(`output/metrics_history/`)은 `output/` 이
gitignore 라 보존되지 않는다. **의미 있는 시점의 측정을 날짜 폴더로 커밋해 공유·추적**한다.

## 규약

- 위치: `mydocs/metrics/{YYYY-MM-DD}/` — `metrics.json` + `metrics_history.json`(추세 요약) +
  `dashboard.html` 세트(자체 완결 열람 — 델타 카드/추세 차트 포함).
- `output/metrics_history/` 는 실행별 단기 이력(최근 30개 롤링, 비커밋)이고, 본 보관소는
  **의미 있는 시점의 장기 보존** — 두 축은 상보적이다.
- **커밋 주기: 의미 있는 시점만** — 리팩토링 Phase 경계(#1883), 릴리즈, 코드 리뷰(r-code-review)
  시점. 매 실행 커밋은 하지 않는다.
- 생성: `./scripts/metrics.sh --snapshot` (수집 후 오늘 날짜 폴더로 자동 복사).
- 열람: 폴더에서 `python3 -m http.server 8080` 후 `http://localhost:8080/dashboard.html`
  (file:// 는 CORS 제한 — [manual/dashboard.md](../manual/dashboard.md) 참조).

## 스냅샷 목록

| 날짜 | 맥락 | 핵심 지표 |
|------|------|----------|
| 2026-07-04 | **#1883 리팩토링 계획 영점** (마지막 리팩토링 후 재진단) | 1,200줄 초과 70개 / CC>25 80개(최대 288) / clippy 0 / 테스트 2,820 pass |
| 2026-07-04-r1 | **#1904 라운드 1 재평가** (object_ops 분할 + typeset 미주 분리 후) | typeset CC 282→104(+분리 179) / 초과 72·CC>25 81(분할 과도기 +2/+1) / 테스트 2,858 |
