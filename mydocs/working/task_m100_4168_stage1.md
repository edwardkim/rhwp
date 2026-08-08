---
kind: working
status: completed
issue: 4168
last_verified: 2026-08-08
---

# Task #4168 Stage 1 - find_metric 선계산 색인

## 구현

- `find_metric`을 `OnceLock` 선계산 색인 조회로 교체했다. 색인은
  `HashMap<&'static str, [(metric, bold_fallback); 4]>` — 이름 단독 키라 임의 `&str`
  조회(`Borrow<str>`)가 유지되고, (bold,italic) 4조합 슬롯을 legacy 3단 폴백 사다리와
  같은 우선순위(정확 → bold-only(italic 요청 한정) → 이름 첫 엔트리 + `bold_fallback=bold`)로
  선주입했다.
- 기존 선형 스캔은 `legacy_find_metric`(cfg(test))으로 바이트 단위 보존해 등가성 오라클로
  쓴다. `resolve_metric_alias`는 종전대로 조회 진입부에서 적용된다.

## 검증 결과

- 신규 `index_matches_legacy_linear_scan_exhaustively`: FONT_METRICS 600개 이름 전체 +
  alias 좌변 67개 전체 + 미등록 이름(빈 문자열 포함) × bold/italic 4조합 전수에서 metric
  포인터 동일성 + `bold_fallback` 일치 — pass.
- `cargo test --profile release-test --lib font_metrics`: 8 passed / 0 failed.
- 적대 리뷰(별도 세션): 테이블 중복 트리플 0건 확인, alias 양방향 누락 0, NFD 자모·NUL·
  30,000자 장문 등 ~3,600 이름 × 4조합 공격 전수 일치, 16스레드 최초 사용 경합 3회 통과,
  wasm32 check 통과 — 반박 실패(등가성 결함 0건).
- 실측(거대 셀 문서, release-test): `build_page_tree` ≈20.7ms → ≈17.2ms, 캐럿 질의
  ≈21.5ms → ≈17.2ms (프로파일 귀속 ~16%와 부합).
- 알려진 후속: `src/tools/font_metric_gen.rs` 생성기가 구식 선형 스캔을 방출한다(선재
  드리프트) — 재생성 시 본 색인·테스트가 소실되므로 생성기 갱신을 별도 이슈로 권고.

## 부수 발견 — #4181

게이트 실행 중 `tests/issue_2007_nested_cell_pagination`(p15-16)이 이 레이어에서 비결정
실패함을 발견, 원인 격리 결과 **devel 선재의 힙 할당 순서 민감성**으로 확정했다(#4181):
의미 무변경 대조군(색인 빌드만 하고 legacy 스캔 사용, 캐시 value 8바이트 확장) 둘 다 동일
플레이크를 재현하고, 색인 결과 등가성은 전수·10회 프로세스 반복으로 증명됨. 본 레이어
게이트는 해당 바이너리를 직렬 재시도로 통과 확인(3회 내 pass)하고 나머지 전 게이트는
결정적 통과. #4181 해소 전까지 이 fixture 의 실패는 회귀 신호로 오독하지 말 것.
