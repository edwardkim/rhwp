---
kind: working
status: completed
canonical: mydocs/working/task_m100_6662_stage4_cell_visual_row_units.md
last_verified: 2026-09-05
---

# 열린 이슈 재검증 4단계: 셀 분할의 물리 글줄 회계

이 단계의 완료는 분할 단위 보정과 focused 계약 검증을 뜻한다. #6712 전체 해결 또는
PR 제출 준비 완료를 뜻하지 않는다.

## 분석과 범위

- Issue: #6712, 상위 현황 #6662. 직전 단계는 `6ca00d6b5`로 커밋했다.
- 한국어 가정통신문 손 씻기 그림 주변의 문단은 같은 `vertical_pos`에 좌우 두 `LINE_SEG`를
  가진다. `HeightMeasurer`와 문단 paint는 #6299에서 이를 한 물리 줄로 다루지만,
  `cell_units_uncached`의 일반 텍스트 분기는 합성 조각마다 높이를 더하고 분할 가능하게 둔다.
- 이번 단계는 이 높이 중복과 좌우 조각 사이의 잘못된 페이지 분할만 다룬다.
  Square 그림·중첩 표의 추가 높이와 소유권 배선은 다음 단계다.
- 같은 vpos만으로 줄을 합치지 않는다. 기존 `stored_seg_is_row_fragment`의 비합성·다른
  column_start 조건과 저장/합성 줄수 1:1 조건을 함께 적용한다.
- 같은 물리 줄의 visible 범위를 하나의 unit에 담아 글자 조각을 누락하지 않는다.
  서로 다른 vpos, 합성 lineSeg, 같은 cs의 리셋은 기존 경계를 유지한다.

## 검증 계획

1. 작은 RowBreak 표에서 좌우 조각이 셀 높이·쪽수를 두 배로 만들고 서로 다른 쪽으로 갈라지는지
   공개 DocumentCore API로 재현한다.
2. 문단의 모든 글자가 보존되고 같은 시각 줄의 좌우 조각이 같은 쪽에 남는지 확인한다.
3. 같은 cs 또는 합성 seg는 조각 합치기 대상이 아닌 반례로 확인한다.
4. #6299의 기존 높이/렌더 계약과 이번 focused 테스트를 실행한다.
5. 결과와 미완료 항목을 이 문서에 분석보고로 추가한 뒤 코드·테스트와 함께 커밋한다.

가정통신문이 이 변경 하나만으로 2쪽이 된다고 약속하지 않는다. 다음 단계의 어울림 높이 수정과
전체 회귀·시각 검증 전에는 #6712를 닫거나 통합 PR 준비 완료로 표시하지 않는다.

## 분석보고

### 재현

- 처음 합성 입력은 셀의 span과 RowBreak 진입 조건이 빠져 분할 경로를 실행하지 않았다.
  이를 제품 실패로 계산하지 않고, 명시적인 2행 표와 본문보다 큰 두 번째 행으로 바로잡았다.
- 직전 커밋 `6ca00d6b5`의 rlib에 새 계약을 링크한 수정 전 재현: 6개 중 3 passed,
  3 failed, exit 101. 첫 4개 계약만 있을 때는 좌우 `C/D`가 각각 0/1쪽으로 분리됐다.
- 최종 6개 재현에서는 16px 물리 줄 6개가 56px 본문에서 2쪽이 아닌 4쪽으로 분리됐고,
  좌측 조각 높이가 작은 경우 3쪽, 한 줄이 세 조각인 경우 4쪽이었다.
- 같은 열 좌표, 합성 LINE_SEG, 표 전체가 한 쪽에 들어가는 반례 3개는 수정 전부터 통과했다.

### 구현

- `cell_units_uncached`의 일반 텍스트 분기만 변경한다. 저장/합성 줄 수가 같고 기존
  `stored_seg_is_row_fragment`가 참인 연속 조각을 하나의 visible 범위에 넣는다.
- 원본 LINE_SEG, 문자 오프셋과 렌더링 코드는 변경하지 않는다. 모든 조각을 범위 안에 유지해
  글자는 보존하고, 조각 사이에서 페이지를 자르지 않는다.
- 기존 문단 paint처럼 마지막 조각의 높이·줄간격으로 한 번 전진한다. 문단 전/후 간격은
  각각 첫/마지막 물리 줄에 한 번만 적용하고, 줄 시작의 vpos gap과 reset 판정을 유지한다.
- 비합성·동일 vpos·서로 다른 열 조건을 모두 요구한다. 합성 줄 수가 달라진 재래핑은 기존
  줄별 계산으로 돌아간다. Picture/Square/nested-table 별도 높이 계산은 변경하지 않는다.

### 단계 검증

- focused nextest: **10 passed, 0 failed, 473 skipped, exit 0**.
  새 계약 6개, 기존 #6299의 높이 계약 2개와 실물 렌더 계약 2개다.
  빌드 6분 06초, 테스트 0.087초. 실행 명령:

  ```bash
  CARGO_BUILD_JOBS=2 cargo nextest run --locked --cargo-profile release-test \
    --target-dir target/pr-review \
    --test regression_suite_001 --test regression_suite_014 --test regression_suite_026 \
    -E 'test(issue_6712_cell_visual_row_units) | test(issue_6299_same_vpos_seg_is_one_row) | test(issue_6299_same_vertpos_one_line)' \
    --test-threads 2 --no-fail-fast
  ```

  suite 번호는 `--prepare` 결과다. 새 원본은
  `tests/cases/issue_6712_cell_visual_row_units.rs`이며 생성 harness는 커밋하지 않는다.
- `cargo fmt --all -- --check`, `git diff --check`, suite manifest `--check`: exit 0.
  manifest는 1172 sources, 4940 static test attrs, 28 suites + 20 exceptions로 확인했다.
- 새 CLI로 한국어·중국어 공개 원본의 `info --json`을 재실행했다. 둘 다 rhwp **3쪽**, 보존한
  한컴 PDF **2쪽**이다. 저장 제품은 각각 2020/2024로 유지된다.
- 한국어본에 `LAYOUT_OVERFLOW` 10.4px 경고가 발생한다. 파일별 stdout/stderr를 분리해
  한국어본 경고임을 재확인했으며 중국어본의 stderr는 비어 있다.
  기존 1단계와 이번 render tree를 비교하면 한국어본 첫 쪽 바깥 표의 bbox는 모두
  `(75.6, 56.7, 638.8, 1021.4)`로 같다. 이 비교만으로 문서 전체 무회귀를 판정하지 않는다.
- 전체 회귀, native/WASM/workspace lint, 최종 시각 증적은 아직 수행 전이다.
  이번 focused 결과를 #6712 전체 해결이나 PR 제출 준비 완료로 대체하지 않는다.
- 로그와 임시 SVG/JSON은 커밋하지 않는다.

### 다음 단계 경계

- 여기서 코드·테스트·분석보고를 일반 커밋으로 고정한다. 이 커밋에 어울림 개체 보정을 계속
  추가하거나 amend하지 않는다.
- 다음 단계는 새 분석 문서를 먼저 만들고, 실제 저장된 옆 글줄이 있는 Square 그림·표의
  높이 중복과 렌더 소유권을 조사한다. 그림 또는 표라는 이유만으로 높이를 일괄 제거하지 않는다.
- 실물 3/2쪽 불일치와 한국어본 넘침이 해결되기 전에는 #6712를 `Closes` 대상으로 삼지 않는다.
