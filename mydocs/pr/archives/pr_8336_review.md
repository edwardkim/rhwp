# [PR #8336](https://github.com/edwardkim/rhwp/pull/8336) 리뷰 — fix: HWPX 읽기·쓰기가 쪽 번호 줄표와 셀 세로쓰기(영문 세움)를 보존한다 ([이슈 #7633](https://github.com/edwardkim/rhwp/issues/7633), [이슈 #8334](https://github.com/edwardkim/rhwp/issues/8334))

## 최종 판정

**메인터너 보정 후 수용 가능** — 원 PR의 적용 내용은
[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)의 머지 SHA
`b8338ba38be288b6e9385f5372d184649323aa9f`에 반영됐다. 원 head를 중복 병합하지
않는다. 후속 제품 보정 `5aa6ee08dd`와 회귀 보강 `c8e4f0ffac` 기준으로 이 PR의
입력과 독립 한컴 Print PDF를 대조한 전쪽 Native/fresh WASM 검증에서 90% 미만과
고정 devel 대비 2%p 초과 하락은 없다. [원 PR 입력별 전쪽 TSV](../assets/review_8438_followup/final-original-inputs/README.md)와
아래의 주장 페이지 review·overlay를 함께 판독했다. 최소 글꼴 전체 Rust 회귀
12,387건 통과·45건 skipped, 선행 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·
[PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 2,153쪽 Native/fresh WASM
표시값 하락 0쪽이다. 후속 보정 PR의 최신 head CI와 병합을 확인한 뒤 원 PR을
중복 반영 사유로 종료한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `2d17f00451` → `eba13ba3a4`, `eb6ae84654` → `b2b0100674`, `7b2495fb8b` → `b426c12d8a`, `ccd0256d6b` → `f7ab73dc54`, `1c8edc9b07` → `a090f19131`, `e6bf361188` → `054e0c49a5`, `4e004d650a` → `a2541933cb`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_7633_8334_hwpx_attr_roundtrip` 5건, `issue_7633_hwpx_page_number_side_char` 2건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

2026-11-03 00시경 KST에도 원 PR head와 열린 상태를 재확인했다. [일괄 조회 결과](../assets/review_8438_followup/original-pr-heads-20261103.md)는 기능·시각 최종 판정을 대신하지 않는다.

## 접수와 적용

- 원 PR: [PR #8336](https://github.com/edwardkim/rhwp/pull/8336), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `4e004d650a5d32920b1337b41e3c5cf7c4b8b58e`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `4e004d650a5d32920b1337b41e3c5cf7c4b8b58e`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [이슈 #7633](https://github.com/edwardkim/rhwp/issues/7633), [이슈 #8334](https://github.com/edwardkim/rhwp/issues/8334).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

쪽 번호 줄표 없음과 영문 세움 셀 세로쓰기 속성을 HWPX 읽기·쓰기에서 보존한다.
쪽 번호 문자와 셀 방향의 서로 다른 저장·재열기 경로를 확인한다.

## 체리픽과 보정

- 원본 `2d17f00451d9989f6ef33deb39d2b8fc34a90efa` → 통합 `a740f282e2986611fe83b9cbcf07b146c72f3345`.
- 원본 `eb6ae8465403d554e20c509f5ad84edcaffad563` → 통합 `38aaa6e39f3d05b67530efa9f28e3e48ed0694a4`.
- 원본 `7b2495fb8b2705b32ce290c44b25ef8eff4b5777` → 통합 `0737f4adb7b31a7df646fadfda63893abdb78091`.
- 원본 `ccd0256d6b79a3dfe777f960ba067c64a756bb87` → 통합 `da05a76bb4d98d4859f372753b91588f3d4e1588`.
- 원본 `1c8edc9b079e0357671b87fc9c1cd99f8dbc4027` → 통합 `eaf5d44d01a26f435e545f653042504536c7f3c4`.
- 원본 `e6bf3611882010740717dd965677b775054bc4e2` → 통합 `c8436fcf5dedebf06a98cd29f6d5764989c4415d`.
- 원본 `4e004d650a5d32920b1337b41e3c5cf7c4b8b58e` → 통합 `abbdd6e3c770ce6522b09e3e77a3259ecc444406`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

### 2026-11-03 현재 저장본의 독립 한컴 2020 Print

현재 코드에서 `samples/hwp-3.0-HWPML.hwp`를 HWPX로 다시 저장하고
`--verify --verify-pages`로 IR 차이 0건과 122쪽을 확인했다.
[현재 저장본](../assets/issue_7633_8334/followup8438/hwpml-final-resave.hwpx)의
SHA-256은 `c6ae1c49859ada19115f7e07c1af5a4e4ea84a2ff57470af0c13554ece058eaa`다.
이 저장본을 한컴 2020 MCP의 Print method 0으로 출력한
[독립 PDF](../../../pdf/issue7633/hwpml-final-resave-2020.pdf)는 122쪽이며,
SHA-256은 `c508a1510a28a5e5631adc8a83d8125b25b04a79765ba1379d8bf804c632d868`이다.
MCP 작업 ID는 `21652151-d025-4b86-872d-b81f42b4587f`이고 변환 상태는 성공이다.
페이지별 Native·fresh WASM 시각 측정과 직접 판독 결과는 다음과 같다.

96 DPI, compat 2022의 색상 경계 보정 전 raw 실루엣을 122쪽 전체에서 측정했다.
Native 최저는 119쪽 96.72%, fresh WASM 최저는 같은 쪽 96.55%이며,
양쪽 모두 독립 Print와 쪽수가 같고 90% 미만은 0쪽이다.
줄표 없음이 드러나는 56쪽은 두 경로 모두 100.00%다.
56쪽의 [Native 비교](../assets/issue_7633_8334/followup8438/hwpml-final2020/native-p056-review.png)·
[오버레이](../assets/issue_7633_8334/followup8438/hwpml-final2020/native-p056-overlay.png),
[WASM 비교](../assets/issue_7633_8334/followup8438/hwpml-final2020/wasm-p056-review.png)·
[오버레이](../assets/issue_7633_8334/followup8438/hwpml-final2020/wasm-p056-overlay.png)에서
줄표가 없는 쪽 번호 `48`의 모양·위치를 직접 확인했다.
전쪽 원본 TSV는 [Native](../assets/issue_7633_8334/followup8438/hwpml-final2020/native.tsv)와
[fresh WASM](../assets/issue_7633_8334/followup8438/hwpml-final2020/wasm.tsv)에 보존했다.
최저 119쪽의 [Native 비교](../assets/issue_7633_8334/followup8438/hwpml-final2020/native-p119-review.png)·
[오버레이](../assets/issue_7633_8334/followup8438/hwpml-final2020/native-p119-overlay.png),
[WASM 비교](../assets/issue_7633_8334/followup8438/hwpml-final2020/wasm-p119-review.png)·
[오버레이](../assets/issue_7633_8334/followup8438/hwpml-final2020/wasm-p119-overlay.png)를
직접 판독했다. 본문 내용·순서·쪽 번호는 같고 제목의 글꼴 굵기에는 차이가 있다.
이 차이는 쪽 번호 줄표 보존이나 셀 방향 주장에 해당하지 않으며, 수치만으로
글꼴 자체의 완전 일치를 주장하지 않는다.

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_7633_8334_hwpx_attr_roundtrip.rs`](../../../tests/cases/issue_7633_8334_hwpx_attr_roundtrip.rs): 이전 실행 5건 통과, 0건 실패, 0건 건너뜀.
- [`issue_7633_hwpx_page_number_side_char.rs`](../../../tests/cases/issue_7633_hwpx_page_number_side_char.rs): 이전 실행 2건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`issue6029-3200477_icao_procedure-2024.pdf`](../../../pdf/issue6029-3200477_icao_procedure-2024.pdf) — SHA-256 `75ce9ab871a588ecbc8bd07d3ca05a61a8c4f70c2487cd66983b70f950d6d1d0`.
- [`148751598-briefing-rhwp-hwpx-2024.pdf`](../../../pdf/page-number-side-char/148751598-briefing-rhwp-hwpx-2024.pdf) — SHA-256 `9384c32e21d1cdef626af78147c61be96ee3bebc46811a5d557668afdc1b6ae1`.
- [`hwp-3.0-HWPML-rhwp-hwpx-2024.pdf`](../../../pdf/page-number-side-char/hwp-3.0-HWPML-rhwp-hwpx-2024.pdf) — SHA-256 `f145f4e5655febfc210bc36e96e74e78a39e42f4e546e1c5bc099090cc23eb20`.

기여자가 제출한 기준 자료: [`hwpml_hwpx_devel_silhouette.tsv`](../../../mydocs/pr/assets/issue_7633_8334/native/hwpml_hwpx_devel_silhouette.tsv). 해당 자료만으로 새 통합 후보의 시각 검증을 통과 처리하지 않는다.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·[PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. [PR #7905](https://github.com/edwardkim/rhwp/pull/7905) 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. [PR #8040](https://github.com/edwardkim/rhwp/pull/8040) 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. [PR #8410](https://github.com/edwardkim/rhwp/pull/8410)·[PR #8412](https://github.com/edwardkim/rhwp/pull/8412)의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

## 2026-11-02 재저장본의 전쪽 검사 진행

원 PR의 원본 세 문서를 현재 검토 엔진으로 HWPX 저장한 뒤 같은 저장본을
한컴 2024 MCP Print했다. [입력·PDF 해시와 작업 ID](../assets/issue_7633_8334/followup8438/print-sources.md)를 보존했다.
`hwpspec` 178쪽 Native 최저는 92.66%이고 90% 미만은 없다.
`hwpml` 122쪽 fresh WASM 최저는 92.55%, 6쪽 `brief`는 Native 95.95%·WASM 96.03%,
쪽 번호 줄표 없는 5쪽 대조군은 양쪽 99.74% 이상이었다. 각 전쪽 TSV와 핵심 쪽의
양 경로 PNG는 최종 제출 전에 정식 증거 경로에 정리해야 한다.

178쪽 `hwpspec`의 fresh WASM은 글꼴 공급 단계에서 실패했다. 첫 쪽 출력 준비 중
문서의 WMF 전체 글꼴 요청을 모으고 실제 공급 프로그램 6개에 64,693,200바이트를
사용한 뒤 `Noto Serif CJK KR`을 추가할 때 64MiB 총량 제한을 초과했다.
등록된 후보와 파일 크기를 진단 로그로 대조했으며 서체 중복 제거만으로는
6개 프로그램이 서로 달라 이 합계를 없앨 수 없다. 이는 출력·정답 여부를 판단할
수 없는 실패로, 총량 한도를 임의로 높이거나 일부 글꼴을 숨겨 통과 처리하지 않는다.
문서 전체 요청·쪽별 사용 범위와 실제 paint 공급 계약을 검토해 해결 후 전쪽을 재검증한다.

추가 진단에서 WMF 요청 9건 중 두 건은 문자 목록이 비어 있었다. 원본
`image34.WMF`와 `image35.WMF`의 변환 SVG를 확인한 결과 각각 독립 공백 텍스트만
가진 Times New Roman 요청이며, 해당 텍스트와 조상에 글자 장식 또는 CSS 클래스가
없었다. 기존 추출기가 공백을 요청 문자에서 제거해 빈 요청으로 만들고, 공급기는
빈 요청에 글꼴 대체 사다리 전체를 등록하고 있었다. 임시 진단에서 이 두 요청을
제외하거나 공백 글리프를 명시한 두 방법 모두 1쪽 Canvas 출력이 완료됐으며 PNG
SHA-256은 모두 `23ba4556f4ea2d254f96e23d0038febce2496103b9b464a88ffa27f65db81230`이다.
이는 원인 분리용 기존 WASM 실험이며 정식 수정이나 178쪽 시각 검증 결과가 아니다.
공백도 실제 glyph coverage로 선택하는 방향을 검토하고, 장식이 있는 공백과 빈
텍스트의 반례를 포함해 최종 경로를 검증한다.
다만 기존 `fontMissingText`는 공백을 검사에서 제외하고 Studio의 WMF 공급기는
현재 글자별 후보 선택을 사용하지 않는다. 따라서 위 공백 지정 실험은 첫 사용
가능 글꼴을 선택했다는 증거이지, 해당 글꼴의 공백 cmap을 확인했다는 증거가 아니다.
검증 도구에서 공급 바이트의 공백 cmap과 fallback 계약을 함께 확인해야 한다.
기존 Studio의 WMF 전체 후보 공급은 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)의
검사로 보호되는 계약이므로 유지한다. 공백 cmap을 확인하는 별도 API와 시각 검증
선택 경로를 미커밋 후보로 적용했으며, API 부재 시 전체 후보를 유지하는 반례를
포함한 Node 검사 15건이 통과했다. 실제 글꼴을 공급하는 Rust 검사와 새 WASM
전쪽 재검증은 아직 진행 중이다.
기존 테스트 자원 두 개의 실제 cmap을 사용한 최소 글꼴 집중 검사에서는 공백이
없는 첫 글꼴과 공백이 있는 다음 글꼴을 구분했으며, 기존 일반 텍스트 API의
동작도 유지됐다. 검사 2건은 통과했지만 이후 XML 공백 처리와 줄 구성이 추가로
바뀌어 현재 후보의 묶음 검사와 fresh WASM을 다시 실행한다.

### 2026-11-02 새 base 최소 글꼴 집중 회귀

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`에서 `issue_7633_8334_hwpx_attr_roundtrip` 5건을 실행해 모두 통과했다.
전체 선택 범위는 28건 통과이며 전체 회귀 실행 결과가 아니다. 시스템 글꼴을 격리하고
저장소의 재배포 가능한 최소 글꼴만 노출했다. 기존 Print 시각 증거와는 별도로 판정한다.

### 2026-11-02 의존성 리베이스 후 전체 Rust 회귀

최소 글꼴 환경에서 전체 선택 12,369건을 두 실행으로 완료해 모두 통과했다.
첫 실행의 중단과 나머지 307건의 재실행, 정확한 목록 대조는
[공통 실행 근거](../assets/review_8438_followup/rebase-de2295-rust.md)에 기록했다.
이는 이 PR의 시각 검증이나 최종 판정을 대신하지 않는다.

### 2026-11-02 그림 참조가 없는 글머리표 보정

독립 한컴 2024 Print의 94쪽에는 검은 원 글머리표가 표시된다. 원본 HWPX의
`hh:bullet`은 `char="●" useImage="1"`이지만 실제 `hh:img` 참조가 없다.
기존 파서는 참조가 없을 때도 BinData 1번을 지정해 다른 그림을 표시했다.
[PR #3210](https://github.com/edwardkim/rhwp/pull/3210)의 실제 이미지 ID 파싱은
유지하고, 참조가 없는 경우 문자로 출력하며 `useImage` 원본 속성은 저장 왕복에서
보존하도록 보정했다. 실제 원본의 문자·문단 소유·본문·왕복과 명시 이미지 ID
대조군을 포함한 최소 글꼴 집중 검사 9건이 통과했다. 글꼴 프로그램 자체는 배포하지 않는다.

같은 후보에서 그림 보정 전 178쪽 전체 raw 최저는 Native 92.66%, fresh WASM
92.56%였으며, 94쪽에 후속 본문 시작 위치 차이도 확인했다. 글머리표 수정 후
새 실행 파일의 전쪽 TSV·변경 영역 직접 비교와 표 뒤 원점 분석이 남아 있으므로
이 기록은 최종 판정이 아니다. 기준 파일은
`pdf/issue7633/batch-8336-hwpspec-2024.pdf`다.

94쪽 본문 위치 추가 조사: 원본 구역 4의 문단 6911은 빈 문단이며 저장 첫 줄
`vertpos=10310`, 바로 다음 문단 6912의 세 줄은 각각 `12410`, `14010`, `15610`이다.
독립 PDF에서 다음 문단 첫 글줄은 이 저장 위치에 대응한다. 변경 전후 RHWP는 세 줄의
상대 간격은 유지하지만 문단 전체가 아래에 있다. 바로 앞 문단 6910에는 25행 떠 있는
표와 5행 글자처럼 취급하는 표가 함께 있고, 5행 표는 이전 페이지에 먼저 배치되며
25행 표는 다음 페이지로 이어지고 아래 캡션이 붙는다. 현재 종료 진단은 다음 빈 문단
저장 위치와 같은 흐름 높이에서 끝나지만, 그 높이가 문단 원점인지 앞 간격을 이미
포함한 첫 줄 원점인지 소비 경로를 확인해야 한다. 이 사실만으로 커서를 되감거나
특정 좌표를 보정하지 않는다. 표 분할 예산·실제 배치·다음 문단의 동일 원점 소비를
검증하고 정상 대조군을 확인한 뒤 수정한다.

추가 보정 단계에서는 같은 호스트의 다른 표가 이전 쪽에서 통째로 배치된 사실을
페이지네이션 항목에서 확인한 뒤에만 종료 표 뒤 저장 줄 원점을 수용한다. 그 외
미배치·부분 배치 개체가 있는 경우에는 기존 경로를 유지한다. 수용한 원점을 조판과
그리기에 전달하며, 빈 문단과 다음 본문의 본문 영역 대비 위치 비율을 검사한다.
수정 전 관계형 회귀는 실제 `0.83899`, 원본 `0.83078`로 실패했고 수정 후 관련
최소 글꼴 검사 10건은 통과했다. 현재 단계의 독립 Print 직접 비교와 정상 분할 표
대조군은 계속 확인한다.

`0b3cbeb360` 제품 코드에서 기존 한컴 2024 Print PDF와 같은 178쪽을 다시 비교했다.
Native 최저 96.47%, fresh WASM 최저 96.45%이며 두 경로 모두 90% 미만은 없다.
문제가 지적된 94쪽은 두 경로 모두 100.00%로, 기존 HWPX 저장본의 글머리표와 표 뒤
본문 위치를 직접 확인했다. 이 저장본은 원본의 책 그림을 누락한 별도 입력이므로
원본 그림 보존의 증거는 아니다. 수치는 96 DPI의 색상 경계 보정 전 2px 내용 실루엣이다.

- 전쪽 결과: [Native TSV](../assets/review_8438_followup/pr8336-hwpspec-0b3cbeb360/native.tsv), [WASM TSV](../assets/review_8438_followup/pr8336-hwpspec-0b3cbeb360/wasm.tsv)
- 94쪽: [Native 비교](../assets/review_8438_followup/pr8336-hwpspec-0b3cbeb360/native-p094-review.png), [Native 오버레이](../assets/review_8438_followup/pr8336-hwpspec-0b3cbeb360/native-p094-overlay.png), [WASM 비교](../assets/review_8438_followup/pr8336-hwpspec-0b3cbeb360/wasm-p094-review.png), [WASM 오버레이](../assets/review_8438_followup/pr8336-hwpspec-0b3cbeb360/wasm-p094-overlay.png)
- 입력 SHA-256: `a57f69af3faacda68af7c424b0b645e00a8adec55854c10b92b02defd807c48a`.
  독립 Print PDF SHA-256: `206ff1ecc19e4af3812ec88cc73be93afbdbab0a247b97cf837c83b62e2dc8cd`.
- 최저점 175쪽에서는 원 PR의 쪽 번호·셀 방향 주장과 별개인 제목 시작 위치 차이가
  보인다. 원본의 빈 개요 머리 형식과 50% 머리 간격을 확인했다. 이 현상은 기존
  출력에도 있었으며 94쪽 표 원점 보정에 따른 후퇴는 아니다.

이 자료는 해당 문서의 보정 결과이며, 다른 입력의 전쪽 대조·전체 회귀 및 CI를
대신하지 않는다.

#### 원본 그림 글머리표 보존 추가 확인

위의 검은 원은 **기존 RHWP HWPX 저장본**을 한컴에서 Print했을 때의 결과다.
원본 `samples/hwpspec.hwp`의 한컴 Print는 검은 원이 아니라 파란 책 그림이다.
원본 DocInfo의 다섯 번째 글머리표는 그림 사용 값 `1`과 그림 데이터 ID `13`을
별도로 저장하며, BinData 레코드 순번 13은 저장소 ID 43의 파란 책 PNG를 가리킨다.
기존 렌더러는 그림 사용 값 `1`을 참조 ID로 오인하고, HWPX 저장은 그림 참조를
누락한다. 따라서 위 저장본 기준 100%는 기존 잘못 저장된 파일에 대한 일치이며
원본 HWP에서 HWPX로 변환할 때 책 그림이 보존된다는 증거가 아니다. 원본 BinData
순번과 HWPX manifest ID를 공통 매핑하고 렌더·저장·다시 열기 및 독립 원본 Print를
대조한 뒤 다시 판정한다. 기존 쪽별 본문 회귀를 완화하는 것으로 대신하지 않는다.

94쪽 본문 굵기 추가 조사: 원본 HWP의 구역 3 문단 240은 함초롬바탕 글자 모양
14를 사용하며 굵게 속성은 꺼져 있다. 반면 기존 원본 Print
`pdf/hwpspec-2024.pdf`에서 같은 본문에 사용된 내장 글꼴은 함초롬바탕 Bold다.
기존 HWPX 저장본의 별도 한컴 Print에는 함초롬바탕 Regular가 포함되어 있다.
원본 속성과 기존 Print의 굵기가 다르므로 2px 실루엣이 100.00%여도 굵기 일치로
판정하지 않는다. 두 Print의 글꼴 선택 환경과 실제 표기 차이를 확인하며 문서별
굵기 강제나 독립 근거 없는 회귀 기대값 변경은 하지 않는다.

원본 레코드의 그림 사용 여부와 4바이트 그림 정보의 ID를 분리하고 HWPX manifest의
실제 항목에 연결하는 보정 후보를 확인했다. 원본 HWP와 새 HWPX 저장본의 94쪽
Native raw는 기존 원본 한컴 Print 대비 각각 100.00%이며, 둘 다 파란 책을 표시했다.
이 선택 쪽 측정 바이너리는 패턴 효과의 HWPX 사양 확인 전 빌드이며 해당 원본의
그림 효과는 원본 그대로(`REAL_PIC`)다. 최종 코드에서는 새로 빌드해 전쪽을 재검증한다.
HWP→HWPX→HWP→HWPX 왕복에서도 참조된 PNG 바이트가 원본과 같다. 독립 Print의
본문 굵기 차이는 위와 같이 별도로 남아 있다. 최소 글꼴 집중 회귀 13건은 모두
통과했고 앞서 실패한 원본 쪽별 본문 비교도 기존 기대값을 유지한 채 통과했다.
이는 선택한 94쪽 Native 및 집중 검사 결과이며 전쪽·fresh WASM 검증은 아니다.

#### 책 그림 보정의 버전별 최종 시각 대조

원본 `samples/hwpspec.hwp`는 한컴 2010 저장본이고 새 HWPX의 버전 정보는 한컴 2020이다.
따라서 저장 버전에 맞는 기준은 동일 원본의 기존 [한컴 2020 Print](../../../pdf/review-integration-20261026/hwpspec-2020.pdf)이며,
원본 입력 SHA와 Print method 0은 [출력 기록](../../working/assets/review_integration_20261026/hancom_print_inputs.json)에 있다.
원본 SHA-256은 `64df877f3c4accda0111ecf39d837da889741bb821b220cf61b6c126dc88364d`,
PDF SHA-256은 `257f02c14ce843f38398c6702274434ec129c73b9da8ed41f9f61dc107a6bbc4`다.
새 [HWPX 저장본](../assets/issue_7633_8334/followup8438/book-reference/hwpspec-book-saved.hwpx)의
SHA-256은 `31c0dbbbf71bb0561d2be517e979b348f90b8ec49d195f0f98a0f960cefd2104`다.

96 DPI, compat 2022, 색상 경계 보정 전 raw 기준으로 원본과 저장본은 각각 178쪽이며
Native 최저 96.51%, fresh WASM 최저 96.36%다. 90% 미만은 없고 94쪽은 두 경로와
두 입력 모두 100.00%다. 책 그림과 뒤 문단, 굵기와 줄 간격을 직접 대조했다.

| 입력 | Native 전쪽 | fresh WASM 전쪽 |
| --- | --- | --- |
| 원본 HWP | [TSV](../assets/issue_7633_8334/followup8438/book-reference/native-source-hwp.tsv) | [TSV](../assets/issue_7633_8334/followup8438/book-reference/wasm-source-hwp.tsv) |
| 저장 HWPX | [TSV](../assets/issue_7633_8334/followup8438/book-reference/native-saved-hwpx.tsv) | [TSV](../assets/issue_7633_8334/followup8438/book-reference/wasm-saved-hwpx.tsv) |

94쪽 저장본: [Native review](../assets/issue_7633_8334/followup8438/book-reference/native-saved-p094-review.png) ·
[Native overlay](../assets/issue_7633_8334/followup8438/book-reference/native-saved-p094-overlay.png) ·
[WASM review](../assets/issue_7633_8334/followup8438/book-reference/wasm-saved-p094-review.png) ·
[WASM overlay](../assets/issue_7633_8334/followup8438/book-reference/wasm-saved-p094-overlay.png).

동일 원본의 별도 2020 Print에서도 해당 본문은 보통체다. 앞서 확인한 2024 HWP Print의
Bold 선택은 이 2020 기준과 혼용하지 않으며 원인을 확정하지 않는다.
[본문 네 출력 확대 비교](../assets/issue_7633_8334/followup8438/book-reference/font-four-way.png)는
위에서부터 RHWP 원본, 2024 HWP Print, 2020 HWP Print, 2024 HWPX Print다. 보정 전 저장본은
책 그림 참조 검출에 실패했고, 보정 후 저장본은 manifest를 통해 원본 PNG와 같은
바이트를 가리킨다. 원본 PNG SHA-256은
`7ea93ebbb90fb9df4ef75b16b393a4aa393fe129408e7d373c5c76a09820130a`다.

2026-11-03 현재 제품 코드 `dd284b5cda`로 원본 HWP를 다시 HWPX 저장했다.
`--verify --verify-pages`에서 IR 차이 0건, 전후 178쪽을 확인했으며 저장본의
SHA-256은 기존 검증본과 같은 `31c0dbbbf71bb0561d2be517e979b348f90b8ec49d195f0f98a0f960cefd2104`다.
같은 2020 Print·96 DPI·compat 2022에서 94쪽 원본과 저장본의 현재 Native PNG도
각각 이전 실행의 같은 입력 PNG와 SHA-256
`c01b60724f8c791bf53119f6c3dd08216ecd1ae169de0b7c95c5945f322a5525`로 일치했다.
기존 94쪽 [비교](../assets/issue_7633_8334/followup8438/book-reference/native-saved-p094-review.png)와
[overlay](../assets/issue_7633_8334/followup8438/book-reference/native-saved-p094-overlay.png)의
책 그림과 본문을 재확인했다. 이는 94쪽의 현재 Native 확인과 저장 바이트 비교이며,
현재 코드의 전쪽 Native·fresh WASM을 새로 산출했다는 뜻은 아니다.

이후 같은 코드의 Native로 원본 HWP 전 178쪽을 기존 한컴 2020 Print와 다시
대조했다. 최저 96.51%, 90% 미만 0쪽이며 새 TSV의 SHA-256
`83f22291a8dac6ee773e38a18a3635d17f888022b05e7b9812949729b9da58d3`은
위 [기존 원본 Native TSV](../assets/issue_7633_8334/followup8438/book-reference/native-source-hwp.tsv)와
바이트 단위로 같아 기존 파일을 재사용한다. 현재 코드의 저장본 전쪽 Native와
두 입력의 fresh WASM 결과는 별도로 확인한다.
현재 코드의 저장 HWPX도 같은 Print와 전 178쪽을 Native로 비교했다. 최저 96.51%, 90% 미만 0쪽이며 결과 SHA-256은 `83f22291a8dac6ee773e38a18a3635d17f888022b05e7b9812949729b9da58d3`으로 [기존 저장본 Native TSV](../assets/issue_7633_8334/followup8438/book-reference/native-saved-hwpx.tsv)와 바이트 단위로 일치해 재사용한다. 원본과 저장본의 fresh WASM은 아래에 따로 기록한다.

현재 제품 코드의 fresh WASM으로 원본 HWP 전 178쪽을 같은 한컴 2020 Print와 다시 비교했다. 최저 96.36%, 90% 미만 0쪽이며 TSV SHA-256 `8a796c7307b5e05e0b5ca6cbeb688fa2c14bcf176f4631a8935001e02c62bd36`은 [기존 원본 WASM TSV](../assets/issue_7633_8334/followup8438/book-reference/wasm-source-hwp.tsv)와 일치해 재사용한다. 저장 HWPX의 결과는 아래에 기록한다.

같은 제품 코드의 저장 HWPX fresh WASM도 전 178쪽 측정을 마쳤다. 최저 96.36%, 90% 미만 0쪽이며 [기존 저장본 WASM TSV](../assets/issue_7633_8334/followup8438/book-reference/wasm-saved-hwpx.tsv)와 SHA-256 `8a796c7307b5e05e0b5ca6cbeb688fa2c14bcf176f4631a8935001e02c62bd36`으로 바이트 단위 일치한다. 이 결과는 이후에 작성 중인 WMFC 인쇄 장치 변경 전 제품 코드의 결과다.

원본·HWPX·HWP 왕복에서 그림 소유와 내용을 비교하는 관계형 검사를
추가했다. 전쪽 비교는 제품 코드 `6896862b1c`와 그 코드로 빌드한 fresh WASM을 사용했다.
최소 글꼴 집중 실행은 기존 검사 13건 통과와 신규 검사 1건 실패로 끝났다.
실패는 테스트가 HWPX→HWP 변환 어댑터를 거치지 않은 원시 직렬화 API를 호출한
것이었다. 실제 CLI와 같은 `export_hwp_with_adapter` 경로로 수정한 후 신규 검사만
다시 실행해 통과했다. 두 실행을 하나의 전체 통과 실행으로 합산하지 않는다.
첫 로그 SHA-256은 `8d0026b08539537168032b5f6d14a695030fdefc5b9f97d9c0bddbc73840b46a`,
재실행 로그는 `fa07491503d17f43c4de2979e07eed6ad3790aac7624cbd215659f0cfa9c97eb`다.

#### 최신 devel 리베이스 기능 보존

기준 `ea86816b5316dfeeb47ecf20df97abfd4b02da73`, 후보 `6896862b1cfb638304dc1106915fcbc7bbbb9d03`.
[PR #8379](https://github.com/edwardkim/rhwp/pull/8379)의 연속 형광펜과 기존
[이슈 #6635](https://github.com/edwardkim/rhwp/issues/6635) 형광펜 색상·선택 보존을
각 기준에서 별도 fresh WASM과 동일한 headless Chrome으로 실행했다. 연속 형광펜은
base·후보 각각 34개, 기존 기능은 각각 64개가 통과했다. 브라우저가 받은 WASM SHA-256은
base `73f8a9443b500ff684b972db0947f8dfbe28402cc05d7a0abfc42bd9001345ff`, 후보
`c7e6eb6f43570abcbdda1920f8baea208858cc883e1fb4f87f9876922d7f1ede`로 각 빌드와 일치했다.
셀 블록 즉시 적용, 빈 선택, undo·redo를 포함한다. 두 실행의 선택적 플러그인 조건은 같았다.

[PR #8421](https://github.com/edwardkim/rhwp/pull/8421)의 시험지 ingest 지침과 생성기,
그림 직렬화·ingest 구현은 후보와 base의 해당 경로에 차이가 없다. 이번 변경은 기존 그림
삽입 지원 범위를 바꾸지 않고 글머리표 그림 참조의 읽기·저장과 렌더링을 보정한다.

### 2026-11-03 현재 저장본의 셀 세로쓰기 검증

`samples/k-water-rfp.hwp`를 현재 제품 코드 `d56930dda7`로 HWPX 저장한
[입력](../assets/issue_7633_8334/followup8438/kwater-final-resave.hwpx)의 SHA-256은
`9f02906bca2968fba8bf39149af9178de3a225f49a281f55682ac23f43956190`이다.
`export-hwpx --verify --verify-pages`에서 IR 차이 0건, 저장 전후 27쪽,
XML의 `VERTICALALL` 셀 3건을 확인했다. 원본 HWP의 독립 한컴 2022
[Print PDF](../../../pdf/k-water-rfp-2022.pdf)는 27쪽이며 SHA-256은
`fe135ed3f457092303fa4997a02a4f64f8f93d39fc49381dd753c8ad51bca36b`다.

현재 저장본과 이 PDF를 96 DPI·compat 2022로 전쪽 비교한
[Native TSV](../assets/issue_7633_8334/followup8438/kwater-final/native.tsv)는
최저 26쪽 97.24%, [fresh WASM TSV](../assets/issue_7633_8334/followup8438/kwater-final/wasm.tsv)는
최저 같은 쪽 97.30%다. 누락·90% 미만은 모두 0쪽이다.
셀 세로쓰기 3건이 보이는 13쪽은 Native 99.69%, fresh WASM 99.67%다.
[Native 비교](../assets/issue_7633_8334/followup8438/kwater-final/native-p013-review.png)·
[오버레이](../assets/issue_7633_8334/followup8438/kwater-final/native-p013-overlay.png),
[WASM 비교](../assets/issue_7633_8334/followup8438/kwater-final/wasm-p013-review.png)·
[오버레이](../assets/issue_7633_8334/followup8438/kwater-final/wasm-p013-overlay.png)에서
세로 방향 글자·셀 원점·본문 순서를 직접 대조했다. 원본과 같은 셀 방향을
유지하며 13쪽의 내용 누락이나 겹침은 보이지 않는다.

### 2026-11-03 최종 코드의 원 PR 입력 전쪽 재검증

제품 코드 `d56930dda7`의 Native와 같은 코드로 새로 빌드한 fresh WASM으로
이 PR과 관련된 9개 원본·저장본을 각 독립 한컴 Print PDF와 다시 대조했다.
전쪽 최저는 Native 95.95%(1쪽), fresh WASM 96.03%(1쪽)이며, 모든 입력에서 쪽수 일치·누락 0쪽·90% 미만 0쪽이다.
[입력별 SHA-256·전체 Native/fresh WASM TSV](../assets/review_8438_followup/final-original-inputs/README.md)에
정확한 원본·PDF·페이지 수와 전쪽 결과를 연결했다. 이 검사는 기존 본문의
주장 영역 review/overlay 직접 판독과 함께 판단한다.
