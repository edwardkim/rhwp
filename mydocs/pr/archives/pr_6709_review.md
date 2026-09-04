# PR #6709 검토 - 용지 기준 Square 그림 이동 뒤 본문 재투영

## 메타데이터

| 항목 | 값 |
| --- | --- |
| 원 PR | [#6709](https://github.com/edwardkim/rhwp/pull/6709) |
| 작성자 | `planet6897` |
| base / 원 head | `devel` / `36b5500891e750be7680c2559e2c278d4cbbe175` |
| 통합 브랜치 | `review/green-ci-batch-20260904-full` |
| 적용 commit | `e6b9a3ed5`, `ffd47191e` (`-x`) |
| 메인터너 보정 | `872f3d4c5` 및 정식 fixture 등록 후속 commit |
| 관련 이슈 | [#6202](https://github.com/edwardkim/rhwp/issues/6202) |

## 검토 결과

- reviewer `jangster77`을 지정했고 원 head required CI는 선정 시 성공 또는 정책상 skip이었다.
- `PaperOrigin`으로 용지 기준 좌표를 host band의 로컬 좌표로 옮겨 그림 이동 뒤 본문을
  새 배제 밴드로 재투영한다.
- 기존 private-path 탐색과 fixture 부재 성공 처리는 제거한다. 원본 HWP는
  `samples/issue6202/`에 SHA-256 manifest와 함께 등록한다.
- 그림 이동 API 오류를 성공 처리하던 경로는 `872f3d4c5`에서 fail-closed로 보정했다.
- 통합 head의 lint, test, 시각 검증은 아직 실행하지 않았다.

## 최종 판정

- 판정: **메인터너 보정 후 수용 가능**
- 원 head: private fixture와 편집 실패를 엄격히 검증하지 못한다.
- 보정 뒤 통합 head: 정식 fixture와 fail-closed 테스트를 포함한다.
- 수용 전 조건: 통합 Rust 검증과 그림 이동 전후의 직접 시각 증적.
- 원격 조치: 수행하지 않았다.

## 2026-09-04 통합 검증 갱신

### 정식 fixture 전환

- 비공개 Windows 경로와 환경 변수 탐색을 제거했다.
- 검증 입력은 `samples/issue6202/156483689-turmeric-industry-standardization.hwp`로 고정했다.
- 원본 SHA-256은 `bd24e80fda9e298ffb05dcdb64c22752a4ed78716b358076db26b2e721e41dc4`이며, `rhwp info --json` 기준 한컴오피스 2018 저장본, 논리 8쪽, `printMethod=4` N-up 문서다.

### 실제 결과

- `CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full cargo test --profile release-test --tests`는 전체 실행 중 IR field-sweep baseline 실패로 종료 코드 `101`을 반환했다.
- 이 fixture에서는 HWP5 재저장 뒤 `sections[].paragraphs[].controls[].cells[].list_header_width_ref`가 `0 -> 35`로 바뀌었다.
- `native-skia` release 빌드에서 `--compat 2022 --profile high-quality`로 8쪽 PNG와 SVG를 생성했다.
- 1쪽 PNG에서 일부 글자가 대체 글리프 상자로 관찰됐다. 원인은 이 실행 환경의 글꼴 가용성으로 단정하지 않았으며, 한컴 기준 PDF 또는 동일 글꼴 환경의 비교 증적이 추가로 필요하다.

### 현재 판정

**머지 보류**. 정식 fixture는 등록됐고 회귀가 더 이상 건너뛰지 않지만, IR baseline 증가와 시각 비교의 글꼴 대체 관찰을 해소하기 전에는 수용 판정을 유지할 수 없다. 상세 산출물은 [통합 시각 sweep](pr_6683_6710_green_ci_batch_visual_sweep.md)에 기록한다.

## 2026-09-04 메인터너 보정 재검증

`src/serializer/control.rs`를 보정했다. 파싱한 HWP5 셀처럼 `raw_list_extra`가 있는 경우에는 `list_header_width_ref=0`을 원본값으로 그대로 기록한다. 반대로 확장 바이트가 없는 새 셀은 한컴 호환 47바이트 `LIST_HEADER` 계약을 위해 기본값 `0x0400`을 유지한다.

- `CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full cargo test --profile release-test --test regression_suite_006 ir_field_sweep_baseline::ir_field_sweep_does_not_regress`: 통과 (`1 passed`, `171 filtered`)
- `CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full cargo test --profile release-test --test regression_suite_024 issue_1623_cellzone_diagonal::`: 통과 (`19 passed`, `144 filtered`)
- `CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full cargo test --profile release-test --tests`: 통과 (`exit 0`)

#6202 정식 fixture에서 관측됐던 `list_header_width_ref` 기준선 발산(`0 -> 35`)은 해소됐다. N-up 물리 페이지와 설치 글꼴 차이에 관한 시각 증적의 범위는 [visual sweep](pr_6683_6710_green_ci_batch_visual_sweep.md)에 기록한 한계를 그대로 적용하며, 한컴 PDF와의 1:1 동일성을 주장하지 않는다.

### 최종 판정

**메인터너 보정 됨 수용 가능.**
