# Task #7949·#7950 — 최신 devel 통합 검증

- 공동 PR: [#7953](https://github.com/edwardkim/rhwp/pull/7953)
- 날짜: 2026-10-17 (Asia/Seoul)
- 사용자 지시: 다른 PR 병합 뒤 devel 동기화 → 현재 작업에 병합 → 후속 검증·self-review.

## 통합과 시각 증거 재사용

기본 작업공간의 devel을 `fe5725dc384689b45d60ef8ea4c83138bc2701d8`로 fast-forward했다.
이를 원격 녹색 head `7f7c77c332cdbbec107d86e8af56dce425acbacc`에 충돌 없이 병합한
통합 code head는 `722a13d23f05455ada976ebecf014f70df67599f`다. #7954의 rhwp-q
조회 도구 통합, 명시적 bin 목록과 `[profile.dev] debug=1`이 들어왔다.

양쪽 비교 `7f7c77c33 → 722a13d23`, `552514f423 → fe5725dc3`에서 runtime 경로 변경은
`src/bin/`뿐이다. PDF·renderer·WASM source와 features/dependencies/target/lib,
release/release-test profile은 각각 동일하다. 개발 profile의 debug 정보 변경과 조회 CLI
통합은 아래 새 통합 검증으로 확인했다. `runtime-invariance.json`에 실제 경로·table 비교를 남겼다.

[stage7](task_m100_7949_stage7.md)의 제품 source `337cde286`에서 만든 Native/fresh Docker
WASM 시각 증거 41쪽씩을 재사용한다. **통합 SHA에서 다시 빌드·캡처했다고 표시하지 않는다.**
Native SVG raster 비교와 별도 Native Skia PNG 검사, WASM Print Canvas와 Studio Canvas2D의
구분도 유지한다. 같은 WASM을 다시 만들거나 같은 전쪽 출력을 반복 생성하지 않았다.
독립 기준은 입력 PDF의 Poppler 출력이며, 사용자 두 PDF와 공개 대조군 모두 쪽수·누락·90% 미달이 없다.

## 통합본에서 실행한 검증

- 공유 target `/home/edward/mygithub/rhwp/target/pr-review`, Cargo jobs 14, nextest threads 8.
- manifest prepare, fmt check, Native Clippy, WASM32 Clippy, workspace build,
  workspace all-target Clippy, manifest·unit-tier 정책 검사: 모두 통과.
- `cargo nextest run --locked --cargo-profile release-test --tests --features native-skia
  --test-threads 8 --no-fail-fast`: **10,685 PASS, 실패 0, 기존 45 skip**.
  실제 검사 344.544초, 컴파일 포함 명령 730.97초다. 소스·baseline/golden·허용치를 바꾸지 않았다.
- 이 실행에 새 rhwp-q dispatch 계약 5건과 PDF paint 계약 4건(Native Skia 실제 PNG 포함)이
  모두 들어갔고 통과했다. 별도 같은 전체 검사를 다시 실행하지 않는다.
- 실제 debug 실행 파일의 `rhwp-q --help`, `rhwp-q page-images --help`,
  `rhwp info explicit-mask.pdf --json`도 통과했다. 기본 feature의 PDF 열기와 조회 도구 공존을 확인했다.
- `fe5725dc3`와 통합 head의 merge-tree 계산은 종료 코드 0,
  공백 검사·변경 문서 상대 파일 링크 95개·기존 20261017 오늘 기록 보존도 통과했다.
- 원격 push 뒤 실제 PR head와 한국어 본문을 API로 재조회했고, 본문 이미지 13개도
  head SHA의 HTTP 응답과 로컬 asset 해시가 같음을 확인했다.

## 최신 기준과의 전쪽 비교

기준 캡처의 source는 `552514f423`이고, 위 동일성 근거로 `fe5725dc3`의 동일한 PDF 실행 경로에
재사용한다. candidate 시각 producer는 stage7의 `337cde286`이다. baseline Native와 Docker WASM은
실제 빌드해 아래 공통 입력 18쪽씩을 실행했다. 첫 sweep의 90% 미달 종료 코드는 baseline 결함이며,
생성된 원본 TSV를 Decimal로 비교했다. **0.01%p 이상 하락은 양쪽 모두 0쪽**이다.

| 입력 / 쪽 | Native 기준 → 보정 | WASM 기준 → 보정 |
| --- | --- | --- |
| control / 1 | 92.29407 → 92.29407% | 92.33023 → 92.33023% |
| control / 2 | 97.90261 → 97.90261% | 97.80694 → 97.80694% |
| control / 3 | 96.84817 → 96.84817% | 96.82916 → 96.82916% |
| control / 4 | 97.91511 → 97.91511% | 97.91906 → 97.91906% |
| control / 5 | 98.41593 → 98.41593% | 98.34511 → 98.34511% |
| control / 6 | 98.75536 → 98.75536% | 98.67174 → 98.67174% |
| control / 7 | 93.54561 → 98.90948% | 93.52793 → 98.89924% |
| control / 8 | 92.67498 → 98.98216% | 92.69398 → 98.84977% |
| control / 9 | 98.67029 → 98.67029% | 98.52512 → 98.52512% |
| control / 10 | 99.07268 → 99.07268% | 98.83573 → 98.83573% |
| control / 11 | 90.14609 → 90.14609% | 90.16832 → 90.16832% |
| control / 12 | 93.36064 → 93.36064% | 93.31080 → 93.31080% |
| paint-state / 1 | 2.50059 → 99.99778% | 2.40631 → 100.00000% |
| compound-winding / 1 | 0.00000 → 100.00000% | 0.00000 → 100.00000% |
| path-image-order / 1 | 14.78164 → 100.00000% | 14.47527 → 100.00000% |
| contiguous-table / 1 | 20.58937 → 99.94191% | 20.53872 → 99.89551% |
| interleaved-table / 1 | 20.74684 → 99.94147% | 20.69571 → 99.89471% |
| plain-table / 1 | 93.08176 → 98.68496% | 92.17861 → 97.75313% |

나머지 5개 입력·23쪽(type002 19쪽, MOF 1쪽, 공개 Mask/고해상도 SMask/Matte 각 1쪽)은
기준 Native와 WASM에서 모두 `PDF_IMAGE_UNSUPPORTED`였다. 이번 신규 지원 범위이며,
기준 TSV나 점수를 만들었다고 주장하지 않는다. 보정본 41쪽씩의 전체 TSV와 독립 출력 대조,
사용자 판정 및 확대 판독은 stage7과 [공개 증거](../pr/assets/issue7949_7950_20261017/README.md)를 따른다.
최저 일치율 Native 90.14609%, WASM 90.16832%다. 큰 채움·흰 글자·투명 영역·표 내용 가림은
보정됐고, 글꼴·미세 간격과 전체 paint 지원의 남은 범위는 [백로그](../plans/task_m100_7949_backlog.md)에 유지한다.
`parserAllocationBoundsProven=false`도 유지한다.

## 증적과 후속 조건

검증 원본은 ignored `output/pr-review/issue7949/`에 두었다. 종료 후 영구 보존 위치는
`/mnt/e/rhwp-local-archives/issue7949-pr7953-20261017/`이며 `output/` 하위 경로를 그대로 보존한다.
`verified-output.json`은 원본·사본의 SHA-256과 symlink 대상을 확인한 목록이다.
초기 3,987개 항목·5,876,244,618바이트가 일치했고 이후 추가 기록은 종료 전에 다시 확인한다.
개인 PDF·글꼴·TSV·로그·JSON·생성 suite·.env는 Git에 추가하지 않는다.

code head `722a13d23`의 GitHub Full CI가 성공했다.
[CI](https://github.com/edwardkim/rhwp/actions/runs/38978215700),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38978214975),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38978214962),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/38978214866),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/38978215110)와 GHAS CodeQL,
CI Impact Policy status의 exact-head 성공을 모두 확인했다. CI preflight의 실제 결과는
`fast_pass=false`, `reason=ci-execution-surface-change`였고 archive 4개와 Native Skia를 실행했다.
Native Skia 로그에서 PDF paint 4건의 실제 통과도 확인했다.

이 성공을 확인한 뒤 [self-review](../pr/archives/pr_7953_review.md)와 오늘할일을 작성했다.
후행 문서는 단일 parent 문서 commit으로 반영하고 새 head의 실제 CI/재사용 판정을 확인한 후
승인된 정상 merge·관련 이슈 종료·devel 동기화·이번 작업의 worktree/output 정리로 진행한다.
이 기록 작성 시점에는 후행 head CI·merge·정리를 완료하지 않았다.
