# 혼합 자식 컷의 독립 Print와 프레임 회귀

전체 회귀가 실패한 기존 두 `nested-mixed-cell.hwp`를 보정했다. 새 HWP나 합성 변형은
추가하지 않았다. [원본·PDF SHA 및 실제 Print 출처](independent_print.json)를 따른다.

두 기준 PDF는 Windows 한컴 2020 `11.0.0.9136`, PrintMethod 0·1-up으로 출력했으며,
각각 8쪽이다. 원본은 변경하지 않았고 MCP의 `scripts_disabled` 전처리를 별도 기록했다.
기존 8쪽 PDF도 보존했다. 원본이나 기준 쪽수를 바꿔 검사를 통과시키지 않았다.

## 보정 전후와 전체 페이지 자격

최종 고정 base는 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`, 검증한 production source는
`04c92da78165ccbf63b467855591df183655a98d`다. 초기 혼합 컷 보정 source `4ab8b438f8d244e1d341c527ed9b5bf8b9d8acb1`의 이력과 최종 재검증을 구분한다. Native276은 6쪽으로 잘못 분할했고,
실패한 4쪽의 raw는 72.30%였다. 최종 Native284와 표준 최적화 fresh WASM092는
두 입력 모두 독립 Print와 8쪽이 일치한다. 다음은 각 입력 전체 8쪽의 최저 raw다.

| 기존 입력 계열 | base Native | 현재 Native | base fresh WASM | 현재 fresh WASM |
| --- | ---: | ---: | ---: | ---: |
| `valid_orientation` | 99.96129% | 99.96132% | 99.95690% | 99.95693% |
| `valid_generated` | 99.74063% | 99.74063% | 99.77021% | 99.77021% |

양쪽 전체 32쪽을 비교했으며, 같은 원본·PDF에 대한 base 대비 0.1%p 초과 하락은 없다.
평균이나 색상 경계 보정값으로 raw 미달을 면제하지 않았다. 각 파일의 TSV 원문은
아래에 보존했다. [독립 좌표·페이지·해시·실행물 및 직접 판독](independent-frame-oracle.json)을
새 정식 회귀의 근거로 사용한다.

| 입력 계열 | Native 전체 TSV | fresh WASM 전체 TSV |
| --- | --- | --- |
| `valid_orientation` | [Native](native/valid_orientation-silhouette.tsv) | [WASM](wasm/valid_orientation-silhouette.tsv) |
| `valid_generated` | [Native](native/valid_generated-silhouette.tsv) | [WASM](wasm/valid_generated-silhouette.tsv) |

## 독립 기대값과 관계 검사

기대 좌표는 RHWP의 계산 결과를 복사하지 않고 독립 PDF의 선 연산자에서 추출했다.
좌표계는 96dpi CSS px이며, PDF point에 `96/72`를 곱한다. 완료 페이지의 Native와
fresh WASM raw가 각각 98% 이상이고 실제 review·standalone overlay를 직접 확인한
뒤, 중요한 위치만 ±2px로 검사한다. 소유·순서·포함·중복 방지 관계도 함께 유지한다.

새 검사 모듈은 `tests/cases/issue_7518_mixed_print_frame.rs`다.

- `following_overlay_owns_its_outer_margin_before_and_after_the_first_cut`:
  `valid_generated`의 첫 원점과 다음 쪽 이어받기에서 원본 바깥 위 여백의 소유를 확인한다.
  첫 원점은 저장 LineSeg와 원본 세로 오프셋에 상대적인 관계로 검사한다. ±2px만으로
  이전 1.88px 이동을 놓치지 않도록 원본 여백 비율도 검사한다.
- `mixed_stream_print_slice_preserves_owned_lines_and_the_final_physical_frame`:
  `valid_orientation` 4쪽의 `UNIT 000~017`을 정확히 한 번씩 순서대로 자식 표 안에
  배치하는지 확인한다. 8쪽의 최종 프레임과 이미 소비한 유닛의 중복 없음도 검사한다.

이름이 다른 기존 원문 보존·마지막 문단의 쪽/셀 소유 검사도 그대로 유지했다. 새 두 함수와 마지막 원문 소유 검사의 실제 수정 전 FAIL/수정 후 PASS는
[동일 바이너리 실행 증거](../s21_20261027/execution-evidence.json)에 연결한다. 독립 좌표는 변경하지 않았다. 시각 자격과
fixture 등록만으로 전체 Rust나 PR 준비가 완료됐다고 판정하지 않는다.

## 대표 PNG와 남은 차이

다음 대표 페이지의 Native와 fresh WASM review·overlay를 모두 직접 열었다.
첫 원점과 이어받기, 혼합 컷, 마지막 종료 경계를 포함한다. PR 본문에서는 첫 원점과 혼합 컷의
대표 쌍을 표시하고, 통합 PR의 최종 증적 코멘트에서는 아래 페이지별 두 backend 자료를
연결한다. 이미지 전체를 생성하거나 중간 raster·render-tree JSON·로그를 커밋하지 않는다.

| 입력·쪽 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| `valid_generated` 1쪽 | [review](native/valid_generated-p001-review.png) / [overlay](native/valid_generated-p001-overlay.png) | [review](wasm/valid_generated-p001-review.png) / [overlay](wasm/valid_generated-p001-overlay.png) |
| `valid_generated` 2쪽 | [review](native/valid_generated-p002-review.png) / [overlay](native/valid_generated-p002-overlay.png) | [review](wasm/valid_generated-p002-review.png) / [overlay](wasm/valid_generated-p002-overlay.png) |
| `valid_orientation` 4쪽 | [review](native/valid_orientation-p004-review.png) / [overlay](native/valid_orientation-p004-overlay.png) | [review](wasm/valid_orientation-p004-review.png) / [overlay](wasm/valid_orientation-p004-overlay.png) |
| `valid_orientation` 8쪽 | [review](native/valid_orientation-p008-review.png) / [overlay](native/valid_orientation-p008-overlay.png) | [review](wasm/valid_orientation-p008-review.png) / [overlay](wasm/valid_orientation-p008-overlay.png) |

raw 100%는 2px 이웃의 실루엣 일치이며 RGB 동일성을 뜻하지 않는다. 선의 진하기와
안티앨리어싱 차이는 남는다. `valid_generated` 첫 쪽 하단은 base와 현재가 모두 Print보다
약 2.9 CSSpx 아래에 있으므로, 이 하단을 독립 ±2px 기대 좌표로 등록하지 않았다.
실제 보정한 첫 원점·이어받기와 완료된 혼합 컷·마지막 종료를 보호한다.

## 실행 조건

Native는 `cargo build --locked --profile release-test --target-dir target/pr-review
--features native-skia,pdf-jbig2,pdf-jpeg2000 --bin rhwp`로 빌드한 봉인 실행물이다.
fresh WASM은 공유 `target/pr-review`에서 `scripts/wasm-pack-locked.sh --target web
--release --out-dir pkg`를 실행했고 실제 `wasm-opt -O`까지 완료했다. pkg와 Studio public의
JS/WASM 해시가 같다. Cargo 8 jobs를 사용했으며 빌드·최적화를 중첩 실행하지 않았다.

Visual Sweep은 `--silhouette-only`, `--compat 2022`, `--dpi 96`, `--page-workers 1`로
전체 페이지를 측정했다. WASM에는 같은 실행 명령에 `--wasm-pkg`로 봉인 패키지를 전달했다.
`compat`는 RHWP 렌더 호환 설정이며 기준 PDF를 출력한 한컴 제품 연도를 뜻하지 않는다.
글꼴 공급은 `/usr/share/fonts`, `/usr/local/share/fonts`, `/home/tsjang/.local/share/fonts`,
저장소 `assets/fonts`이며 backend별 공급 집합 해시도 oracle에 보존했다.

전체 통합 범위의 Rust·WASM·Studio 검증은
[현재 작업 기록](../../../working/task_semanticist21_20261027_rebase_8116.md)에
별도로 연결한다. 이 두 원본의 통과를 다른 문서나 전체 검증의 통과로 확대하지 않는다.
