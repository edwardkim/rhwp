---
kind: report
status: active
last_verified: 2026-10-04
---

# PR #7482 review — TAC physical rows and Square paragraph flow

## 최종 판정

**머지 보류.** 공통 TAC 줄 구성·Square 후속 흐름·legacy 커서 소유권 보정의 로컬 구현과 검증을 완료했다. 전체 회귀 10,265/10,265(50 skipped), focused 24/24, 필수 lint·Skia, CDP 18문서/45검사 PASS. Native/fresh WASM 각 27쪽을 직접 판독했다. 50% 쪽 수용의 적용 조건은 미검증이며 관련 이슈 전체 해결을 선언하지 않는다. 이번 #7482의 90% threshold만 사용자 예외를 적용하고 원 점수/남은 차이는 보존한다. 사용자 승인 뒤 보정 branch push와 [별도 PR #7563](https://github.com/edwardkim/rhwp/pull/7563) 등록을 완료했다. 원 #7482의 comment·approve·close·merge는 수행하지 않았다.

## 접수와 검토 head

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#7482](https://github.com/edwardkim/rhwp/pull/7482) / davindev |
| 원 contributor head | `kidsnote/rhwp@9bc8478d5ddf43dc87c06278b8c0deb8c7640959` |
| 보정 code head | `0373fb43c4189a138482f72ebbb5ecb0a081b4a5` |
| 고정 base | `6b3faf77d8085441f9f26d88d65a49791e910352` (`devel`) |
| 로컬 branch | `review/davindev-pr7482-maintainer-20261003` — 원 head와 기존 보정 commit을 보존 |
| 원격 참고 상태 | open / mergeable / clean / maintainer_can_modify=true; 원 head Build & Test/Lint/Skia success |
| 충돌 simulation | `git merge-tree --write-tree HEAD upstream/devel`: exit 0, tree `acd4d81f95fa5e05ebeaabf8f6bfc1728fc8e920` |

원 contributor CI는 위 보정 code head의 CI를 대신하지 않는다. merge 전에는 최신 원격 head/base/required checks를 다시 조회해야 한다. #7518의 보정·통합은 이 기록의 범위가 아니다.

## 기여자 진단과 보정 범위

기여자는 저장 LineSeg가 없는 글앞/글뒤 TAC가 paper layer에서 그려져 layout 반환 커서가 시작점에 머물고 뒤 문단이 겹치는 위치를 정확히 찾았다(`728224157`). Square host 글자가 표 옆 가용 띠를 쓰지 못하는 개선도 제목 복원과 2→1쪽 출력으로 입증했다(`512cda03a`). 그러나 table bottom까지 커서를 움직이는 국소 보정만으로는 같은 줄의 여러 TAC·중간 텍스트·개행·여백과 측정/실제 배치의 공통 줄 소속이 정해지지 않는다.

보정은 실제 단 너비, 문단 여백/들여쓰기, 표 바깥여백, 텍스트·개체 순서와 명시적 개행을 공통 `InlineFlowPlan`으로 구성한다. 같은 줄에 들어가는 B/표/A/표/C는 같은 줄에, 가용 너비 부족 또는 개행은 다음 줄에 놓인다. 줄별 기준선/점유 끝을 사용하고 문단 표 높이 합산이나 단순 최대 높이로 대신하지 않는다. 빈 장식 host도 줄 공간을 유지한다.

Square 표의 바깥 점유는 후속 문단의 제외 영역으로 유지하지만 글줄의 흐름 끝을 표 하단까지 밀지 않는다. 저장 LineSeg와 standalone 여러 행 RowBreak의 기존 분할 owner는 유지한다. 큰 혼합 문단·caption/notes·중첩으로 기존 owner에 반환되는 경로는 신규 정책 검증으로 확대하지 않는다.

## 생산과 소비 경로

| 값 | 생산 → 측정/예산 → 실제 배치 |
| --- | --- |
| TAC row/box/end | `inline_flow.rs::plan` → `typeset/inline_flow/plan.rs::build_plan`의 실제 frame → `typeset_inline_flow` 예산/`commit_inline_flow` → `layout/paragraph_layout.rs::layout_inline_flow_plan` |
| Square text end | `plan_square_table_host` → `FormattedParagraph::use_square_host_plan.total_height`/host advance → `layout.rs`의 `col_area.y + plan.end`; table cursor와 max로 덮어쓰는 분기 제거 |
| Square object extent | 같은 plan의 `occupied_bottom`/`square_host_exclusion` → 별도 `height_for_fit` → `record_square_host_flow`의 단/host 원점 변환 → 후속 문단 frame와 확정한 rows |
| standalone split table | `controls/paragraph_flow.rs` → `controls/flow_table.rs` → `TypesetEngine::typeset_tac_table`의 기존 fit/fragment owner; 이번 보정은 split/continuation 컷을 재정의하지 않음 |

## 입력과 실제 검사

공개 원본과 수동 NO_LS/줄 구성 변형을 구분한다. 입력 생성·독립 한컴 2020 PDF job·SHA-256·기대 관계는 [fixture README](../../samples/issue7482/README.md)와 [provenance](../../samples/issue7482/provenance.json)에 있다. 저장 LineSeg 수용을 수동 메타데이터로 완화하지 않았다.

정식 [TAC/Square tests](../../tests/cases/issue_7482_shared_paragraph_table_rows.rs)의 15개 검사는 최종 render tree의 줄/표 원점·개수·순서·점유와 뒤 문단을 확인한다. 이전 원 contributor binary에서 11개 배치 검사가 의도한 원인으로 실패한 기록을 보존했다. p38 반례는 이전 보정 `1b937ea9`의 표 y=478.4px 때문에 실패했고 독립 PDF 괘선 518.67px에 대응하는 518.4px로 복구했다. 새 Square 후속 반례는 이전 `d86b35184` binary에서 첫 줄 y=124.0px 때문에 실패하고 현재 91.4px로 통과한다(독립 PDF ink 시작 91.03px). 22개 문단의 문자·순서·옆 lane와 2쪽 큰 TAC의 단일 소속도 검사한다.

#2319 검사는 내부 owner 문자열을 실제 표 개수·행/열·높이/소속 검사로 바꿨다. 이 변경을 실제 결함 수정 전 FAIL로 보고하지 않는다. 새 Square 입력의 oracle 2/2 항목만 추가하며 기존 baseline/golden/래칫 허용치는 변경하지 않는다.

## 현재 검증과 시각 증적

검증 code SHA `0373fb43c4189a138482f72ebbb5ecb0a081b4a5`, base `6b3faf77d8085441f9f26d88d65a49791e910352`다. 아래 모든 실행은 이 source에서 수행했고 이전 head의 결과를 재사용하지 않았다. `CARGO_TARGET_DIR`는 공유 `/home/edward/mygithub/rhwp/target/pr-review`다.

| 검사 | 결과 |
| --- | --- |
| fmt / Native Clippy / WASM Clippy / workspace build / workspace all-target Clippy | 모두 PASS, `--locked` 및 `-D warnings` |
| manifest / unit-tier base 비교 | PASS, 1,427 case source / 28 generated suites + 20 exceptions; source unit 4,199 / 298 |
| focused | 15개 정식 TAC/Square/legacy 검사 + #6970 경계 9개 = 24/24 PASS |
| 전체 nextest | 10,265/10,265 PASS, 50 skipped, 792.835s |
| Skia lib | workspace 합계 4,109 PASS(그중 rhwp 3,927), 13 ignored |
| Skia missing picture | 2 PASS, 211 skipped |
| Skia direct PDF | 4 PASS, 218 skipped |
| root fresh WASM wrapper / pkg-public byte equality | PASS, WASM `91654b0814b6cadc38225cf9eaab48ef1e3a35d9a7ad2b08cd7a02761ed4d95b` |
| 실제 Studio CDP | 18문서 / 45검사 PASS, pageErrors 0, browser served WASM 동일 해시 |
| Visual Sweep | Native/fresh WASM 각각 22문서 27쪽 review PNG와 대표 standalone overlay 직접 판독 |

```sh
node scripts/run-rust-test.mjs issue_7482_shared_paragraph_table_rows -- --cargo-profile release-test --target-dir <shared-target>
cargo nextest run --locked --cargo-profile release-test --target-dir <shared-target> --tests --test-threads 4 --no-fail-fast
CARGO_TARGET_DIR=<shared-target> scripts/wasm-pack-locked.sh --target web --out-dir pkg
cargo test --locked --profile release-test --target-dir <shared-target> --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir <shared-target> --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir <shared-target> --features native-skia
```

전체 회귀에는 공개 신규 fixture의 `RHWP_SECURITY_SWEEP_SAMPLES_JSON` 범위를 전달했다. sweep는 `scripts/visual_sweep.py --hwp <input> --pdf <independent-PDF> --rhwp-bin <immutable-binary> [--wasm-pkg pkg] --embed-fonts full --dpi 96`를 동일 영향 페이지에 실행했다. CDP는 `mydocs/manual/e2e-cdp.md` helpers와 준비된 Chrome/전용 Vite를 사용한다. 기존 17문서는 Vite HTTP, 추가 #6970 문서는 기존 `tests/fixtures/`의 동일 원본 bytes를 CDP 응답으로 공급해 실제 `loadHwpFile`/WASM/canvas를 실행했다. 첫 404는 fixture 경로 환경 실패로 보존하며 layout 결함 증거로 계산하지 않는다.

c8 전체 검사 10,263 PASS / 1 FAIL / 50 skipped는 실제 #6970 겹침 회귀였다. 새 정식 사례는 immutable c8에서 1 run FAIL / 현 source PASS이며 마지막 절의 16px 독립 기대 관계로 검사한다. 잘못 지정한 generated suite에서 0 run인 실행은 결함 증거에서 제외했다. 최종 master, 개별 필수 검사, 실패 기록과 CDP script는 로컬 `output/pr-review/davindev-20261003/candidate-final11-validation/`에 있다. direct-review JSON과 각 sweep `run_manifest.json`은 source·입력/PDF·명령·파일 해시를 고정한다.

최종 Native binary SHA-256은 `fbed82fbb72ed49fe1ef766b599dff18fd74277a73cf000671cd2b98b3cf6b71`이다. 대표 새 캡처 28 PNG·raw 점수는 [시각 증적](assets/pr7482_shared_rows_20261004/README.md)과 [집계](assets/pr7482_shared_rows_20261004/validation-summary.json)에 있다. Square 후속 p1 100.0%, p2 99.43386%, 새 #6970 p3 74.11148%다. #6970 오른쪽 글줄 겹침은 해결했지만 왼쪽 단의 PDF 줄바꿈 차이는 남는다. distribution p3 85.83661%, masked p2 43.1166%, regulatory HWP/HWPX p38 77.99388/78.00141%, form p2 65.49734%의 raw gate와 직접 관측한 남은 차이를 유지한다. 완전 시각 일치로 판정하지 않는다.

## 조판 원칙 판정과 남은 범위

| 범위 | 판정 | 근거/한계 |
| --- | --- | --- |
| fresh TAC 물리 줄/순서/폭/개행/뒤 문단 | 충족 | 정식 좌표 계약과 독립 PDF, Native 및 CDP |
| Square 후속 free lane와 점유/흐름 분리 | 충족 | 수정 전 FAIL/후 PASS, 22개 실제 문단과 독립 PDF |
| stored/standalone 분할 owner 보존 | 부분 검증 | p38 HWP/HWPX와 기존 관련 회귀; 모든 분할 조합 재검증으로 확대하지 않음 |
| 50% 쪽 수용 | 미검증 | Hancom 2020 기존 대조군과 2024 10개 진단은 큰 TAC를 다음 쪽에 통째로 이월 |
| 90% raw gate | 미충족·사용자 예외 | 이번 #7482만 threshold 예외. 원 점수/gate를 보존하며 전역 정책은 변경하지 않음 |
| 남은 전체 시각 일치 | 미충족 | 기존 masked 사진 표 높이/후속 문단, form p2 간격, regulatory 셀/글꼴/괘선, distribution 표 왼쪽, Paper 장식 차이 |

50% 진단은 22/30 일반 문단×CELL/NONE, 작은 편집 영역 60%/40%, `flowWithText=0`/`affectLSpacing=1` 변형을 포함한다. 이는 수동 생성 진단이며 정상 저장본의 모든 적용 조건을 입증하지 않는다. [한컴 여러 쪽 지원](https://help.hancom.com/hoffice130/ko-KR/Hwp/table/tableattribute/table%28many%29.htm)은 TAC 여러 쪽 설정 비적용을 설명하지만 50% 정책을 설명하지 않는다. 적용 조건의 독립 사례 또는 새 정책 채택 결정이 필요하다.

## 원격 반영과 후속 처리

원 PR head에 fast-forward 보정을 추가하는 dry-run은 실제 원격 source SHA가 고정 head와 일치하고 LFS 대상이 없음을 확인한 뒤 실행했다. GitHub가 `Permission to kidsnote/rhwp.git denied to edwardkim` / 403으로 거부했다. `maintainer_can_modify=true`를 실제 write 권한으로 간주하지 않는다. 원격 변경은 없으며 contributor commit rewrite/force push도 하지 않는다. 대체 경로는 이 검증 branch를 원 저장소에 공개해 별도 보정 PR로 검토하는 것이다. 이 경로로의 push/PR 생성/comment/원 PR 처리는 각각 작업지시자의 승인 범위를 따라야 하고, 50% 미검증 범위를 포함한 부분 제출 결정도 필요하다. 공개 직전 최신 base 호환/LFS를 재확인하고 새로운 head의 full CI를 기다린다.

merge 후 contributor comment는 한국어 존댓말로 원 기여의 개선과 추가 보정, 실제 CI·merge SHA, 이 SHA로 고정한 대표 review/overlay Markdown 이미지를 알린다. 남은 차이를 적고 `--body-file`로 게시한 뒤 API 본문을 재조회한다. 50% 정책까지 해결하지 않았다면 전체 이슈 종료 표현을 쓰지 않는다.


## #6970 소유권 경계 보정

앞 legacy owner 뒤에서 과거 제외 영역만으로 unchanged plain rows의 절대 원점 소유를 바꾼 것이 c8 회귀의 원인이다. 마지막 실제 PageItem의 확정 shared plan.end와 현재 흐름 커서가 연결된 경우만 같은 rows를 이어받는다(`typeset/inline_flow.rs`). 해당 제외 영역이 실제 줄을 carve하는 경로/TAC 줄 구성은 유지한다. 그림 host 자체를 공통 계획으로 바꾸거나 좌표를 clamp하지 않는다.

독립 한컴 p3 글줄 top 227.773/239.773pt의 12pt=16px 간격을 새 정식 `unrelated_exclusion_preserves_picture_host_successor_pitch`의 기대 관계로 삼았다. 실제 최종 141/142/143번 줄 간격·비겹침·뒤 빈 문단·문자를 검사하며, immutable c8 binary에서는 1 run FAIL, 현재 Native는 PASS다. 기존 TAC/Square 14개와 새 1개, #6970 경계 9개를 합쳐 24/24 PASS(0.308s). Native review/standalone overlay의 직접 판독에서도 오른쪽 세 줄이 분리되며 원 gate 74.11148%와 남은 왼쪽 단 차이는 보존한다. c8 full 실패 로그와 최신 head의 별도 검증 로그를 연결한다.

## 승인된 부분 보정 공개

사용자가 준비된 부분 보정의 push·별도 PR 생성을 승인해 `fix/pr7482-shared-tac-rows`와 Open [#7563](https://github.com/edwardkim/rhwp/pull/7563)을 공개했습니다. [새 PR self-review](archives/pr_7563_review.md)는 원 검토 head와 공개 head, 검증·남은 CI/검토 조건을 구분합니다. 원 #7482와 #7518의 disposition은 별도이며 merge 승인은 받지 않았습니다.
