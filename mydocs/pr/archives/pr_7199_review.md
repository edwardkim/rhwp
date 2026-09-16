---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7199_review.md
last_verified: 2026-09-16
---

# PR #7199 검토

## 최종 판정

**머지 보류** — P2 1건: 실제 저장 줄 배정과 새 소유자 탐색의 줄 소속이 달라,
이전 줄 표의 여백이 다음 줄 표의 위치에 유입된다. 원본 한 쪽 개선과 기존 focused 10건 통과는 확인했지만,
별도 경계 진단에서 최신 devel 통과 / PR 적용 실패를 재현했다. 메인터너 보정은 아직 적용하지 않았다.

## 검토 대상과 경로

| 항목 | 확인값 |
| --- | --- |
| 원 PR / 이슈 | [#7199](https://github.com/edwardkim/rhwp/pull/7199) / [#7150](https://github.com/edwardkim/rhwp/issues/7150) |
| 제목 | Task #7150: 줄을 소유한 TAC 표를 제 바깥여백에 앉힌다 |
| 작성자 / reviewer | lpaiu-cs, 기존 기여자 / jangster77 사전 할당 완료 |
| 원 head | `5356d0c0a3f1c6e687993defc5831c3e28f3cf6d` |
| 원 CI base | `8d45f242baa1a565357aaa38e9f459595b1e756c` |
| 최신 통합 base | `6cd3c0692a3ed9def7f7e7af1ea03cad0c1a0aaa` |
| 로컬 code head | `ca0db01b2f8d9e6fc73e290d32f9d1f7fc9a3282` |
| 로컬 branch | `codex/pr7199-review-20260916` |
| 적용 | `cherry-pick -x`, 충돌 없음, 원본 작성자/메시지 보존 |
| 변경 | renderer 1파일 + 실물 focused test 1파일, +174/-11 |
| 원격 상태 재확인 | OPEN, non-draft, MERGEABLE/CLEAN, source head 불변 |

route: collaborator_external_pr + intake_and_review + local_validation + visual_fixture_evidence.
원격 push·리뷰 게시·merge는 하지 않았다. source CI를 최신 통합 head의 CI라고 표기하지 않는다.

## P2 — 실제 줄 배정을 재사용하지 않는 소유자 탐색

위치: `src/renderer/layout/paragraph_layout.rs:7924–7928` (로컬 code head 기준).

새 코드는 전체 `composed.tac_controls`를 가시 문자 구간으로 다시 필터링한다.
그러나 같은 함수는 이미 `stored_tac_line_assignment`로 원시 UTF-16 줄 소속을 복원하고,
그 결과로 `tac_offsets_px`를 줄별 필터링해 실제 `run_tacs`를 배치한다
(`paragraph_layout.rs:4566–4583`, `:7139–7153`; `composer.rs:1551`의 #6706 계약).
줄 끝 표와 다음 줄 첫 개체가 같은 가시 위치에 투영되는 경우 새 탐색은 이전 줄 표를
다음 줄의 소유자로 받아들인다. 실제 배치에는 없는 표의 여백에서 기준선을 유도하는 오류다.

[합성 입력과 생성 내역](../../../tests/fixtures/issue7150_cross_line_owner/README.md)은
기존 issue2470 문단에 둘째 줄 표를 추가한 경계 진단이다. 두 파일은 첫 줄 큰 표의
상하 여백만 140/140 → 240/40 HU로 바꾸며 여백 합과 모든 줄 높이는 유지한다.
둘째 줄 자체를 바꾸지 않았으므로 둘째 줄 표의 y는 불변이어야 한다.
한컴 재저장본이나 합성 입력의 한컴 PDF 일치 판정으로 제시하지 않는다.

| 실행 | 다음 줄 표 y: 140/140 | 다음 줄 표 y: 240/40 | 판정 |
| --- | ---: | ---: | --- |
| 최신 devel `6cd3c0692` | 320.0px | 320.0px | PASS, 차이 0.0px |
| PR 적용 `ca0db01b2` | 318.7px | 320.0px | FAIL, 차이 1.3px |

실행 trace의 첫 줄 `run_tacs=[0,1]`, 둘째 줄 `run_tacs=[2]`와 대조했다.
이전 줄 큰 표는 둘째 줄에 그려지지 않지만 새 `line_owner` 후보에 들어간다.
[재현 검사](../assets/pr7199_review/check_cross_line_owner.py)는 성공 시 0, 이 오류 재현 시 1을 반환한다.

해제 조건: 실제 배치가 소비하는 동일한 줄별 TAC 배정 결과에서 소유자를 선택하고,
이 경계의 기존 devel PASS / 수정 전 FAIL / 보정 후 PASS를 정식 focused test로 고정한다.
원본 결재표 위치와 #7049의 동반 표 하단차도 유지해야 한다. 바뀐 범위의 Native/WASM sweep과
최종 head CI를 확인한 뒤 재판정한다. 줄 소속을 또 다른 조건식으로 추측하는 보정은 피한다.

## 완료한 검증과 범위

| 검증 | 결과 |
| --- | --- |
| 원 head CI | [CI](https://github.com/edwardkim/rhwp/actions/runs/35075772850) Build & Test, archive A/B/C/D, Lint, Native Skia 성공; Render Diff·CodeQL·Adapter·Proptest도 실패/대기 없음 |
| 통합 head 빌드 | `cargo build --locked --bin rhwp` 성공 |
| focused | #7150 4건 + #7049 4건 + #6754 2건 = **10/10 PASS**; 필터 제외 588건 |
| 줄 경계 불변성 | 위 P2, **devel PASS / PR FAIL** |
| fresh WASM | locked wrapper `--target web --no-opt --dev` 빌드 성공; 새 JS/WASM을 Chrome/153.0.8010.47에서 실제 실행 |
| Visual Sweep | Native와 fresh WASM 각각 1–2쪽 compare·overlay·review 생성/직접 확인, 페이지 누락 없음 |
| 독립 PDF 보조 검사 | fidelity_compare text-only + export-all-svg + layout-ledger, 2쪽 완료 |
| whitespace / 입력 | `git diff --check` 통과; 기존 입력 Git blob 일치, HWPX ZIP CRC 통과 |

환경: macOS, `DEVELOPER_DIR=/Library/Developer/CommandLineTools`, 전용
`CARGO_TARGET_DIR=/Users/tsjang/rhwp/target/pr7199-review-20260916`.
Docker CLI는 있으나 daemon 연결 실패였다. host dev/no-opt WASM 결과를 최적화된 Docker 배포 빌드 통과로 쓰지 않는다.
전체 회귀·lint·Native Skia는 원 head의 성공한 CI 근거를 사용했고 로컬에서 중복 실행하지 않았다.
최신 devel 통합 head에는 위 focused/경계/시각 검증만 수행했다.

## 직접 Visual Sweep 판정 — overlay 포함

기준: 기존 Git `pdf/issue2470/36382471_masked-2022.pdf`, Creator `Hwp 2022 12.0.0.4547`,
Producer `Hancom PDF 1.3.0.550`, 2쪽. 원본 HWPX의 lastSavedWith는 Hancom Office 2020
`11.0.0.8227`이다. 이번에 새로 PDF 변환하지 않았으며 기존 독립 한컴 PDF를 그대로 사용했다.

144 DPI, Chrome webfont 경로, 기본 diff threshold 32. 글꼴 대체 차이는 좌표 개선과 구분했다.
Native/WASM의 아래 지표는 같았다. 두 backend 모두 사람이 review와 overlay를 직접 열었다.
라벨·한글 설명은 판독 가능했다. 대표 저장 이미지는 fresh WASM 실행본이다.

| 페이지 | devel pixel / ink match | PR Native·WASM pixel / ink match | 직접 판정 |
| --- | --- | --- | --- |
| 1 | 97.08770% / 39.97946% | 97.61166% / 46.74444% | 결재표 위치 개선. 제목/기관명 두께 등 기존 차이는 남음 |
| 2 | 94.65651% / 15.32564% | 94.65651% / 15.32564% | 사진칸 높이 및 후속 본문 위치 차이. devel과 PR의 SVG/render tree가 byte-identical하여 이번 변경의 회귀는 아님 |

자동 후보 **0/2쪽**은 시각 일치 판정이 아니다. 큰 여백을 포함한 pixel match와 내용 잉크 중심
일치율(visual_accuracy_proxy=ink match)을 구분한다. 특히 2쪽은 직접 판독에서 큰 차이가 보인다.
완전 일치나 문서 전체 해결로 승인하지 않는다. 1쪽 제목 두께는 PR 본문도 #7151 범위로 분리했다.

![1쪽 Native와 같은 결과를 보인 fresh WASM·한컴 PDF·overlay](../assets/pr7199_review/wasm_review_001.png)

![1쪽 standalone overlay](../assets/pr7199_review/wasm_overlay_001.png)

![2쪽 기존 차이를 확인한 fresh WASM·한컴 PDF·overlay](../assets/pr7199_review/wasm_review_002.png)

![2쪽 standalone overlay](../assets/pr7199_review/wasm_overlay_002.png)

시각 절차 정본: [Visual Sweep](../../manual/verification/visual_sweep_guide.md#github-merge-comment).
임시 산출 루트는 `/private/tmp/rhwp-pr7199-evidence-20260916`이며 Native `sweep/issue2470`,
fresh WASM `wasm-sweep/issue2470`, devel `base-sweep/issue2470` 아래에 각각
`compare/`, `overlay/`, `review/`를 생성했다. devel 비교 실행의 checkout 표시는 로컬 review head이지만
실제 exporter는 devel에서 따로 빌드한 `rhwp-base`다. 코드/바이너리 복원 후 fresh WASM sweep을 수행했다.
원시 log/TSV/JSON은 로컬 임시 산출물로만 두고 커밋 대상에서 제외했다.

## 재현 명령

저장소 루트에서 실행했다. 아래 `$EVIDENCE`는 임시 디렉터리로 설정한다.

```bash
export CARGO_TARGET_DIR=/Users/tsjang/rhwp/target/pr7199-review-20260916
export DEVELOPER_DIR=/Library/Developer/CommandLineTools
EVIDENCE=/private/tmp/rhwp-pr7199-evidence-20260916
node scripts/rust-test-suite-manifest.mjs --prepare
cargo build --locked --bin rhwp
cargo nextest run --locked --test regression_suite_012 --test regression_suite_004 --test regression_suite_020 \
  -E 'test(/issue_7150_tac_line_owner_anchor::/) | test(/issue_7049_inline_tac_table_baseline::/) | test(/issue_6754_tac_picture_and_table_share_a_line::/)'
venv/bin/python mydocs/pr/assets/pr7199_review/check_cross_line_owner.py "$CARGO_TARGET_DIR/debug/rhwp"
scripts/wasm-pack-locked.sh --target web --out-dir "$EVIDENCE/wasm-pkg" --no-opt --dev
venv/bin/python scripts/visual_sweep.py \
  --file-target issue2470 samples/issue2470/36382471_masked.hwpx pdf/issue2470/36382471_masked-2022.pdf \
  --rhwp-bin "$CARGO_TARGET_DIR/debug/rhwp" --wasm-pkg "$EVIDENCE/wasm-pkg" \
  --pages 1-2 --dpi 144 --out "$EVIDENCE/wasm-sweep"
```

Native는 위 sweep에서 `--wasm-pkg`를 빼고 `--out "$EVIDENCE/sweep"`으로 실행했다.
`RHWP_BIN="$CARGO_TARGET_DIR/debug/rhwp" venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 1`
에 `--source`, `--reference-pdf`, `--text-only --export-all-svg --layout-ledger --out-dir`을 지정해
동일 입력 전수 후보를 수집했다. 의도적으로 실패하는 경계 진단과 통과한 기존 focused를 구분한다.

## 공통 조판 원칙 심사

| 원칙 | 판정 | 근거 |
| --- | --- | --- |
| 독립 근거 | 충족(원본 범위) | 기존 한컴 PDF의 결재표와 직접 Native/WASM overlay |
| 측정·배치의 공통 결과 / 저장 줄 소속 | **미충족** | 새 탐색이 기존 저장 줄 배정과 다른 구간 필터 사용, P2 실행 재현 |
| 일반성 / 경계 증거 | 미충족 | 기존 4개 신설 테스트는 같은 단일 줄 표본. 줄 경계에서 devel PASS / PR FAIL |
| 분할·이어받기 | 비해당 | 이 PR은 분할 컷·소비량·예약 높이를 수정하지 않음 |
| 기준값 완화 | 비해당 | 테스트 오차·baseline 변경 없음 |
| 입력 Git 포함 | 충족 | 아래 기존 경로 재사용 및 새 합성 경계 입력 2개를 리뷰 증적에 포함 |
| 증거 주장 범위 | 제한 명시 | 합성 입력은 불변성 진단. 한컴 생성/시각 오라클로 주장하지 않음 |

`같은 lh를 만족하면 높이도 여백도 같다`는 코드 주석은 수학적으로 성립하지 않는다.
복수 후보 정책도 별도 근거/반례가 필요하지만 이번 판정은 추정 결함을 늘리지 않고 위 재현된 P2에 한정한다.
텍스트·그림·수식이 섞인 모든 조합의 공유 기준선은 이번 검토에서 완전 검증하지 않았다.

## 검증 입력 Git 확인

기존 7파일은 `ca0db01b2f8d9e6fc73e290d32f9d1f7fc9a3282`의 Git blob과 실행 파일 내용이 일치했다.
기존 파일은 복제·이름 변경하지 않았다. 새 경계 입력의 생성 방식/해시는 fixture README에 있다.

| 실제 사용한 저장소 경로 | SHA-256 |
| --- | --- |
| `samples/issue2470/36382471_masked.hwpx` | `43572dad5e17395aa02d1b0000b736b8467278931086604776ef30393dd0f54b` |
| `pdf/issue2470/36382471_masked-2022.pdf` | `814492b502a46e56e3a3be253e7beb386d752d2f5d47bfe2bfb9c646d41747cb` |
| `samples/hwpx/opengov/36384689_결재문서본문_화재발생종합보고서(제2026-298호).hwpx` | `9de5b2b17aba9c51bfbab27f5e571780aa8e49f39f059e2d1ba8665da11df4cd` |
| `samples/issue2083_hide_fill_page.hwpx` | `7758c15c57b1ef14fda6e6d29409ae3425f344931f2901641af84a40ef413d2e` |
| `samples/21_언어_기출_편집가능본.hwp` | `905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15` |
| `samples/issue6542/156678235_mid_para_vpos_rewind.hwp` | `bd2a04f4f969bdad693b21cb26fe61c16f36a14bc5b442c79a2c862777d2419c` |
| `samples/issue6754/156585314-ssagirang-barley.hwp` | `18350ed16867552fbaef12b0677531c0b37fcc8e585cf7d4ec0a953f812644cf` |

## Merge 후 contributor PR comment 계획

현재는 머지 보류이므로 게시하지 않았다. 보정·최종 검증·별도 merge 승인 후 실제 merge SHA와 CI URL,
수정된 줄 소속 계약, 실제 검증 범위와 남은 차이를 한국어로 설명하고 감사한다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을
연결하고 대표 PNG를 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7199_review/wasm_review_001.png`
형식으로 넣는다. 현재 snapshot 지표를 보정 후 결과로 재사용하지 않는다. 본문은 UTF-8 파일과
`--body-file`로 게시한 뒤 이미지 URL·한국어·실제 head를 다시 확인한다.
