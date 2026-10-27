# PR #7559 리뷰 — 새 표의 저장 전후 원점

## 최종 판정

**메인터너 보정 후 수용 가능 — 새 표의 저장 전후 원점 범위의 로컬 검증 완료.**

원 contributor head의 직접 merge 판정과 메인터너 보정한 통합 후보의 판정을 구분한다. 원격 CI·merge·후속 처리는 실제 수행 후 별도로 기록한다.

## 접수 정보와 적용 계보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#7559](https://github.com/edwardkim/rhwp/pull/7559) / semanticist21 |
| 원 head | `5f1f1057bd1b7981b3801b815aa28b98860932a2` |
| 통합 브랜치 | `review/semanticist21-20261019` |
| 검증 코드·테스트 source | `04c92da78165ccbf63b467855591df183655a98d` |
| 고정 upstream/devel | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 원격 조회 시 상태 | 2026-10-27 조회에서 원 head 불변·open·reviewer jangster77 확인. 원 Draft 여부와 통합 후보 판정은 별개 |

| 원 커밋 | 현재 통합의 대응 |
| --- | --- |
| `ebba2950fe97e00d38715e05236f6182f1a74da8` | `15f1ed01500cbbf3ace1be1b34f1c64f52b9c66d` |
| `2ee1ad24b403d384262d499db071bca10f6be5d5` | `d7e9463ef247987a73f7cea27ce438f12f644c95` |
| `5f1f1057bd1b7981b3801b815aa28b98860932a2` | 원 PR의 devel merge 전용 커밋; 별도 기능 체리픽 없음 |

## 구현 주장·공통 소비 경로·반례

공개 API로 생성한 새 표의 저장 전후 원점 불일치를 재현한 기여다. 공통 computed_host_frame의 표 원점·점유 끝을 fit과 실제 배치가 함께 소비하도록 보정했다. 작은/큰 빈 줄, 양수 offset, 다음 문단 앞 간격, 표 위로 들어가는 글줄과 회피 글줄을 구분한다. 원점 불변의 상대 오차는 독립 Print의 절대 좌표 핀과 구분한다.

## 원 PR의 실제 회귀 실행

아래 함수와 실제 최종 PASS 행을 대조했다. 계약 검사 통과와 독립 Print의 시각 일치를 구분한다.

- [tests/cases/table_hwpx_save_origin_contract.rs](../../../tests/cases/table_hwpx_save_origin_contract.rs): SHA-256 `e5ef237b0abe215694b4c3ec0e6492d576ef18474d58aa11d8c42ae8e8f2bb2d`.
  - `generated_table_padding_origin_survives_hwpx_save`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-full-nextest-4697.log:8598`.
  - `generated_table_offset_origin_survives_hwpx_save`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-full-nextest-4697.log:8597`.

## 독립 Print·전쪽 TSV·대표 페이지

이 PR의 기존 검증 입력은 최종 조건 번호 059, 060, 061, 062, 063, 064, 065, 066, 067, 068, 069, 070, 071, 072, 076, 077, 078, 079에 연결한다. 다른 PR의 평균 점수로 이 범위를 대신하지 않는다.

| 조건 | 전체 쪽수 | Native 최저 raw·전체 TSV | fresh WASM 최저 raw·전체 TSV |
| --- | ---: | --- | --- |
| 059 | 1 | [100.00000](../assets/s21_20261027/n/c/059.tsv) | [100.00000](../assets/s21_20261027/w/c/059.tsv) |
| 060 | 1 | [100.00000](../assets/s21_20261027/n/c/060.tsv) | [100.00000](../assets/s21_20261027/w/c/060.tsv) |
| 061 | 1 | [100.00000](../assets/s21_20261027/n/c/061.tsv) | [100.00000](../assets/s21_20261027/w/c/061.tsv) |
| 062 | 1 | [100.00000](../assets/s21_20261027/n/c/062.tsv) | [100.00000](../assets/s21_20261027/w/c/062.tsv) |
| 063 | 1 | [100.00000](../assets/s21_20261027/n/c/063.tsv) | [100.00000](../assets/s21_20261027/w/c/063.tsv) |
| 064 | 1 | [100.00000](../assets/s21_20261027/n/c/064.tsv) | [100.00000](../assets/s21_20261027/w/c/064.tsv) |
| 065 | 1 | [100.00000](../assets/s21_20261027/n/c/065.tsv) | [100.00000](../assets/s21_20261027/w/c/065.tsv) |
| 066 | 1 | [100.00000](../assets/s21_20261027/n/c/066.tsv) | [100.00000](../assets/s21_20261027/w/c/066.tsv) |
| 067 | 1 | [100.00000](../assets/s21_20261027/n/c/067.tsv) | [100.00000](../assets/s21_20261027/w/c/067.tsv) |
| 068 | 1 | [100.00000](../assets/s21_20261027/n/c/068.tsv) | [100.00000](../assets/s21_20261027/w/c/068.tsv) |
| 069 | 1 | [100.00000](../assets/s21_20261027/n/c/069.tsv) | [100.00000](../assets/s21_20261027/w/c/069.tsv) |
| 070 | 1 | [100.00000](../assets/s21_20261027/n/c/070.tsv) | [100.00000](../assets/s21_20261027/w/c/070.tsv) |
| 071 | 1 | [100.00000](../assets/s21_20261027/n/c/071.tsv) | [100.00000](../assets/s21_20261027/w/c/071.tsv) |
| 072 | 1 | [100.00000](../assets/s21_20261027/n/c/072.tsv) | [100.00000](../assets/s21_20261027/w/c/072.tsv) |
| 076 | 1 | [100.00000](../assets/s21_20261027/n/c/076.tsv) | [100.00000](../assets/s21_20261027/w/c/076.tsv) |
| 077 | 1 | [100.00000](../assets/s21_20261027/n/c/077.tsv) | [100.00000](../assets/s21_20261027/w/c/077.tsv) |
| 078 | 1 | [100.00000](../assets/s21_20261027/n/c/078.tsv) | [100.00000](../assets/s21_20261027/w/c/078.tsv) |
| 079 | 1 | [100.00000](../assets/s21_20261027/n/c/079.tsv) | [100.00000](../assets/s21_20261027/w/c/079.tsv) |

[입력·Print 경로와 SHA, producer·DPI·글꼴, base TSV 및 대표 review·overlay](../assets/s21_20261027/README.md)를 각 조건 번호로 대조한다. raw는 색상 경계 보정 전 2px 내용 실루엣이며 RGB 완전 일치가 아니다. 실제 빈 페이지의 raw 100%는 가시 내용의 정확성 증거로 사용하지 않고, 원본·독립 Print의 쪽수와 소유 계약을 별도로 검사한다.

신규 검사는 문단·줄·개체의 소유·순서·누락·중복·포함·겹침 관계를 기본으로 한다. 독립 Print와 실제 판독을 마치고 Native/fresh WASM이 모두 98% 이상인 완료 페이지만 독립 좌표 ±2px를 허용하며 관계 검사도 유지한다. 원본 HU·크기·비율 계약과 임의 화면 좌표 핀을 구분한다.

## 리베이스와 전체 검증의 경계

새 base #8116의 기존 네 입력은 같은 Print·글꼴·DPI에서 현재 보정 범위와 함께 비교했다. 확정된 중첩 표 원점에서 측정과 배치가 같은 가용 높이를 소비하도록 보정했으며 기존 15개 검사의 기대값을 바꾸지 않았다. 사용자 지시의 페이지별 하락 한계는 0.1%p다. base의 90% 미만은 변경 전 비교 자료로 구분하고 후보는 모든 관련 쪽에서 90% 이상을 유지한다.

필수 로컬 게이트는 최종 실행 결과와 source SHA를 대조해 완료했다. 전체 Rust·Skia·Studio의 실제 종료 집계는 공통 증거의 실행 요약을 따른다.

## 원격 처리와 후속 기록

통합 PR을 실제 템플릿에 맞춰 제출한 뒤 정확한 head의 CI·mergeability를 확인한다. 최신 지시에 따라 code head CI 성공 뒤 같은 통합 PR의 trailing commit으로 self-review·오늘할일을 반영하고 trailing head CI 후 일반 병합한다. 원 PR별 리뷰는 각각 유지하고 실제 통합 merge·devel 포함을 확인한 뒤 존댓말로 원 기여와 보정 범위, CI·merge SHA 및 그 SHA에 고정한 대표 review·overlay를 연결한다. contributor fork·branch와 원 history는 보존한다.

대표 쪽 선정: 새 표를 HWPX로 저장·재열기한 표와 뒤 본문의 원점. 다른 PR의 임의 페이지로 대체하지 않는다.

## Merge 후 contributor PR comment 계획

[Visual Sweep merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라 통합 merge와 자산의 devel 포함을 확인한 뒤에만 게시한다. 이 계획을 원 PR의 직접 merge 승인으로 쓰지 않는다.

검증 범위는 위 조건 059, 060, 061, 062, 063, 064, 065, 066, 067, 068, 069, 070, 071, 072, 076, 077, 078, 079 전체 쪽이다. raw 90% 미만·누락·측정 불가 후보는 양쪽 0쪽이며, 대표 페이지와 전체 범위의 최저값을 구분한다. 평균·RGB 진단값으로 raw 미달을 면제하지 않았다.

대표는 조건 077의 1쪽이다. 다음 수치는 실제 새 출력·같은 독립 Print를 대조한 값이며 완전한 RGB 일치의 선언이 아니다.

| 출력 | raw 실루엣 | pixel_match | visual_accuracy_proxy_percent | review / overlay |
| --- | ---: | ---: | ---: | --- |
| native | 100.00000% | 99.60387% | 1.97222% | [review](../assets/s21_20261027/panels/native/077-p001-review.png) / [overlay](../assets/s21_20261027/panels/native/077-p001-overlay.png) |
| wasm | 100.00000% | 99.60297% | 1.80455% | [review](../assets/s21_20261027/panels/wasm/077-p001-review.png) / [overlay](../assets/s21_20261027/panels/wasm/077-p001-overlay.png) |

직접 판독과 남은 차이는 공통 증거의 해당 조건·쪽 기록을 따른다. 새 출력 쌍과 실제 이전 판독의 동일성을 검증한 재사용 범위를 구분했다.

게시 문안은 원 기여에 감사드리고, 원 PR 번호·원 head와 통합 후보의 보정 범위·실제 CI·merge SHA를 존댓말로 설명한다. 같은 asset의 아래 URL에서 `<merge-commit-sha>`만 실제 merge SHA로 바꾼다.

`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/native/077-p001-review.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/native/077-p001-overlay.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/wasm/077-p001-review.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/wasm/077-p001-overlay.png`

실제 LF 본문 파일을 `--body-file`로 게시하고 API로 본문·한글·이미지 URL을 재조회한다. 동일 merge·동일 증거의 기존 comment가 있으면 중복 게시하지 않는다.

## 통합 PR #8319의 code head CI 확정

이 PR의 원 contributor head와 위 개별 검증 범위를 유지한 통합 후보 `aee691ae596d288382369b9d5243f4cd06e7fa65`의
[최종 code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)가 성공했다. [PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 문서-only trailing head CI와 정책 성공 뒤 일반 병합한다.
원 contributor PR 자체를 직접 merge한 것으로 표시하지 않는다. 제품 source 불변인 추가 테스트
보정과 최종 원격 검증은 [통합 리뷰](pr_8319_review.md)·[CI 증거](../assets/s21_20261027/final-ci.json)에 구분했다.
최신 지시대로 같은 PR에 self-review·오늘할일을 trailing commit으로 반영한다. 아직 병합하지 않았으며 실제 병합 뒤 원 PR별 결과 comment와 종료를 진행한다.
