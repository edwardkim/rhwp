# PR #9083 리뷰 — 다섯 원본 389쪽 피델리티와 공통 paint 보정

## 최종 판정

**승인 — 다섯 원본389쪽의 피델리티·내용 소유·회귀 보호와 정확한 code head CI를 확인했다.**
남은 병합 조건은 이 기록의 문서-only trailing head 검사와 merge 직전 최신 head/CLEAN 재확인이다.

검토한 code head는 `e9b3114b3506483b423541349639d18b3731e6ac`, 실제 최종 제품 검증 source는
`3f7e4340ae3038ea77917ff0f45f816e8342770e`, 비교 base는
`53da4d2da28ae0cacf8feac3a989cc6a69fb2622`다. 둘 사이에는 분석·검증 증적만 추가했고
Rust·Studio·회귀·스크립트의 바이트가 같은지 확인했다. CI 변경은 없다.

## 범위와 실제 검증

[PR #9083](https://github.com/edwardkim/rhwp/pull/9083)은 collaborator self PR이다.
대표 #7445의 아래 다섯 원본을 완료 범위로 하며 다른 미완료 문서는 포함하지 않는다.

| 이슈 | 원본 | 독립 Print 쪽수 | Native 최저 | 최종 fresh WASM 최저 |
| --- | --- | ---: | ---: | ---: |
| [7727](https://github.com/edwardkim/rhwp/issues/7727) | CBTA 연구 | 204 | 98.13% | 98.13% |
| [7729](https://github.com/edwardkim/rhwp/issues/7729) | 제도개선 권고안30269 | 22 | 98.69% | 98.68% |
| [7730](https://github.com/edwardkim/rhwp/issues/7730) | 영어 시험지 | 8 | 99.49% | 99.49% |
| [7731](https://github.com/edwardkim/rhwp/issues/7731) | HWP 컨트롤 API | 105 | 99.55% | 99.55% |
| [7732](https://github.com/edwardkim/rhwp/issues/7732) | RowBreak 각주 표 | 50 | 98.75% | 98.75% |

같은 원본·독립 한컴 Desktop Print·정상 설치 글꼴에서96DPI/compat2022/print/raw2px/채널 문턱32로
대조했다. 웹한글기안기 PDF는 판정 기준이 아니다. CBTA·각주 PDF의 Creator는2020,
권고안·시험지·API는2022다. 파일명의2020을 제품 버전으로 추정하지 않았다.
[입력/PDF 해시·출처](../assets/issues7727_7732_20261114/final-1123f322/inputs.json)와
[검토 commit의 실제 blob 일치](../assets/issues7727_7732_20261114/final-1123f322/committed-original-input-audit.json)를 확인했다.

[최종 증적 진입점](../assets/issues7727_7732_20261114/README.md)에 각 원본의 전쪽 Native/fresh WASM
TSV와 대응 쪽의 raw/PDF/review/overlay를 연결했다. 제출 본문 raw 링크48개를 내려받아
제출 commit의 blob와 바이트·SHA-256이 같음을 확인했다.

## 구현과 조판 원칙 준수 검토

[원인·반례·원 해결 이력과 생산→소비 분석](../../working/task_m100_7727_stage1.md)을 따랐다.
번호 문단의 현재 문자 경계와 저장 세로 슬롯, 그림 캡션의 공통 물리 원점, 병합 칸의 완료 행 종료
점유를 구분했다. 표 앞뒤의 중복 예약과 주석 전용 줄 소유도 측정·실제 paint 소비에서 확인했다.
그라데이션 공통 채널 보간·원형 반경, 영문 반각 공백 em·밑줄 원점과 RIGHT 탭의 수용된 저장
전진을 수정했다. 문서 ID 예외나 판정 문턱 완화로 수용하지 않았다.

실제 텍스트 편집은 과거 탭 전진을 무효화한다. Undo 메타 복원과 빈 입력은 이 편집과 달리
기존 값을 보존하도록 분리했다. HWP/HWPX 공개 편집 API로 편집→저장→재열기→재저장→재열기한
독립 Print 대조군은 Native99.90%·WASM100.00%다.
[반복 저장 근거](../assets/issues7727_7732_20261114/final-1123f322/edited-save-print/validation.json)를 보존했다.
기존 HWPX raw numbering namespace alias 진단은 별도 실패로 남겼으며 canonical prefix 대조군을
일반 alias 직렬화 결함까지 해결한 것으로 설명하지 않는다.

회귀는 `tests/cases/` 자동 수집 경로에 추가했다. 독립 PDF 기대값이 정확한 기존 검사는 유지했다.
실제 조판 보정·전쪽 시각 수용 뒤 잘못된 수집/정렬 기대만 관계형으로 바로잡았다. CBTA118·162·166쪽의
양쪽98% 이상과 직접 판독을 확인한 독립 기대 좌표는±2 CSSpx로 보호한다. RIGHT 탭 편집·Undo·빈 입력,
그라데이션의 실제 layer JSON 물리 원과 스타일 동점 소유는 각각 별도 반례와 정상 대조군으로 검사한다.

## 로컬 검증과 증적 재사용

- Mac 최소 글꼴 전체 `cargo nextest run --locked --cargo-profile release-test --tests --test-threads 8 --no-fail-fast`: **13,565 PASS / 0 FAIL / 45 skip**, 테스트461.749초.
- fmt·Native/WASM/all-target Clippy·workspace build·manifest·unit tier를 포함한14단계 전부exit0.
- Native Skia lib **3,852+15+165+2 PASS**,12ignore, placeholder2건·독립P37 PDF 경로도 통과.
- Studio 전체 **2,791 PASS / 0 FAIL / 2 skip**, tsc·Vite build·renderer replay 계약 PASS.
- Mac fresh WASM은 호스트 `scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt` 대체 빌드다. Docker/wasm-opt 완료로 쓰지 않는다. 새 WASM389쪽을 전부 재산출했고 기존389 PNG와 바이트가 전부 같았다.
- Native389쪽·보호36조건3,542쪽은 source1123f322의 실제 실행을 재사용한다. 최종 제품과 추가 Rust 차이는 paint JSON뿐이며 Native/직접 Canvas의 읽기·그리기 경로 SHA가 같음을 [재사용 감사](../assets/issues7727_7732_20261114/canvaskit-common-3f7e4340/unchanged-path-reuse-audit.json)로 입증했다. 과거 실행을 새 source의 재실행으로 기록하지 않았다.
- 보호 공식 두 자리 값 하락0·상승0, 필수 편람 HWPX Native383쪽 gate PASS. 고정 base Native/fresh WASM 대조에서2%p 초과 하락0쪽. Native3쪽의 최대0.06%p 하락은 같은 쪽 직접 판독으로 내용 소유·정렬 보존을 확인했다.

[최종 Rust 실행](../assets/issues7727_7732_20261114/canvaskit-common-3f7e4340/rust-validation.json) ·
[최종 Studio 실행](../assets/issues7727_7732_20261114/canvaskit-common-3f7e4340/studio-validation.json) ·
[389쪽 새 WASM](../assets/issues7727_7732_20261114/canvaskit-common-3f7e4340/fresh-wasm-389-validation.json) ·
[고정 base](../assets/issues7727_7732_20261114/final-1123f322/fixed-base-comparison.json) ·
[보호3,542쪽](../assets/issues7727_7732_20261114/final-1123f322/protected-validation.json).

## 실제 소비 경로·직접 판독과 한계

정상 설치 SFNT의 실제 bytes·metadata를 공급하는 플랫폼 API adapter와 공통 Rust 프로그램 선택
계약으로 실제 Studio Canvas2D Print15쪽을 비교해 최저98.21%를 확인했다. 실제 Chrome OS 글꼴
열거가 검증됐다는 뜻은 아니다. 다른 자동 선택 조건의 진단값을 이 수용 결과와 섞지 않는다.
대표7쪽의 review·standalone overlay에서 목차 번호, 캡션·그림, 병합 행, 표 다음 본문과 주석 소유를
직접 판독했다. [판독 기록](../assets/issues7727_7732_20261114/canvaskit-common-3f7e4340/studio-direct-reading.json).

지원 실제 CanvasKit 권고안4쪽은53.71%→98.26%다. Native/Canvas의 공통 radial_circle을
paint JSON으로 전달해 소비하게 했다. 동점 face 보존만 바꾼 분리 진단은 PNG가 동일했으므로
그 face 변경을 이 표본의 점수 상승 원인으로 설명하지 않았다. 기존128쪽·작업 예산·font preflight를
보존했고 실제 Canvas2D fallback은 CanvasKit PASS로 세지 않았다.
일부 글리프 가장자리·테두리 농도·미세 간격 차이를100%로 주장하지 않는다.

## 코드 CI와 남은 병합 조건

정확한 code head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/43649004508),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43649004545),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43649004458),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/43649004492),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/43649004502)가 모두success다.
Archive A–E 합계13,380PASS·0FAIL·55skip과 Native Skia의 실제 선택 검사도 통과했다.
최종 CI Impact Policy까지 성공하고 MERGEABLE/CLEAN인 시점을 확인했다.
[정본 CI 원장](../assets/issues7727_7732_20261114/code-ci-e9b3114b.json)과
[실제 테스트 요약](../assets/issues7727_7732_20261114/code-ci-test-summaries.json)을 보존한다.

위 code head CI 성공 뒤 이 리뷰·오늘할일·정본 CI 요약만 문서-only trailing으로 반영한다.
trailing head의 실제 required checks/fast-pass와 최신 head/CLEAN을 다시 확인한 뒤 승인 범위에서
병합한다. CI workflow 변경이 없으므로 옵션2의 별도 기록 PR 대상이 아니다.

## 병합 후 댓글·이슈·정리 계획

병합 commit이 devel에 실제 포함된 뒤 #7727·#7729·#7730·#7731·#7732 각각에 해당 원본·독립 Print,
전쪽 두 TSV, 실제 CI와 같은 쪽 raw/PDF/review/overlay를 이미지 Markdown으로 게시하고 `completed`로 닫는다.
이미지 응답의 형식·SHA를 commit blob와 대조하고 게시 후 API를 재조회한다.
#7445 표의 이 다섯 행만 취소선·PR 링크로 갱신한다. 다른 미완료 하위 범위와 대표 이슈는 유지한다.

병합 후 Duration 갱신 성공 또는 증거 부족 보류를 확인하고 devel을 동기화한다. CI를 새로 재실행하지
않는다. 사용자 Vite7700·원본/PDF·공유 target·46개 stash와 다른 작업은 보존하며 이번 소유 임시
branch/worktree·루트 output 전체·target/pr-review/debug를 실제 사용 종료 후 정리한다.
