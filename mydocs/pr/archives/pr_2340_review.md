# PR #2340 검토 — 한컴돋움/한컴바탕 → Haansoft 실메트릭 연결 (#2279 반복분)

- PR: https://github.com/edwardkim/rhwp/pull/2340 (planet6897) — #2279 umbrella open 유지
- 충돌 0

## 변경 본질

한컴돋움/한컴바탕이 함초롬(HCR) 계열로 치환·측정되던 것을 **실체 폰트**
(TTF name table lang=0x412: HDOTUM=한컴돋움=Haansoft Dotum)의 실메트릭으로
연결 — 3중 증거(name table + 한글 PDF Justify 무신축 run dx 실측('*'
0.583em·음절 1.0em = Haansoft hmtx 정확 일치) + 종전 경로의 줄수 ±1 오차
귀속). "렌더링 의미 추정 금지 — 권위 자료 확정" 원칙의 모범 적용.

- **alias 2계층 동기화 규칙 준수 확인**: style_resolver(치환 제거) +
  font_metrics_data(metric alias 연결) 양쪽 정합, SVG 폴백 체인은 svg.rs
  직접 처리로 불변 명시. 함초롬돋움은 종전 HCR 유지(미실측 축 보수 — #2156
  바탕만 확정된 상태 존중)
- area_dot fallback 도 Haansoft 계열 embedded 신뢰로 확장 + 회귀 테스트

## 로컬 재실증 (merged tree)

| 게이트 | 결과 |
|--------|------|
| 신규 단위 테스트 | 1/1 (한컴→Haansoft + 함초롬→HCR 유지 고정) |
| 핀 | byeolpyo 4/26 · 시장 312 · 연결맵 385 · 36395825=2 유지 |
| `cargo test --tests` | 실패 0 / fmt / clippy 0 |

92셋 86→88·358 recount FIXED 39/REGRESSED 0·픽셀 90.88→93.63% 는 그들
오라클/도구 산출(대상 샘플이 10k 코퍼스라 저장소 밖) — 저장소 내 핀·스위트
전면 무회귀와 방법론(3중 증거·base/head 픽셀 대조 도구 동봉)으로 방향 확인.

## 판단

**merge 권고.** 폰트 정체 실증 기반의 정밀도 전진, 미실측 축(함초롬돋움
ASCII)을 보수적으로 남긴 절제 포함.
