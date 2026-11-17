# PR #9147 리뷰 — Native 실제 대체 글꼴 진단과 말미 공백 보완

## 최종 판정

**승인 — #9104의 잔여 진단과 공통 말미 공백 보완의 필수 검증을 완료했습니다.**
작성자 `edwardkim`의 self-review입니다. 별도 reviewer는 지정하지 않았습니다.
2026-11-17 13:14:43 KST 기준 code head `6f99912b5013b3a82da9869f536e1488859d4f83`의
Full CI·CodeQL·Render Diff·Adapter·Proptest·최종 CI Impact Policy가 모두 성공했습니다.
이 기록을 더한 문서 전용 trailing head의 검사와 최신 병합 시뮬레이션은 별도로 확인한 뒤
정상 merge commit으로 병합합니다. 제품·테스트를 추가로 변경하지 않았습니다.

## 접수 정보

| 항목 | 확인한 값 |
| --- | --- |
| PR·작성자·base | [#9147](https://github.com/edwardkim/rhwp/pull/9147) / `edwardkim` / `devel` / Open |
| code head | `6f99912b5013b3a82da9869f536e1488859d4f83` |
| 검증 기준 devel | `4f44355f5ff234abd1289d340cb4a3fe13297982` |
| 제품 producer / 최종 Rust 검사 head | `1da7afbd64bbc59d9338a187ebcd21860ddc0842` / `13bb8191967f7e2efc9c182deaf0e7e075576205` |
| 관련 이슈·선행 수용 | [#9104](https://github.com/edwardkim/rhwp/issues/9104), [#9143](https://github.com/edwardkim/rhwp/pull/9143) |
| 규모 | code head 256파일, +56,656 / -56줄. 대부분 전쪽 TSV·재현 조건·검증 증거이며 제품 변경은 349줄 추가·10줄 삭제입니다. 대형 diff의 코드·증거를 구분해 검토했습니다. |
| 작성 시점 상태 | code head CI 성공·충돌 없음. trailing 검사는 미실행 상태이므로 이 결과로 대신하지 않습니다. |

## 변경과 선행 분석

다른 collaborator의 #9143에 #9105·#9127이 수용돼 가져오기·저장과 설치 CJK fallback이
이미 반영됐음을 확인했습니다. #9103은 CLOSED이며 두 원 PR도 CLOSED입니다. 이들을 다시
수용하거나 fork branch를 갱신하지 않았습니다. 남은 #9104의 실제 대체 기록과 기존 로컬
말미 공백 보완만 현재 devel 위에서 검증했습니다.

`for_text`가 실제 고른 후보와 최종 Font의 family를 관측해 `export-png --json`의
`nativeFontSupply.localFallbacks`에 담습니다. 요청/해결 family·출처·요청 스타일을 기록하고
본문 문자·codepoint는 보관하지 않습니다. 공급 mappings·FontEnvironment·선택 순서를
바꾸지 않으며 관측은 최대 256개·이름 64 KiB입니다. `complete`는 관측 범위의 완전성이고
문서 전체 잉크의 동일성 보증이 아닙니다. metadata-only는 `notRendered`, 상한·부분 실패는
`partial`입니다. embedded SVG/메타파일 내부의 별도 글꼴 선택은 관측 범위에 포함하지 않습니다.

공통 `VisibleTextLineMeasure`는 같은 전체 글자모양에서 이어진 장식 SP와 독립 입력란을
구분합니다. fit 결과의 `TextRun.layout_positions`를 공통 Paint·Native·Canvas·SVG가
소비하며 본문 원점·원문·밑줄을 보존합니다. 문서 ID·쪽수별 분기나 Paint trim을 추가하지 않았습니다.

## 검증 입력과 실제 결과

[결과 보고](../../report/task_m100_9104_report.md),
[최종 로컬 증거 원장](../../working/assets/issue9104/final/README.md),
[code CI 영수증](../assets/pr9147/code-ci.json)을 연결합니다.

| 검사 | 실제 결과와 출처 |
| --- | --- |
| 최소 공개 글꼴 로컬 전체 회귀 | 13,710 PASS·0 FAIL·45 skip. [최종 실행](../../working/assets/issue9104/final/final-gates.json) |
| Native 공식 3종 / 진단 집중 | 4,073 PASS / 12 PASS. [범위 동등성](../../working/assets/issue9104/final/native-scope-equivalence.json), [진단](../../working/assets/issue9104/final/native-observation-focused.json) |
| lint·build·정책 | fmt·Native/WASM32/workspace Clippy·workspace build·suite/unit 정책 모두 PASS |
| GitHub Full CI | [44011682101](https://github.com/edwardkim/rhwp/actions/runs/44011682101) 성공. A–E 실제 worker 로그는 13,516 PASS·0 FAIL·45 skip이며 로컬 수와 구분했습니다. Native·Frontend·lint·Build & Test 모두 성공 |
| CodeQL | [44011682064](https://github.com/edwardkim/rhwp/actions/runs/44011682064), Rust·JavaScript·Python·Actions와 GHAS check 성공 |
| Render Diff | [44011682098](https://github.com/edwardkim/rhwp/actions/runs/44011682098), Canvas·PDF 비교와 선택 CanvasKit gate 성공 |
| Adapter / Proptest | [44011682001](https://github.com/edwardkim/rhwp/actions/runs/44011682001) / [44011682062](https://github.com/edwardkim/rhwp/actions/runs/44011682062) 성공 |
| 최종 정책 감사 | [44015649036](https://github.com/edwardkim/rhwp/actions/runs/44015649036), exact code head의 `CI Impact Policy: success` |
| Native/fresh WASM 전쪽 보호 | 각 1,771쪽·합계 3,542행. raw 최저 98.86043% / 98.86472%, 하락·누락 0. [전쪽 TSV](../../working/assets/issue9104/final/visual-validation.tsv) |
| 정확한 base와 실제 비교 | PNG 3,536쌍 동일·변경 6쌍 직접 판독, 하락 0. [전체 비교](../../working/assets/issue9104/final/devel-whole-comparison.json) |
| 새 base 기능 보존 | #9022·#9105·#9107·#9109·#9127·#9137·#9141, 66종·74쪽씩 전쪽 PNG 동일·동작 38 PASS. [기능별 표](../../working/assets/issue9104/final/incoming-feature-preservation.json) |
| 편람·하한 연결 | 실제 Native HWPX 383쪽 `--handbook-gate` PASS, 갱신 consumer compare PASS·Python 267 PASS |

공개 진단 입력은 [offline-font-families.hwpx](../../working/assets/issue9104/inputs/offline-font-families.hwpx),
SHA-256 `ad4a3660999e616731cd55d0135c6d06119eda8c8041c9e7a3e9f38713eab044`입니다.
검토 commit에 포함된 Git blob과 실행 파일의 동일성 및 보안 검사 3종을 확인했습니다.
보호 문서·독립 Print PDF의 기존 정식 경로·SHA·producer·글꼴 조건·전쪽 대응도
[전쪽 manifest](../../working/assets/issue9104/final/visual-validation.json)로 확인했습니다.
대용량 PDF의 로컬 보관과 기존 커밋 자원을 구분하며 새 글꼴 프로그램·WASM·내장 글꼴 SVG는 배포하지 않습니다.
공개 API로 만든 진단 문서를 독립 한컴 Print 좌표 정본으로 사용하지 않았습니다.

## 조판 원칙과 주장·증거 대조

| 검토 항목 | 판정 | 확인한 근거 |
| --- | --- | --- |
| 제출 전 로컬 검증과 Open PR | 충족 | 해당 source의 필수 게이트를 먼저 완료한 뒤 Open PR을 생성했습니다. |
| 기존 이슈·해결 PR 선행 분석 | 충족 | #9143의 실제 통합·현재 base와 원 7건의 기능 보존을 각각 확인했습니다. |
| 구현 근거·일반성 | 충족 | 같은 글자모양의 말미 SP와 독립 입력란을 구분합니다. 독립 Print·정상 대조군과 공통 계약을 확인했습니다. |
| 측정·배치 일관성 | 충족 | `VisibleTextLineMeasure` → fit → 공통 `layout_positions` → 실제 Paint/backend 소비를 추적했습니다. [호출 경로](../../working/assets/issue9104/final/implementation-review.md) |
| 새 base 기능 보존 | 충족 | 정확한 4f44355/후보의 66종·74쪽씩 PNG와 동작 38건을 대조했습니다. |
| 최종 출력 소비 | 충족 | 실제 Native PNG·fresh WASM 브라우저·SVG 5건 및 관측 on/off 픽셀 동일성을 확인했습니다. |
| 분할·이어받기 계약 | 비해당 | 컷·소유 유닛·행 예약 높이·표 이월·페이지 높이를 변경하지 않습니다. |
| 줄 소속·점유 높이 | 충족 | 저장 LineSeg와 소속을 유지하고 SP의 가로 전진만 소비합니다. 독립 스타일·공백 전용 줄·탭/개체·격자 대조군을 보존했습니다. |
| 사례·증거 독립성 | 충족 | 실제 네트워크 차단·빈 cache에서 다운로드 실패와 대체 8건을 관측했습니다. 같은 PNG의 base는 진단이 없으며, 본문 관계 음성 대조는 base 1 FAIL → 후보 PASS입니다. |
| 기준값 변경 | 충족 | 동일한 전쪽 producer의 중첩 표 10쪽 두 행만 99.86→100.00으로 상향했습니다. 다른 3,540행·34개 dataset 출처·허용치는 그대로입니다. |
| 회귀 추가·기존 검사 점검 | 충족 | Native/fresh WASM 1,566행 선행 측정 뒤 관계 검사 4건을 등록했습니다. 대상 참조 57개에서 기존 관계·독립 근거가 있는 완료 페이지 핀을 유지했고 제거·이관은 0건입니다. |
| 확대 판독 | 충족 | 중첩 표 10쪽·양 형식 편람 60/63쪽의 본문·수식·입력란·밑줄·뒤 내용을 같은 배율에서 직접 대조했습니다. |
| 관측 주장·보안 경계 | 충족 | 실제 선택을 다시 계산하지 않습니다. 요청 family는 불신 데이터이며 provenance 검사로 확인했습니다. 상한·캐시·미지원·일반/굵게/기울임·metadata-only를 실행했습니다. |

## 시각 증적과 남은 차이

[직접 판독](../../working/assets/issue9104/final/direct-review.json)의 다섯 영역과
[중첩 표 10쪽](../../working/assets/issue9104/final/nested10-direct-review.png),
[편람 60쪽 변경 본문](../../working/assets/issue9104/final/handbook60-changed-body-detail.png)을 확인했습니다.
중첩 표 10쪽과 양 형식 편람 60/63쪽은 두 backend raw가 각각 100.00%입니다.
본문 압축이 해소되고 원문·장식·입력란·뒤 내용이 보존됐습니다.

회색 서식 영역의 기존 Canvas 색 차이는 같은 devel의 해당 영역과 픽셀이 같습니다.
이 색 차이의 수정이나 전체 색상·픽셀 일치를 주장하지 않습니다. 실루엣 100%와 직접 판독을
구분했으며 정본 하한 완화·글꼴 예외·새 회귀 수용 예외는 적용하지 않았습니다.

## 병합과 후속 처리 계획

사용자가 CI 성공·self-review·머지·후속 처리를 승인했습니다. 이 기록 head의 실제 fast-pass와
required check 성공·최신 head/base 병합 시뮬레이션을 확인한 뒤 exact-head 정상 merge합니다.
CI 실행 정의·재사용 정책은 변경하지 않았으며 두 visual consumer는 정본 표 해시 연결만 바뀌었습니다.
따라서 문서 전용 trailing 경로를 적용합니다.

병합 후 duration 갱신 결과 또는 증거 부족 보류를 확인하고, 검증 CI를 새로 실행하지 않습니다.
최종 devel을 동기화한 뒤 #9104에 실제 merge·CI·관측 범위와 재현 결과를 게시해 닫습니다.
원 #9105·#9127·#9103에 중복 수용 조치는 하지 않습니다.

PR 결과 댓글에는 merge SHA·CI URL·[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment),
merge SHA로 고정한 위 중첩 표/편람 비교 PNG와 실제 raw 수치·기존 색 차이를 연결합니다.
게시 전 raw PNG의 해시를 커밋 원본과 대조하고 게시 후 UTF-8·BOM·본문·이미지를 다시 확인합니다.
실제 사용 종료 후 이번 task branch/worktree·루트 output 전체·공유 debug를 정리하며
사용자 글꼴·기존 WIP/worktree·Studio/CDP는 보존합니다.
