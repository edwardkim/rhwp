# Issue #6389 단계 2 — 명시적 폰트 환경에 따른 조판

Issue: [#6389](https://github.com/edwardkim/rhwp/issues/6389)

## 분석과 구현 전 계약

- 사용자 지시: 추가 질문 없이 #6389를 진행한다. 이 이슈가 끝나기 전에 다른 이슈를 시작하지 않는다.
- 시작 commit: `e5a305cb4` (1단계 재검증·줄 경계 회귀 검증).
- 입력/독립 근거: 기존 편람 HWP와 KoPub PDF p68, no-ttf PDF p69. 같은 원본의 KoPub 환경과
  Haansoft Batang 환경이 공존한다. #6389 댓글의 86712는 저장 LineSeg/substFont만으로 PDF 생성
  환경을 식별할 수 없다. 따라서 문서 ID나 페이지 수로 환경을 자동 선택하지 않는다.
- 원인 계층: 글꼴 face 자체의 0.872em 값은 맞지만, 사용자가 다른 조판 환경을 명시할 입력과
  공통 스타일 결정 경로가 없다. 출력에서만 fallback을 바꾸면 측정은 여전히 KoPub를 쓰게 된다.
- 구현 범위: 세션의 명시적 폰트 환경(id + source→target face 매핑), DocumentCore와 SVG/render-tree/
  PDF CLI 및 WASM 바인딩. 해소된 스타일과 폰트 진단에서 동일 결정을 소비한다.
- 원본 FontFace/LineSeg는 변경하지 않는다. 내장 폰트는 환경상 미설치로 취급하지 않는다.
  명시하지 않은 폰트와 기본 환경은 기존 결과를 유지한다. 빈 값·CSS family chain 등 잘못된
  환경은 적용 전에 거부하며, 반복 설정은 no-op, 설정 해제는 원래 조판을 복구한다.
- 저장 정보 재사용 계약은 1단계 실제 문서 16줄 검증으로 유지한다. 미설치 환경의 재조판은
  명시적으로 바탕 계열을 선택했을 때 해당 face 메트릭과 paint가 함께 선택되는지 별도로 검증한다.
- 새 매핑은 자동 시스템 폰트 탐지가 아니며, 프로필 선언만으로 특정 한컴 PDF와 전체 일치한다고
  주장하지 않는다. 원본 저장·재열기에 세션 설정을 영구 삽입하지 않는다.

## 검증 계획

1. 기본/대체/해제, 독립 세션, 잘못된 환경, 내장 폰트 보호, 저장 원본 보존을 공개 API로 검증한다.
2. 실제 편람의 저장 줄과 86712의 재조판 결과·폰트 결정을 확인하고 Visual Sweep으로 비교한다.
3. 영향 범위 집중 검증, Rust fmt/Clippy(native·WASM·workspace), 필요한 fresh WASM을 검증한다.
4. 결과와 한계를 이 문서에 기록하고 해당 회차 변경을 함께 커밋한다.

## 실행 결과

### 구현

- `FontEnvironment`를 문서 IR과 분리한 세션 설정으로 추가했다. 정확한 원본 face → 최종 face를
  한 번만 선택하며, 공통 스타일이 측정·조판·paint에 전달된다. 환경 변경 시 기존 Canvas 측정과
  페이지 캐시를 폐기하고 파생 상태를 재구성한다. 편집 후 스타일 재구성도 같은 결정을 유지한다.
- `DocumentCore::set_font_environment`, WASM `setFontEnvironment`, CLI `export-svg`,
  `export-render-tree`, `export-pdf --font-environment`를 연결했다. `fontsUsed`와 폰트 진단도
  동일한 선택을 보고한다. 선언은 `oracle.status=declared`로 표시하며 PDF 일치 인증으로 표현하지 않는다.
- Visual Sweep의 Native·WASM export와 글꼴 CSS 생성에 같은 JSON을 전달하고 환경 해시를
  provenance/resume 조건에 포함했다. 사용법은 [폰트 환경 안내](../manual/font_environment.md)에 기록했다.
- 기존 HWP/PDF를 그대로 사용했다. 새 문서 복제본은 없으며 환경 JSON만 fixture로 추가했다.

### 실행 완료한 검증

검토 worktree는 `/Users/tsjang/rhwp-issue6389-review-20260916`, 전용 target은
`/Users/tsjang/rhwp/target/issue6389-20260916`이다. 기준 `devel`은
`5720d3f1646d6c25e51c8d7cbb0b4cc6bc5feb4f`, 직전 단계 commit은 `e5a305cb4`다.
검증은 커밋 전 수정 소스로 수행했다. [소스 파일별 해시](assets/issue6389-stage2-20260916/validated-source.json)는
실제 빌드 worktree와 작업 트리의 소스·스크립트·fixture 바이트 일치를 확인한 기록이다.

| 검증 | 실제 결과 |
|---|---|
| #6389 집중 Rust test | 환경 계약 5 + 기존 실물 줄 경계 1 = **6 통과** |
| 측정/출력 계약 | KoPub 872HU → 바탕 1000HU, 원본 폰트·저장 bytes 보존, 세션 분리/해제/반복/내장 폰트 보호 확인 |
| 저장 LineSeg 없는 재조판 | 18,000HU 줄에 60글자: 872HU는 20/20/20, 1000HU는 18/18/18/6. 독립 산술 기대값과 일치 |
| CLI 실행 | SVG·render tree·PDF 출력, 잘못된 환경 exit 2 및 출력 미생성 확인 |
| Rust lint | fmt, native Clippy, WASM Clippy, workspace build, all-targets Clippy 통과 |
| fresh WASM | `scripts/wasm-pack-locked.sh --target web --out-dir pkg-issue6389` 통과, Chrome에서 실행 |
| Visual Sweep Python | **51 통과**, Node 구문 검사 통과 |
| suite 정책 | base SHA를 고정한 `rust-test-suite-manifest --check` 통과; generated 파일 미포함 |
| 문서 | 변경 manual 3개 metadata, 보고서 포함 5개 링크, `git diff --check` 통과 |
| Native Skia lib | rhwp **3,930 통과 / 13 ignored**, workspace 보조 crate **182 통과** |
| Native Skia 그림·직접 PDF | 그림 **2/2**, 직접 PDF **4/4** 통과, 실행 묶음 exit 0 |

Rust 집중 test는 생성 suite의 현재 배치를 원본 test 경로로 찾은 후 `issue_6389` filter로 실행했다.
초기 테스트 작성 중 API 이름·임시 디렉터리·JSON key 오기를 수정했으며, 잘못된 suite 선택으로
0개가 실행된 시도는 통과 근거에서 제외했다. 최종 6개 결과만 사용한다. Sweep의 Path 전달 오류도
수정 후 실제 Native/WASM export를 완료했다.

### Visual Sweep 직접 판정

| 입력과 환경 | 범위 | Native / WASM | 직접 확인 |
|---|---|---|---|
| 편람, 기본 KoPub 환경 | 한컴 KoPub PDF p68 | pixel 91.43950 / 91.44526%, ink 79.25197 / 79.26593% | 문제 셀의 16줄과 ※ 3줄, 셀 내부 표시 유지. 페이지 전체의 장식·예시 상자 잔차는 동일성 주장에 포함하지 않음 |
| 86712, KoPub돋움체 → 함초롬바탕 | 한컴 2024 PDF p28 | 양쪽 pixel 86.51529%, ink 7.62931% | 최종 face 변경 확인. 위쪽 이전 행 조각·일부 줄 경계·표 하단 위치·글자 굵기가 다르므로 **PDF 시각 일치 보류** |

- [편람 Native](assets/issue6389-stage2-20260916/manual-native-p068.png) /
  [편람 WASM](assets/issue6389-stage2-20260916/manual-wasm-p068.png)
- [86712 Native](assets/issue6389-stage2-20260916/reflow-native-p028.png) /
  [86712 WASM](assets/issue6389-stage2-20260916/reflow-wasm-p028.png)
- 각 PNG와 같은 이름의 `*-provenance.json`/`*-summary.json`에 입력·PDF·binary·WASM·환경
  SHA256와 원본 명령 산출물의 수치를 보존했다. `flagged=0`은 자동 경고가 없다는 뜻이며 시각 통과 판정이 아니다.
- 86712는 Native/WASM render tree **65/65쪽 JSON 일치**. 편람은 **313/384쪽 JSON 일치**이며,
  나머지 71쪽의 차이 231건은 모두 음수 문단 sentinel `pi`의 usize 64bit/32bit 표현뿐이다.
  모든 페이지의 텍스트와 좌표는 같다. [대조 기록](assets/issue6389-stage2-20260916/backend-differences.json).
- 86712 기본 환경은 64쪽, 명시적 대체 환경은 65쪽이지만 쪽수만으로 한컴 재현을 주장하지 않는다.
  PDF p28 본문 폰트는 `pdftohtml -xml`에서 HCRBatang이며, 이에 따라 최종 fixture는 함초롬바탕을
  선택했다. 최초 바탕 매핑 비교는 최종 증적으로 사용하지 않는다. 전체 글꼴 미설치 상태를 재현한 프로필도 아니다.
- 편람 no-ttf PDF의 대응 내용은 p69이다. 잘못된 p68 대 p68 비교는 배제했다. 이 단계에서
  no-ttf PDF p69와의 최종 대체 환경 시각 일치를 입증한 것은 아니다.

### 전체 검사에서 발견한 분류 누락 보정

전체 nextest의 `classification_drift_is_blocked`가 새 `set_font_environment`를 미분류
`&mut self` API로 검출했다. 이 API는 직렬화할 문서 IR을 바꾸지 않으므로 원본 stream을
무효화하면 오히려 세션 설정 계약을 훼손한다. 기존 `EXEMPT` 분류 표에 `SessionState`로
등록하고, 실제 저장 bytes/원본 face 보존을 확인한 #6389 테스트 이름을 근거로 연결했다.
가드 로직·Pending 상한·baseline 수치는 바꾸지 않았다. 최초 전체 실행은 **9,905개 중 9,904 통과,
분류 누락 1 실패, 51 skipped**(271.140초)였다. 종료 후 분류를 반영하고 해당 가드 전체를 다시
실행해 **5/5 통과**(0.096초)했다. 최종 fmt와 all-targets Clippy도 통과했다.
최초 전체 실행을 전량 통과한 것으로 소급하지 않는다.

### 재현 명령과 로그

검토 worktree에서 `node scripts/rust-test-suite-manifest.mjs --prepare`를 먼저 실행했다.
아래 `$rhwp_review_target`은 위 전용 target 경로다. 파생 suite는 커밋하지 않는다.

```bash
rhwp_review_target=/Users/tsjang/rhwp/target/issue6389-20260916
cargo nextest run --locked --cargo-profile release-test --target-dir "$rhwp_review_target" \
  --tests --no-fail-fast --status-level fail --final-status-level fail
node scripts/run-rust-test.mjs issue_2724_passthrough_invalidation_guard -- \
  --cargo-profile release-test --target-dir "$rhwp_review_target"
cargo test --locked --profile release-test --target-dir "$rhwp_review_target" --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- \
  --cargo-profile release-test --target-dir "$rhwp_review_target" --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- \
  --cargo-profile release-test --target-dir "$rhwp_review_target" --features native-skia
```

[집중 test 로그](assets/issue6389-stage2-20260916/focused-test.log),
[전체 최초 실행](assets/issue6389-stage2-20260916/full-regression-initial.log),
[가드 보정 후 재실행](assets/issue6389-stage2-20260916/passthrough-guard-final.log),
[suite 정책](assets/issue6389-stage2-20260916/suite-policy.log).

### 결과보고와 회차 종료

- 이번 회차의 분석 → 구현·검증 → 결과보고를 완료했다. Native Skia 필수 3종도 모두 통과했다.
  전체 release-test는 9,904 통과/분류 누락 1 실패였으며, 분류 보정 후 해당 가드 5개를 재실행해
  전부 통과했다. 보정은 테스트 분류 표에만 적용했고 생산 코드는 바꾸지 않았다. 보정 후 전체
  9,905개 재실행은 하지 않았으며, 최초 실행을 “최종 전체 전량 통과”로 표현하지 않는다.
- [Skia lib](assets/issue6389-stage2-20260916/native-skia-lib.log),
  [그림](assets/issue6389-stage2-20260916/native-skia-picture.log),
  [직접 PDF](assets/issue6389-stage2-20260916/native-skia-pdf.log)에 실제 종료 결과를 남겼다.
- 사용자에게 이 결과와 잔여 범위를 먼저 보고하고, 이 회차의 분석·소스·테스트·증적·결과보고를
  함께 로컬 커밋한다. 다음 단계 분석은 이 커밋에 미리 포함하지 않는다.
- 명시적 환경을 공통 측정·출력으로 전달하는 기능 계약은 입증했다. 86712의 전체 PDF 일치와
  실제 backend 폰트 가용성 자동 판정은 입증하지 않았다. 이 차이를 face 상수나 baseline 완화로 숨기지 않는다.
- #6389의 기존 저장 줄 넘침과 KoPub 폭 수정은 이미 병합되어 있고 단계 1에서 재검증했다.
  현재 단계는 후속 폰트 환경 경계를 구현한 회차다. 잔여 시각 문제를 포함한 이슈 전체 해결·종료로 보고하지 않는다.
