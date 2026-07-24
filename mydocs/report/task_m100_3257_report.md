# 처리 결과 보고서 — Task M100 #3257

## 이슈와 결론

- 이슈: [#3257 — 웹한글기안기 제품규격서 V1.1 4페이지 렌더링 깨짐](https://github.com/edwardkim/rhwp/issues/3257)
- 결론: 4쪽 `용도별 서버 구성` TAC 그림을 정렬 폭에서 누락하던 경로를 고쳤다. 수정 후
  그림은 본문 중앙 `x=129.8px`에서 시작하고 본문 우측을 넘지 않는다. 수정 전에는
  `x=416.9px`에서 시작해 우측에서 잘렸다.

## 원인과 변경

문단 마지막 공백 뒤의 `treat_as_char` 그림은 실제 paint 경로에서는 현재 줄에 방출됐지만,
가운데/오른쪽 정렬의 폭 계산은 반열림 줄 범위만 사용해 그 그림을 제외했다. 따라서 공백만
가운데로 옮긴 뒤 그림을 이어 붙였다.

- `tac_offsets_for_line_width()`가 마지막 run 또는 명시 줄바꿈의 끝 TAC를 실제 방출 규칙과
  같은 줄의 폭에 포함한다.
- 다음 composed line이 같은 문자 위치에서 시작하면 앞 줄에는 포함하지 않아 #1219 줄 경계
  수식 회귀를 막는다.
- TAC-only 예외는 수식에만 적용한다. 그림·표는 문단의 Center/Right 정렬 폭을 따른다.
- fixture render-tree 테스트가 `pi=75`, `ci=0` 그림의 정상 좌표와 우측 경계를 고정한다.

## 재현·시각 검증 자료

| 항목 | 값 |
|---|---|
| 원본 fixture | `samples/issue3257/webhangul_product_spec_v1.1.hwp` |
| HWP SHA-256 | `40c7dfb901acb834cd2acb1726b15d2cb95d49e1e745d50e3cb95b2c37162fd6` |
| HWP 2020 기준 PDF | `pdf/issue3257/webhangul_product_spec_v1.1-2020.pdf` |
| PDF SHA-256 | `e160258befe6524e1eb5b0c66d1de6fa69690f892dd8c903f0769b86c23b9aaf` |
| 변환 job | `0d1524c3-dec7-4d0c-b3d5-89b2b279e690` (`run_status=0`, `validation=ok`, 6쪽) |
| visual sweep | `output/visual-issue3257-p004/issue3257-webhangul-product-spec-p004/` (4쪽, 144dpi) |

![#3257 4쪽 rhwp·HWP 2020 비교 및 overlay](assets/task_m100_3257/visual_sweep_review_004.png)

- sweep 자동 후보: `0/1`
- pixel match: `86.11708%`
- visual_accuracy_proxy_percent: `11.30076%` — 한컴 전용 글꼴과 기존 전체 줄 위치 차이를
  함께 반영하는 보조값이므로, 그림의 본문 내 완전 표시 여부는 위 실제 비교 이미지와
  render-tree 좌표로 별도 판정했다.
- 임시 산출물: `compare/compare_004.png`, `overlay/overlay_004.png`,
  `review/review_004.png`.

## 검증

| 명령 | 결과 |
|---|---|
| `CARGO_INCREMENTAL=0 cargo test --lib trailing_tac_width_tests -- --nocapture` | 3 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib issue_3257_centered_trailing_picture_uses_full_line_width -- --nocapture` | 1 passed |
| `CARGO_INCREMENTAL=0 cargo test --test issue_1219_equation_line_hangul_advance --test issue_1285_tac_sequence_right_align -- --nocapture` | 3 passed |
| `CARGO_INCREMENTAL=0 python3 scripts/task1274_visual_sweep.py ... --page 4 --dpi 144` | 성공, 후보 0/1 |
| `cargo fmt`, `git diff --check` | 통과 |

### IR field sweep baseline

`cargo test --profile release-test --tests`의 첫 실행은 새 #3257 fixture에 baseline이 없어
`hwp5rb`의 `cells[].list_header_width_ref` 16건을 회귀로 보고했다. 상세 비교에서 원본값은
모두 0, 재생성 저장본은 모두 `0x0400`이었다. 이는 #1633 HWP 저장 호환 보정이 이미 기존
코퍼스에 남긴 알려진 정규화이며 TAC 레이아웃 수정에 따른 발산은 아니다. 새 fixture의
동일 관측값 16건을 `tests/fixtures/ir_field_sweep_baseline.tsv`에 기록했다. 이후
`CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 전체 재실행은 exit 0으로
완료했고, `ir_field_sweep_baseline`은 2 passed였다.

전체 GitHub Actions와 최종 시각 승인 판정은 ready 전환 후 CI 결과 및 작업지시자 확인으로 진행한다.

## PR 준비 상태

이 변경은 #3259와 함께 `task/3257-3259-renderer-recent-reopen` 단일 브랜치에서 PR로 낸다.
오늘할일 문서는 PR 생성 승인 직전에만 추가한다.
