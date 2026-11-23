# 그림 배치 비교 방법과 입력

검증 대상은 Windows 한컴 2024로 새로 인쇄한 공개 HWPX 두 개와 단일 그림 속성만 바꾼 열 개의 대조군이다. 한컴 2024 13.0.0.3901과 Hancom PDF 1.3.0.550의 작업 번호, 입력과 PDF의 SHA-256, 저장소 경로는 [인쇄 원장](../print-inputs.tsv)에 있다. 원본 두 개를 다시 인쇄한 전쪽 픽셀은 기여자가 제출한 데스크톱 인쇄본과 같았다. 기여자가 보고한 비공개 23건 가운데 서로 다른 구성이었던 6건의 원문은 제공되지 않았다. 메인테이너의 실측 대상은 공개 재현물이다.

## 소스와 재현 조건

| 쪽별 TSV 열 이름 | 사용한 소스 |
| --- | --- |
| `base-native`, `base-wasm` | #9350과 #9355를 포함하는 devel `17b17c934b527aa523f037ca12d8713394e43c82` |
| `candidate-native`, `candidate-wasm` | 접수 시점 devel `7d6628982c`와 원 기여 `1cc8f4c457`의 실제 병합 `50aa1a326c`. 이후 기여자가 다시 제출한 `f5bd466536`과 이전 head의 파일 트리가 같다. |
| `fixed-native`, `fixed-wasm` | 현재 원 기여 `f5bd466536`과 제품 보정 `de678c8325`·회귀 검사를 최신 devel과 실제로 병합한 `c1504cb12199081c010188557ff2f1e14d36d5ef`를 사용했다. 빌드별 소스·Cargo.lock·실행 파일 해시는 [빌드 기록](renderer-builds.json)에 있다. |

기본 비교는 96 DPI, `--compat 2024`, `--font-path ttfs/opensource`, Print 프로필이다. Native Skia 또는 Docker에서 새로 최적화한 WASM Canvas로 그림을 출력했다. 같은 기준 PDF에서 만든 전쪽 PNG를 사용해 2px 이웃의 보정 전 내용 실루엣을 측정했다. [12개 문서의 소스·백엔드별 1쪽 실측](../page-scores.tsv) · [좌표 대조](../picture-bounds-96dpi.tsv)

## 직접 판독

- `grid2x2`와 `row3-mixed`에서 각 열의 그림이 위아래 두 줄을 차지한다. 뒤의 `END`는 칸 아래에 남는다. 네 번째 그림에 왼쪽·오른쪽·가운데 정렬로 같은 좌표를 지정한 세 인쇄본은 동일한 배치를 보였다. [Native 비교](native/controls-1.png) · [WASM 비교](wasm/controls-1.png)
- 앞 그림만 겹침을 허용하면 왼쪽 열, 마지막 그림만 허용하면 오른쪽 열에서 두 그림이 겹친다. 네 번째 그림의 중심을 칸 오른쪽 밖으로 13.4px 옮겼을 때 나머지 세 그림의 줄·위치는 같다. [Native 비교](native/controls-2.png) · [WASM 비교](wasm/controls-2.png)
- 네 번째 그림 전체가 칸 밖에 있는 입력은 칸 경계에서 그 그림이 가려지며 나머지 세 그림과 뒤 문단이 보인다. 바깥 여백과 양수 세로 오프셋을 바꾼 대조군은 각각의 저장값에 따라 줄·칸·뒤 문단의 위치가 달라진다. [Native 비교](native/controls-3.png) · [WASM 비교](wasm/controls-3.png)

각 백엔드 폴더의 문서별 PNG에는 한컴 Print, 제품 출력과 겹침 비교가 들어 있다. [2×2 Native](native/grid2x2.png) · [2×2 WASM](wasm/grid2x2.png) · [3×2 Native](native/row3-mixed.png) · [3×2 WASM](wasm/row3-mixed.png)

## 글꼴과 보호 문서 재현

#9350의 명시적 공급 대상 312개를 모두 확인했다. 기존 해시 305개, 허용된 Windows 11 설치본 6개, 화학물질 표시 기준에만 허용된 DotumChe Regular 2.21 한 개다. 기준과 후보는 같은 입력·PDF·DPI·compat·문서별 공급 순서로 측정했다. [공식 공급 목록](../../../../verification/protected-font-supply/manifest.json) · [파일별 해시와 돋움체 대조](font-supply.md)

공식 목록의 확보된 프로그램은 명시 순서대로 공급하고 자동 선택 문서에는 사용 가능한 기록 글꼴을 격리된 Fontconfig에 노출해 devel과 후보를 비교했다. 재현할 수 있는 프로그램이 제한된 문서는 양쪽 소스에서 같은 조건으로 측정했다. 입력이나 프로그램이 빠진 행은 보호 통과에 포함하지 않는다. [정본 판정 기준](../../../../manual/verification/visual_verification_governance.md#정본-보호-tsv의-공식-표시값-보존)

| 측정 조건 | 백엔드 | 비교한 문서와 쪽 | 로컬 devel과 후보의 PNG 차이 | 두 소스에서 공식 하한에 미달한 쪽 |
| --- | --- | ---: | ---: | ---: |
| 기존 공급 | [Native](protected-system-native.tsv) | 16종 1,644쪽 | 0쪽 | 19쪽 |
| 기존 공급 | [fresh WASM](protected-system-wasm.tsv) | 16종 1,644쪽 | 0쪽 | 25쪽 |
| 허용된 Windows 설치본을 공급한 문서 | [Native](protected-windows-fonts.tsv) | 6종 276쪽 | 0쪽 | 65쪽 |
| 허용된 Windows 설치본을 공급한 문서 | [fresh WASM](protected-windows-fonts.tsv) | 6종 276쪽 | 0쪽 | 63쪽 |
| 화학물질 문서에 허용된 DotumChe Regular 2.21 | [Native](chemical-dotumche-diagnostic.tsv) | 1종 103쪽 | 0쪽 | 0쪽 |
| 화학물질 문서에 허용된 DotumChe Regular 2.21 | [fresh WASM](chemical-dotumche-diagnostic.tsv) | 1종 103쪽 | 0쪽 | 0쪽 |

23종의 2,023쪽을 각 백엔드에서 비교했다. 정본의 쪽별 하한 미달은 기준과 후보에서 똑같이 Native 84쪽·WASM 88쪽이다. 화학물질 문서는 새 SPJGT와 허용된 돋움체를 지정해 두 백엔드 모두 103쪽의 하한을 충족했다. [화학물질 문서의 최종 공급·측정](chemical-dotumche-verification.json)

회수한 굴림 파일은 `outline-demo`와 `outline-minimal`의 Native 자동 선택 환경에도 반영했다. Fontconfig에서 기록 목록 1,219개 중 로컬의 575개를 확인한 뒤 두 문서의 4쪽을 devel·보정본으로 다시 그렸다. 굴림 추가 전후와 두 소스 사이의 픽셀 차이는 모두 0쪽이었다.

이전 검토에서 수용한 #9335의 공급본으로도 devel과 후보를 실행했다. 23종·양 백엔드의 5,811개 항목과 고유 프로그램 559개의 해시·이름·순서를 확인했다. Native에서 동명 글꼴을 선택하는 검색 디렉터리 묶음과 환경 JSON도 복원했다. 이 결과는 새 글꼴 목록이 생기기 전의 조건에서 그림 보정으로 바뀐 출력을 확인하는 데 사용한다. [이전 수용 조건](../../semanticist21-20261123-table-notes-shadows/protected-inputs.json) · [검색 순서 대조](font-search-groups.tsv) · [쪽별 정본](../../visual-protected-baseline/final.tsv)

이전 공급 조건에서 양쪽 백엔드 각각의 2,023쪽 모두 로컬 devel과 보정본의 PNG가 같았다. 이 조건으로 현재 하한표에 대조하면 Native의 미달은 0쪽이고 WASM은 시행령 문서의 8쪽이 미달이다. [Native 전쪽](protected-previous-supply-native.tsv) · [WASM 전쪽](protected-previous-supply-wasm.tsv)

형식 사양서 16쪽은 공식 하한 100.00%에 대해 이 호스트의 Native에서 98.71%였다. devel과 보정 후보의 전쪽 PNG가 같았다. 독립 PDF와 대조한 차이는 문단 79의 삽입 그림에 쓰인 문자·선의 생김새에 집중된다. 렌더 트리에서도 해당 영역은 단일 그림이다. [16쪽 같은 배율의 확대 비교](spec13-native-16.png) · [확보한 글꼴의 Native 실측](protected-system-native.tsv)

## devel의 보호 기준값 재현 후속

2026-11-23 작업지시자는 그림 배치 보정본과 기준 `devel`의 PNG와 점수가 같은 기존 미달 172건을 이 PR에 한해 수용했다. 4,046개 측정 행을 `devel`의 정본과 원본 SHA·PDF SHA·백엔드·호환 버전·DPI·쪽 번호로 다시 연결했다. 미달 172건의 정본값·기준값·보정값·동일한 PNG 해시는 [승인 범위 TSV](accepted-existing-floors.tsv)에 있다. [쪽별 공식 하한표](../../visual-protected-baseline/final.tsv)

후속 조사는 `devel`에서 수행한다. 시행령(`decree`) Native 59쪽·WASM 60쪽과 `hwp3-sample10` Native 11쪽·WASM 16쪽부터 생산 당시 글꼴 선택·렌더링 환경을 대조한다. 형식 사양서 16쪽의 삽입 그림은 독립 PDF와 래스터화 경로도 대조한다. 완료 기준은 같은 기준 PDF와 입력·DPI에서 양쪽 백엔드의 전쪽 공식값 재현 및 원인 기록이다. [기준 입력·측정 조건](../../visual-protected-baseline/manifest.json) · [형식 사양서 16쪽 확대](spec13-native-16.png)

## 샘플 추가와 저장 검증

`grid-outside.hwpx`는 의도적으로 그림 하나를 칸 밖에 둔다. Native 진단에서 기준 devel과 보정 후보가 같은 그림 한 개를 가리켰고, 데스크톱 Print에서도 그 그림은 칸에 보이지 않는다. 나머지 열한 문서에서 새 off-canvas 항목은 0개였다. 새 샘플 12개는 독립 PDF와 쪽 수가 모두 1로 일치했다. 표 뒤 문단과 그림 소속·순서·누락 여부는 회귀 검사에서도 확인한다. [입력별 보정 전후 진단](fixture-diagnostics.tsv)

두 원본과 `header-footer-paper-frame.hwpx`의 HWPX 왕복을 기준·후보 양쪽에서 실행했다. 최초·재저장 모델 차이가 모두 0개였고 저장 파일의 ZIP 무결성 검사도 통과했다. 외부 본문 clipping 문서 목록 92건의 원문은 이 호스트에 없었다. 공개 입력 12건에 대해서는 양쪽 실행 파일로 본문 영역 이탈과 표 clipping을 직접 확인했다.

## 새 devel의 기능 검증

#9335의 입력 17종을 같은 호스트에서 출력한 기준과 보정본은 양쪽 백엔드 각각 19쪽의 PNG가 모두 같았다. #9350의 아홉 이슈에 쓰인 입력 10종도 같은 방식으로 전쪽을 비교했다. 기준과 보정본의 583쪽은 Native·WASM에서 각각 모두 바이트 단위로 같았다. #9350 생산자가 기록한 PNG 자체와 일치한 쪽은 Native 3쪽·WASM 2쪽이었다. 표에는 양쪽 로컬 출력과 선행 생산자의 측정값과 해시를 함께 보존했다. [#9335 비교](upstream-regression-pages.tsv) · [#9350 Native](upstream-nine-native.tsv) · [#9350 WASM](upstream-nine-wasm.tsv)

Studio 검사 2,832개가 통과했고 기존 건너뛰기 항목은 2개였다. 실제 Chromium에서 입력 5개를 세 가지 렌더링 방식과 화면·인쇄 프로필로 열어 30개 결과를 확인했다. 한글 조합·붙여넣기 후 실행 취소 검사는 보정본 첫 실행에서 예상보다 글자를 더 지웠다. 같은 테스트와 각각 새로 빌드한 WASM으로 기준·보정본을 다시 실행해, 글꼴 목록이 없는 경우·빈 경우·채워진 경우 모두에서 조합과 붙여넣기 직후의 문자열이 실행 취소·다시 실행으로 복원되는 것을 확인했다. [소스·WASM 해시와 실제 편집 문자열](host-font-input.json)

검증 도중 devel에 병합된 #9359(`321ab8f977`)의 589개 변경 경로는 모두 `mydocs/`에 있었다. 검토 기록 `95b80ef88b`와 실제로 병합한 로컬 커밋은 `f6bc02a423`이고, 트리 `21d2287a9d49225822421cde0cd83ba94b1d35f8`은 병합 시뮬레이션 결과와 같다. `mydocs/` 밖의 파일은 제품 검증 소스 `c1504cb121`과 같았다. 새 devel과 실제 병합본 두 곳에서 편람 기준 검사 21건·기여자 공통 보호 검사 9건이 각각 통과했다. [새 글꼴 목록과 보호값 대조](font-supply.md)

## Rust 검증

Fontconfig에 저장소의 공개 글꼴 5개만 노출해 전체 회귀 14,017개, 집중 회귀 25개가 통과했다. 전체 회귀에서 건너뛴 항목은 45개다. Native Skia 라이브러리 검사 4,033개, 누락 그림 대조 2개, 직접 PDF 검사 37개가 통과했다. Release 라이브러리 검사는 3,976개, 문서 예제는 8개가 통과했다. 라이브러리 검사에서 명시적으로 무시한 항목은 각 프로필에서 12개, 문서 예제에서는 3개였다. 포맷 검사, 기본·WASM·워크스페이스 전체 타깃 Clippy, 워크스페이스 빌드와 Release 빌드도 성공했다. [실행 명령·결과·로그 해시](local-checks.json)
