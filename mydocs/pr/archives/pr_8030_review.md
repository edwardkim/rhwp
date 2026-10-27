# PR #8030 리뷰 — fix(render): 반각 구두점은 글리프가 전진폭보다 넓을 때만 눌러 그린다

## 최종 판정

**메인터너 보정 후 수용 가능 — fix(render): 반각 구두점은 글리프가 전진폭보다 넓을 때만 눌러 그린다 범위의 로컬 검증 완료.**

원 contributor head의 직접 merge 판정과 메인터너 보정한 통합 후보의 판정을 구분한다. 원격 CI·merge·후속 처리는 실제 수행 후 별도로 기록한다.

## 접수 정보와 적용 계보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#8030](https://github.com/edwardkim/rhwp/pull/8030) / semanticist21 |
| 원 head | `08d0a12a60b74da18eeb0cd9b834c7bfc7730522` |
| 통합 브랜치 | `review/semanticist21-20261019` |
| 검증 코드·테스트 source | `04c92da78165ccbf63b467855591df183655a98d` |
| 고정 upstream/devel | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 원격 조회 시 상태 | 2026-10-27 조회에서 원 head 불변·open·reviewer jangster77 확인. 원 Draft 여부와 통합 후보 판정은 별개 |

| 원 커밋 | 현재 통합의 대응 |
| --- | --- |
| `08d0a12a60b74da18eeb0cd9b834c7bfc7730522` | `1fe9e27a5ca17172fb06f1ac24a0abe40fc30431` |

## 구현 주장·공통 소비 경로·반례

halfwidth_punct_paint_scale은 전진폭과 실제 glyph 폭의 비율을 최대 1로 제한한다. 이미 좁은 따옴표를 0.5배로 다시 누르거나 작은 glyph를 늘리지 않는다. 폭을 얻지 못한 기존 fallback과 정상 전각 측정은 별도 경계로 남긴다. 측정의 논리 advance를 바꾸는 수정이 아니라 Canvas·Skia가 소비하는 paint 배율을 공유하는 수정이다.

tests/cases/halfwidth_punct_paint_scale.rs의 넓음/좁음/같음/폭 없음 네 경계. 수치 예제의 20·10은 합성 함수 인자이며 실물 문서의 절대 위치 핀이 아니다.

## 원 PR의 실제 회귀 실행

아래 함수와 실제 최종 PASS 행을 대조했다. 계약 검사 통과와 독립 Print의 시각 일치를 구분한다.

- [tests/cases/halfwidth_punct_paint_scale.rs](../../../tests/cases/halfwidth_punct_paint_scale.rs): SHA-256 `2fa454245e11afb784f572dfb76b89aab84057fdeba9eb68798306a3d35687d4`.
  - `halfwidth_punct_paint_only_squeezes_glyphs_wider_than_advance`: PASS, `rhwp::regression_suite_008`, 로컬 `logs/coherent-full-nextest-4697.log:6076`.

## 독립 Print·전쪽 TSV·대표 페이지

이 PR의 기존 검증 입력은 최종 조건 번호 113에 연결한다. 다른 PR의 평균 점수로 이 범위를 대신하지 않는다.

| 조건 | 전체 쪽수 | Native 최저 raw·전체 TSV | fresh WASM 최저 raw·전체 TSV |
| --- | ---: | --- | --- |
| 113 | 19 | [96.64038](../assets/s21_20261027/n/c/113.tsv) | [96.90367](../assets/s21_20261027/w/c/113.tsv) |

[입력·Print 경로와 SHA, producer·DPI·글꼴, base TSV 및 대표 review·overlay](../assets/s21_20261027/README.md)를 각 조건 번호로 대조한다. raw는 색상 경계 보정 전 2px 내용 실루엣이며 RGB 완전 일치가 아니다. 실제 빈 페이지의 raw 100%는 가시 내용의 정확성 증거로 사용하지 않고, 원본·독립 Print의 쪽수와 소유 계약을 별도로 검사한다.

신규 검사는 문단·줄·개체의 소유·순서·누락·중복·포함·겹침 관계를 기본으로 한다. 독립 Print와 실제 판독을 마치고 Native/fresh WASM이 모두 98% 이상인 완료 페이지만 독립 좌표 ±2px를 허용하며 관계 검사도 유지한다. 원본 HU·크기·비율 계약과 임의 화면 좌표 핀을 구분한다.

## 리베이스와 전체 검증의 경계

새 base #8116의 기존 네 입력은 같은 Print·글꼴·DPI에서 현재 보정 범위와 함께 비교했다. 확정된 중첩 표 원점에서 측정과 배치가 같은 가용 높이를 소비하도록 보정했으며 기존 15개 검사의 기대값을 바꾸지 않았다. 사용자 지시의 페이지별 하락 한계는 0.1%p다. base의 90% 미만은 변경 전 비교 자료로 구분하고 후보는 모든 관련 쪽에서 90% 이상을 유지한다.

필수 로컬 게이트는 최종 실행 결과와 source SHA를 대조해 완료했다. 전체 Rust·Skia·Studio의 실제 종료 집계는 공통 증거의 실행 요약을 따른다.

## 원격 처리와 후속 기록

통합 PR을 실제 템플릿에 맞춰 제출한 뒤 정확한 head의 CI·mergeability를 확인한다. 최신 지시에 따라 code head CI 성공 뒤 같은 통합 PR의 trailing commit으로 self-review·오늘할일을 반영하고 trailing head CI 후 일반 병합한다. 원 PR별 리뷰는 각각 유지하고 실제 통합 merge·devel 포함을 확인한 뒤 존댓말로 원 기여와 보정 범위, CI·merge SHA 및 그 SHA에 고정한 대표 review·overlay를 연결한다. contributor fork·branch와 원 history는 보존한다.

## 이 PR의 증거 종류

전진폭보다 넓음·좁음·같음·폭 없음 네 독립 함수 경계를 검사한다. 표시용 굵은 글꼴 PNG를 반각 배율의 결함 검출로 사용하지 않는다.

위 실제 함수 PASS가 주장한 계약의 증거다. 공통 시각 대조군을 해당 결함의 수정 전 FAIL 또는 실제 브라우저 이벤트 실행으로 바꾸어 설명하지 않는다.

## 통합 PR #8319의 code head CI 확정

이 PR의 원 contributor head와 위 개별 검증 범위를 유지한 통합 후보 `aee691ae596d288382369b9d5243f4cd06e7fa65`의
[최종 code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)가 성공했다. [PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 문서-only trailing head CI와 정책 성공 뒤 일반 병합한다.
원 contributor PR 자체를 직접 merge한 것으로 표시하지 않는다. 제품 source 불변인 추가 테스트
보정과 최종 원격 검증은 [통합 리뷰](pr_8319_review.md)·[CI 증거](../assets/s21_20261027/final-ci.json)에 구분했다.
최신 지시대로 같은 PR에 self-review·오늘할일을 trailing commit으로 반영한다. 아직 병합하지 않았으며 실제 병합 뒤 원 PR별 결과 comment와 종료를 진행한다.
