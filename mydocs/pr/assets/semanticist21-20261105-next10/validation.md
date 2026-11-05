# 다음 10건의 누적 검증 기록

**로컬 검증과 최종 head CI·self-review를 확인한 뒤 통합 PR #8640을 병합했다.**

대상은 #8368·#8369·#8385·#8484·#8488·#8490·#8492·#8494·#8500·#8504다. 원 head 10개를 merge 조상으로 보존했으며 contributor branch에 push하지 않았다. 원격 처리 승인을 받아 원 PR 10건의 자동 MERGED까지 확인했다.

## 완료해 보존한 이전 source

제품 source `6337fe4d102b591b65c61b92a8da8b2df71eb8cb`, base `ed057276c1e0e07e743218d95641e015b46b6715`(중간 d4a72efd7c와 제품 tree 같음).

- 문서별 글꼴 한도 안의 실제 Native/fresh WASM: 각각 46상태·124쪽, 최저 96.49% / 97.10%, 90% 미만·누락·독립 PDF 쪽수 불일치·2%p 초과 하락 없음.
- 최소 글꼴 전체 nextest 12,544 PASS·45 SKIP·exit 0. 실제 Native 공식 3종 4,064 PASS·12 IGNORE, 모든 Rust lint·workspace build, Studio TypeScript·2,720 tests·build 통과.
- 최적화 WASM의 실제 API 9묶음과 한컴 계산 기대값 37개, 실제 Studio Canvas2D 6장면, CanvasKit의 문자 획·공통 테두리/각주 소비 검사를 별도로 실행했다.
- Canonical KoPub/HCR 편람 Native 383쪽 passed·최저 99.96%다. 일반 샘플 글꼴 팩을 공급한 초기 실패와 구분한다.
- 검증 입력 오류를 보정한 보안 suite 6개 및 fixture 45개 × CLI 세 탐지기 검사를 완료했다. `samples/`의 신규/변경 문서는 없다.

[실제 실행 기록](pre8622-validation.md), [전쪽 TSV·입력/PDF 해시](validation/pre8622/summary.json), [확대 판독과 대표 이미지](validation/pre8622/direct-reading.md), [글꼴 공급 보정](font-qualification.md), [원형 점선 독립 측정](circle-print-geometry.md)을 연결한다.

## 새 base의 실제 검증

최신 devel [#8622](https://github.com/edwardkim/rhwp/pull/8622)의 `6e1de406209ba9a58032f6ca6de45b18aa687d1c`를 통합했다. 실제 시각 producer `d53115fabc92bd392044d9b906c599baddb4368d`의 Native SHA-256은 `53a260e289c11c8c08471a839b036874dc168678c4f3fac3805aa34142e153da`, optimized fresh WASM은 `8b1db6f82ad0ff39bd79f588a8bc691daeab000def4c9a792523299a506a56c7`이다.

- 이번 10건: [46상태·전 124쪽](validation/post8622/summary.json), Native 최저 96.49% / fresh WASM 최저 97.10%. 전쪽 누락·쪽수 불일치·90% 미만·2%p 초과 하락 없음.
- 새 base의 39 PR 보존: [80개 입력/Print/compat·전 406쪽](validation/inherited8622/summary.json), Native 최저 92.45% / WASM 최저 92.99%. 각 경로의 PNG 403쪽은 base와 byte 단위로 같고, 달라진 3쪽도 점수 하락이 없다. 나머지 비시각 2 PR은 실제 CLI와 전체 회귀의 대응 사례로 확인한다.
- [직접 확대 판독](validation/post8622/direct-reading.md), [이미지와 crop·SHA](validation/post8622/image-index.tsv)를 별도로 완료했다. 변경 영역과 뒤 본문·표 하단·참조 순서를 직접 확인했다.
- canonical Native 편람은 [383쪽 gate](validation/post8622/handbook-native-gate.json) passed, [실제 TSV](validation/post8622/handbook-native.tsv)의 최저 99.96%·370쪽 100%다.

## 마지막 계산 API 보정과 최종 fresh 산출물

제품 source `584c9503baa651c62d0403dad9cb77c5cb9ffde6`에서 일반 계산 API의 비유한 결과를 JSON/셀 기록 전에 거부했다. 저장 FORMULA에는 이미 같은 검사가 있었으므로 [공통화 경로와 동일 subtree](validation/post8622/finite-fix-scope.json)를 확인했다. 위 530쪽·편람 383쪽의 producer를 이 최종 source로 바꾸어 표시하지 않는다.

[최종 빌드 명령·source tree·SHA](validation/finite-final/candidate-finite-builds.json), [WASM manifest](validation/finite-final/wasm-finite-manifest.json)를 보존한다. 최종 Native SHA-256은 `b530ec0d4976241c04d994b5f9f5532f994e0654630618708f83a69f0a23916a`, optimized fresh WASM은 `11012c56c9463f4eca27426ab118a2c73a54d90418597315f3df7c0211d2b78d`다.

- 영향받는 계산식 7상태·전 7쪽을 [최종 Native/fresh WASM으로 추가 실행](validation/finite-final/summary.json)했다. 양쪽 최저 100.00%, 14개 PNG가 이전 producer와 byte 단위로 같다.
- [실제 최종 WASM API 10묶음](validation/finite-final/wasm-api-probe-finite.json): 셀 균등화·양쪽 저장, LOG/LN/ROUND/각도, 한컴 기대값 37개·FORMULA 33개, 생성/삭제/자기·순환 거부, 각주·개요 ID·장식 속성·미주·문단 머리와 비유한 결과의 원자적 오류를 확인했다.
- [실제 계산식 대화창](validation/finite-final/formula-dialog-finite.json): public dispatcher로 LOG(100)=2·LN(EXP(1))=1, undo/redo, LN(-1)의 정상 안내·셀 불변·취소를 확인했다. [수정 전 실제 API의 잘못된 JSON과 NaN 기록](validation/finite-final/nonfinite-api-before.json)과 구분한다.
- [최종 브라우저/Studio 실행](validation/finite-final/wasm-runtime.json): TypeScript, Studio 2,720 PASS·2 기존 SKIP, build, bindings, 실제 Canvas2D 6장면, CanvasKit outline 6종과 공통 테두리/각주 소비를 모두 통과했다. CanvasKit fixture의 실제 outline 경로는 textRun이다. glyphRun의 별도 fixture 실행을 주장하지 않는다.
- [공개 fixture 45개 × CLI 3종 보안 탐지](validation/finite-final/fixture-security-finite-results.json) 모두 clean·exit 0이다. 신규/변경 samples는 없으며 samples 전용 환경변수를 fixtures 경로에 사용하지 않았다.

## 최종 Rust 필수 게이트

[실제 명령·소스·종료 코드·실행 시간](validation/finite-final/rust-gates.json) 19개 단계를 모두 완료했다. [계수와 원 실패 회귀의 실제 PASS](validation/finite-final/completion.json)를 연결한다.

- release lib: **3,983 PASS·12 IGNORE**.
- 전체 nextest: **12,629 PASS·0 FAIL·45 SKIP**. 새 비유한 결과의 문서 불변 회귀와 원 CI에서 실패했던 WMF 자원 검사의 실제 PASS를 확인했다.
- Native 공식 3종: **4,064 PASS·12 IGNORE**.
- 모든 fmt·Rust lint·workspace build·doc test가 exit 0이다. [실제 최소 글꼴 5개](validation/finite-final/minimum-fonts.txt)를 보존하며 외부 상용 글꼴을 볼 수 없는 환경에서 회귀를 실행했다.

| 실행 | 결과 | 초 |
| --- | --- | ---: |
| `manifest-prepare` | exit 0 | 1.45 |
| `manifest-check` | exit 0 | 0.79 |
| `unit-tiers-check` | exit 0 | 10.64 |
| `release-build` | exit 0 | 1042.40 |
| `release-lib-build` | exit 0 | 1003.82 |
| `release-lib-runtime` | exit 0 | 16.12 |
| `nextest-build` | exit 0 | 663.68 |
| `nextest-runtime` | exit 0 | 642.33 |
| `native-lib-build` | exit 0 | 300.59 |
| `native-lib-runtime` | exit 0 | 18.47 |
| `issue_2225_missing_picture_placeholder` | exit 0 | 258.58 |
| `render_p37_direct_pdf_export` | exit 0 | 15.83 |
| `fmt` | exit 0 | 25.33 |
| `diff-check` | exit 0 | 0.41 |
| `clippy-root` | exit 0 | 81.67 |
| `clippy-wasm32` | exit 0 | 75.24 |
| `workspace-build` | exit 0 | 110.03 |
| `clippy-workspace-all-targets` | exit 0 | 106.99 |
| `doc-test` | exit 0 | 159.44 |

## 재현과 검증 환경 보정

[최신 base 보존 계획](rebase8622-plan.md), [독립 Print 출처](print-provenance.md), [실제 편집/저장 입력의 Print](edited-print-provenance.md), [문서별 글꼴 공급](font-qualification.md)을 따른다. 입력·PDF·per-page TSV는 검토 commit에 포함하며 폰트 program·인증·서비스 원시 응답·생성 harness는 포함하지 않는다.

처음 사용한 큰 글꼴 팩과 system-only Native 결과는 최종 증거에서 제외하고 올바른 문서별 구성을 실제 재출력했다. 공용 Cargo target에서 base 산출물이 잘못 재사용된 두 짧은 빌드도 무효로 분류했다. source 내용은 바꾸지 않고 소유 worktree의 root source mtime을 갱신해 실제 컴파일한 뒤 source·binary/API·SHA를 다시 확인했다. 마지막 대화창 하네스의 기본 host 접속 실패는 headless를 명시한 실행으로 바로잡았으며 제품 코드는 바꾸지 않았다. 이후 실제 앱의 오류 표시와 셀 불변 검사를 모두 통과했다.

원격 검증과 merge의 실제 결과는 [최종 기록](../../archives/pr_8640_review.md)에 확정했다. [원격 처리안](remote-action-plan.md)과 [PR 본문](pr-body.md)으로 다음 원격 단계 승인을 확인한다.


## 마지막 base 전진의 별도 호환 확인

원 code head 584c9503ba는 유지했다. 현재 devel a95d1d5afa와의 실제 병합 probe 06bf0855b4에서 [별도 기능·출력 확인](current-base-compatibility.md)을 완료했다. 이 프로그램의 Native/fresh WASM은 이번 124쪽씩과 새 경로 3·4쪽을 실제 재출력했고, 전쪽은 기존 후보와 동일하며 영향 쪽은 양쪽 100%다. 집중 Rust 18건, 실제 그룹 글상자 입력/선택/Esc·계산식·이번 API와 Studio 2,721건도 통과했다. 고정 code head의 전체 19단계와 새 병합 probe의 검증을 합산하거나 같은 producer라고 표시하지 않는다.
