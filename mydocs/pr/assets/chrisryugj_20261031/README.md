# chrisryugj PR #8224·#8233 검증 증거

최종 PR 판정은 [#8224 검토](../../pr_8224_review.md)와 [#8233 검토](../../pr_8233_review.md)를 따른다.
이 문서는 2026-10-31 실제 실행한 저장·시각·공통 검증의 근거다. 필수 로컬 검증을 완료했으며,
원격 push·새 PR·CI·머지는 별도 승인 및 실행 대상이다.

## 비교 기준과 producer

- 고정 devel: `55f40ecd99485a1c9b1f7cdc709cb6efcb9e9b29`.
- 원 head: #8224 `caabfdc2c4568f8f151c79d9d86c8df56779c52a`, #8233 `d9c8a71b642fad1ee8221f4036817a23dfe15f10`.
- 통합 제품 변경: `f85410c616`; 후속 테스트·기록 head: `b8c0ef2e4ba1b827db13b9b86bf355f7e27f5e4f`.
- Native는 두 source worktree에서 `cargo build --locked --profile release-test --features native-skia --bin rhwp`로 각각 재컴파일했다.
- 후보 WASM은 Docker에서 release 컴파일과 wasm-opt까지 완료했다. 빌드 13분 5초.
- base WASM은 직전 검증 제품 `c3f5cbd6d7beecdaa231cd3a40080f2e7f3c8c1e`를 재사용했다.
  고정 devel과 `src`, `crates`, `vendor`, Cargo manifest/lock, `build.rs`의 diff가 비어 있음을 확인했다.
- 파일 SHA-256과 producer source: [validation-provenance.json](validation-provenance.json).
  Sweep manifest의 `git_head`는 실행한 기본 checkout이며 producer source와 구분한다.
- 첫 후보 집중 실행에서 공용 target의 base rlib가 재사용된 결과는 제외했다.
  source 수정 시각을 갱신하고 실제 재컴파일한 [후보 결과](candidate-fresh-results.json)는 8개 통과다.
  [base 결과](base-focused-results.json)는 탭 4개·그룹 데이터 3개 실패, 데이터 없는 탭 1개 통과다.

## 입력과 독립 Print

원본은 [lseg-05-tab.hwp](../../../../samples/lseg-05-tab.hwp), 기준은
[lseg-05-tab-hwp-2020.pdf](../../../../pdf/lseg-05-tab-hwp-2020.pdf)다.
Hancom Office 2010 저장본이므로 한컴 2020 engine을 사용했다. 새 Print 작업의 96 DPI 래스터가
기존 기준 PDF와 픽셀 단위로 동일한 것도 확인했다.

| 편집 | 실제 실행 | 저장본 | 한컴 Print |
| --- | --- | --- | --- |
| 삽입 | 문단 0.2 시작에 `새\t` 삽입 | [HWP](../../../../samples/pr8224/lseg-tab-insert.hwp) | [PDF](../../../../pdf/pr8224/lseg-tab-insert-2020.pdf) |
| 삭제 | 문단 0.2의 첫 탭 삭제 | [HWP](../../../../samples/pr8224/lseg-tab-delete.hwp) | [PDF](../../../../pdf/pr8224/lseg-tab-delete-2020.pdf) |
| 분할 | 문단 0.2의 첫 탭 직후 분할 | [HWP](../../../../samples/pr8224/lseg-tab-split.hwp) | [PDF](../../../../pdf/pr8224/lseg-tab-split-2020.pdf) |
| 재병합 | 위 분할 후 문단 0.3을 앞 문단으로 병합 | [HWP](../../../../samples/pr8224/lseg-tab-split-merge.hwp) | [PDF](../../../../pdf/pr8224/lseg-tab-split-merge-2020.pdf) |

각 편집 저장본의 SHA를 고정한 뒤 원격 Print를 생성했다. 모든 작업의
`pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, 한컴 `11.0.0.9136`을 확인했다.
[작업·입력 SHA](print-edit-jobs.json), `print-*-status.json`과
[임시 입력에서 정식 입력으로의 매핑](input-file-map.json)에 원본성을 기록했다.
초기 탐색용 Print와 최종 고정 입력의 Print는 구분하며 이 디렉터리에는 최종 결과만 보존한다.

## 전쪽 시각 비교

모두 Print profile, compat 2022, 96 DPI, PDFium, 흰 배경, 보정 전 2px raw 실루엣이다.
Native Skia와 실제 Chrome의 fresh WASM Canvas2D로 각각 출력했다. `/mnt/c/Windows/Fonts`를
동일하게 공급했으며 대체 글꼴 선언은 추가하지 않았다. 실제 등록 family·face·글꼴 프로그램 SHA는
`*-wasm-export.json`의 font receipt에 기록했다. 글꼴 파일 자체는 저장소에 추가하지 않았다.

아래는 같은 저장 입력을 devel과 후보에서 연 결과다. 각 입력은 전체 1쪽이며 독립 PDF와 쪽수가 일치한다.

| 입력 | Native base → 후보 | WASM base → 후보 |
| --- | --- | --- |
| 원본 | 98.05964 → 98.05964% | 97.90605 → 97.90605% |
| 삽입 | 98.04957 → 98.04957% | 97.89441 → 97.89441% |
| 삭제 | 98.04650 → 98.04650% | 97.89252 → 97.89252% |
| 분할 | 98.09736 → 98.09736% | 98.00278 → 98.00278% |
| 재병합 | 98.05964 → 98.05964% | 97.90605 → 97.90605% |

전쪽 최저 Native 98.04650%, WASM 97.89252%. 90% 미달·누락·쪽수 불일치·2%p 초과 하락은 모두 0건이다.
[비교 TSV](visual-summary.tsv)와 [Decimal 원 수치 비교](visual-summary.json)를 사용한다.
`*-silhouette.tsv`는 원 TSV, 같은 이름의 manifest는 반올림 전 수치·실행 조건·PNG 해시를 보존한다.
자동 gate의 `not_evaluated`는 승인으로 해석하지 않으며 아래 직접 판독과 함께 판단했다.

저장·재열기와 별도로 원본을 브라우저에서 직접 편집한 Canvas도 측정했다.
삽입은 devel 97.32876% → 후보 97.89491%, 삭제·분할·재병합은 각각 97.89252·98.00278·97.90605%로 같다.
`*-live-canvas.tsv`와 manifest를 참고한다. 삽입의 Native CLI 저장본과 WASM 저장본은 SHA-256까지 동일하다.

## 직접 확대 판독

[삽입](insert-detail.png), [삭제](delete-detail.png), [분할](split-detail.png),
[재병합](split-merge-detail.png)을 같은 영역·2배 배율로 대조했다. 각 패널은 독립 Print,
Native 재열기, WASM 편집 직후 순서다. `*-native.png`, `*-wasm.png`, `*-pdf.png`는 전쪽 원 PNG이고,
`*-wasm-live.png`는 편집 직후 PNG다. 재열기와 편집 직후의 미세한 raster 차이를 서로의 해시로 대신하지 않는다.

`새` 뒤 첫 탭, 기존 `탭시작`과 `탭끝`, 분할 뒤 오른쪽 문단 시작, 후속 본문의 줄바꿈·내용 순서와
줄 간격을 확인했다. 글자 누락·중복·겹침이나 새 배치 결함을 발견하지 않았다. 이 입력에는 수식이 없다.
남는 미세한 글리프 외곽·농도 차이는 기존 backend 차이와 함께 기록하며 100% 일치로 주장하지 않는다.
새 절대 px 좌표·쪽수 golden은 추가하지 않았다.

## 실제 저장 경로와 그룹 데이터

[WASM 편집 비교](wasm-edit-comparison.json)는 실제 `insertText`·`deleteText`·`splitParagraph`·
`mergeParagraph` → `exportHwp` → 재열기를 실행한 결과다. 탭 입력의 확장 word가 탭 문자 순서에
맞게 이동하며 텍스트는 유지됐다. [Native CLI 삽입 기록](native-cli-insert-records.json)도 같은 수정 효과를 확인했다.

그룹 데이터는 세 원본에서 실제 글자 삽입·삭제 후 저장했다. 자식 도형의 원본 payload 순서·SHA가
각각 `sungeo.hwp` 24개, issue6542 1개, `hwp-multi-001.hwp` 1개 모두 일치한다.
devel은 각각 0개였다. 기본 글꼴 조건에서 base와 후보의 94·7·9쪽 렌더 트리와 텍스트도 동일했다.
이 결과는 CTRL_DATA의 비시각적 저장 보존 검사이며, 이 세 문서의 한컴 시각 일치율로 해석하지 않는다.

`sungeo.hwp`의 전체 CTRL_DATA는 60개에서 48개가 된다. [payload 감사](group-payload-audit.json)에서
원본·후보의 고유 payload 25종은 모두 유지되며, 한 payload의 중복 수만 24→12로 줄었다.
원 PR의 자식 보존 범위와 기존 최상위 중복 표현 차이를 구분하고 #8225 전체 해결로 확대하지 않는다.

## 검증 입력과 기존 검사 범위

[기존 입력 원본성](existing-input-provenance.json)은 실행 파일과 고정 commit의 Git blob을 대조한 결과다.
새 편집 HWP/PDF 8개도 `input-file-map.json`의 SHA와 정식 경로 파일이 동일함을 확인했다.
[소스 일치 기록](validated-source-files.json)은 기본 작업공간과 실제 검증 worktree의 변경 Rust 파일을 비교한다.

동일 탭 원본을 쓰는 #6520의 SVG 경로 동일성 검사는 이미 관계형이라 유지했다. #6800의 400px 폭 핀은
탭 뒤 가시문자의 공개 문자 끝이 bbox에 포함되는 관계로 교체했다. 집중 실행 1건이 실제로 통과했으며
다른 원본의 검사는 유지했다. 이 변경은 제품·WASM 산출물을 바꾸지 않는다.

자동 suite 재배정에 따른 0건 선택과 중첩 worktree의 fmt metadata 오류는 통과 증거에서 제외했다.
`test-harness-finalization.json`, `harness-correction.json`, `fmt-workspace-correction.json`에
재실행 경로를 구분했다. fmt는 동일 소스인 기본 작업공간에서 통과했고, 최종 suite prepare 뒤 검증을 진행했다.

## 최종 로컬 게이트

제품 구현은 `f85410c616`, 테스트·증거 시작점은 `b8c0ef2e4b`이며 최종 테스트 내용은
이 증거와 같은 커밋에 포함된다. 검증 worktree와 기본 작업공간의 변경 Rust 9개 파일은
`validated-source-files.json`의 SHA-256으로 동일함을 확인했다. 시각 producer 생성 뒤 변경은
테스트·문서·fixture뿐이고 제품 소스는 바뀌지 않았다.

| 검사 | 결과 |
| --- | --- |
| 원 계약 집중 대조 | base 7개 실패·1개 통과 → 후보 8개 통과 |
| 최종 전체 release-test nextest | 12,207개 통과, 실패 0, 기존 skip 45; 실행 496.765초 |
| 최종 전체에 포함된 이번 계약 | 탭 6개, 그룹 자식 데이터 3개 모두 통과 |
| Native Skia lib / placeholder / direct PDF | 4,040 / 2 / 22개 통과; lib 기존 ignored 12 |
| fmt / Native Clippy / WASM Clippy / workspace all-target Clippy | 모두 통과, Clippy `-D warnings` |
| workspace build / suite manifest / unit-tier | 모두 통과 |
| 새 편집 HWP 4개 명시 보안 검사 | security corpus·injection contract 20개 통과, 실패 0 |
| 변경 테스트 3개 rustfmt / 문서 링크 / diff whitespace | 통과 |

명령·종료 코드: [gate-results.json](gate-results.json),
[new-samples-security.json](new-samples-security.json), 실제 로그: [logs/](logs/).
전체 회귀의 신규 sample 검사는 환경변수 없는 실행과 구분하여 4개 입력을 명시한 후 별도 실행했다.
보안 집중 실행에서 제외된 12,232개는 필터 비선택 건수이며 전체 실행의 skip 45개와 구분한다.
workspace build의 orchestration 시간에는 후속 테스트 보정 대기가 포함되어 있고 실제 Cargo 빌드는 2분 50초다.
