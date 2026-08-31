# PR #6489 검토 - WMF 미지원 래스터 연산

- 원 PR head: 2dc0c84cfb5bd08fb9ebe02bf333e6eadb519967
- 통합 cherry-pick: b061a2a208776b202ef05b77bdffd4c1b5b51133
- 통합 기준: 76532b4da0e720026fb24211ad0c382884d3b970

## 판정: 메인터너 보정 됨 수용 가능

### [P1] 문서 전체 이력의 동일 키 PATINVERT를 무조건 상쇄한다

src/wmf/converter/svg/ternary_raster_operator.rs는 브러시 전용 PATINVERT의 기하·fill 키가 이전 어느 시점에라도 있으면 기존 항목을 제거하고 현재 연산을 생략한다. 중간 draw의 존재나 세 연산 관용구의 인접 순서는 확인하지 않는다.

따라서 같은 기하·브러시의 후속 PATINVERT가 독립적인 그리기여도 출력이 사라질 수 있다. 현재 시험은 의도된 쌍 상쇄만 확인하며 중간 draw 뒤 같은 키를 보존하는 경우를 다루지 않는다.

## 필요한 최소 보정

- 상쇄를 확인된 인접 관용구로 한정하거나 레코드 순서와 상태 전이를 모델링한다.
- 중간 draw 뒤 동일 키 PATINVERT가 생략되지 않는 회귀를 추가한다.

## 검증 및 증적

- issue_6469_wmf_brush_only_raster_ops: 2/2 통과
- 전체 nextest, Native Skia, 배포용 WASM: 공통 검증 통과
- 원 PR 시각 자료: mydocs/report/6469-wmf-raster-ops/{before,after,hangul2022}.png
- 현재 통합 head의 WMF oracle PDF 재검증은 수행하지 않았다.
## 2026-08-31 메인터너 보정 검증

**최종 판정: 메인터너 보정 됨 수용 가능.**

- `PATINVERT -> DPA -> PATINVERT`의 연속 의도 패턴만 상쇄하도록 보정했고, 중간 draw 뒤의 같은 key `PATINVERT`는 보존하는 회귀를 추가했다.
- focused #6469/#6145/#5886/#5301/#5677/#5696/#6495/#6494, `issue_1139_inline_picture_duplicate` 86/86, `svg_snapshot` 8/8 및 전체 nextest `8888 passed, 0 failed, 46 skipped`를 확인했다.
- 현 보정 후보는 아직 commit되지 않은 작업트리이므로 이 판정은 특정 새 commit SHA가 아니라 현재 후보와 위 검증 결과에 귀속된다.
