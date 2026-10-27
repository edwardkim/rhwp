# PR #8043 리뷰 — feat(hyperlink): 문서 내부 책갈피 링크를 넣고 고치고 읽는다

## 최종 판정

**메인터너 보정 후 수용 가능 — feat(hyperlink): 문서 내부 책갈피 링크를 넣고 고치고 읽는다 범위의 로컬 검증 완료.**

원 contributor head의 직접 merge 판정과 메인터너 보정한 통합 후보의 판정을 구분한다. 원격 CI·merge·후속 처리는 실제 수행 후 별도로 기록한다.

## 접수 정보와 적용 계보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#8043](https://github.com/edwardkim/rhwp/pull/8043) / semanticist21 |
| 원 head | `a0bb863a08edf4e1461fbbf18ac2baaeaaea4450` |
| 통합 브랜치 | `review/semanticist21-20261019` |
| 검증 코드·테스트 source | `04c92da78165ccbf63b467855591df183655a98d` |
| 고정 upstream/devel | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 원격 조회 시 상태 | 2026-10-27 조회에서 원 head 불변·open·reviewer jangster77 확인. 원 Draft 여부와 통합 후보 판정은 별개 |

| 원 커밋 | 현재 통합의 대응 |
| --- | --- |
| `a0bb863a08edf4e1461fbbf18ac2baaeaaea4450` | `b83facb57ce70ff49716fbcce801ee6cb3fb217b` |

## 구현 주장·공통 소비 경로·반례

책갈피 내부 링크의 encoding/classification을 URL 문자열 추정과 구분한다. insert/update/query와 HWP/HWPX 저장 뒤 재열기에서 bookmark 대상이 보존되는지 확인하고, 기존 웹 링크가 내부 책갈피로 바뀌지 않는 대조군을 둔다. 기존 passthrough invalidation guard의 호출 소비와 연결되며 테스트 통과를 URL 외부 이동의 실제 실행 증거로 확대하지 않는다.

## 원 PR의 실제 회귀 실행

아래 함수와 실제 최종 PASS 행을 대조했다. 계약 검사 통과와 독립 Print의 시각 일치를 구분한다.

- [tests/cases/issue_6963_bookmark_hyperlink.rs](../../../tests/cases/issue_6963_bookmark_hyperlink.rs): SHA-256 `fb9ace1ce4c092a6ace6b6feadbd5678e8525e07d10da783d6bbc4284ea21a91`.
  - `bookmark_command_matches_hancom_encoding`: PASS, `rhwp::regression_suite_012`, 로컬 `logs/coherent-full-nextest-4697.log:7342`.
  - `hancom_sample_bookmark_link_reports_its_bookmark`: PASS, `rhwp::regression_suite_012`, 로컬 `logs/coherent-full-nextest-4697.log:7346`.
  - `bookmark_link_inserts_updates_and_survives_hwp_and_hwpx`: PASS, `rhwp::regression_suite_012`, 로컬 `logs/coherent-full-nextest-4697.log:7343`.
  - `web_link_is_not_turned_into_bookmark_link`: PASS, `rhwp::regression_suite_012`, 로컬 `logs/coherent-full-nextest-4697.log:7345`.
- [tests/issue_2724_passthrough_invalidation_guard.rs](../../../tests/issue_2724_passthrough_invalidation_guard.rs): SHA-256 `e2663ebd831dc39e4787a31c4cacd11a95cc2136a915d5cfae59047e37ebe7dd`.
  - `classification_drift_is_blocked`: PASS, `rhwp::regression_suite_011`, 로컬 `logs/coherent-full-nextest-4697.log:6991`.
  - `stale_exemptions_are_reclaimed`: PASS, `rhwp::regression_suite_011`, 로컬 `logs/coherent-full-nextest-4697.log:7005`.
  - `delegation_targets_actually_invalidate`: PASS, `rhwp::regression_suite_011`, 로컬 `logs/coherent-full-nextest-4697.log:6990`.
  - `invalidation_density_ledger_is_ratcheted`: PASS, `rhwp::regression_suite_011`, 로컬 `logs/coherent-full-nextest-4697.log:6992`.
  - `guard_scanner_self_check`: PASS, `rhwp::regression_suite_011`, 로컬 `logs/coherent-full-nextest-4697.log:7002`.

## 독립 Print·전쪽 TSV·대표 페이지

이 PR의 기존 검증 입력은 최종 조건 번호 197에 연결한다. 다른 PR의 평균 점수로 이 범위를 대신하지 않는다.

| 조건 | 전체 쪽수 | Native 최저 raw·전체 TSV | fresh WASM 최저 raw·전체 TSV |
| --- | ---: | --- | --- |
| 197 | 1 | [100.00000](../assets/s21_20261027/n/c/197.tsv) | [100.00000](../assets/s21_20261027/w/c/197.tsv) |

[입력·Print 경로와 SHA, producer·DPI·글꼴, base TSV 및 대표 review·overlay](../assets/s21_20261027/README.md)를 각 조건 번호로 대조한다. raw는 색상 경계 보정 전 2px 내용 실루엣이며 RGB 완전 일치가 아니다. 실제 빈 페이지의 raw 100%는 가시 내용의 정확성 증거로 사용하지 않고, 원본·독립 Print의 쪽수와 소유 계약을 별도로 검사한다.

신규 검사는 문단·줄·개체의 소유·순서·누락·중복·포함·겹침 관계를 기본으로 한다. 독립 Print와 실제 판독을 마치고 Native/fresh WASM이 모두 98% 이상인 완료 페이지만 독립 좌표 ±2px를 허용하며 관계 검사도 유지한다. 원본 HU·크기·비율 계약과 임의 화면 좌표 핀을 구분한다.

## 리베이스와 전체 검증의 경계

새 base #8116의 기존 네 입력은 같은 Print·글꼴·DPI에서 현재 보정 범위와 함께 비교했다. 확정된 중첩 표 원점에서 측정과 배치가 같은 가용 높이를 소비하도록 보정했으며 기존 15개 검사의 기대값을 바꾸지 않았다. 사용자 지시의 페이지별 하락 한계는 0.1%p다. base의 90% 미만은 변경 전 비교 자료로 구분하고 후보는 모든 관련 쪽에서 90% 이상을 유지한다.

필수 로컬 게이트는 최종 실행 결과와 source SHA를 대조해 완료했다. 전체 Rust·Skia·Studio의 실제 종료 집계는 공통 증거의 실행 요약을 따른다.

## 원격 처리와 후속 기록

통합 PR을 실제 템플릿에 맞춰 제출한 뒤 정확한 head의 CI·mergeability를 확인한다. 최신 지시에 따라 code head CI 성공 뒤 같은 통합 PR의 trailing commit으로 self-review·오늘할일을 반영하고 trailing head CI 후 일반 병합한다. 원 PR별 리뷰는 각각 유지하고 실제 통합 merge·devel 포함을 확인한 뒤 존댓말로 원 기여와 보정 범위, CI·merge SHA 및 그 SHA에 고정한 대표 review·overlay를 연결한다. contributor fork·branch와 원 history는 보존한다.

대표 쪽 선정: 책갈피 링크 문단의 표시와 순서; 링크 대상 읽기·갱신은 별도 API 검사. 다른 PR의 임의 페이지로 대체하지 않는다.

## Merge 후 contributor PR comment 계획

[Visual Sweep merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라 통합 merge와 자산의 devel 포함을 확인한 뒤에만 게시한다. 이 계획을 원 PR의 직접 merge 승인으로 쓰지 않는다.

검증 범위는 위 조건 197 전체 쪽이다. raw 90% 미만·누락·측정 불가 후보는 양쪽 0쪽이며, 대표 페이지와 전체 범위의 최저값을 구분한다. 평균·RGB 진단값으로 raw 미달을 면제하지 않았다.

대표는 조건 197의 1쪽이다. 다음 수치는 실제 새 출력·같은 독립 Print를 대조한 값이며 완전한 RGB 일치의 선언이 아니다.

| 출력 | raw 실루엣 | pixel_match | visual_accuracy_proxy_percent | review / overlay |
| --- | ---: | ---: | ---: | --- |
| native | 100.00000% | 99.95196% | 35.54217% | [review](../assets/s21_20261027/panels/native/197-p001-review.png) / [overlay](../assets/s21_20261027/panels/native/197-p001-overlay.png) |
| wasm | 100.00000% | 99.96913% | 59.49926% | [review](../assets/s21_20261027/panels/wasm/197-p001-review.png) / [overlay](../assets/s21_20261027/panels/wasm/197-p001-overlay.png) |

직접 판독: 링크 문단·책갈피 문단의 내용·순서·간격을 Print와 대조했다. 실제 링크 대상과 왕복 보존은 별도 편집 검사로 확인한다.

게시 문안은 원 기여에 감사드리고, 원 PR 번호·원 head와 통합 후보의 보정 범위·실제 CI·merge SHA를 존댓말로 설명한다. 같은 asset의 아래 URL에서 `<merge-commit-sha>`만 실제 merge SHA로 바꾼다.

`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/native/197-p001-review.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/native/197-p001-overlay.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/wasm/197-p001-review.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/wasm/197-p001-overlay.png`

실제 LF 본문 파일을 `--body-file`로 게시하고 API로 본문·한글·이미지 URL을 재조회한다. 동일 merge·동일 증거의 기존 comment가 있으면 중복 게시하지 않는다.

## 통합 PR #8319의 code head CI 확정

이 PR의 원 contributor head와 위 개별 검증 범위를 유지한 통합 후보 `aee691ae596d288382369b9d5243f4cd06e7fa65`의
[최종 code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)가 성공했다. [PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 문서-only trailing head CI와 정책 성공 뒤 일반 병합한다.
원 contributor PR 자체를 직접 merge한 것으로 표시하지 않는다. 제품 source 불변인 추가 테스트
보정과 최종 원격 검증은 [통합 리뷰](pr_8319_review.md)·[CI 증거](../assets/s21_20261027/final-ci.json)에 구분했다.
최신 지시대로 같은 PR에 self-review·오늘할일을 trailing commit으로 반영한다. 아직 병합하지 않았으며 실제 병합 뒤 원 PR별 결과 comment와 종료를 진행한다.
