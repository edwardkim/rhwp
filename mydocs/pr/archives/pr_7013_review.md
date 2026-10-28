# PR #7013 검토

## 최종 판정: 승인

칸 안 마지막 문단의 문단 기준 자리차지 중첩 표 리드를 세로 정렬 콘텐츠 높이에 반영하는
변경을 승인한다. 집중 테스트, 전체 회귀, lint, WASM 빌드와 동일 재현 문서의 한컴 PDF
대조를 완료했으며, upstream 통합 PR #7033의 Full CI와 CodeQL도 성공했다.
추가 메인터너 제품 코드 보정은 하지 않았다.

이 판정은 기능 범위의 승인이다. **#6815의 문서 후행 및 post-merge 재사용 검증은 아직 미완료**이며,
이번 기능 승인이나 verifier job의 성공만으로 #6815를 닫지 않는다.

## 대상과 계보

| 항목 | 확인 내용 |
| --- | --- |
| 원 PR / 작성자 | [#7013](https://github.com/edwardkim/rhwp/pull/7013), `davindev` |
| 통합 PR | [#7033](https://github.com/edwardkim/rhwp/pull/7033) |
| 기능 closing 이슈 | [#7012](https://github.com/edwardkim/rhwp/issues/7012), `Closes #7012` |
| 운영 검증 이슈 | [#6815](https://github.com/edwardkim/rhwp/issues/6815), 자동 closing 대상 아님 |
| 선행 기여 | [#6591](https://github.com/edwardkim/rhwp/pull/6591), 첫 기여자 아님 |
| 원 fork / branch | `kidsnote/rhwp`, `fix/6697-valign-content-height-lead` |
| 원 head | `0bb4135daafe07ad35cbba67a409059352048711` |
| 기준 devel | `c378fbe3c7a856e5b75001d1207fdcf8532d1850` |
| 로컬 제품 검증 merge | `28c33b91e281faf1e0d5dd679ba542c49921b993` |
| 원본 및 증적 commit | `8fa53c81b0354baddf759e57796328191f310ec4` |
| upstream Full 후보 | `8ecce5d654fb244389d0dfc8b7e035226361903c` |
| Full 후보 부모 | 첫 부모 `8fa53c81b0354baddf759e57796328191f310ec4`, 둘째 부모 위 devel |
| 통합 branch | `review/7013-6815-reuse-20260911` |
| 제품 변경 | Rust source 1개, 기존 integration test source 1개, 51줄 추가 / 3줄 삭제 |
| reviewer | 원 PR에 `jangster77` 지정, 통합 PR owner 자동 지정 없음 |

원 기여 commit은 그대로 보존했다. 원 PR의 `maintainerCanModify=true`와 별개로 현재 계정의
fork push 권한이 없었고 dry-run도 403으로 거부됐다. 원격 원 PR에는 변경하지 않았다.
사용자 승인에 따라 upstream 검증 경로로 전환했으며, 실패한 원 PR용 로컬 기록 commit을
통째로 가져오지 않았다. 원본 HWP/PDF/PNG를 먼저 넣고 최신 devel을 실제 merge한 뒤 Full CI를 실행했다.
Full 후보의 `src`, `tests`, `crates`, `.github`, Cargo manifest/lock은 로컬 검증 merge와 동일했다.
contributor commit의 rebase/amend/force-push 및 fork branch 삭제는 하지 않는다.

## 코드 검토

- `calc_nested_controls_bottom_height`는 호스트가 칸의 마지막 문단일 때만 중첩 표 높이에
  `para_relative_float_table_lead`를 포함한다. 뒤 형제 문단의 저장 위치가 있으면 추가하지 않는다.
- 마지막 문단의 `vpos_height`에도 같은 리드를 포함해 콘텐츠 높이의 최대값 후보가 렌더 위치와
  같은 높이 근거를 사용하도록 한다.
- 기존 Top 정렬과 음수 offset 검증을 유지하고, Center 정렬에서 표의 이동량이 리드의 절반이라는
  독립적인 기하 계약을 기존 integration source에 추가했다. baseline이나 허용 오차를 완화하지 않았다.
- `height_measurer::cell_nested_controls_bottom` 전반의 장부 통합은 범위 밖이다.
  선언 높이가 콘텐츠보다 큰 이번 재현 문서의 Center 정렬 문제와 구분한다.

## 실제 검증 결과

환경은 macOS, 전용 `target/pr7013-review-20260911`, Docker 미사용이다. Cargo는 순차 실행했다.
원 PR 이후 base의 제품 변경도 있으므로 로컬 통합 merge에서 전체 회귀를 별도로 실행했다.

| 검증 | 실제 결과 |
| --- | --- |
| generated suite prepare / manifest check | 통과, 생성 파일 커밋 제외 |
| `issue_6697_cell_nested_table_vert_offset` / `issue_1510` 집중 nextest | 10개 통과, 선택 밖 358개 제외, 실행 0.022초 |
| 전체 nextest, release-test, 8 threads, no-fail-fast | **9,472개 통과, 기존 46개 skip**, 실행 327.742초 |
| `cargo fmt --all -- --check` | 통과 |
| native / WASM32 lib / workspace all-target Clippy, `-D warnings` | 모두 통과 |
| workspace build | 통과 |
| `wasm-pack-locked.sh --target web` | 통과, wasm-opt 완료, 3분 32초 |
| 한컴 PDF / native SVG webfont sweep | 전체 2쪽 대조, 상세 아래 |
| 신규 HWP 명시 보안 검사 | 정상 corpus 3종 탐지 검사 1개 및 injection contract 14개 통과 |

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr7013-review-20260911 --tests --test-threads 8 --no-fail-fast
```

집중 검증은 generated `regression_suite_024`와 `regression_suite_012`의 두 module을 선택했다.
WASM은 `/tmp/rhwp-7013-review-20260911/pkg`에 생성해 기존 Studio pkg를 덮어쓰지 않았다.
wasm-bindgen 사전 빌드 플랫폼 판정 실패 뒤 cargo install fallback과 최종 빌드는 성공했다.
신규 HWP 보안 검사는 검증 merge에서 빌드한 `regression_suite_018`과 `regression_suite_014`로 실행했다.
`RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 `["samples/issue7012/synth_cell_nested_float_lead_center.hwp"]`를
명시했으며 env 미지정으로 신규 sample 검사를 생략한 결과를 사용하지 않았다.

원 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/34563331026),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34563331045),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34563330909)는 성공했다.
로컬 Native Skia 전체 묶음은 중복 실행하지 않았다. Studio 실제 UI 조작, Windows 실행 및
macOS 이외 환경의 시각 일치는 이번에 검증하지 않았다.

upstream Full 후보 `8ecce5d654fb244389d0dfc8b7e035226361903c`의 실제 CI:

- [CI 34581439791](https://github.com/edwardkim/rhwp/actions/runs/34581439791): 성공.
  lint, Native Skia, frontend package, archive A~D 빌드 및 회귀 worker, Build & Test aggregate 성공.
- [CodeQL 34581439508](https://github.com/edwardkim/rhwp/actions/runs/34581439508): 성공.
  Rust, Python, JavaScript/TypeScript Analyze가 실제 실행되어 성공했다.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34581438570),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34581439285),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34581439271),
  [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/34581438615): 모두 성공.
- 초기 preflight는 `fast_pass=false`, `direct-source-missing-build-and-test:8fa53c81b0354baddf759e57796328191f310ec4`였다.
  증적 부모에 별도 CI가 없으므로 현재 merge 후보에서 Full 실행한 것을 오류로 판정하지 않는다.

## 원본과 한컴 기준

- 원본: [#7012 첨부 fixture의 고정 commit](https://github.com/kidsnote/rhwp/blob/4d1f1c56e1588b127995848bbd7e5a32c603af1f/samples/synth_cell_nested_float_lead_center.hwp).
  작성자가 한글 음절 치환, 이미지 더미화, 기하 보존으로 익명화한 공개 fixture라고 설명했다.
- 보존: [원본 HWP](../../../samples/issue7012/synth_cell_nested_float_lead_center.hwp), 79,872 bytes.
  Git blob `49b76c162ecccc6e81a75958cb7e51ba576b0767`이 원격 첨부와 일치했다.
- `info --json`: hwp5, `lastSavedWith.product=hancom-office-2020`, `version=11.0.0.8808`, 2쪽.
- 유효한 첨부 PDF가 없어 MCP 비동기 start/status/download에서 engine `2020`으로 변환했다.
  job `07fe8343-6b21-4be6-8e93-4f8162c6780b`, succeeded / download success.
- [한컴 PDF](../../../pdf/synth_cell_nested_float_lead_center-2020.pdf): 262,877 bytes,
  2쪽, 595 x 841pt, PDF 1.6. `Creator: Hwp 2022 0.0.0.0`, `Producer: Hancom PDF 1.3.0.550`.
  `2020`은 서비스 호환 engine 식별자이며 실제 PDF Creator와 구분한다.
- HWP SHA-256: `3fcd280bf793e4205c76fcf15bddcbe1337a09e05db1989d9ee9cd9fe39557c1`.
- PDF SHA-256: `5004407c79e030339272aefa307fa9f624effb904f16b148a3f3eaa5525aa527`, client 검증값과 로컬 일치.
- PDF SHA-1: `9965ae9842da9e48cef201dd6b7c1ef60dfc6bb4`.

## 시각 검토

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라
기준 devel과 로컬 통합 merge를 각각 새로 빌드한 바이너리로 1~2쪽 대조했다.
기본 webfont rasterizer를 사용했으며 원본의 익명화 음절을 글꼴 손상으로 판단하지 않았다.

```bash
venv/bin/python scripts/visual_sweep.py --key pr7013-after \
  --hwp samples/issue7012/synth_cell_nested_float_lead_center.hwp \
  --pdf pdf/synth_cell_nested_float_lead_center-2020.pdf --pages 1-2 \
  --rhwp-bin target/pr7013-review-20260911/debug/rhwp \
  --out /tmp/rhwp-7013-review-20260911/after
```

수정 전은 `/tmp/rhwp-7013-review-20260911/rhwp-before` 및 before 출력 경로를 사용했다.
바이너리 SHA-256은 전 `f9ce4cbadadd71d900492fc43a51af144f88d9c557c955d65c8973b517a4ad78`,
후 `c2689d67af1d1a694491fdad73b462455d73754ce1f13a9de9176be4220c52be`다.

| 2쪽 관찰값 | 수정 전 | 수정 후 |
| --- | --- | --- |
| 대상 칸 y / 높이 | 373.4..974.1 / 600.7px | 동일 |
| 제목 줄 y | 744.6..765.9px | 714.5..735.9px |
| 중첩 표 y | 829.8..936.8px | 799.7..906.7px |
| 표 높이 | 107.0px | 동일 |
| 제목 줄 하단에서 표 상단까지 | 약 63.9px | 약 63.8px |
| 표 하단에서 칸 하단까지 | 37.3px | 67.4px |
| pixel_match | 89.31826% | 90.15950% |
| visual_accuracy_proxy_percent | 50.16873% | 48.44138% |

표와 제목이 함께 약 30.1px 위로 이동하고 상대 간격은 유지됐다. 한컴 기준과 나란히 연 이미지에서
콘텐츠가 아래로 처지던 현상이 개선됨을 확인했다. 1쪽 rhwp PNG는 전후 SHA-256이
`00ba8632329868ecce3c59b978fafd3c8d9e8fcd02bf757afb6023902b89039b`로 동일했다.
자동 flagged는 전후 0/2쪽으로 기존 정렬 결함도 검출하지 못했다. flagged=0만으로 승인하지 않았다.
글꼴과 페이지 크기 차이로 glyph 주변 diff가 남고 proxy는 감소했다. 전체 fidelity 개선을 주장하지 않으며
이번 표 정렬 범위만 승인한다.

직접 열어 확인한 최종 증적:

- [수정 전 2쪽](../assets/pr_7013_6815_validation_20260911/before-p002.png), SHA-256
  `51f5cefd5a7812da8588918b6d8da92a32acf74d8e1a6fbe95637b107a927f43`.
- [수정 후 2쪽](../assets/pr_7013_6815_validation_20260911/after-p002.png), SHA-256
  `e8e38180f5b1ad71ba9f7917497ed6f1c3b8d4bb213cb82955ae2295a3084872`.

## #6815 문서 후행 및 post-merge 검증 경계

- 이번 후행 commit은 본 리뷰와 `mydocs/orders/20260911.md` 두 Markdown만 변경한다.
  신규 HWP/PDF/PNG는 Full 후보에 이미 포함했다. 신규 sample을 허용 경로로 옮겨 정책을 우회하지 않는다.
- Full CI와 실제 CodeQL 분석이 성공한 current-base merge 후보는 `8ecce5d654fb244389d0dfc8b7e035226361903c`다.
  후행 head의 preflight가 어떤 후보 SHA와 source run을 선택하는지 실제 로그로 확인한다.
- merge 전 최종 head, required checks, 실행된 worker, MERGEABLE/CLEAN을 다시 확인한다.
  merge 후에는 CI와 CodeQL 각각의 `reuse=true`, 실제 reason/source run, heavy worker skip을 확인한다.
  nextest timing 갱신도 성공해야 하며 verifier job의 성공과 재사용 성공을 구분한다.
- 기존 #6998에서는 CI 재사용은 성공했지만 CodeQL이 `candidate-full-lane-evidence-unavailable`로 거부됐다.
  이번에도 Full fallback이 발생하면 실행 성공과 별개로 #6815의 재사용 검증 실패로 기록한다.
- 이 문서 작성 시점에는 후행 head 및 merge SHA가 아직 없으므로 해당 CI 결과를 선기록하지 않는다.
  실제 결과는 PR/이슈 코멘트에 SHA와 run URL을 포함해 남긴다.

## Merge 후 contributor PR 및 이슈 comment 계획

원 PR #7013은 직접 merge한 것이 아니라 #7033 통합으로 수용한 사실과 실제 merge SHA를 남긴다.
기능 이슈 #7012에도 실제 PR/devel CI, 전체 회귀, 위 정렬 관찰과 미검증 범위를 기록한다.
#6815에는 기능 close와 구분해 current-base 후보, Markdown 후행 head, merge SHA 및 verifier 결과를 남긴다.

[Visual Sweep 정본 안내](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)와
전후 2쪽 이미지 두 장을 코멘트에서 직접 표시한다. 실제 merge SHA로 치환한다.

```markdown
![수정 전 한컴 대조](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_7013_6815_validation_20260911/before-p002.png)
![수정 후 한컴 대조](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_7013_6815_validation_20260911/after-p002.png)
```

flagged 0/2쪽, pixel_match와 proxy의 전후 값, 숫자와 사람이 확인한 정렬 개선의 차이를 함께 적는다.
asset이 실제 merge를 통해 devel에 반영되고 후속 처리 조건을 충족한 뒤 UTF-8 body file로 게시한다.
게시 후 API로 body와 이미지 Markdown을 재조회한다. 동일 merge의 기존 코멘트는 수정하며 중복 등록하지 않는다.
원 PR 상태를 확인한 뒤 필요한 close를 처리하고 contributor fork branch는 보존한다.
임시 로그, 원시 raster, SVG, metric JSON, MCP 응답은 커밋하지 않는다.
