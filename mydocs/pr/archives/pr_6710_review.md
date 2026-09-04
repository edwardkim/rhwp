# PR #6710 검토 - 출처 표식과 무관한 첫 조각 초과 허용치

## 메타데이터

| 항목 | 값 |
| --- | --- |
| 원 PR | [#6710](https://github.com/edwardkim/rhwp/pull/6710) |
| 작성자 | `planet6897` |
| base / 원 head | `devel` / `4a1eb7c27552dd9dd619a9aef1b9aaaf997a6fdb` |
| 통합 브랜치 | `review/green-ci-batch-20260904-full` |
| 적용 commit | `c340bd7a8`, `61cd71fb9` (`-x`) |
| 메인터너 보정 | `872f3d4c5` 및 정식 fixture 등록 후속 commit |
| 관련 이슈 | [#5057](https://github.com/edwardkim/rhwp/issues/5057) |

## 검토 결과

- reviewer `jangster77`을 지정했고 원 head required CI는 선정 시 성공 또는 정책상 skip이었다.
- 저장된 첫 조각 source frame 허용치를 native HWP5와 direct-HWPX 저장 조판 경로 모두에
  적용하고, 기존 페이지 수 차이를 매개로 하던 `#4658` 계약도 별도 차이 입력으로 유지한다.
- source PNG는 참고 자료일 뿐 통합 head 시각 증적이 아니다.
- 기존 private-path 탐색과 export/ZIP 오류 성공 처리를 제거한다. 원본 HWP는
  `samples/issue5057/`에 SHA-256 manifest와 함께 등록하고 모든 fixture 발견 뒤 오류는 실패한다.
- 통합 head의 lint, test, 시각 검증은 아직 실행하지 않았다.

## 최종 판정

- 판정: **메인터너 보정 후 수용 가능**
- 원 head: private fixture와 export/ZIP 오류를 엄격히 검증하지 못한다.
- 보정 뒤 통합 head: 정식 fixture와 fail-closed 테스트를 포함한다.
- 수용 전 조건: 통합 Rust 검증 및 HWP5/direct-HWPX의 페이지·표 영역 직접 시각 증적.
- 원격 조치: 수행하지 않았다.

## 2026-09-04 통합 검증 갱신

### 정식 fixture 전환

- 비공개 Windows 경로와 환경 변수 탐색을 제거했다.
- 검증 입력은 `samples/issue5057/21484591-gimcheon-sewage-ordinance.hwp`로 고정했다.
- 원본 SHA-256은 `87ac4934e661fd0e177d35f355030bff26d2b764bc25b0ccf69aa7cc66f6dfc7`이며, `rhwp info --json` 기준 한컴오피스 2010 저장본, 논리 13쪽, `printMethod=4` N-up 문서다.

### 실제 결과

- `CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full cargo test --profile release-test --tests`는 전체 실행 중 IR field-sweep baseline 실패로 종료 코드 `101`을 반환했다.
- 이 fixture에서는 HWP5 재저장 뒤 `sections[].paragraphs[].controls[].cells[].list_header_width_ref`가 `0 -> 367`로 바뀌었다.
- `native-skia` release 빌드에서 `--compat 2022 --profile high-quality`로 13쪽 PNG와 SVG를 생성했다.
- 렌더 로그에는 표 겹침 진단 4건과 표 하단 `2.6px` 초과 진단 1건이 남았다. 7쪽 PNG에서는 일부 글자가 대체 글리프 상자로 관찰됐다.

### 현재 판정

**머지 보류**. 정식 fixture 등록과 오류 전파 보정은 완료됐으나, HWP5 `list_header_width_ref` 재저장 차이와 표 레이아웃 진단, 글꼴 대체 관찰을 해소하거나 한컴 기준과의 허용 근거를 확보해야 한다. 상세 산출물은 [통합 시각 sweep](pr_6683_6710_green_ci_batch_visual_sweep.md)에 기록한다.

## 2026-09-04 메인터너 보정 재검증

`src/serializer/control.rs`를 보정했다. 파싱한 HWP5 셀처럼 `raw_list_extra`가 있는 경우에는 `list_header_width_ref=0`을 원본값으로 그대로 기록한다. 반대로 확장 바이트가 없는 새 셀은 한컴 호환 47바이트 `LIST_HEADER` 계약을 위해 기본값 `0x0400`을 유지한다.

- `CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full cargo test --profile release-test --test regression_suite_006 ir_field_sweep_baseline::ir_field_sweep_does_not_regress`: 통과 (`1 passed`, `171 filtered`)
- `CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full cargo test --profile release-test --test regression_suite_024 issue_1623_cellzone_diagonal::`: 통과 (`19 passed`, `144 filtered`)
- `CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full cargo test --profile release-test --tests`: 통과 (`exit 0`)

#5057 정식 fixture에서 관측됐던 `list_header_width_ref` 기준선 발산(`0 -> 367`)은 해소됐다. `LAYOUT_TABLE_OVERLAP` 및 2.6px overflow 진단과 N-up/설치 글꼴 차이는 PNG/SVG 산출물의 관찰 사실로 [visual sweep](pr_6683_6710_green_ci_batch_visual_sweep.md)에 남겨 둔다. 이번 직렬화 보정이 해당 렌더러 진단을 해결했다고 주장하지 않는다.

### 최종 판정

**메인터너 보정 됨 수용 가능.**
