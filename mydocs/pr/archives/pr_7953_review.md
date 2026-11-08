# PR #7953 리뷰 — 기본 PDF 가져오기와 이미지 마스크·채움 보존

## 최종 판정

**승인 — #7949·#7950의 기본 feature·마스크·채움 보존 범위 구현과 통합 검증을 완료했다.**
사용자의 self-review·merge 지시를 적용한다. 검증 code head의 Full CI는 성공했고,
이 문서와 오늘할일의 single-parent trailing head에 대한 실제 CI·mergeability는 마지막 조건이다.
아직 수행하지 않은 trailing CI·merge·정리를 완료로 기록하지 않는다.

## 접수와 범위

| 항목 | 값 |
| --- | --- |
| PR / 작성자 / base | [#7953](https://github.com/edwardkim/rhwp/pull/7953) / edwardkim / devel |
| 관련 이슈 | [#7949](https://github.com/edwardkim/rhwp/issues/7949), [#7950](https://github.com/edwardkim/rhwp/issues/7950), 병합 뒤 종료 |
| 검증 code head | `722a13d23f05455ada976ebecf014f70df67599f` |
| 통합 base | `fe5725dc384689b45d60ef8ea4c83138bc2701d8` (#7954 병합) |
| 선행 녹색 head | `7f7c77c332cdbbec107d86e8af56dce425acbacc` |
| 실제 시각 producer source | `337cde2865bf6c926fdcd53018d420fb19207b24`, 변경 없는 실행 경로의 증거 재사용 |
| 경로 | collaborator 본인 PR self-review. 외부 첫 기여자 절차는 비해당 |

기본 Native/Docker WASM의 pdf-import, explicit Mask·SMask·Matte, page operator/codec scratch
재사용, 경로 채움·clip·경로/그림 순서와 표 추론의 내용 가림을 검토했다. feature-off의 오류·
capability 계약은 유지한다. 19쪽 입력의 잔여 출력까지 보정하라는 추가 승인과, 최신 devel 병합
지시를 반영했다. #7954의 CLI 변경에 PDF 코드 보정을 섞지 않았다.

## 구현 주장과 실제 검증

| 주장과 적용 경로 | 독립 계약·검사 | 판정 |
| --- | --- | --- |
| 기본 feature → Native/Worker PDF 감지·열기 | 기본 Docker WASM capability·실제 PDF·HWP, 명시적 off 대조 | 충족. off API는 선행 off bundle 재사용이며 새 시각 출력 아님 |
| 이미지 Mask/SMask → decoded sample·alpha·Matte → 저장 PNG → paint | PDF Reference의 Decode·행 패딩·SMask 우선순위와 색 복원, 공개 3개 입력·양쪽 전쪽 sweep | 충족. 색상 키·stencil·graphics-state SMask는 백로그 |
| 요청의 immutable page operation 및 종료된 codec scratch 재사용 | 실제 반복 호출·요청 격리·변하는 픽셀, decoded 버퍼 선계상·sticky 한도 오류 | 충족한 범위만 수용. codec 내부 전체 할당 증명은 미완료 |
| graphics state/source_order → path paint·이미지·lowering → 실제 글자·그림 표시 | 독립 PDF의 채움/구멍/clip·alpha와 순서, 공개 paint 경계·저장 후 검사 | 충족. 모든 text/path 혼합·blend 지원은 주장하지 않음 |
| table_paint_layer → 표 승격 가능 여부 → flow/positional paint | 표 글자가 가려지는 수정 전 실제 PNG FAIL → 보정 후 PASS, 일반 표 승격 정상 대조군 | 충족. 셀 포함·가시 잉크·채움과 순서로 검증 |
| Native Skia workflow 명령 → 영향 분류 목록 → 해당 검사 실행 | 기존 classifier Python 계약의 전 FAIL/후 PASS, Node 224·Python 120, exact-head CI의 PNG 검사 4건 실제 PASS | 충족. 임계값·baseline 완화 없음 |

[stage7](../../working/task_m100_7949_stage7.md)에 위반 규칙·실제 소비 지점·독립 기대값·
반례와 수정 전후를 연결했다. reviewer는 실제 Mask/paint/table·요청 cache 호출 경로를 대조했다.
원본 ID·쪽수·우연한 상수 조합으로 화면을 맞춘 예외나 사후 좌표 clamp를 추가하지 않았다.
레이아웃 엔진의 줄 분할·pagination·rowspan·continuation 알고리즘 변경은 없으므로 해당
엔진의 컷/요구 높이 변경 입증은 비해당이다. PDF lowering의 점유·source 순서·실제 배치는 해당한다.

## 로컬 및 exact-head CI

[최신 통합 보고](../../working/task_m100_7949_stage8.md)에 명령·환경·base/head 비교를 모았다.
통합 head에서 필수 Rust lint 전부·manifest·unit-tier, Native Skia nextest **10,685 PASS / 45 skip**,
실제 rhwp-q와 기본 CLI의 Mask PDF 열기를 통과했다. 새 조회 계약 5건·PDF paint 4건도 이 실행에 포함됐다.
선행 실제 기본 Docker WASM API/Worker 85 assertion·5 Worker, Studio Canvas2D와 SHA 확인은
PDF/runtime·의존성·release profile 동일성 증거로 재사용했다. 통합 head의 fresh 재빌드로 쓰지 않는다.

code head의 다음 실행은 모두 성공했다.

- [Full CI 38978215700](https://github.com/edwardkim/rhwp/actions/runs/38978215700): Lint·Frontend package·Native Skia·archive A/B/C/D·Build & Test.
- [CodeQL 38978214975](https://github.com/edwardkim/rhwp/actions/runs/38978214975): 네 언어 분석과 별도 GHAS CodeQL check.
- [Render Diff 38978214962](https://github.com/edwardkim/rhwp/actions/runs/38978214962).
- [Adapter 38978214866](https://github.com/edwardkim/rhwp/actions/runs/38978214866),
  [Proptest 38978215110](https://github.com/edwardkim/rhwp/actions/runs/38978215110).
- CI Impact Policy는 같은 head/base에 대해 성공했다. 실제 preflight는 fast-pass가 아닌 Full이었다.

후행 commit은 이 review·stage8·오늘할일만 포함한다. source·test·workflow·asset은 추가 변경하지 않는다.
정책 실행면을 바꾼 PR이므로 trusted controller의 정확한 candidate와 `rfp=1`·base·head 증명 및
최종 aggregate를 확인한다. 증명이 불충분하면 실제 Full 결과를 기다리며 검증을 우회하지 않는다.
정상 squash merge에 `--match-head-commit`을 사용하고 `--admin` 예외를 적용하지 않는다.

## 시각 증거와 남은 범위

[공개 입력 9개·입력 SHA·대표 비교/overlay](../assets/issue7949_7950_20261017/README.md)와
stage7을 대조했다. 두 사용자 PDF의 원본은 재배포 권한이 확인되지 않아 로컬에만 보존했다.
크기는 각각 1,199,820·194,530바이트이고 50MiB 이상 자료는 이번 PR에 없다. 전체 TSV 원문은
PR 본문에 41쪽씩 있으며 로그·중간 JSON·원본 TSV·글꼴은 Git에 포함하지 않는다.

후보의 전쪽 Native/WASM은 각각 41쪽, 쪽수·누락·90% 미달이 없고 최저는 90.14609%/90.16832%다.
같은 입력이 열리는 기준 18쪽씩에서 0.01%p 이상 하락은 0쪽이다. 나머지 23쪽은 기준이
Mask 오류로 거절한 신규 지원 범위로 구분했다. 현재 base의 동일 실행 경로 증거와 수치 전부는
stage8에 있다. 미측정 기준 TSV를 측정했다고 표시하지 않았다.

점수와 별개로 공개 Mask의 투명 방향·alpha 단계·Matte, 도형의 색·Bezier 구멍·겹침 순서,
표의 실제 글자 잉크·셀 내부 포함을 같은 배율로 확대 대조했다. 사용자는 19쪽 PDF의 사각형
채움·흰 글자·18쪽 표 배경을 Studio에서 **시각 통과**로 판정했다. 새 수식 조판 변경은 없으며
개별 글꼴/미세 간격 차이와 [후속 백로그](../../plans/task_m100_7949_backlog.md)를 유지한다.
실루엣 수치를 모든 색/글리프의 동일성이나 사람의 판정 정확도로 해석하지 않는다.
`parserAllocationBoundsProven=false`를 유지하며 전체 codec·paint·배포 완료라고 주장하지 않는다.

새 회귀는 공개 독립 PDF의 양쪽 sweep과 직접 판독 뒤 추가했다. 기존 baseline/golden·
래칫 기대값을 완화하거나 기존 실패 검사를 이관하지 않았다. 완성되지 않은 실물 배치의 전체 SVG
해시·절대 px 핀으로 기대 출력을 고정하지 않았다. Native Skia 가시성 검사는 원본 grid 영역에서
글자 잉크와 채움색을 확인하며 IR 출력 좌표를 정답으로 재사용하지 않는다.

영구 로컬 증거: `/mnt/e/rhwp-local-archives/issue7949-pr7953-20261017/`.
output 사본 3,987개 항목의 해시·symlink 검증 뒤 새 CI·후속 기록도 추가 보존한다.

## Merge 후 PR comment 계획

같은 self PR에 한국어 존댓말로 정상 merge SHA·최종 head와 CI URL·실제 재사용/Full 판정,
로컬 10,685 PASS·41쪽씩의 최저 점수와 기준 비교 범위·사람의 통과 범위를 남긴다.
#7949·#7950 종료 상태와 남은 백로그를 구분한다. GitHub 자동 close가 적용되지 않으면 승인된
해결 범위로 수동 종료하고 중복 없는 후속 기록을 남긴다.

[Visual Sweep 방법](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 연결하고,
merge commit에 존재하는 `table-visible-text-before-after.png`와 Native/WASM compare·overlay를
실제 Markdown 이미지로 게시한다. URL은 `raw.githubusercontent.com/edwardkim/rhwp/<merge SHA>/mydocs/pr/assets/issue7949_7950_20261017/`에 고정한다.
각 이미지는 같은 원본·같은 배율의 검증 자산이고 새 merge 시각 캡처라고 부르지 않는다.
`--body-file` 또는 정확한 UTF-8 JSON 파일을 local gh에 전달한 뒤 API로 본문을 재조회한다.

이 PR에 필요한 archive review·오늘할일·stage 보고는 head에 포함하므로 별도 기록 PR은 만들지 않는다.
merge 뒤 duration 갱신 결과를 확인하고 devel을 동기화한다. 증거 보존·사용 종료·clean 상태와
정확한 branch SHA를 확인한 뒤 이 작업의 임시 remote/local branch·두 worktree 및 output 전체를 정리한다.
기본 Studio·다른 작업·공유 target은 보존한다.
