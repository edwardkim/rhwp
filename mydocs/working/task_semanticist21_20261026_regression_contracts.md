# 실제 글꼴 공급자와 회귀 검사의 독립 계약

## 검증 대상과 결과

런타임 소스는 `11dc162752ebcd30084757a4f9e20d18d0d8189a`다. Native253과
fresh WASM072로 기존 17조건·각 213쪽을 전체 비교했다. 최저 raw 일치율은 각각
92.04708%, 91.53306%이며, 모든 쪽이 90% 이상이고 독립 Print PDF와 쪽수가 일치했다.
고정 base의 같은 입력·PDF와 대조한 페이지별 0.1%p 초과 하락은 양쪽 모두 0쪽이다.
#6133의 base는 쪽수가 달라 raw 비교값을 만들지 않았고, 현재의 2쪽 내용 보존을 별도로
확인했다. 이 결과를 전체 Rust 검증 완료로 기록하지 않는다.

## #5846 기호 선택 조건의 교정

기존 검사는 원본이 요청한 `휴먼명조` 문자열로 실제 U+2024 기호를 찾았다. 독립
Print PDF와 현재 SVG의 실제 공급자는 한컴바탕이며 첫 paint family는
`Haansoft Batang`이다. 요청 face와 실제 공급자를 구분하도록 선택 조건만 교정했다.
기존 좌표·±2px 허용 오차·소유·기호 폭과 다음 글자의 관계 검사는 유지했다.

같은 원본 66쪽의 Native/fresh 최저는 97.15069/97.06823%다. 중요한 65쪽은 양쪽
raw 100%이며 canonical review와 standalone overlay에서 기호·다음 글자·간격을
직접 확대 판독했다. 집중 회귀는 교정 전 98 PASS / 1 FAIL에서 교정 후
99 PASS / 0 FAIL로 종료했다. 이 선택 조건 교정은 렌더링 기대 위치를 바꾸지 않는다.

## 증적

아래 경로의 공통 루트는 ignored `output/pr-review/semanticist21-20261019/`다.

| 항목 | 증적 |
| --- | --- |
| Native 전체 비교 | `affected-current-full-4019-result.json`, `remaining-active-native253-4022-result.json` |
| fresh WASM 전체 비교 | `affected-current-wasm072-4026-result.json`, `remaining-active-wasm072-4027-result.json` |
| base 대비 페이지별 비교 | `native-page-preservation-4025.json`, `wasm-page-preservation-4033.json` |
| 집중 99건 종료 | `verified-boundaries-4032-result.json`, `logs/verified-boundaries-4032.log` |
| Native 65쪽 직접 판독 | `selected-native253-4024/5846-native253/review/review_065.png`, 같은 조건의 `overlay/overlay_065.png` |
| fresh WASM 65쪽 직접 판독 | `selected-wasm072-4030/5846-wasm072/review/review_065.png`, 같은 조건의 `overlay/overlay_065.png` |

## 당시 남은 검토와 삭제 유지

#8045의 수동 LineSeg 재현본과 한컴 저장본은 같은 입력으로 취급하지 않는다. 같은
재현본의 한컴 저장본에서 드러난 표 높이 차이와 #8093의 혼합 텍스트·양식 개체 기준선
차이는 공통 측정·배치 경로에서 분석 중이다. 기존 재현본의 raw가 90%를 넘더라도
직접 판독에서 확인한 차이가 남아 있으므로 완료로 판정하지 않는다.

sample16·편람의 기존 삭제를 유지한다. 이전 파생 참조를 되살리거나 제외 입력을
재생성하지 않는다. `retired-reference-readonly-4038.json`은 manifest를 저장하지 않고
현재 1,767개 수집 소스와 전용 검증 worktree의 실제 1,767개 참조를 대조했다.
누락·제외 대상 참조는 0개이며 주 작업공간의 generated 파일은 변경하지 않았다.

## 후속 보정과 최종 Rust 검증 — 2026-10-26

위 #8045·#8093의 당시 미완료 상태는 공통 보정과 독립 Print 대조로 해소했다.
[Form·정상 한컴 중첩 표 증거](../pr/assets/inline_wrap_qualified_20261026/README.md)의
세 조건은 전쪽 Native255·최적화 fresh WASM073 모두 raw 100%다. 수동 LineSeg 입력과
정상 한컴 저장본을 구별하며, 정상 저장본의 통과를 수동 입력의 완전 일치로 바꾸어 쓰지 않는다.
코드·회귀 후보는 `61528013511c87c732524f91c6f224286a055337`, 실제 runtime은
`bbf3aed18dd82bbf344072b77b5ac590e7a71f3a`다. 이후 문서·증거 커밋과 해당 코드·검사
파일이 동일함을 대조했다. 새 회귀의 수정 전 실패·수정 후 통과는 4057·4059 기록에 있다.

| 필수 Rust 검증 | 실제 최종 결과 | ignored output의 로그·영수증 |
| --- | --- | --- |
| 준비·fmt·suite 정책·unit tier 정책·Release build | 각각 exit 0 | `final-full-rust-4052-result.json`의 완료 단계 |
| Release 라이브러리 | 3,983 PASS·0 FAIL·12 ignored | `logs/final-release-lib-4052.log` |
| 전체 nextest | 11,708 PASS·0 FAIL·45 skipped | `logs/final-full-nextest-4052.log`, `full-nextest-terminal-audit-4092.json` |
| Native Skia 라이브러리 | 4,040 PASS·0 FAIL·12 ignored | `logs/final-native-skia-lib-4093.log` |
| Native Skia placeholder·직접 PDF 검사 | 각각 exit 0 | `remaining-rust-4093-result.json` |
| Native·WASM32 Clippy | 각각 exit 0 | `logs/final-native-clippy-4093.log`, `logs/final-wasm-clippy-4093.log` |
| workspace build·all-target Clippy | 각각 exit 0 | `logs/final-workspace-build-4100.log`, `logs/final-workspace-clippy-4100.log` |
| 문서 테스트 | 8 PASS·0 FAIL·3 ignored, exit 0 | `logs/final-doctests-4108.log` |

표의 공통 루트는 `output/pr-review/semanticist21-20261019/`다.
최종 집계 `final-rust-validation-4111.json`은 각 로그의 SHA-256과 실제 종료 결과를 연결한다.
전체 nextest의 바깥 driver는 SIGTERM으로 끝났고 child 종료 코드는 수집되지 않았다.
따라서 driver를 성공으로 표시하지 않고, 별도 감사로 확인한 명시적 terminal summary와
연속된 11,708개 PASS를 전체 nextest의 완료 근거로 쓴다. 이후 미실행 단계는 따로 완료했다.

workspace query cache 쓰기는 ENOSPC로, 첫 문서 테스트의 다섯 항목은 같은 시점의
링커 SIGBUS로 실패했다. 종료된 정확한 증분 캐시와 사용하지 않는 output 실행 파일 사본을
정리하고 같은 소스로 재실행했다. 문서 테스트는 임시 실행 파일 링크가 겹치지 않도록
한 개씩 실행했으며 빌드 잡은 8개, 공유 target은 `target/pr-review`로 유지했다.
기대값·golden·검사 항목은 바꾸지 않았다. 실패 기록을 삭제하거나 이전 driver의 상태를
성공으로 덮어쓰지 않는다.

현재 원 PR 재현 입력 62조건의 후보 Native·fresh WASM 캡처 124개는 모두 완료됐다.
base 대조는 디스크 부족으로 중단된 뒤, 출처가 일치하는 완료 캡처는 canonical
`--png-pair`로 다시 측정하고 누락된 캡처만 실제 export해 계속한다. 일반 PNG 모드의
`--resume`을 실루엣 전용 모드에 적용했던 4109는 인자 오류이며 통과 증거에서 제외한다.
이 대조와 대표 경계의 직접 판독이 완료되기 전에는 전체 PR 준비 완료로 판정하지 않는다.

sample16·편람의 삭제된 회귀·fixture·보관본·파생 참조는 생성하지 않았다.
`retired-reference-preservation-4103.json`은 manifest를 쓰지 않고 현재 수집 소스와
주 작업공간·검증 worktree의 generated 참조를 대조했다. 삭제 파일을 가리키는 참조와
수집 소스의 누락은 0개다. OLE 원본의
[현재 Native·WASM 증거](../pr/assets/legacy_chart_physical_layout_20261020/current-20261026/README.md)는
기존 HWP·추적 중인 동일 Print PDF를 재사용했으며 제외 문서의 복원과 무관하다.


## 원 PR별 실제 실행 대조와 삭제 유지 재확인 — 2026-10-26

[기존 재현본 62조건의 최종 증거](../pr/assets/source_pr_qualified_20261026/README.md)를
보존했다. 전쪽 Native·fresh WASM 후보 각각 62쪽의 최저 raw는 99.98107%이고,
독립 Print 쪽수가 같으며 base 대비 0.1%p 초과 하락은 0쪽이다. 대표 23조건의
review·standalone overlay 46쌍은 기존 캡처의 입력/PDF·producer·실제 PNG 해시를
확인한 뒤 합성했다. 원본 RGB가 같은 경우의 판독 재사용과 새 캡처를 구분했다.
원문 TSV 248개·비교표 2개·PNG 92개의 해시를 재확인했다.

원 PR 45개의 현재 검사 함수 196개를 실제 PASS 행과 연결했다. 이는 PR 간 공용
함수의 중복을 포함한 연결 수이며 서로 다른 검사 196개를 새로 만들었다는 뜻이 아니다.
기본 nextest에서 조건부로 빠진 Native font trace 두 함수는 별도 native-skia 모듈
실행의 12 PASS·0 FAIL·exit 0으로 확인했다. Studio 네 소스는 전체 Node 명령에
명시된 실제 검사 파일과 그 파일이 import한 runner를 구분했다.

| 원 PR | 실제 PASS와 연결한 Rust 함수 수 | Studio 검사/runner 소스 수 |
| --- | ---: | ---: |
| [#7559](../pr/archives/pr_7559_review.md) | 2 | 0 |
| [#7572](../pr/archives/pr_7572_review.md) | 3 | 0 |
| [#7573](../pr/archives/pr_7573_review.md) | 3 | 0 |
| [#7588](../pr/archives/pr_7588_review.md) | 11 | 0 |
| [#7589](../pr/archives/pr_7589_review.md) | 2 | 0 |
| [#7662](../pr/archives/pr_7662_review.md) | 6 | 0 |
| [#7694](../pr/archives/pr_7694_review.md) | 19 | 0 |
| [#7819](../pr/archives/pr_7819_review.md) | 14 | 0 |
| [#7833](../pr/archives/pr_7833_review.md) | 38 | 0 |
| [#7904](../pr/archives/pr_7904_review.md) | 0 | 4 |
| [#8014](../pr/archives/pr_8014_review.md) | 1 | 0 |
| [#8025](../pr/archives/pr_8025_review.md) | 1 | 0 |
| [#8027](../pr/archives/pr_8027_review.md) | 4 | 0 |
| [#8029](../pr/archives/pr_8029_review.md) | 1 | 0 |
| [#8030](../pr/archives/pr_8030_review.md) | 1 | 0 |
| [#8031](../pr/archives/pr_8031_review.md) | 2 | 0 |
| [#8032](../pr/archives/pr_8032_review.md) | 2 | 0 |
| [#8033](../pr/archives/pr_8033_review.md) | 2 | 0 |
| [#8035](../pr/archives/pr_8035_review.md) | 3 | 0 |
| [#8036](../pr/archives/pr_8036_review.md) | 2 | 0 |
| [#8037](../pr/archives/pr_8037_review.md) | 2 | 0 |
| [#8038](../pr/archives/pr_8038_review.md) | 2 | 0 |
| [#8039](../pr/archives/pr_8039_review.md) | 1 | 0 |
| [#8041](../pr/archives/pr_8041_review.md) | 7 | 0 |
| [#8042](../pr/archives/pr_8042_review.md) | 11 | 0 |
| [#8043](../pr/archives/pr_8043_review.md) | 9 | 0 |
| [#8045](../pr/archives/pr_8045_review.md) | 1 | 0 |
| [#8049](../pr/archives/pr_8049_review.md) | 2 | 0 |
| [#8050](../pr/archives/pr_8050_review.md) | 1 | 0 |
| [#8051](../pr/archives/pr_8051_review.md) | 1 | 0 |
| [#8052](../pr/archives/pr_8052_review.md) | 2 | 0 |
| [#8053](../pr/archives/pr_8053_review.md) | 1 | 0 |
| [#8054](../pr/archives/pr_8054_review.md) | 1 | 0 |
| [#8055](../pr/archives/pr_8055_review.md) | 1 | 0 |
| [#8056](../pr/archives/pr_8056_review.md) | 1 | 0 |
| [#8057](../pr/archives/pr_8057_review.md) | 3 | 0 |
| [#8058](../pr/archives/pr_8058_review.md) | 4 | 0 |
| [#8061](../pr/archives/pr_8061_review.md) | 1 | 0 |
| [#8062](../pr/archives/pr_8062_review.md) | 1 | 0 |
| [#8065](../pr/archives/pr_8065_review.md) | 9 | 0 |
| [#8066](../pr/archives/pr_8066_review.md) | 1 | 0 |
| [#8090](../pr/archives/pr_8090_review.md) | 1 | 0 |
| [#8093](../pr/archives/pr_8093_review.md) | 4 | 0 |
| [#8094](../pr/archives/pr_8094_review.md) | 11 | 0 |
| [#8096](../pr/archives/pr_8096_review.md) | 1 | 0 |

공통 로그 루트는 ignored `output/pr-review/semanticist21-20261019/`다.
`individual-pr-execution-bindings-4125.json`에는 현재 검사 파일 SHA-256, 모듈·함수의
실제 PASS 행과 로그 SHA-256을 기록했다. `native-trace-contracts-4119-result.json`과
`logs/native-trace-contracts-4119.log`는 Native 조건부 검사 12건의 실제 종료 근거다.
개별 리뷰의 검사 통과는 독립 Print 일치나 PR의 전체 수용 판정을 대신하지 않는다.
아직 게시하지 않은 통합 PR의 CI·merge·후속 처리를 완료했다고 기록하지 않았다.

`retired-reference-preservation-4123.json`은 `deriveManifest`의 persist=false 경로로
현재 수집 소스 1,787개를 읽었다. 누락 0개·삭제 대상 참조 0개이며 현재 generated
harness의 삭제 대상 참조도 0개다. manifest·fixture·보관본·삭제된 파생 파일을
재생성하지 않았다. 텍스트 검색의 과거 사유 주석과 제외 필터는 실제 원본을 여는
활성 참조와 구분해 유지했다. 검사 대상과 실제 실행 코드는 변경하지 않았다.

## 삭제된 파생 참조의 재생성 금지 재확인 — 2026-10-26

사용자 지시에 따라 sample16·편람의 삭제를 유지한다. HEAD
`588130c50b2e5a5a6f14cded810f68198880c988`에서 `deriveManifest`의
`persist=false` 경로로 수집한 소스 1,787개와 현재 generated harness를 읽고,
새 harness 문자열도 메모리 안에서만 대조했다. 누락·삭제 대상 소스·현재 파생 참조·
재생성 예정 참조가 모두 0건이다. `--prepare`나 fixture·보관본 생성은 실행하지 않았다.
결과는 ignored `output/pr-review/semanticist21-20261019/retired-reference-preservation-4148.json`이다.
텍스트 검색의 과거 근거 주석·정상 합성 계약과 실제 파일을 여는 활성 참조는 구분했다.

같은 범위의 [#8061 개별 검토](../pr/archives/pr_8061_review_impl.md#독립-print의-레이어-반례--2026-10-26)에서
불완전한 초기 기하 정보와 레이어 의미의 반례를 분리했다. 같은 재현의 기하 보완본·
단일 속성 대조 4조건은 후보/base × Native/fresh WASM 16회 모두 raw 100%·1쪽이지만,
`flowWithText=0, zOrder=0` 그림은 Print와 달리 쪽 도형 위에 보였다.
같은 불투명 영역의 색상 차이는 실루엣만으로 판정하지 않는다. 해당 PR의 수용 판정은
보류를 유지하고 공통 레이어 소비 조건의 보정 대상으로 기록했다.

`pr8061-controls-panels-4150-result.json`의 compare·standalone overlay·review 8쌍은
현재 source와 sweep·입력/PDF·원 PNG 해시를 확인해 기존 캡처에서 정본 함수로 합성했다.
그중 Native의 첫 조건과 `flowWithText=1` 조건의 review·overlay 4개를 실제로 열어
서로 다른 그림 표시와 같은 셀 본문 순서를 직접 대조했다. 나머지 생성 이미지를 모두
판독했다고 기록하지 않는다. source·검사·지침·Visual Sweep 도구는 바꾸지 않았다.

## 셀 그림의 쪽 순서와 편집 소유 분리 보정 — 2026-10-26

앞 절 이후 #8061의 필수 반례에 따라 공통 renderer를 보정 중이다. 기존 검증 코드
후보 `61528013511c87c732524f91c6f224286a055337`와 렌더 소스
`bbf3aed18dd82bbf344072b77b5ac590e7a71f3a`의 결과를 이 새 소스의 완료 결과로
재사용하지 않는다. source·검사·지침을 바꾸지 않았다는 앞 절은 그 단계 당시의 기록이다.

`layout/anchored_cell_paint.rs`로 기존 그림 wrapper·셀 순서 조립을 옮겼다.
`flow_with_text=0`의 일반 셀 그림은 셀의 편집 주소와 절대 좌표를 보존하면서 쪽
개체들과 정렬한다. `flow_with_text=1`과 이미 증명된 셀 프레임 paint는 셀의 그리기
소유를 유지한다. 쪽 순서를 따르는 한 그림의 bitmap·테두리·캡션은 같은 묶음으로
이동하며 임시 NodeId를 순서 비교에 쓰지 않는다. 원본 주소가 없는 테두리가 bitmap
앞으로 재정렬되지 않도록 원본 그림의 정렬키와 내부 부분 순서를 함께 보존한다.
공개 JSON 계약에 내부 묶음 필드를 추가하지 않는다.

같은 원 재현을 떠 있는 표로 바꾸고 겹침 영역을 명시한 Print 3조건에서도
`flowWithText`와 z-order의 차이가 확인됐다. 별도의 2px 파란 테두리 대조군은 같은
원 재현에서 테두리만 추가했다. 다른 실물 HWP·이관 문서·sample16·편람을 추가하거나
이전 파생 참조를 재생성하지 않았다.

| 단계 | 실제 결과와 남은 검증 |
| --- | --- |
| Native 256 | 초기 공통 분리 보정, release-test 빌드 exit 0·4분 39초·source 해시 고정. 8조건·9쪽의 수정 전후를 확인했다. |
| 집계 오류 복구 | 4158 집계기는 시스템 Pillow API에서 실패했다. 완료된 PNG·producer·입력/PDF 해시를 검증하고 4159의 정본 PNG-pair TSV로 재집계했다. 재렌더하지 않았다. 후보 8조건 raw 100%, 두 레이어 반례는 수정 전 FAIL·수정 후 PASS다. |
| Native 257 | 한 개체 내부의 bitmap·테두리 순서까지 보존한 보완본, release-test 빌드 exit 0·4분 17초. 4163의 9조건·10쪽 Native 대조는 모두 raw 100%·쪽수 일치·색상 존재 진단 통과다. |
| fresh WASM 074 | 같은 공통 소스의 표준 release·실제 wasm-opt 빌드 exit 0·17분 33초. 아래 신규 양쪽 시각 검증과 focused 회귀를 완료했다. |

기존 #8061 합성 검사는 `flowWithText=0` 그림도 무조건 쪽 도형 위에 두어야 한다고
고정하므로 독립 Print와 맞지 않는다. 실제 원본·Print·양쪽 시각 확인을 마친 뒤
새 이름의 관계 검사와 자격을 충족한 페이지의 중요 좌표 검사로 교체한다. 기존의
전체 Rust·Studio PASS는 이전 코드 후보의 결과이며 새 공통 소스에서 필요한 필수
게이트를 다시 완료해야 한다. PR은 아직 제출하지 않았다.

로그·영수증은 ignored `output/pr-review/semanticist21-20261019/` 아래
`pr8061-runtime-overlay-4156.json`, `pr8061-runtime-overlay-4161.json`,
`rebase-native256-build-4157-result.json`, `cell-layer-boundaries-audit-4159-result.json`,
`pr8061-floating-controls-print-4154-result.json`, `pr8061-border-control-print-4160-result.json`,
`rebase-native257-build-4162-result.json`, `cell-layer-boundaries-validation-4163-result.json`,
`rebase-candidate-wasm074-build-4164-result.json`에 구분했다. 임시 source overlay와
Git head를 같은 것으로 보고하지 않으며 실제 파일별 SHA-256을 보존했다.


## 셀 그림 보정의 양쪽 검증과 파생 참조 보존 — 2026-10-26

[정식 증거](../pr/assets/cell_picture_layer_qualified_20261026/README.md)에 같은 #8061
재현의 HWPX 8개·독립 MCP Print PDF 8개·Native/fresh WASM TSV·대표 review/overlay를
등록했다. 8조건은 양쪽 전쪽 raw 100%이며, 정상 RowBreak 2쪽도 양쪽 100%다.
기존 영향 문서 6조건·94쪽을 같은 새 런타임으로 재출력했다. 선행 검증 합계는
15조건·104쪽/backend, 전쪽 90% 이상·독립 PDF 쪽수 일치다.
base 대비 페이지별 비교 208개에서 0.1%p 초과 하락 0쪽을 확인했다.

새 이름의 셀 그림 검사 9개와 RowBreak 검사 1개는 focused nextest 10 PASS·0 FAIL이다.
첫 실행의 8개 실패는 같은 원본 좌표의 약 1e-14px f64 차이였고, 정수 HWPUNIT의
동등·포함 관계로 수정했다. 독립 PDF ±2px 조건은 유지했다. 별도 순서 증거 3조건은
수정 전 FAIL / 수정 후 PASS다. raw만으로 드러나지 않는 잘못된 색상 가림도 실제 PNG로
검사했다. PHOTO의 래스터 경계와 일부 약 1.12px 원점 차이는 숨기지 않고 기록했다.

삭제 이력에서 관련 검사·fixture를 추적하고 `deriveManifest(persist=false)` 및 메모리의
harness로 두 작업공간의 소스 1,788개를 대조했다. 누락·삭제 대상 source·새로 생성될
삭제 대상 참조는 0건이다. 준비한 검증 worktree의 실행 harness도 대상 참조 0건이다.
주 작업 트리의 과거 ignored generated 파일에는 삭제된 `diag_1042_version_check.rs`
참조 1건이 남아 있으나 복원·재생성·실행·stage하지 않는다. 과거 사유 주석·공용 제외
필터·제품 catalog enum은 실제 파일을 여는 활성 검사와 구분한다. sample16·편람의
삭제된 파생 참조를 다시 생성하는 변경은 추가하지 않았다.

ignored 영수증은 `cell-scope-focused-4175-result.json`,
`cell-scope-negative-proof-4176.json`, `retired-reference-preservation-4177.json`,
`cell-scope-finalization-4178.json`, `cell-scope-base-comparison-4179.json`이다.
새 코드의 전체 Rust·lint·Studio 및 최종 제출 범위 검증은 별도로 완료해야 하며,
이전 런타임의 전체 PASS를 재사용하지 않는다. 아직 PR을 제출하지 않았다.


## 사용자 지시에 따른 손상된 이동 HWP 삭제 — 2026-10-26

기존 일반 이동 `before.hwp`·`after.hwp`의 한컴 2020 손상 경고를 확인한 뒤 삭제했다.
두 파일의 SHA 목록과 재열기 경로도 정리했다. HWPX 두 파일 및 차트 HWP/HWPX 네 파일의
모델·저장 계약은 별도 `valid_inline_move_fixture_contract`에 유지하고, 공용 helper를
분리해 기존 원본은 983줄로 줄였다. 새 HWP로 대체하거나 삭제 파일을 재생성하지 않는다.

현재 실행 중인 전체 nextest·Native/Studio 후속 작업은 삭제 전 고정 source의 결과로
구분한다. 종료 뒤 검증 worktree에도 삭제를 반영하고 변경한 검사의 집중 회귀·lint를
완료한다. 새 source에 과거 PASS를 적용하지 않는다. 실제 삭제와 근거는 #7694 개별
구현 기록 및 ignored `damaged-move-removal-4195.json`에 연결한다.

## 손상 표본 삭제 뒤 자동 수집 유지 — 2026-10-26

손상 HWP 두 개를 삭제한 `5e71af95aa` 뒤 일반 HWPX의 기존 배치 결함은
`a1fe7fb50f`와 #8266으로 구분했다. 모델·저장 계약과 정상 차트 계약은 유지한다.
검사 분리 과정에서 추가한 `#[path] mod` 두 개가 자동 수집기의 singleton 예외로
분류되어 integration target 예산이 48에서 50으로 늘어나는 문제를 확인했다.
기존 `tests/cases/`의 공유 helper 포함 방식처럼 명시적인 내부 모듈에서 `include!`하도록
수정했다. target 예산·정책·CI 호출을 완화하지 않으며, 같은 helper를 실제 suite 안에서
컴파일·실행해 보존 여부를 확인한다. `.rs.txt` 보존본은 제거 전 source 바이트 그대로 둔다.

`current-full-nextest-4184.log`는 생산 코드 `a90a2057a7`에서 11,716 PASS·0 FAIL·45 skipped,
전체 Native Skia 3종과 문서 테스트도 exit 0이다. Studio의 TypeScript·전체 Node·fresh WASM
실브라우저·production build·오류 표시 검증은 4186에서 모두 exit 0이다. 이 결과를 삭제 후
변경한 두 검사 source의 실행으로 바꾸어 기록하지 않는다. 변경된 검사와 필수 lint는
`logs/current-*-4205.log`에서 별도로 검증하고 terminal 결과를 추가 기록한다.

## 생산 코드의 최종 실행과 정리 후 검사 — 2026-10-26

| 범위 | 실제 terminal 결과 | ignored 로그 (`output/pr-review/semanticist21-20261019/`) |
| --- | --- | --- |
| `a90a2057a7` 전체 Rust nextest | 11,716 PASS·0 FAIL·45 skipped, exit 0 | `logs/current-full-nextest-4184.log` |
| 같은 생산 코드 Native Skia lib | 3,858 + 15 + 165 + 2 = 4,040 PASS·0 FAIL, exit 0 | `logs/current-native-skia-lib-4184.log` |
| Native placeholder / direct PDF | 2 PASS / 14 PASS, 각각 exit 0 | `logs/current-native-skia-{placeholder,direct-pdf}-4184.log` |
| 문서 테스트 | 8 PASS·0 FAIL·3 ignored, exit 0 | `logs/current-doctests-4184.log` |
| Native font trace | 12 PASS·0 FAIL, exit 0 | `logs/current-native-trace-4191.log` |
| Studio TypeScript·Node·실브라우저·production build | Node 2,597 PASS·0 FAIL, 5단계 모두 exit 0 | `logs/current-studio-*-4186.log` |
| `116735e8df` 삭제·분리 뒤 이동 계약 | 19 PASS·0 FAIL, exit 0·source 불변 | `logs/move-cleanup-focused-4207.log` |
| 같은 정리 뒤 전체 필수 lint | fmt·Native/WASM/workspace Clippy·workspace build·정책 모두 exit 0 | `logs/current-*-4205.log` |

전체 실행 뒤 바뀐 것은 손상 표본 삭제와 이동 계약 검사 정리다. 생산 Rust 코드는
`a90a2057a7`과 동일함을 확인했고, 변경된 검사 19개는 새 source로 다시 실행했다.
전체 회귀 결과와 새 focused 결과를 서로 다른 source 경계로 기록한다.

[최종 전쪽 TSV와 대표 review·overlay](../pr/assets/semanticist21_final_20261026/README.md)에
129조건·각 344쪽을 등록했다. 전쪽 최저는 Native 92.04708%, fresh WASM 91.53306%이며
90% 미만·누락·독립 PDF 쪽수 불일치는 0쪽이다. 수치 대응 684행에서 base 대비 0.1%p
초과 하락 0행이며 #6133의 1→2쪽 복원은 내용·쪽 소속의 구조 개선으로 따로 기록했다.
나머지 108조건의 새 Native 캡처는 이전 직접 판독 대상과 전쪽 픽셀이 같음을 확인했다.
새로 출력한 것을 이전 캡처의 재사용으로 대신하지 않았다. HWPX 차트 네 경로도 별도로
출력했으며 판독한 HWP 차트와 RHWP/PDF 픽셀이 각각 동일함을 확인했다.
Print 증거·입력·원문 TSV를 보존하고 font 목록만 동일 해시별 한 번 저장했다.

## #8240 문서 전용 리베이스와 원 PR별 실행 대조 — 2026-10-26

최신 base `4e690c930105fa09fef484787ed0a2e5e182d444` 위로 리베이스했다.
새 base의 변경은 `THIRD_PARTY.md` 41줄뿐이다. 이전 후보 `53ef4bcfc6`과 새 후보
`d6ea108372`의 src·crates·tests·scripts·Studio·npm·Cargo tree ID가 모두 같고,
replay한 356개 commit은 range-diff에서 전부 같은 patch다. #8228의 조상 관계와
삭제한 표본을 유지했다. 새 base의 suite·unit-tier 정책 검사도 통과했다.
[리베이스 증거](../pr/assets/semanticist21_final_20261026/rebase_equivalence.json)와
[원 PR별 현재 commit 대응](../pr/assets/semanticist21_final_20261026/source_commit_map.json)을
등록했다. 실제 생산 코드는 리베이스 뒤 `e436d8aeefb5e46265f5f15f71ccfb0925742197`이며,
실행 당시 `a90a2057a7`의 코드와 같다. 이를 새로 빌드·실행한 결과로 재표기하지 않았다.

45개 개별 review·impl의 실제 실행 절을 현재 소스에 맞춰 갱신했다.
[실제 실행 대응](../pr/assets/semanticist21_final_20261026/source_execution_map.json)의
206개 함수 대응과 Studio 소스 4개에 누락이 없다. 94개 문서의 내부 Markdown 링크도
검사했다. 이 작업으로 최종 승인 판정을 일괄 변경하지 않았다.

## 원 PR 재현의 잔여 게이트 — 진행 중

129조건·344쪽/backend는 **완료한 통과 자료의 부분집합**이다. 별도의 원 PR 재현
59조건은 과거 Native 미달로 실행이 중단됐던 기록이 있었다. 그중 정상 RowBreak는
현재 같은 입력·Print의 완료 증거를 재사용하고, 초기 #8061의 불완전한 기하 입력은
독립 Print로 검증한 같은 재현 8조건으로 교체한 경계를 유지한다. 나머지 57조건은 기존
원본과 PDF의 바이트를 그대로 사용해 현재 후보/base × Native/fresh WASM을 다시 관측한다.
새 문서 수집·입력 생성·Cargo 빌드는 하지 않는다. `remaining-contracts-current-4213-result.json`과
`logs/remaining-contracts-current-supervisor-4213.log`에 실제 상태·명령·종료를 기록한다.
Native 후보 57회 중 36조건이 통과했고, 20조건은 raw 미달, 1조건은 쪽수 불일치다.
전 경로의 측정과 원인 판정 전에는 통합 PR을 제출하지 않는다.

### 확인한 실제 회귀: #8036

`pr8036-saved-vpos-0.hwpx`는 같은 원본의 독립 Hancom Print와 새 base가 모두 1쪽이며
base Native raw는 100%다. 현재 후보만 2쪽이라 TSV 산출이 쪽수 불일치로 중단된다.
`stored_reset_after_tall_line`의 2쪽 기대값과
`typeset/section/stored_reset.rs`에서 `pv > 5000`을
`pv > 0 && prev_vpos_end > 5000`으로 바꾼 강제 분기부터 재검토한다.
수동으로 만든 저장 줄의 큰 높이만으로 쪽 경계 수용을 넓힌 가정이 독립 Print와 다르다.
그대로 승인하거나 base 미달 문서로 이관할 수 없다. 원본 입력·Print는 보존하고,
공통 경계 판단을 보정한 뒤 두 vpos 대조군의 전쪽 Native/fresh WASM을 다시 확인한다.
잘못된 2쪽 기대를 새 정상 baseline으로 고정하지 않으며 새 이름의 관계 검사로 보호한다.

### 별도로 구분한 기존 중첩 표·문자 전진

#7833의 첫 nested 표 재현은 base raw 18.58277%, 후보 18.37031%다. 직접 Native 재출력의
모든 픽셀 차이는 `(175,367)..(247,387)`의 수달 문자와 후속 글자 구간에 한정된다.
노드에서는 문자 폭 7→18px와 뒤 세 요소의 x 이동만 달랐고, 표의 좌표·높이는 같았다.
같은 Print에서 해당 기호(F000으로 매핑)와 다음 `나`의 원점을 직접 읽은 전진은
18.23011px다. 따라서 raw 하락과 실제 문자 전진의 정확성을 구분해야 한다. 이 사실로
기존 표의 배치 미달을 통과 처리하거나 px 고정을 정당화하지 않는다.

`cell_boundary_object_fixture`는 본문 문단·호스트를 복제해 중첩 셀로 옮기면서 저장
LineSeg를 함께 가져오는 구조다. 수동 이동 뒤 저장 좌표가 유효한지 먼저 확인하고,
정상 저장본의 수용 조건을 완화하는 대신 입력 생성 계약과 실제 renderer 문제를 구분한다.
이 원인 가설은 아직 완료 판정이 아니며, 고정 원본의 대조 결과를 보존한 뒤 검증한다.
21개 Native 실패는 같은 producer로 묶어 노드·PNG 차이를 추가 수집했다.
`failed-tree-comparison-4221-result.json`과 독립 문자 원점의
`pr7833-base-candidate-diagnosis-4219.json`은 ignored output에 있다.
잔여 판정과 필요한 변경을 모은 뒤 한 번에 보정·빌드·재검증한다.

## 손상된 #5715 HWP 보관본 삭제 — 2026-10-26

작업지시자의 “손상된 파일 삭제” 지시에 따라
`mydocs/pr/assets/issue_5715_corrupt_sample_20261025/float_chart_ghost_ladder_gap.hwp`
보관본도 삭제했다. 삭제 전 21,504바이트와 SHA-256
`fe6570871a907d8cff07f9a672286c26ff15a42c665c315ee634a44e39ac8282`를 확인했다.
별도 보관본이나 대체 입력을 만들지 않았다. 기존 `before.hwp`·`after.hwp` 삭제도 유지한다.
이 단계에서는 활성 검사나 제품 코드를 변경하지 않았으며 #5715의 해결을 주장하지 않는다.

검증: Git 추적 대상 코드·검사·스크립트·baseline 4,664개 파일에서 삭제 대상의
활성 참조 0건을 확인했다. 변경한 README의 내부 링크 3개와 `git diff --check`도
통과했다. 로그는 ignored `output/pr-review/semanticist21-20261019/logs/`의
`corrupt-file-deletion-references-4238.log`와 `corrupt-file-deletion-links-4238.log`다.
기본 링크 검사기의 전체 탐색은 파일 I/O 지연으로 중단하고 해당 README의 링크만
한정 검사했다. 코드 변경이 없어 이 삭제 단계에서 Rust 재빌드를 실행하지 않았다.

## 남은 원 PR 재현의 공통 원인 보정 — 2026-10-26

기존 #8032의 현재 public API 산출물과 새 독립 Print를 대조했다. 큰 TAC 그림은
다음 줄에 정상 배치되지만, 앞 텍스트의 자동 줄바꿈을 확정하는 `inline_flow::plan`
이 줄 종료 이유를 보존하지 않아 양쪽 정렬 전진을 발행하지 않았다. 측정의
`InlineFlowPlan.rows/boxes`에서 한 번 확정하고 `layout_inline_flow_plan`이 같은
원점을 소비하도록 수정한다. 명시 개행·문단 끝·좁은 그림 대조군은 별도로 확인한다.
이미지 nearest 보간 진단은 오히려 85%대로 낮아져 적용하지 않았다.

#8036은 동일 원본의 base와 독립 Print가 1쪽인데 후보만 2쪽인 실제 회귀다.
이전 줄 시작이 아니라 큰 개체의 끝만으로 새 쪽을 추정하던 조건을 보정한다.
잘못된 2쪽 기대 검사는 아직 변경하지 않았으며, 새 Native/fresh WASM 시각 검증
선행 조건을 만족한 뒤 새 이름의 관계 검사로 교체한다.

#7833은 수동으로 본문 문단을 셀에 복제하면서 원래 저장 줄의 소유 권한을
무효화하지 않은 생성 문제를 확인했다. 공개 무효화 계약을 적용한 별도 진단
대조군은 8개 중 6개 Native 100%다. 남은 깊은 TAC 표 2개는 공통 폭 1200과
셀 폭 합 41954가 충돌하므로, 대체 표의 1행 격자도 공통 상자에 맞춘 대조군을
검증한다. 원래 입력·Print·미달 결과는 그대로 구분해 보존하고 현재 정식 helper의
통과로 보고하지 않는다. 아직 정식 테스트나 기대값을 변경하지 않았다.

이전 Native257/fresh074 전체 PASS와 양성 129조건은 위 새 코드의 검증으로
재사용하지 않는다. 변경 후 같은 기존 입력 범위에서 새로 확인한다.

## Native258/fresh075 대조와 검사 등록 — 2026-10-26

같은 큰 그림 API 입력은 Native 90.69451%, fresh WASM 89.93941%다. 기준 devel에서는
각각 89.19482%, 88.37924%였다. 양쪽 개선이지만 fresh가 90% 미만이므로 해당 그림
검사는 정식 등록하지 않았다. 좁은 대조군은 Native 100%, fresh 99.95997%다.
큰 표 뒤의 vpos=0/67200 두 입력은 모두 후보·base Native/fresh 각 100%·전체 1쪽이다.
대표 PNG를 두 backend에서 직접 판독하고 이어짐 입력의 같은 페이지 RGBA도 확인했다.
잘못된 기존 2쪽 모듈을 제거하고 새 이름의 관계·완료 페이지 ±2px 검사로 교체한다.
수정 전 FAIL/수정 후 PASS 실행은 `independent-regression-drafts-4257-result.json`,
전쪽 비교는 `current-source-contracts-4254-result.json`과 정식 asset에 연결했다.
그림 관련 초안은 ignored output에만 남긴다.

공개 무효화 계약을 적용한 주석 8조건은 Native 100%다. 공통 상자에 맞춰 셀 격자를
축소하거나 원래 표 template 크기를 유지하는 진단은 깊은 중첩 표 2조건을 해결하지
못했고 정식 helper에 적용하지 않았다. 기본 글꼴 정보만 추가한 #8031/#8039 대조군도
미달이어서 원본 해결이나 정식 검사 변경으로 보고하지 않는다. 원래 입력·Print를 보존한다.

원격 devel에 #8188이 병합돼 새 head는 `2546c8b2ab7041399fb15fe82e0f50c1ade21724`다.
79개 runtime 파일이 달라 코드 동등성으로 대체할 수 없다. 객체 전용 merge 사전 분석에서
충돌 18곳을 확인했다. 현재 WASM 패키지는 리베이스 전 코드로 명확히 구분하며,
관련 소스·테스트와 기능을 보존해 리베이스한 뒤 새 source로 다시 검증한다.
삭제한 sample16·편람의 옛 파생 참조를 재생성하지 않는다.

## #8188 리베이스와 우선 보존 검증

`2546c8b2ab7041399fb15fe82e0f50c1ade21724` 위로 리베이스를 완료했다.
이전 head는 `432416bf08`이며 backup ref와 전체 실행 로그는 ignored 검토 경로에 보존한다.
새 base의 원본 글꼴 프로그램·수식 좌표계·이미지 Print frame·Canvas glyph ink 및
`issue_8160` 검사는 유지했다. 삭제한 옛 sample16·편람 검사와 파생 참조를 재생성하지 않는다.

첫 Native 빌드는 기존 독립 자모 분기의 `CharWidthDecision` 초기화에 새 base의
`measurement_character` 필드가 없어 실패했다. 실제 측정 문자를 선택하지 않는
0 전진폭 분기이므로 다른 비측정 분기와 같은 `None`을 지정한다. 조판 폭은 바꾸지 않는다.
composer의 import 순서는 정식 formatter 결과를 반영한다.

사용자 지시에 따라 다음 우선 검증은 #8188의 동일한 변환본 3개 × 64쪽 × Native/fresh WASM이다.
기준 Print PDF·입력 SHA와 실제 공급 글꼴 프로그램을 원 PR 증적과 대사한다.
결과가 나오기 전에는 이 384쪽의 raw 100% 유지나 통합 PR 준비 완료를 선언하지 않는다.

### #8188의 실제 회귀와 좌표축·AST 연결 보정

우선 재측정 source `68eb7e204dc63b67f0687992bb53c6b30d6a42c1`의 Native는
3문서 전 192쪽 중 184쪽만 raw 100%였다. HWPX는 6·24·25·26·53·56쪽,
HWP2024와 HWP2022는 각각 6쪽이 달랐다. HWPX 최저는 25쪽 49.23069%다.
입력/PDF 및 실제 공급 글꼴 217개 SHA는 #8188의 `rebase-8231` 증적과 같다.
원본 글꼴 프로그램은 다른 호스트의 비공개 로컬 보존본에서 가져와 ignored output에서만
공급했다. 공개 OFL 배포본이나 재배포 허가를 확보한 파일로 취급하지 않는다.

실제 `RHWP_DIAG_FLOW=1`에서 HWPX 문단466의 저장 후보는 502.08px,
`same_frame=false`, 앞 내용과 저장 상대 간격이 인증한 원점은 486.9866667px다.
표의 실제 높이·선언 높이는 둘 다 43.4666667px로 맞는다. 높이 결함이 아니라,
같은 좌표축이 아닌 두 원점에 `max`를 적용한 현재 분기가 약15.09px를 더했다.
문단841·876에도 같은 불일치가 있다. #8188의 분기처럼 축 미일치 시에는
앞 내용으로 인증한 원점을 사용하고, 같은 축의 독립 개체 원점과 실제 내용 하한을
함께 지키는 현재 경로는 보존한다. 문서 ID·쪽 번호·목표 좌표 조건은 추가하지 않는다.

수식은 구형 파서가 명시 중괄호를 새 `Group`으로 보존하지만 #8188의
`legacy::supports`·`compound_frame` 등이 이를 모르는 연결 누락이다.
AST의 그룹 소유는 유지하며 해당 레거시 프레임의 표시 가능성·연산자/분수 구조
조회에서만 `ungrouped()`를 사용한다. 다른 세대의 파싱·그룹 글꼴 범위는 유지한다.

변경 전 같은 보관 라이브러리로 기존 `issue_8160_all_pages.rs`를 실제 실행해
3개 모두 FAIL을 확인했다. 세 원본 6쪽의 분수선 기대105.383992px 대비
101.760000px가 대표 실패다. 기존 독립 기대값과 ±2px는 바꾸지 않는다.
진단·원본 전쪽 TSV·정수 미대응 수·review/overlay 및 red 실행은 ignored 검토
경로의 4269·4272·4274·4275 기록에 있다. 보정 후 새 제품의 전쪽 검증이 필요하다.

보정 전 fresh WASM076 빌드는 완료되지 않았고 실행 세션이 exit143으로 종료됐다.
이를 최적화 완료나 fresh 출력으로 재사용하지 않으며 보정 소스를 새로 빌드한다.

### 문단 끝 글자모양의 같은 줄 높이 소비

두 연결 보정의 Native260에서 기존 #8188 전쪽 내용·소유·중요 좌표 검사3개는
모두 PASS로 바뀌었다. 보정 전0 PASS/3 FAIL과 같은 검사 source·기준값이다.
실제 PNG의 전쪽 raw100 판정은 별도이며 검사 통과로 대신하지 않는다.

추가로 HWPX26쪽의 정수 미대응9/23은 표490의 `주센타와`·`교체활용` 두 문단에만
있다. 원본은 가시 글자800HU, 종단 빈 run1000HU, 저장 줄 높이1000HU/기준선850HU다.
`stored_text_metrics::nominal_height`가 종단 글자모양을 누락해 유효 줄을800HU로
투영했다. 재조판이 이미 소비하는 `paragraph_end_font_size_px`를 이 정규화에도
공유해 원본 UTF-16 종단 주소의 줄 높이를 보존한다. 문서별·문구별 보정은 추가하지 않는다.
같은 원본의 Print와 전체 Native/fresh 재측정으로 완료 여부를 판정한다.

### #8188 전쪽 복원과 누락된 26쪽 회귀 검사 완료

제품 source `c2513250ef976cc8189671a5766cd646d75bc4ed`의 Native261과 최적화한
fresh WASM077에서 같은 3개 원본 × 64쪽을 각각 다시 산출했다. 양쪽 192/192쪽,
총 384쪽 모두 raw 100%이고 양방향 정수 미대응 수는 0이다. 입력과 독립 Print의
쪽수도 각각 64/64다. 이전 base 증적의 100%를 현재 출력으로 대신하지 않았다.
217개 실제 공급 글꼴 프로그램은 기존 증적과 해시가 같으며 글꼴 파일은 커밋하지 않는다.

새 `issue_8160_terminal_style_cell_frame` 검사는 기존 26쪽 검사가 놓친 셀26의
`주센타와`·`교체활용`을 보호한다. 독립 Print 기준선·가로 원점 ±2px와 셀 소유·
서로 다른 문단·위아래 순서·셀 내부 포함·누락/중복 없음을 함께 검사한다.
Native/fresh 양쪽 100%와 26쪽 review·standalone overlay 직접 판독 후 등록했다.
같은 정식 검사 source의 Native260 실행은 exit101, Native261은 exit0이다.
자동 수집 `regression_suite_007`의 실제 nextest는 1 PASS/0 FAIL, 필터 제외291개,
exit0으로 끝났다. 기존 `issue_8160_all_pages`의 독립 기대값은 유지했다.

fresh077의 실제 Canvas 브라우저 계약도 8 PASS/0 FAIL/0 skip이다. 생성된
`rhwp-studio/public/rhwp.js`는 검증한 pkg와 SHA가 같고, #8188의 Print metafile·
frame resampling 관련 export를 포함한다. 이후 검사·문서·증적 커밋은 엔진 Rust를
바꾸지 않으며, 제품 source c2513250에서 실행한 결과임을 명시한다.

최종 전체 TSV 6개와 대표 Native/fresh 26쪽 review·overlay는
[정식 증적](../pr/assets/issue_8160/semanticist21-rebase-20261026/README.md)에 보존한다.
중간 JSON·로그·원시 페이지 PNG와 글꼴 파일은 output에만 유지한다.
로그는 `logs/8188-new-regression-nextest-4290.log`, 수정 전후 실행은
`8188-terminal-formal-red-green-4289-result.json`, 전쪽 판정은
`8188-priority-{native,wasm}-4282-result.json`이다. 모두 같은 검토 output 아래에 있다.
이 완료는 #8188 보존과 새 회귀 검사에 한정하며 통합 PR 전체 검증 완료를 뜻하지 않는다.

### CI 운영 후속 12커밋 반영과 기능 보존 — 2026-10-26

기준 devel이 `2546c8b2ab7041399fb15fe82e0f50c1ade21724`에서
`49bc935a149c20b942d3045c2f5c733aeccf57d0`으로 바뀌어 리베이스했다.
#8273·#8274·#8278·#8280의 duration 갱신 CLI·문서 전용 판정·후속 기록
12커밋이며 새로운 외부 contributor PR은 체리픽하지 않았다.
364개 커밋을 충돌 없이 재적용한 head는 `21af47c759`다. 기존 삭제는 유지한다.

리베이스 전 `3b13d8c65d`와 새 head의 `src`·`crates`·Cargo 설정·build.rs·
Studio source·Visual Sweep 두 구현 파일에 차이가 없다. c2513250에서 빌드한
Native261/fresh077의 제품 검증은 이 소스 동일성으로 연결하고, 새로 빌드했다고
기록하지 않는다. 새 base의 duration 정책·CLI 계약 17개 Node 검사와 workflow
계약 22개 Python 검사를 현재 head에서 실제 실행해 모두 통과했다.

리베이스 로그는 `logs/rebase-49bc-4293.log`, 기능 보존 근거는
`rebase-49bc-4295-result.json`, 정책 로그는 `logs/rebase-duration-policy-4295.log`와
`logs/rebase-duration-workflow-4295.log`다. ignored 검토 output에 보존한다.
기존 #8032·#8036의 같은 입력 4개와 #7833 소유 무효화 대조군 8개는
현재 제품의 양쪽 출력으로 재측정 중이다. 기존 미달의 해소나 통합 PR 준비 완료를
실행 결과보다 먼저 선언하지 않는다. 새 HWP 원본으로 범위를 넓히지 않는다.

### 지정 사양과 현재 재현의 대조 — 진행 중

작업지시자가 지정한 본문 5.0 revision1.3의 표69·70(개체 공통 크기·여백·TAC),
표83~85(그룹 원점과 변환 행렬), 표107(그림 사각형·crop·안쪽 여백)을 읽고
현재 #8032 재현의 저장 속성과 대조했다. 1 HWPUNIT=1/7200인치이며, 문서의
크기·그룹 좌표·그림 원본 좌표를 구분한다. 현재 그림의 세 변환 행렬은 모두
항등 행렬이고 그룹 offset·안/바깥 여백은 모두0이다. 원점 차이를 해소하려고
문서명이나 측정한 px 차이를 하드코딩할 근거는 없다.

같은 넓은 그림 입력의 독립 Print에는 16×16 RGB 이미지와 3200×1280 soft mask가
있다. PDF 이미지 변환은 x=95.947998046875pt, 너비=383.79400634765625pt다.
96dpi CSS px로 변환한 x는 약127.930664px인데 현재 RenderTree의 명목 원점은
113.4px다. PDF의 첫 본문 글자는 x=85.0339126586914pt이므로 그림과 본문 사이의
가로 차이는10.914085387pt다. 이 값이 글자처럼 개체의 줄바꿈·정렬 소비와 관계있는지,
최종 paint의 crop·프레임 차이인지부터 구분한다. 보간만 바꾸는 진단을 해결로
격상하지 않는다. 앞줄 분배 간격의 절반과 비슷하지만 정확히 같지 않으므로 원인으로
확정하거나 다음 줄에 그 값을 더하지 않는다.

수식 revision1.3의1.1.1~1.1.2는 기본 이탤릭·rm/it/bold·SCALE의 상대 크기와
중괄호의 항 묶음을 구분한다. 본문 표105의 수식 글자 크기·baseline과 함께,
#8188 보정에서 Group 소유를 유지하면서 레거시 구조 조회만 ungrouped로 처리한
범위를 대조한다. AST 묶음 제거로 글자모양 범위를 무너뜨리는 수정은 하지 않는다.

차트 revision1.2의 VtChart TwipsWidth/TwipsHeight와 VtFont.Size(point),
PrintInformation의 여백·중앙 정렬·LayoutForPrinter·ScaleType은 서로 다른
좌표·배율 계약이다. OLE 그림 프레임이나 기본 문서 HWPUNIT와 같은 값으로
취급하지 않는다. 이 사양에는 현재 PNG 로고의 보간 필터·픽셀 위상 정답이 없으므로
위치·보간 판단은 같은 원본의 독립 Print와 실제 소비 경로를 계속 대조한다.
사양 설명본 자체는 수정하지 않았다.

현재 제품으로 다시 실행한 기존12조건 × 양쪽 출력24건은23 PASS/1 FAIL이다.
#8032 넓은 그림은 Native90.69451%/fresh89.93941%, 좁은 그림은100%/99.95997%다.
#8036 두 조건과 #7833의 소유 무효화 주석8대조군은 양쪽 모두100%다.
이 결과는 #7833의 원래 미달 생성기 전체가 해결됐다는 뜻이 아니며, 별도의
중첩 표 기하 대조군2조건과 #8031/#8039의 원래 실패는 남아 있다.
원본/PDF를 바꾸거나 새 실제 HWP로 범위를 넓히지 않았다.
실제 결과는 `rebased-current-contracts-4294-result.json`, 로그는
`logs/rebased-current-contracts-supervisor-4294.log`에 보존한다.

### #8188로 해결된 하위 이슈의 원격 후속 처리

작업지시자의 명시 요청에 따라 #7721·#7722를 completed로 종료하고, 본문에
#8188 merge2546c8b2의 실제3변환본·전체384쪽 raw100 및 독립 증적 링크를 남겼다.
당시 등록 입력·별칭·PDF의 이력은 보존했다. #7445의 해당 두 표 행을 취소선으로
바꾸고 실제 sub-issue 연결에서도 해제했다. API 재조회로 상태·본문·연결을 확인했다.
현재 연결은95개(OPEN82/CLOSED13)다. 부모7445와 다른 미완료 이슈는 열어 둔다.
이 후속 처리는 현 브랜치의 통합 PR 제출·머지 완료를 뜻하지 않는다.

### 두 사양 기반 가설의 실제 분리 결과

부모 셀의 내용 폭에 맞춰 common 폭·셀 격자를 정합하게 만든 #7833 대조군8개를
새 MCP2020 Print로 출력했다. 실제 method0·hancom2020_pdf_driver_one_up 성공을
확인했다. 양쪽16실행 중4개만90% 이상이며, 깊은 wrapper와 작은 TAC 표의 배치는
여전히 다르다. 기하 변경만으로 원본 생성기나 제품의 해결을 선언하지 않는다.
표69의 고유 ID 조건에 따라 표·그림 ID만 고유하게 바꾼 별도8조건도 출력했지만,
96dpi 전쪽 Print raster는8/8 모두 전 버전과 바이트 동일했다. 중복ID는 해당 차이의
원인으로 확인되지 않았으며, 이 값으로 제품 분기를 추가하지 않았다.
결과는 `cell-parent-frame-validation-4301-result.json`과
`cell-unique-identity-effect-4306-result.json`이다. 정식 helper·fixture·회귀 기대값은
변경하지 않았다.

직접 review·overlay에서 깊은 wrapper의 한컴 Print는 부모 행의 물리 frame과
그 밖의 부동 자식을 구분하지만 현재 fallback은 자식 높이까지 부모에 합산한다.
`float_placement/reflow_block_cell.rs::has_reflow_block_cell_host`는 NO_LS 가시 텍스트가
한 문단이라도 있으면 false로 끝난다. 빈 prefix인 주석8대조군은 양쪽100%지만,
`중간` prefix가 있는 같은3단계 wrapper는 이 조건에서 공통 계획을 사용하지 못한다.
`HeightMeasurer::cell_reflow_block_flow` → `reflow_cell_frame_rows`와
Layout의 `reflow_block_cell_flow` → `sequential_nested_cell_layout`·
`closed_reflow_frame`·외부 자식 원점 소비를 함께 확인했다. fallback의
`text_height + nested_sum + outgrown`과 동일한 내용의 실제 배치를 먼저 대조한다.
수정한다면 실제 셀 가용 폭에서 구성한 텍스트 줄 결과를 공유해야 하며, 가시 텍스트
조건만 풀거나 문구별 높이를 더하는 방식은 사용하지 않는다. 아직 구현하지 않았다.

#8032는 PDF RGB XObject의 bbox를 개체의 명목 bbox와 직접 비교해서는 안 된다.
PDF의16×16 RGB와3200×1280 1bit soft mask는 별도의 source 창이다. 실제 잉크만
비교한 진단에서 RHWP/PDF 중심은375.7788/361.1644px였고, 이미지를 오른쪽14px로
옮기면 오히려 차이가 커진다. 앞선 원점 차이만으로 가로 정렬 결함이나 보정 방향을
확정하지 않는다. 진단 배열 이동은 metric gate나 제출PNG에 적용하지 않았다.
원본 PNG의 투명도를 기존 `print_jpeg_payload`가 거절하는 경로와 Print의 RGB/soft
mask 전달·표본 위상을 함께 추적한다. `pr8032-image-frame-diagnosis-4298.json`은
가설 구분용일 뿐 원점 보정이나 개선 증거가 아니다.

### #8188 상속 검사88건과 원문/논리 주소 판정

현재 정식 자동 수집21target·42모듈의88검사를 실행해87 PASS/1 FAIL을 확인했다.
실패는 `issue_8160_sample16_inline_object_fragments_keep_each_leader_with_its_source_tab`다.
기존 검사는 TextRun의 공개 논리 시작과 조각 내 위치를 더한 값을 원문 scalar
인덱스로 바로 사용했다. 실제 문단27의 탭은 원문12, 인라인 개체 한 칸을 포함한
논리13이다. 원 PR efec7825/통합3c4d044의 의도와 기존 주소계 보정 기록을 대조해,
점선 배치나 원본 탭 소유의 실제 후퇴가 아니라 검사 가정 오류로 판정했다.
처음 stderr만 보고 소유 주소 회귀로 설명했던 판단을 이 근거로 정정한다.

이 상속 검사가 실제 사용하는 기존 HWP5 원본6a3cdf2c·독립 Print를 그대로 읽어
전64쪽 Native/fresh를 별도 대조했다. 둘 다64/64쪽·최저raw100%이며 2쪽의 목차,
단일 점선과 뒤 번호를 review·standalone overlay로 직접 판독했다. 새로운 실제
HWP를 수집하거나 삭제된 corpus 참조를 복구하지 않았다. 입력·PDF 해시·전체TSV·
대표PNG는 기존 정식 증적README의「상속된 탭 소유 검사」절에 추가했다.

검사는 같은 원본의 공개 `HwpDocument.logical_to_text_offset`로 원문 scalar에
대응시켜 탭 여부를 확인하며, 해당 조각의 전체 문자열이 같은 원문 범위와 일치하는지
추가 검증한다. 리더 수·단일 소유·앞뒤 텍스트 순서·쪽 번호 앞 종료와 독립 px
기대값은 유지했다. 실제 캐럿·제품 출력은 바꾸지 않는다. 정식 관련3개 검사는
모두 PASS다(`logs/8188-tab-source-axes-nextest-4311.log`). 동일88건을 다시 실행해
전체 보존을 확인한 뒤 이 단계의 커밋에 terminal 결과를 기록한다.

동일88건의 최종 재실행은 **88 PASS/0 FAIL/5711 필터 제외**, exit0이다.
로그는 `logs/rebase-8188-all-contracts-nextest-4313.log`, 실행 신원·검사 SHA는
`rebase-8188-all-contracts-4313-result.json`에 보존했다. formatter·내부 링크와
제품 source 동일성도 확인했다. 검증한 엔진은 c2513250의 Native261/fresh077이며
이번 단계는 검사·증적만 수정했다. 전체 통합의 잔여 시각 미달·전체 Rust/lint는
다음 보정 후 최종 head에서 계속 확인한다.

### #8031 API 대조군의 정식 회귀와 원 입력 보존

원 PR의 1×1 셀 안 PageNumberPos와 후속 BODY를 공개 blank/table API로 구성한
같은 기능의 대조군을 한컴 2020 Print와 검증했다. 용지·명목 표 크기는 원 PR과
같으며 DocInfo·기본 셀 여백·괘선은 API의 정상 생성 값을 유지한다. 원래 최소 IR의
시각 미달과 구분하며 원 `page_number_pos_in_table_cell.rs`는 바이트 그대로 유지했다.
새 `page_number_cell_print_contract.rs` 두 함수는 양쪽 raw100·1/1쪽·직접 판독 후
소유·유일성·내용 순서·독립 Print 꼬리말 기준선 ±2px를 보호한다. 자동 suite007에서
실제로 2 PASS/0 FAIL이다. Native261/fresh077의 제품 source는 c2513250과 같다.

초기 standalone 검사에서는 HWPX sideChar를 prefix/suffix 별도 저장이라고 가정해
legacy format assertion이 실패했다. 실제 XML의 sideChar="-"와 파서 모델 dash_char를
대조해 검사 가정을 정정했다. old base247 RLIB의 링크 실패는 실행 RED가 아니며,
base070 WASM도 이미 같은 collector를 포함해 BODY·쪽 번호 각 1회 보존이 PASS다.
이를 수정 전 실패로 사용하지 않는다. 첫 자동 nextest0건은 생성 목록 불일치로,
manifest --prepare 후 새 두 검사 실행·exit0을 확인했다. 로그4322와 정식 asset
`mydocs/pr/assets/pr_8031_page_number_cell/README.md`에 정확한 근거를 연결했다.

현재 단계는 검증된 대조군의 정식 회귀를 보강한 것이며 기존 최소 IR의 미달,
#7833 혼합 NO_LS wrapper·#8032 투명 확대 그림·#8039 control-tail의 차이는
계속 확인한다. 통합 전체나 원 최소 입력의 피델리티 완료로 보고하지 않는다.

기존 모듈은 manifest 재배치 후 `regression_suite_014`에서 별도 2 PASS/0 FAIL을
확인했다(`logs/page-number-cell-original-nextest-4325.log`). 신규 suite007과 합계
4 PASS이며, 4323의 suite007 단독 필터 결과를 4건이라고 세지 않는다.
새 검사 rustfmt·diff 공백 검사와 고정 base manifest 검사를 통과했다. 내부 링크
검사 도구의 동일 helper로 변경 Markdown 4개와 등록부를 검사해 오류0을 확인했다.
기본 CLI는 redirect 인자가 없어도 전체 추적 바이너리를 읽는 비효율 때문에
중단했으며, 이를 통과 기록으로 쓰지 않았다. 로그는 `logs/page-number-cell-links-focused-4328.log`다.

### #7833 가시 NO_LS prefix의 공통 줄 프레임 계획

기준 head35fa0d1296의 위반은 제한 없는 Para/TopAndBottom 자식의 본체가
부모 글줄·행의 점유와 혼합되는 것이다. 기존 Print4300과 직접 판독4302는
부모 행·가시 prefix·행 밖 자식의 별도 소유를 보여 준다. 원 PR의 cut 계약과
기존 공통 reflow 경로는 유지하며 조건을 문구·문서 ID로 제한하지 않는다.

생산은 `measure_table_impl`의 실제 column/grid·padding으로 구한 inner_width,
`compose_paragraph_in_context` → `recompose_cell_lines_in_frame`의 줄 결과다.
측정의 `reflow_cell_frame_rows`와 `measured_reflow_external_flow`, 배치의
`measured_reflow_block_table` → 전체/부분 `reflow_block_cell_flow` 소비 지점이
같은 폭·줄 기반 host 점유를 쓰게 한다. 이미 구성된 문단은 재추측하지 않고
그 ComposedParagraph를 같은 공통 함수에 전달한다. 셀 폭을 전달받지 못한
기존 호출은 가시 NO_LS 문단을 임의 높이로 수용하지 않는다.

먼저 기존8개 wrapper/control의 Native와 직접 PNG로 방향을 확인한다. 개선
확인 뒤 fresh WASM·기존 note 정상 대조군·#8188 보존을 재검증한다. 지금 단계의
코드 후보와 이후 실행은 이전 c2513250 증거로 통과 처리하지 않는다.

Native262를 같은8 wrapper·8 note 입력으로 실제 비교했다. wrapper는
38.10614→79.76226, cut은38.33084→95.88601, 표형은43.96520→70.06839 /
45.10169→77.91698로 개선됐다. flat 그림2조건과 note8조건은 raw100을
유지했다. flat TAC 표81.99184/87.13986은 변화가 없다. 합계11통과/5미달이며
새 head 전체 승인으로 판정하지 않는다. `reflow-visible-host-validation-4330-result.json`과
직접 review·overlay4332에 같은 입력·Print와 실제 producer SHA를 보존했다.

잔여 차이의 실제 width39.52px 셀에서 `가🦦나다라`, 인라인 그림2개를 읽었다.
각각의 슬롯·UTF-16 갭·스타일을 유지한 진단4334/4336에서 첫줄 `가+그림+🦦`가
나온다. 가12.96px·그림16px·🦦18.293333px의 합은47.253333px로 상자보다 크다.
Font trace4335도 실제 fallback Segoe UI Emoji의 advance1372HU를 확인했다.
Print의 🦦 원점은 다음 줄이다. 문제는 `fill_one_interval`이 토큰을 거절한 뒤
`base_char_widths` 없는 단일 문자 fallback에서 이미 측정한 base_width를 버리고
CJK/em 또는 그 밖/0.5em으로 다시 추측하는 것이다. 같은 기호가 약6.67px로
줄어 거절이 뒤집힌다. 같은 단일 토큰의 기측정 폭을 재사용하고, 커닝의 경계
보정·다문자 토큰의 상세 폭 계약은 유지한다. 새 숫자 허용치는 추가하지 않는다.

단일 토큰 폭을 보존한 Native263의 첫 같은 입력은92.48023%, cut은95.39459%다.
별도 원시 줄 경계의 소비 누락도 확인했다. `recompose_cell_lines_in_frame`은
표 실측 높이 또는 낱말 꼬리 재결합 때만 FrameGeometry를 게시하고, 일반
NO_LS 그림·수식은 새 텍스트 줄만 반환했다. 배치의 `control_line_indices`는
이 프레임이 없으면 NO_LS에서 None이 되어 폭을 임의 줄에 채우는 폴백을 썼다.

새 프레임이 소유한 원시 LineSeg 시작을 인라인 개체의 줄 질의와 InlineControl에
함께 게시한다. 예를 들어 그림 슬롯raw1과 뒤 🦦 raw9는 가시 문자 경계1로
합치면 안 된다. FrameGeometry의 raw 시작을 직접 읽으며 기존 source 또는
호출자가 투영한 paragraph의 축을 다시 추정하지 않는다. 측정의 셀 본문 폭도
기존 공통 `ParagraphBox::content_for_style`로 문단 좌우 여백을 적용한다. 이는
새 함수를 넣은 경로가 기존 실제 셀 조판기와 같은 상자를 사용하게 하는 보정이다.
Native263의 실행 중 비교는 고정 실행 파일의 이전 후보 결과로 보존하고 이 추가
수정의 검증으로 사용하지 않는다. 다음 Native/fresh 출력에서 따로 판정한다.

Native264의 실제 16조건은 12 PASS / 4 미달이다. 중첩 그림 before 94.13019%,
cut 97.26027%, flat 그림 2조건과 주석 8조건은 100%다. TAC 표 네 조건은
70.06839/77.91698/81.99184/87.13986으로 남았다. #8188의 88건과 기존 셀
경계 편집 17건·쪽 번호 4건은 정식 자동 수집에서 **109 PASS / 0 FAIL**, exit0을
확인했다. 로그는 `logs/reflow-inherited-contracts-nextest-4339.log`다.
Native264 SHA-256은 `9ac0c73cf03e713d0b76040a70ad3f5bbf897beb88502a986f66fde8b350c8ec`다.
빌드·실행 source별 해시는 4340/4339 receipt에 고정했다. 이후 include 파일의
if/iterator 줄 접기만 변경했으며 4347에서 공백 외 동일성을 확인했다.
이 결과는 전체 Rust·fresh WASM·전체 시각 검증 완료를 의미하지 않는다.

인라인 표 잔여 원인은 더 좁혀졌다. 동일 원본 HWPX를 진단용으로만 MCP 2020에서
HWP로 저장했다(job `473bb1f0-d204-459b-81ac-9fefb896deee`). 기준은 기존 Print 4300을 유지하며
새 저장본을 원본의 통과 대체 자료로 쓰지 않는다. HWP5 Section0의 CTRL_HEADER를
CFB/해제 스트림에서 직접 읽으면 작은 표 4개의 common.width는 1200→2606 HU지만
각 셀의 선언 600 HU는 유지된다. 저장본 SHA-256은
`646bc950ad590de040e49f4ec084f5525088dde195c95285f338752b276b6533`이다.
추가 진단은 같은 원본의 outer0·font20·padding700900 세 변형뿐이다. 바깥 여백 0과
기본 글자 20pt에서도 2606, 안 여백 700/900에서는 3766이다. 각각
2×(510+510+283), 2×(700+900+283)이며 최소 내용 폭 1mm(283 HU)와 일치한다.
변형 과정·원본/산출 SHA는 4345 plan·개별 MCP receipt에 있다. 공식 도움말 검색은
이 최소 폭의 근거를 찾지 못했으므로 사양에 기재됐다고 주장하지 않는다.

정상화 HWP의 common.width가 2606이어도 현재 Native는 실제 작은 표를 16px로
그린다(진단 4346). `Table::closed_inline_grid_width_hu`가 600+600을 완결된 물리
폭으로 받아 source common 폭보다 우선하기 때문이다. 원인은 점수나 PDF 좌표가
아니라 셀 안 여백보다 작은 선언 폭을 실제 물리 경계로 승인하는 가정이다. 다음
단계는 모델의 폭 생산 → paragraph_frame_owner_widths 측정 → column/row 셀 배치와
최종 보조 경계 덮어쓰기를 함께 추적해 최소 물리 폭을 공통 결과로 전달하는 것이다.
아직 이 최소 폭 보정은 구현하지 않았으며 기존 기대값도 변경하지 않았다.


### 최소 물리 폭의 공통 소비 보정 계획 — 8e44c0f233 이후

기본 경로는 collaborator 외부 PR 통합이며, local_validation·visual_fixture_evidence·
multi_pr_update_branch 보조 절차를 적용한다. 원 #7833과 기존 메인터너 보정의 계약을
유지하고 같은 입력의 경계 대조만 사용한다. 다른 HWP나 새 PR을 추가하지 않는다.

원시 셀 폭과 물리 폭을 분리한다. 사양 표 75·77·80은 개체 크기·안 여백·셀 선언
크기를 구분하며, 최소 폭 수치는 사양에 명시됐다고 주장하지 않는다. 같은 입력의
MCP 2020 저장 대조에서 다음을 확인했다. 기존 독립 Print 4300은 계속 기준이다.

| 대조 | 작은 표의 물리 폭(HU) | 관측 |
| --- | ---: | --- |
| 기본·바깥 여백 0·기본 글자 20pt | 2606 | 2 × (510 + 510 + 283) |
| 안 여백 700/900 | 3766 | 2 × (700 + 900 + 283) |
| 글자처럼 취급 해제 | 2606 | 흐름 참여 여부와 최소 물리 폭은 별개 |
| 좌우 안 여백 0 | 1200 | 원래의 600 × 2를 유지 |
| 두 열 병합 셀 하나 | 1303 | 병합 열 수가 아니라 물리 셀 한 번의 하한 |
| tbl noAdjust=1 | 2606 | 높이 자동 조정 속성으로 폭 하한을 생략하지 않음 |

4349의 최초 no-adjust 변형은 속성을 sz에 잘못 넣어 그 조건의 증거로 인정하지
않았다. 4350에서 tbl 속성으로 바로잡고 새 job으로 실제 저장했다. 그 밖의
4349 대조군과 원본은 유지했다. 모든 변형은 ignored output의 진단 자료이며,
정상화 저장본의 통과로 원본의 피델리티를 대체하지 않는다.

공통 생산자는 Table의 셀 물리 폭이다. 유효 안 여백 합과 최소 내용 폭 283 HU의
하한을 선언 폭에 적용하되, 폭 0의 미해결 그리드와 직렬화 바이트는 보존한다.
flow_width/완결 행 합 → paragraph_frame_owner_widths 측정 → 열·병합 제약 →
stored_column_boundaries → build_row_col_x/declared_row_col_x → 전체·부분 셀
배치가 같은 폭을 소비해야 한다. 원시 선언 유효성 검사와 편집 이력은 변경하지 않는다.
행별 구획이 다른 표의 공통 폭 계약도 열별 최댓값 합으로 대체하지 않는다.

수정 전 Native264에서 #8188 세 변환본의 192쪽은 raw100·정수 미대응 0/0이다.
네 번째 기존 탭 원본은 요약에서 옮긴 과거 PDF 해시를 검증기가 거부했다. 실제
4308 manifest와 현재 파일의 ba4b10dee6e35e9c5ee71dd8c8004528ad3dd72b47ef5dd0cd859e0431aae381
일치를 확인하고 4351에서 그 입력만 이어서 검증한다. 앞 192쪽을 다시 실행하거나
새 입력으로 바꾸지 않는다. 4348의 실패와 원인은 terminal receipt에 기록했다.

보정 뒤 같은 16개 wrapper/note와 #8188 전쪽을 먼저 확인하고 표준 최적화 fresh
WASM으로 양쪽을 대조한다. Native264의 기존 109 PASS를 이후 코드의 검증으로
재사용하지 않는다. 새 회귀 등록·기대값 변경은 관련 Native/fresh 전쪽 90% 이상과
직접 판독을 완료한 뒤 수행한다.


### 최소 물리 폭 보정의 Native 실행 결과

생산 변경은 `Table::physical_cell_width_hu`에서 유효 안 여백과 283 HU 내용 하한을
계산하고 원시 선언을 보존하는 것이다. 완결 TAC 합, 문단 소유 폭, 단일·병합 열
제약, 저장 칸 경계 제약, 행별 실제 경계가 같은 값을 소비한다. 기존 선언 유효성
검사·직렬화 값·행별 별도 구획 계약은 유지한다. 새 숫자 공차를 추가하지 않았다.

별도 경계 대조 4353의 양수 내용 폭 128 HU에서도 작은 표는 2606 HU로 저장됐고,
안 여백 0·셀 선언 100 HU에서는 표가 566 HU로 저장됐다. 여백보다 선언 폭이
작을 때만 생기는 특례가 아니라 실제 내용 폭 하한임을 확인했다.

Native265 빌드는 8 jobs·공용 target/pr-review에서 exit0이었다. 실행 파일 SHA-256은
`75991e51cddb71d38dd07923dd7b3ee73d0020437e550fe604358a8603f0fc55`이며
정확한 dirty source 해시·명령은 `minimum-cell-width-native-build-4352-result.json`에
고정했다. 같은 16개 wrapper/note의 4354 비교는 **16 PASS / 0 FAIL**이다.

| 조건 | Native264 → Native265 raw |
| --- | --- |
| 깊은 TAC 표 before | 70.06839% → 100% |
| 깊은 TAC 표 cut | 77.91698% → 100% |
| flat TAC 표 before | 81.99184% → 100% |
| flat TAC 표 cut | 87.13986% → 100% |
| 깊은 그림 before/cut | 94.13019% / 97.26027% 유지 |
| flat 그림·주석 10조건 | 100% 유지 |

깊은 표 before의 새 review/standalone overlay를 직접 열어 작은 표의 칸 경계,
줄 순서·부모 물리 행·그 밖 자손과 뒤 본문을 대조했다. 기존 진단 PNG를 재사용하지
않았다. 경로는 `minimum-cell-width-direct-4354/deep-tables-before-native/`다.
이 대조군은 부모 기하를 정합하게 만든 별도 입력으로, 원래 잘못된 저장 소유·
그리드 생성기의 전체 해결을 뜻하지 않는다. 원 helper·기대값은 아직 변경하지 않았다.

#8188 세 변환본과 기존 탭 입력의 4355 재출력은 **256/256쪽 raw100·양방향 정수
미대응0/0**이다. Native264와 256쪽의 후보 PNG SHA가 전부 같고 기준 PNG도
불변이다. `minimum-cell-width-native-preservation-4358.json`에서 실제 대응을
확인했다. 새 실제 HWP는 추가하지 않았다. #8032 두 그림, #8036 두 vpos, #8031
API 쪽 번호의 기존 5조건도 4357에서 90.69451/100/100/100/100%를 유지했다.

표준 최적화 fresh WASM078은 4356에서 같은 source로 빌드 중이다. Native의
16 PASS·상속256쪽100%를 fresh 검증이나 전체 Rust/lint 통과로 확대하지 않는다.
원 생성 입력의 미달과 #8032 투명 확대 그림·#8039 control-tail 등 통합 필수 항목은
계속 해결하며 최종 PR 준비 완료나 원격 CI 완료로 보고하지 않는다.


### 원 합성 입력의 저장 프레임 차이 추가 확인

4359에서는 기존 generated-fixtures-004의 #7833 flat before/cut 8개와 그때의 독립
Print를 그대로 재검증했다. 모두 미달이며, 새 정합 대조군의 16 PASS로 이 결과를
덮지 않는다. 원 helper·기대값·입력/PDF는 변경하지 않았다. 이번 조치는 기존 범위의
추가 진단이며 새로운 실제 HWP로 검토를 확장하지 않았다.

원 `pr7833-flat-false-tables-false-before.hwpx`의 SHA-256은
`8bacee7f863c62789fb46517fa559dacc4b2e4191cf2deaf3b40ed874028cfaa`다.
4360은 이 같은 입력을 MCP 2020에서 진단용 HWP로 저장했다. job은
`6ad446c0-4184-4cb8-bb22-e86889aeef25`이며 기존 Print를 바꾸지 않았다.
Native265의 source/hancom-save dump를 직접 대조했다.

- 셀의 원시 폭 20977 HU와 안 여백 510/510은 유지됐다. 복제된 문단의 저장
  segment_width 42520은 19956으로 다시 계산됐다.
- 셀 첫 문단의 vertical_pos 3200은 0으로, 그 뒤 비-TAC 표 host의 6800은
  1600으로 바뀌었다. 최내측 첫 그림 문단도 1600→0으로 갱신됐다.
- 바깥 표 common 높이는 1282→2882였으나 셀 선언 높이 282는 유지됐다.
  실제 흐름과 저장 보조 높이를 같은 소유 값으로 볼 수 없다.

자료는 `pr7833-table-normalization-4360-result.json` 및
`original-cell-cut-dump-4360/{source,hancom-save}.txt`다. 정상화 저장본은 원 입력의
시각 통과 자료로 사용하지 않는다. 이 입력은 합성이므로 저장 수용 조건을 바꾸기
전에 정상 저장본·실제 폭/외부 어울림의 반례와 대조해야 한다. 원시 vpos를 clamp하거나
배치를 숨겨 통과시키지 않으며, 아직 이 추가 보정이나 회귀 변경을 구현하지 않았다.


### 양쪽 시각·상속 보존·새 회귀 실행 완료 — 2026-10-27

표준 최적화 fresh078은 실제 wasm-opt -O를 거쳐 exit0·소스 불변·Studio public
해시 일치를 확인했다(4356). 같은 16개 조건의 fresh4361도 16 PASS/0 FAIL이며
최저 94.09928%, 깊은 그림 cut97.27333%, TAC 표4조건·나머지 10조건은100%다.
Native와 합계32조건이며 원 생성기의 별도 미달8조건과 혼합하지 않는다.

#8188의 Native265 256쪽과 fresh078 256쪽은 **512/512 raw100·정수 미대응0/0**이다.
각각의 PNG는 이전에 검증한 Native261/fresh077 증적과 512쪽 모두 바이트 해시가
같다. 새 8개 TSV도 기존 영구 자산과 바이트가 같아 중복 파일을 만들지 않고
`issue_8160/semanticist21-rebase-20261026/README.md`에 재출력·대사 결과를 추가했다.
현재 보정 소스의 상속/셀 편집/쪽 번호 정식 회귀 109건은 4365에서 전부 PASS·exit0이다.

새 `inline_cell_physical_width_print` 한 함수는 한 쪽 Native/fresh raw100과 양쪽
review/standalone overlay 직접 판독 후 등록했다. 같은 기존 깊은 표 before를 정식
fixture/PDF에 바이트 동일하게 보존했으며, 중요한 가로 경계만 독립 PDF ±2px로
검사하고 칸 여백 뒤의 내용 폭·경계 연결·표 순서·비겹침·전체 텍스트 누락/중복도
유지한다. 세로 절대 위치는 고정하지 않는다. 원 helper나 기존 함수는 교체하지 않았다.
[정식 증거](../pr/assets/pr_7833_minimum_cell_width/README.md)에 해시·Print·전쪽 TSV·PNG를 연결했다.

같은 새 검사 소스를 직접 컴파일해 Native264에서 실제 16px / 독립 34.7013244628906px로
1 FAIL·exit101, Native265에서1 PASS·exit0을 확인했다(4369). 두 컴파일은 성공했고
실패 원인을 실제 단언으로 확인했다. 자동 수집 prepare 후 suite027에서 새 함수
1 PASS/0 FAIL/272 필터 제외·exit0을 확인했다(4370). 테스트 빌드 CLI의 SHA는
6e7de7f3c9cd54c22458f3a6ee0c390e6d64f77bb9dc3d3af0b4d718d78b7d8b이며,
시각 검증용으로 봉인한 Native265와 실행 파일을 구분한다. 런타임 소스는 동일하다.

현재 단계의 공통 최소 폭과 해당 한 회귀·양쪽 대조·상속 검증을 완료했다. 통합 PR의
원 생성 입력·#8032 확대 투명 그림·#8039 등의 필수 미달, 최종 전체 Rust/lint와
실제 원격 CI/merge는 여전히 남았다. 준비 완료나 목표 전체 완료로 보고하지 않는다.


새 회귀의 부모 셀 포함·원문 문단/개체 소속을 보강했다. 같은 최종 소스
`3f76a1d68cde30c4c57e7bbc552e9e765ee3e10c15e184f85bb3ebcb0b507872`를
4371에서 직접 재컴파일해 RED 1 FAIL / exit 101 → GREEN 1 PASS / exit 0을 확인했다.
새 자동 수집은 prepare 뒤 suite023에 배정됐으며 4372에서1 PASS / 0 FAIL / 307 필터 제외,
exit0이었다. 처음 버전의 suite027·4369/4370과 혼합하지 않는다. 실제 실행 target은
파일 추가·가중치에 따라 달라지므로 항상 manifest helper와 로그를 확인했다.
