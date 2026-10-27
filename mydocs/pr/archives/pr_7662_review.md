# PR #7662 리뷰 — 수정: 개체 바로 앞에서 나눈 문단이 그 개체로 시작하게 한다

## 최종 판정

**메인터너 보정 후 수용 가능 — 수정: 개체 바로 앞에서 나눈 문단이 그 개체로 시작하게 한다 범위의 로컬 검증 완료.**

원 contributor head의 직접 merge 판정과 메인터너 보정한 통합 후보의 판정을 구분한다. 원격 CI·merge·후속 처리는 실제 수행 후 별도로 기록한다.

## 접수 정보와 적용 계보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#7662](https://github.com/edwardkim/rhwp/pull/7662) / semanticist21 |
| 원 head | `377bcb71e7404a741105a28303a3fae8b671fc9d` |
| 통합 브랜치 | `review/semanticist21-20261019` |
| 검증 코드·테스트 source | `04c92da78165ccbf63b467855591df183655a98d` |
| 고정 upstream/devel | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 원격 조회 시 상태 | 2026-10-27 조회에서 원 head 불변·open·reviewer jangster77 확인. 원 Draft 여부와 통합 후보 판정은 별개 |

| 원 커밋 | 현재 통합의 대응 |
| --- | --- |
| `f902422226983aa4f6ddd282db329791f9b84a32` | `eb4bbe5612b6a5f8ca617fda98275f5725823df6` |
| `377bcb71e7404a741105a28303a3fae8b671fc9d` | `686ace4604b5ac7bad74cad6b027d998273cd31d` |

## 구현 주장·공통 소비 경로·반례

Paragraph::split_at은 logical control 위치와 scalar 문자열 분할점을 분리한다. 새 문단으로 가는 control은 UTF-16 gap 시작부터 이동시키고, 끝 자동번호는 자리표 문자와 앞 문단에 남긴다. Field는 한 글자짜리 일반 개체로 더하지 않고 이관된 FieldRange의 control 참조로 함께 이동한다. split_before_control의 개체 앞/뒤 Enter, merge undo, 개체부터 복사, 자동번호와 제목 차례 표시를 각각 대조한다.

## 원 PR의 실제 회귀 실행

아래 함수와 실제 최종 PASS 행을 대조했다. 계약 검사 통과와 독립 Print의 시각 일치를 구분한다.

- [tests/cases/split_before_control.rs](../../../tests/cases/split_before_control.rs): SHA-256 `26f04d233cdb223cf53b236a27f48a7431716bbfef783991d991e8bcf2a016a7`.
  - `enter_right_before_object_starts_new_paragraph_with_it`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:8049`.
  - `enter_right_after_inline_object_keeps_it_in_first_paragraph`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:8047`.
  - `merge_undo_restores_paragraph_that_starts_with_object`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:8050`.
  - `copy_starting_at_object_pastes_it_first`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:8046`.
  - `split_leaves_auto_number_on_its_placeholder_character`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:8051`.
  - `split_moves_title_mark_slot_with_its_character`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:8052`.

## 독립 Print·전쪽 TSV·대표 페이지

이 PR의 기존 검증 입력은 최종 조건 번호 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194에 연결한다. 다른 PR의 평균 점수로 이 범위를 대신하지 않는다.

| 조건 | 전체 쪽수 | Native 최저 raw·전체 TSV | fresh WASM 최저 raw·전체 TSV |
| --- | ---: | --- | --- |
| 183 | 1 | [100.00000](../assets/s21_20261027/n/c/183.tsv) | [100.00000](../assets/s21_20261027/w/c/183.tsv) |
| 184 | 1 | [100.00000](../assets/s21_20261027/n/c/184.tsv) | [100.00000](../assets/s21_20261027/w/c/184.tsv) |
| 185 | 1 | [100.00000](../assets/s21_20261027/n/c/185.tsv) | [100.00000](../assets/s21_20261027/w/c/185.tsv) |
| 186 | 1 | [100.00000](../assets/s21_20261027/n/c/186.tsv) | [100.00000](../assets/s21_20261027/w/c/186.tsv) |
| 187 | 1 | [100.00000](../assets/s21_20261027/n/c/187.tsv) | [100.00000](../assets/s21_20261027/w/c/187.tsv) |
| 188 | 1 | [100.00000](../assets/s21_20261027/n/c/188.tsv) | [100.00000](../assets/s21_20261027/w/c/188.tsv) |
| 189 | 1 | [100.00000](../assets/s21_20261027/n/c/189.tsv) | [100.00000](../assets/s21_20261027/w/c/189.tsv) |
| 190 | 1 | [100.00000](../assets/s21_20261027/n/c/190.tsv) | [100.00000](../assets/s21_20261027/w/c/190.tsv) |
| 191 | 1 | [100.00000](../assets/s21_20261027/n/c/191.tsv) | [100.00000](../assets/s21_20261027/w/c/191.tsv) |
| 192 | 1 | [100.00000](../assets/s21_20261027/n/c/192.tsv) | [100.00000](../assets/s21_20261027/w/c/192.tsv) |
| 193 | 1 | [100.00000](../assets/s21_20261027/n/c/193.tsv) | [100.00000](../assets/s21_20261027/w/c/193.tsv) |
| 194 | 1 | [100.00000](../assets/s21_20261027/n/c/194.tsv) | [100.00000](../assets/s21_20261027/w/c/194.tsv) |

[입력·Print 경로와 SHA, producer·DPI·글꼴, base TSV 및 대표 review·overlay](../assets/s21_20261027/README.md)를 각 조건 번호로 대조한다. raw는 색상 경계 보정 전 2px 내용 실루엣이며 RGB 완전 일치가 아니다. 실제 빈 페이지의 raw 100%는 가시 내용의 정확성 증거로 사용하지 않고, 원본·독립 Print의 쪽수와 소유 계약을 별도로 검사한다.

신규 검사는 문단·줄·개체의 소유·순서·누락·중복·포함·겹침 관계를 기본으로 한다. 독립 Print와 실제 판독을 마치고 Native/fresh WASM이 모두 98% 이상인 완료 페이지만 독립 좌표 ±2px를 허용하며 관계 검사도 유지한다. 원본 HU·크기·비율 계약과 임의 화면 좌표 핀을 구분한다.

## 리베이스와 전체 검증의 경계

새 base #8116의 기존 네 입력은 같은 Print·글꼴·DPI에서 현재 보정 범위와 함께 비교했다. 확정된 중첩 표 원점에서 측정과 배치가 같은 가용 높이를 소비하도록 보정했으며 기존 15개 검사의 기대값을 바꾸지 않았다. 사용자 지시의 페이지별 하락 한계는 0.1%p다. base의 90% 미만은 변경 전 비교 자료로 구분하고 후보는 모든 관련 쪽에서 90% 이상을 유지한다.

필수 로컬 게이트는 최종 실행 결과와 source SHA를 대조해 완료했다. 전체 Rust·Skia·Studio의 실제 종료 집계는 공통 증거의 실행 요약을 따른다.

## 원격 처리와 후속 기록

통합 PR을 실제 템플릿에 맞춰 제출한 뒤 정확한 head의 CI·mergeability를 확인한다. 최신 지시에 따라 code head CI 성공 뒤 같은 통합 PR의 trailing commit으로 self-review·오늘할일을 반영하고 trailing head CI 후 일반 병합한다. 원 PR별 리뷰는 각각 유지하고 실제 통합 merge·devel 포함을 확인한 뒤 존댓말로 원 기여와 보정 범위, CI·merge SHA 및 그 SHA에 고정한 대표 review·overlay를 연결한다. contributor fork·branch와 원 history는 보존한다.

대표 쪽 선정: 개체 바로 앞 문단 나눔 뒤 인라인 사각형과 뒤 텍스트의 다음 문단 소속. 다른 PR의 임의 페이지로 대체하지 않는다.

## Merge 후 contributor PR comment 계획

[Visual Sweep merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라 통합 merge와 자산의 devel 포함을 확인한 뒤에만 게시한다. 이 계획을 원 PR의 직접 merge 승인으로 쓰지 않는다.

검증 범위는 위 조건 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194 전체 쪽이다. raw 90% 미만·누락·측정 불가 후보는 양쪽 0쪽이며, 대표 페이지와 전체 범위의 최저값을 구분한다. 평균·RGB 진단값으로 raw 미달을 면제하지 않았다.

대표는 조건 192의 1쪽이다. 다음 수치는 실제 새 출력·같은 독립 Print를 대조한 값이며 완전한 RGB 일치의 선언이 아니다.

| 출력 | raw 실루엣 | pixel_match | visual_accuracy_proxy_percent | review / overlay |
| --- | ---: | ---: | ---: | --- |
| native | 100.00000% | 99.94533% | 24.26128% | [review](../assets/s21_20261027/panels/native/192-p001-review.png) / [overlay](../assets/s21_20261027/panels/native/192-p001-overlay.png) |
| wasm | 100.00000% | 99.95106% | 32.4031% | [review](../assets/s21_20261027/panels/wasm/192-p001-review.png) / [overlay](../assets/s21_20261027/panels/wasm/192-p001-overlay.png) |

직접 판독: 앞 텍스트 뒤에서 문단을 나눈 뒤 사각형과 뒤 텍스트가 다음 문단으로 함께 이동한다. 그림을 같은 자리에 남긴 잘못된 base 입력과 구분한다.

게시 문안은 원 기여에 감사드리고, 원 PR 번호·원 head와 통합 후보의 보정 범위·실제 CI·merge SHA를 존댓말로 설명한다. 같은 asset의 아래 URL에서 `<merge-commit-sha>`만 실제 merge SHA로 바꾼다.

`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/native/192-p001-review.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/native/192-p001-overlay.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/wasm/192-p001-review.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/wasm/192-p001-overlay.png`

실제 LF 본문 파일을 `--body-file`로 게시하고 API로 본문·한글·이미지 URL을 재조회한다. 동일 merge·동일 증거의 기존 comment가 있으면 중복 게시하지 않는다.

## 통합 PR #8319의 code head CI 확정

이 PR의 원 contributor head와 위 개별 검증 범위를 유지한 통합 후보 `aee691ae596d288382369b9d5243f4cd06e7fa65`의
[최종 code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)가 성공했다. [PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 문서-only trailing head CI와 정책 성공 뒤 일반 병합한다.
원 contributor PR 자체를 직접 merge한 것으로 표시하지 않는다. 제품 source 불변인 추가 테스트
보정과 최종 원격 검증은 [통합 리뷰](pr_8319_review.md)·[CI 증거](../assets/s21_20261027/final-ci.json)에 구분했다.
최신 지시대로 같은 PR에 self-review·오늘할일을 trailing commit으로 반영한다. 아직 병합하지 않았으며 실제 병합 뒤 원 PR별 결과 comment와 종료를 진행한다.
