# PR #7627 리뷰 — 저장 앞 프레임의 원본 세로 정렬 보존

## 최종 판정

**승인.** 원본 저장 프레임의 정렬을 줄 구성과 최종 배치에 함께 적용하는 최소 보정으로, 관련 회귀·전체 Rust 검증·Native/fresh WASM 직접 비교를 통과했다.

이는 collaborator self-review 기록이며 GitHub의 본인 PR Approve 제출과 구분한다. 병합 전에는 정확한 최종 head의 CI·mergeability·base 변화와 사용자 승인을 다시 대조한다. 작업지시자는 이 최소 선행 보정을 제출·검토·병합한 뒤 #7544로 복귀하는 순서를 승인했다. 최초 제출 head `7718c96d2a`의 원격 CI는 29 SUCCESS / 5 SKIPPED / 1 NEUTRAL(CodeQL 결과 게시)로 종료됐고 실패·대기 없음과 CLEAN을 확인했다. 후행 문서 head의 CI·mergeability를 다시 확인한 뒤 병합한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#7627](https://github.com/edwardkim/rhwp/pull/7627) / postmelee / devel |
| Code source | `cf2b0508fc13812b51b345eacf16a47baa6db6ec` |
| 검토·최초 제출 head | `7718c96d2adde8a9b7978a8a86f308dab996ed07`; source/test는 code source와 동일 |
| 정책 비교 base | `48ff4bb9353bf8d29b7e1c0ac44df9bc5800dce8` |
| 규모 | source 13줄(7추가·6삭제), 기존 회귀 보강 41줄, 계획·대표 PNG 7개. 새 fixture·baseline 없음 |
| 관련 이슈 | [#7207](https://github.com/edwardkim/rhwp/issues/7207) 참조만 함; 종료하지 않음 |
| 원래 작업 | [#7544](https://github.com/edwardkim/rhwp/pull/7544)의 리뷰·병합. 이 선행 PR을 #7544 완료로 해석하지 않음 |
| 작성 시점 상태 | Open, non-draft, MERGEABLE, CLEAN. 후보 CI 종료 확인; 최종 후행 상태는 병합 직전 다시 조회 |

## 변경과 검토 범위

`stored_full_width_row_declared_height`가 수용한 저장 앞 프레임을 줄 구성 창에서는 인정하면서 최종 `effective_align`에서는 Top으로 되돌리던 불일치를 없앴다. 기존 앞 프레임과 이어받기 프레임의 소유 결과를 `frame_owns_alignment`로 함께 소비한다. 새 문서·쪽·행 번호 예외, 좌표 clamp 또는 출력 은폐는 없다.

실제 생산·소비 경로는 [최소 구현·검증 기록](../../plans/task_m100_7207_impl.md)에 연결한다. 생산자는 미편집 HWP5 저장 정보·단일 전폭 소유 조건을 확인한다. `continuation/fragment/emit.rs`의 선언 프레임/가용 예산 → `end_row_height_override`/물리 예약 높이 → `table_partial.rs`의 줄 구성 창 → `effective_align` → `centered_content_height` → 실제 원점을 대조했다. 새 조건은 기존 프레임의 최종 정렬 소비만 복원하며 컷·유닛·높이 생산과 fit 실패·이월·종료를 바꾸지 않는다. Top·여러 셀 소유·편집 후 재조판은 기존 수용 조건에 따른다.

#7207 전체, #7445 교육과정 개선, #7544의 Enter·닫는 guide·Studio는 이 PR에 포함하지 않는다. 기존 7쪽 표 하단, 9쪽 다열 셀 문구, 닫는 선과 19쪽 범례선 차이 및 다른 두 문서의 판정은 남아 있다.

## 조판 원칙 준수 검토

| 검토 항목 | 정확한 코드와 실행 근거 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 실제 원본 셀의 Center 속성·독립 PDF17쪽 빈 밴드. 기존 저장 프레임의 소유 결과를 소비하며 조건을 문서에 한정하지 않음. 기존 정상 RowBreak 전18쪽 tree 동일 | 충족 |
| 측정·배치 일관성 | 기존 프레임/예산 생산은 유지. 동일 `frame_owns_alignment`를 줄 구성 창과 최종 정렬이 소비하고 뒤의 `centered_content_height`는 같은 물리 상자 계약 사용. Native/fresh WASM 39쪽 내용·좌표 동등 | 충족 |
| 분할·이어받기 계약 | 컷·유닛·요구/예약 높이·fit 실패·이월·종료 수정은 비해당. 바뀐 앞 조각의 위치와 다음 내용은 원문 검사·17~21쪽 그림6개/캡션 소속·순서로 확인. 전체 쪽수21 유지 | 충족(기존 컷 생산을 재설계했다는 주장은 없음) |
| 줄 소속과 점유 높이 | 기존 저장 줄·프레임 수용과 줄 구성 결과를 재사용. 재조판/Top/복수 셀은 새 앞 프레임 분기 대상이 아님. 관련 #7518 23개 회귀와 정상 대조군 보존 | 충족 |
| 사례와 증거의 독립성 | 원본 Center와 한컴 PDF의 빈 밴드 관계가 기대 근거. 동일 강화 검사에서 base FAIL(exit101) → 보정 PASS(exit0); 구현 offset·실물 절대px·SVG hash를 기대값으로 고정하지 않음 | 충족 |
| 기준값 변경 | baseline·golden·래칫·허용치 변경 없음 | 비해당 |
| 주장과 검증 범위 | 아래 정확한 source의 필수 검증·전쪽 TSV·직접 PNG 판독 완료. 기존 괘선 차이·다른 문서 및 성능 benchmark는 해결/측정으로 주장하지 않음 | 충족(범위 밖은 미검증/남은 문제로 유지) |

## 검증 입력 커밋 확인

**충족.** 아래 실행 파일은 제출 head `7718c96d2adde8a9b7978a8a86f308dab996ed07`의 Git blob과 바이트·SHA-256이 같다. 원문과 한컴 Print PDF는 기존 repository 자료이며 새 원문·PDF를 추가하지 않았다. focused 기존 #7518 검사의 세 입력도 함께 확인했다. 전체 회귀가 사용하는 나머지 기존 fixture를 신규 시각 일치 증거로 승격하지 않는다.

| 저장소 경로 | SHA-256 |
| --- | --- |
| `samples/task2097/18095317_eogu_geumji.hwp` | `956ad319f493aa8edfd26bb318c97b82a187a5860fc6ce3c19e55a4ae8429ed9` |
| `pdf/18095317_eogu_geumji-2020.pdf` | `98a9378f5b3440cc8c56c03dd483e2af95a194b9340ebc916c08474ceea65eab` |
| `samples/rowbreak-problem-pages.hwp` | `10b6ab6548610e18c82ba78a1c844a00107fedbb28c195cb05e6fd20626d33ed` |
| `pdf/rowbreak-problem-pages-2024.pdf` | `9bfe742e084152eb5e204ba66873bffb0af742adc9d4ca89012dc3ce8434b7a2` |
| `samples/issue7500/picture_band_cell_spaces.hwp` | `70c76b83f86a9d5ee4bd619c32f60c22d16ae406fb7bed452b766e8838d207bb` |
| `samples/76076_regulatory_analysis.hwp` | `3308ba8505391bae2d0d62963e9399f4e48cdae574304cc0f89a311c6efbb6b5` |
| `samples/issue1891/80168_regulatory_analysis.hwpx` | `1e00b54e1139515258260a657217e0b3baa0b1b652a2cd28dd4b9bc48aa262fe` |

macOS arm64에서 같은 로컬 글꼴 공급과 print profile로 비교했다. 기존 한컴 2020 어구 PDF와 한컴 2024 RowBreak PDF를 재사용했다. 사용자 제공 파일의 데이터·경로·메타데이터는 이 문서·첨부·Git에 포함하지 않는다. Docker daemon이 없어 fresh WASM은 루트 locked wrapper의 host web package와 wasm-opt 성공으로 검증했고 Docker 실행 통과로 보고하지 않는다.

## 로컬 검증 결과

code source `cf2b0508fc13812b51b345eacf16a47baa6db6ec` 이후 Rust source/test 변경은 없다. Cargo는 공유 `target/pr-review`에서 순차 실행했다.

| 검사 | 실제 결과 |
| --- | --- |
| `cargo fmt --all`, `cargo fmt --all -- --check` | PASS |
| `cargo clippy --locked --target-dir target/pr-review -- -D warnings` | PASS |
| `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings` | PASS |
| `cargo build --locked --workspace --target-dir target/pr-review` | PASS |
| `cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings` | PASS |
| manifest `--check --base-ref 48ff4bb9353bf8d29b7e1c0ac44df9bc5800dce8` | PASS |
| #7207 기존3개 + #7518 기존23개 | 26 PASS |
| `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 4 --no-fail-fast` | 10,509 PASS / 0 FAIL / 50 skipped |
| `cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib` | 4,109 PASS / 0 FAIL / 13 ignored (네 lib 합계) |
| `run-rust-test.mjs issue_2225_missing_picture_placeholder` / `render_p37_direct_pdf_export`, release-test·native-skia | 2 PASS / 4 PASS |
| source-side cfg(test) unit-tier | 변경 없어 비해당 |
| `git diff --check base...HEAD`·변경 문서 링크 | PASS |

첫 workspace build의 disk-full과 앞선 전체 회귀의 sparse fixture 누락은 환경 실패로 구분했다. 같은 commit의 누락 파일 복원 후 다시 실행한 최종 결과만 위 표에 사용했으며 제품 source·기대값을 바꾸지 않았다. 파생 suite·manifest는 Git에 포함하지 않는다. 자세한 명령·결과는 기존 [구현 기록](../../plans/task_m100_7207_impl.md)에 연결하며 ignored output의 로컬 로그를 공개 증적의 대용으로 사용하지 않는다.

## 시각 증적과 남은 차이

Native/fresh WASM 각각 원문 어구21쪽·정상 RowBreak18쪽 전체를 같은 독립 PDF와 96dpi print profile·고정2px 관용으로 비교했다. 평균·글꼴 예외·마스킹·허용치 완화는 쓰지 않았다. TSV 이진화 원값과 색상 경계 값은 [Native 전체39쪽 TSV ZIP](https://github.com/user-attachments/files/33139118/issue7207-native-silhouettes.zip)에 있다. 인증된 다운로드의 SHA-256은 `4b66fb97743f406ebf1fe5a70c46d725b8f96181c0cd245e79c57f1a03da06cb`로 로컬 원본과 같다.

| 범위 | Native 최저 | fresh WASM 최저 | 90% 미만·누락 |
| --- | --- | --- | --- |
| 어구 전21쪽 | 93.22327%(7쪽) | 93.22327%(7쪽) | 양쪽0 |
| 정상 RowBreak 전18쪽 | 92.48319%(12쪽) | 92.48319%(12쪽) | 양쪽0 |

대표 [변경 전17쪽](../../working/assets/issue7207-center-frame-20261007/before-native-map17-review.png), [Native17쪽 review](../../working/assets/issue7207-center-frame-20261007/after-native-map17-review.png)·[overlay](../../working/assets/issue7207-center-frame-20261007/after-native-map17-overlay.png), [fresh WASM17쪽 review](../../working/assets/issue7207-center-frame-20261007/after-wasm-map17-review.png)·[overlay](../../working/assets/issue7207-center-frame-20261007/after-wasm-map17-overlay.png)를 직접 판독했다. 첫 지도 앞 빈 밴드·그림/캡션 위치·다음 제목·내용 소속이 복원됐다. 17쪽은 양쪽34.67390% → 97.32653%, 18쪽26.38697% → 94.51240%다. 본문11~15쪽은 98.43249/98.24875/98.29738/98.33620/98.14148%이며 [Native11쪽](../../working/assets/issue7207-center-frame-20261007/after-native-body11-review.png)·[WASM11쪽](../../working/assets/issue7207-center-frame-20261007/after-wasm-body11-review.png)을 보존했다.

어구 Native17/19/7쪽과 fresh WASM17쪽 직접 판독에서 남아 있는 7쪽 하단 괘선·19쪽 범례선 차이는 변경 전후 동일했다. 자동 점수를 전체 fidelity100%나 #7207 전체 해결로 바꾸어 쓰지 않는다. 어구 전21쪽은 텍스트 순서 변화0·전체 쪽수21 유지, 바뀐 쪽11~15/17~18이다. 정상 전18쪽의 전후 tree 차이0이며 양쪽 출력39쪽은 구조·내용·좌표가 동등하다(좌표1e-5, 같은 의미의 아키텍처별 pi sentinel만 동등화).

PNG는 code source에서 생성한 것을 문서 후행 commit에 보존한다. 이후 source 변경이 없어 재출력으로 가장하지 않는다. PR 본문의 실제 Markdown 이미지와 첨부 링크가 표시됨을 브라우저 DOM에서 확인했다. 최종 후행 head에는 같은 asset의 정확한 head SHA URL로 갱신한다.

## 원격 CI

최초 제출 head `7718c96d2adde8a9b7978a8a86f308dab996ed07`의 [CI run](https://github.com/edwardkim/rhwp/actions/runs/37563781126)은 성공했다. preflight는 base classifier v7, `classified:rust+rust-render`로 Rust·Render·Native Skia·Rust CodeQL을 요구했다. lint, Native Skia, Archive A/B/C/D의 빌드·실행 및 Build & Test aggregate가 성공했다. CodeQL·Render Diff·Adapter·Proptest와 CI Impact Policy도 종료돼 실패·대기가 없다. source 범위상 frontend·WASM Build 등 5개 생략과 CodeQL 결과 게시 neutral 1개를 실제 성공과 구분한다. 후행 commit은 이 검토·오늘할일만 추가하므로 검증한 source/test와 동일함을 확인하고 그 head의 CI도 따로 확인한다.

## 처리 순서와 Merge 후 contributor PR comment 계획

이 소형 self PR은 별도 review_impl 대신 이 절에 처리 순서를 기록한다. 검토·오늘할일을 같은 branch의 문서 후행 commit으로 push → 정확한 최종 head CI·base·mergeability 확인 → 일반 merge → devel 포함과 duration 갱신 확인 → PR/참조 이슈 후속 댓글 및 안전한 전용 branch/worktree 정리 → #7544 검토로 복귀한다. GitHub self-Approve·branch protection 우회는 하지 않는다.

시각 증적이 merge commit에 실제 포함된 뒤 한국어 존댓말 댓글을 `--body-file`로 게시하고 API로 원문·링크를 재확인한다. 댓글에는 merge SHA·정확한 CI URL·위 로컬 결과·[Visual Sweep merge comment 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)·Native/fresh WASM17쪽 review/overlay 네 이미지의 merge SHA 고정 raw URL과 남은 차이를 포함한다. 이 자동 수치는 사람의 판정 정확도가 아닌 내용 실루엣 비교 보조값이라고 설명한다.

#7207에는 최소 정렬 보정의 merge 사실과 #7544 재검토의 선행 작업이라는 범위만 남기며 OPEN을 유지한다. 종료 표현은 쓰지 않는다. #7544는 기존 원문을 유지하고 재검토 결과를 댓글로 추가한다. 교육과정 전체·#7207 잔여 경로를 이 PR에 추가하지 않는다. 다른 활성 검토가 사용하는 worktree·입력 링크·공유 target은 제거하지 않고 정확한 유지 사유를 남긴다.
