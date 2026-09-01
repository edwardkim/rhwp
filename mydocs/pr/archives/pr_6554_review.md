# PR #6554 검토 - 저장 사다리와 단 넘김 위험 휴리스틱

- 검토일: 2026-09-01
- 작성자: `planet6897`
- 원 PR/source head: `e02cc3b07aa35da9215bddd1326027a28b61669a`
- 누적 순서: 1/3 (`#6554 -> #6559 -> #6560`)
- 통합 적용 commit: `33d12bd8f840164f3190dcabacc8e6b0c06e702f`
- 통합 기준: `upstream/devel@6d3fd65a30cd8e6755b18a1ab44c5035279b110f`
- conflict: 없음
- 판정: 승인

## 판정 범위

저장 `vpos` 사다리가 현재 단에 들어간다고 말하고 조판 엔진의 자체 회계로도 남은 공간 안에 드는
경우에는 `late_compact_text_tail_overflow_risk`가 그 결정을 덮어쓰지 않게 한다. 위험 휴리스틱을
제거하지 않고 두 조건이 함께 참인 경우에만 억제하므로, 저장 증거가 없거나 실제 fit을 넘는 기존 보호
갈래는 유지된다.

이 판정은 #6544의 모든 조판 차이가 해소됐다는 뜻이 아니다. 직접 비교에서도 p13의 `pi=659` 잔여와
전체 페이지의 큰 font·조판 차이가 남는다. 이번 PR이 약속한 것은 저장 사다리를 무시하던 한 문단을
되돌리는 좁은 정정이므로 #6544는 자동 close하지 않고 잔여 원인을 계속 추적한다.

## 누적 적용과 검증

- 원 PR은 최신 `devel`에 conflict 없이 적용됐다.
- focused test 4건 통과:
  - `column_break_follows_stored_ladder_not_risk_heuristic`
  - `issue_1284_2023_sep_page20_question30_title_stays_in_left_tail`
  - `issue_1274_2022_oct_page11_question20_equation_tail_keeps_pdf_bleed`
  - `issue_1274_2022_oct_page16_question30_title_tail_continues_next_column`
- 누적 head에서 oracle page-count와 off-canvas 각 16 partition, IR field sweep 4/4가 통과했다.
- 누적 head `ee4d7c4ad196b6642a4bfc14bd8fa9d11bb2994a`에서 manifest prepare/check,
  `cargo fmt`, native Clippy, WASM lib Clippy, workspace build, workspace all-target Clippy가 모두
  `-D warnings`로 통과했다.
- 같은 누적 head의 `release-test` 전체 회귀는 8,917/8,917 통과, 46건 정책상 skip, 실패 0건이었다
  (313.667초). 로컬 `cargo-nextest`는 저장소 권장 0.9.140보다 낮은 0.9.137이어서
  `junit.report-skipped` 설정을 인식하지 못했으나 test selection과 실행 결과에는 영향이 없었고,
  이 환경 차이를 통합 PR에 명시한다.
- Native Skia 필수 게이트인 전체 `--lib`, 누락 이미지 placeholder 2/2, 직접 PDF export 4/4가 모두
  통과했다.
- Docker 29.7.2의 `docker compose --env-file .env.docker run --rm wasm`은 6분 15초에 성공해
  `/app/pkg`를 생성했다.
- Draft 통합 PR #6563의 code candidate head `ee4d7c4ad196b6642a4bfc14bd8fa9d11bb2994a`에서 Full CI가
  성공했다. CI run `33480418819`, CodeQL `33480418938`, Render Diff `33480418671`, Proptest
  `33480418840`, Adapter inter-diff `33480418917`이 모두 녹색이었다.

기존 #1139 테스트 세 건은 "현재 sweep" 동작을 고정하던 조건을 한컴 PDF의 단 배치로 바꾼다. 세 핀을
개별 실행했고 모두 통과했으므로 단순 기대값 완화가 아니라 대상 조판 경로의 실제 이동으로 확인했다.

## 직접 시각 검증

원본 `samples/3-09월_교육_통합_2023.hwp`의 마지막 저장 제품은
`hancom-office-2022`(`12.0.0.535`)다. 시각 검증 정책에 따라 engine 2020 버킷인
`pdf/3-09월_교육_통합_2023-hwp-2020.pdf`를 기준으로 사용했다.

- 원본 SHA-256: `47a503ea0e92a63ee58b552e661fbde27f8a611afffd67e822be5928319e3c87`
- 기준 PDF SHA-256: `bfd4de7927a82b4ef521fc78663075cae952424a2615ef9b86c36c37325bf01b`
- 비교 절차: [PDF/SVG visual sweep 가이드](../../manual/verification/visual_sweep_guide.md#github-merge-comment)
- 실행 command:

```bash
python3 scripts/visual_sweep.py \
  --key pr6554-2023-sep \
  --hwp samples/3-09월_교육_통합_2023.hwp \
  --pdf pdf/3-09월_교육_통합_2023-hwp-2020.pdf \
  --pages 13,20 \
  --svg-rasterizer rsvg \
  --out output/pr-review-planet/6554-rsvg
```

| page | compare | overlay | review | pixel match | visual accuracy proxy | 자동 후보 |
|---:|---|---|---|---:|---:|---:|
| 13 | `output/pr-review-planet/6554-rsvg/pr6554-2023-sep/compare/compare_013.png` | `output/pr-review-planet/6554-rsvg/pr6554-2023-sep/overlay/overlay_013.png` | `output/pr-review-planet/6554-rsvg/pr6554-2023-sep/review/review_013.png` | 90.19371% | 11.91978% | 0 |
| 20 | `output/pr-review-planet/6554-rsvg/pr6554-2023-sep/compare/compare_020.png` | `output/pr-review-planet/6554-rsvg/pr6554-2023-sep/overlay/overlay_020.png` | `output/pr-review-planet/6554-rsvg/pr6554-2023-sep/review/review_020.png` | 90.71969% | 8.52825% | 1 |

두 review PNG를 직접 열었다. p13은 목표 문단 하나가 왼쪽 단으로 돌아왔고 p20은 목표 수식 줄이 한컴과
같은 단에 남는 방향을 확인했다. 낮은 visual accuracy proxy는 전체 fidelity 통과값이 아니다. p13의
잔여 frame-tail과 p20의 큰 질문 위치 차이를 포함한 별도 조판 차이가 있으므로 좁은 정정 근거로만 쓴다.

- 대표 asset: [p13·p20 review contact sheet](../assets/pr_6554_issue6544_2020_review.png)
- 대표 asset SHA-256:
  `f24f6c3b9feace8abc373de384586f7a504c6157d1581b2f8568312758a6179b`

대표 asset을 직접 열어 도구 라벨과 overlay가 판독 가능함을 확인했다. 통합 묶음의 다른 두 PR도
engine 2020 기준 PDF로 직접 검증한 뒤 이 판정을 최종 원장에 고정했다.

## Merge 후 contributor PR comment 계획

- 원 head `e02cc3b0` -> 적용 `33d12bd8f` -> 통합 merge SHA의 계보를 남긴다.
- p13·p20 두 페이지, 자동 후보 1건, 위 지표와 사람이 확인한 좁은 개선 및 잔여 차이를 함께 알린다.
- 대표 PNG `mydocs/pr/assets/pr_6554_issue6544_2020_review.png`는
  `<merge-commit-sha>`로 고정한 raw URL을 사용한다.
- #6544는 잔여 `pi=659` 원인이 남으므로 close하지 않는다.
- 통합 merge 뒤 UTF-8 without BOM body file로 comment를 게시하고 API로 본문을 재조회한 뒤 원 PR을
  중복 병합하지 않고 close한다.
