# PR #8340 메인터너 검증 증적

중첩 표 문단 정렬·간격과 Undo/Redo, 저장 왕복을 검증했다. 공개 원본 두 개, 원 PR 복원본 두 개,
편집을 유지한 API 저장본 14개와 실제 Studio 저장본 두 개로 **20개 상태**를 비교했다.
Native/fresh WASM의 전쪽 raw 실루엣 최저값은 각각 **100.00%**, 총 쪽수는 모두 독립 Print와
1/1쪽으로 같다. 90% 미만·누락·2%p 초과 하락은 없다.

이 값은 96 DPI, 2px 관용 실루엣 값이다. PDFium·Skia·Canvas의 RGB/안티앨리어싱 차이가 있으며
RGB 픽셀 전체가 동일하다는 뜻은 아니다. 원본은 수동 XML로 작성된 공개 HWPX이고 한컴에서
처음 저장한 문서라는 주장은 하지 않는다.

## 입력과 전체 페이지 TSV

| 상태 | 실행한 입력 | 독립 기준 | Native 전쪽 | fresh WASM 전쪽 |
| --- | --- | --- | --- | --- |
| same-line | [입력](../../../../tests/fixtures/issue7419-boundaries/nested-same-line.hwpx) | [Print](../../../../pdf/issue7419/nested-same-line-print-2024.pdf) | [100.00%](scores/candidate/native/nested-same-line.tsv) | [100.00%](scores/candidate/wasm/nested-same-line.tsv) |
| same-line-restored | [입력](../../../../mydocs/pr/assets/nested-cell-paragraph-format/nested-same-line-restored.hwpx) | [Print](../../../../pdf/issue7419/nested-same-line-print-2024.pdf) | [100.00%](scores/candidate/native/nested-same-line-restored.tsv) | [100.00%](scores/candidate/wasm/nested-same-line-restored.tsv) |
| width-wrap | [입력](../../../../tests/fixtures/issue7419-boundaries/nested-width-wrap.hwpx) | [Print](../../../../pdf/issue7419/nested-width-wrap-print-2024.pdf) | [100.00%](scores/candidate/native/nested-width-wrap.tsv) | [100.00%](scores/candidate/wasm/nested-width-wrap.tsv) |
| width-wrap-restored | [입력](../../../../mydocs/pr/assets/nested-cell-paragraph-format/nested-width-wrap-restored.hwpx) | [Print](../../../../pdf/issue7419/nested-width-wrap-print-2024.pdf) | [100.00%](scores/candidate/native/nested-width-wrap-restored.tsv) | [100.00%](scores/candidate/wasm/nested-width-wrap-restored.tsv) |
| same-line-center-hwpx | [입력](inputs/nested-same-line-center.hwpx) | [Print](pdf/nested-same-line-center-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-same-line-center-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-same-line-center-hwpx.tsv) |
| same-line-right-hwpx | [입력](inputs/nested-same-line-right.hwpx) | [Print](pdf/nested-same-line-right-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-same-line-right-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-same-line-right-hwpx.tsv) |
| same-line-before-hwpx | [입력](inputs/nested-same-line-before.hwpx) | [Print](pdf/nested-same-line-before-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-same-line-before-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-same-line-before-hwpx.tsv) |
| same-line-line-hwpx | [입력](inputs/nested-same-line-line.hwpx) | [Print](pdf/nested-same-line-line-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-same-line-line-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-same-line-line-hwpx.tsv) |
| same-line-before-line-hwpx | [입력](inputs/nested-same-line-before-line.hwpx) | [Print](pdf/nested-same-line-before-line-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-same-line-before-line-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-same-line-before-line-hwpx.tsv) |
| same-line-before-line-hwp | [입력](inputs/nested-same-line-before-line.hwp) | [Print](pdf/nested-same-line-before-line-hwp-print-2024.pdf) | [100.00%](scores/candidate/native/nested-same-line-before-line-hwp.tsv) | [100.00%](scores/candidate/wasm/nested-same-line-before-line-hwp.tsv) |
| same-line-indent-hwpx | [입력](inputs/nested-same-line-indent.hwpx) | [Print](pdf/nested-same-line-indent-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-same-line-indent-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-same-line-indent-hwpx.tsv) |
| width-wrap-center-hwpx | [입력](inputs/nested-width-wrap-center.hwpx) | [Print](pdf/nested-width-wrap-center-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-width-wrap-center-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-width-wrap-center-hwpx.tsv) |
| width-wrap-right-hwpx | [입력](inputs/nested-width-wrap-right.hwpx) | [Print](pdf/nested-width-wrap-right-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-width-wrap-right-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-width-wrap-right-hwpx.tsv) |
| width-wrap-before-hwpx | [입력](inputs/nested-width-wrap-before.hwpx) | [Print](pdf/nested-width-wrap-before-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-width-wrap-before-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-width-wrap-before-hwpx.tsv) |
| width-wrap-line-hwpx | [입력](inputs/nested-width-wrap-line.hwpx) | [Print](pdf/nested-width-wrap-line-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-width-wrap-line-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-width-wrap-line-hwpx.tsv) |
| width-wrap-before-line-hwpx | [입력](inputs/nested-width-wrap-before-line.hwpx) | [Print](pdf/nested-width-wrap-before-line-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-width-wrap-before-line-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-width-wrap-before-line-hwpx.tsv) |
| width-wrap-before-line-hwp | [입력](inputs/nested-width-wrap-before-line.hwp) | [Print](pdf/nested-width-wrap-before-line-hwp-print-2024.pdf) | [100.00%](scores/candidate/native/nested-width-wrap-before-line-hwp.tsv) | [100.00%](scores/candidate/wasm/nested-width-wrap-before-line-hwp.tsv) |
| width-wrap-indent-hwpx | [입력](inputs/nested-width-wrap-indent.hwpx) | [Print](pdf/nested-width-wrap-indent-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-width-wrap-indent-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-width-wrap-indent-hwpx.tsv) |
| same-line-studio-right-hwpx | [입력](inputs/nested-same-line-studio-right.hwpx) | [Print](pdf/nested-same-line-studio-right-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-same-line-studio-right-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-same-line-studio-right-hwpx.tsv) |
| width-wrap-studio-right-hwpx | [입력](inputs/nested-width-wrap-studio-right.hwpx) | [Print](pdf/nested-width-wrap-studio-right-hwpx-print-2024.pdf) | [100.00%](scores/candidate/native/nested-width-wrap-studio-right-hwpx.tsv) | [100.00%](scores/candidate/wasm/nested-width-wrap-studio-right-hwpx.tsv) |

base의 전쪽 TSV와 정밀 metric 원본도 `scores/base/{native,wasm}/`에 있다.
[상태별 입력·PDF·SHA-256](inputs.json), [페이지별 정밀값·base 비교](visual-summary.json),
[빌드·글꼴·도구 출처](producer.json), [검사 결과](validation.json)를 함께 확인한다.

## 편집 저장본의 독립 Print

[Print 실행 기록](print-execution.json)의 16건은 실제 실행한 저장본을 새 한컴 2024에서 출력했다.
모든 결과는 13.0.0.3901, PrintMethod 0, `frame_print_to_pdf_ex_one_up`, 1쪽이다. 다운로드 SHA-256을
검증했으며 원격 주소·토큰·결과 capability는 포함하지 않는다. 변환 서비스는 일반 기여자의 필수
조건이 아니며 기존 제출 증적과 새 메인터너 검증을 구분한다.

API 입력은 `inputs.json`의 원본·최내곽 `targets`에 `props`를 적용한 뒤 실제 HWP/HWPX로 저장했다.
가운데/오른쪽 정렬, 문단 앞 간격 600, 줄간격 200, 두 값 동시 적용, 여백·내어쓰기를 포함한다.
margin/indent는 2배 HWPUNIT이며 속성 조회의 dialog px와 입력 단위를 혼동하지 않는다.
두 HWP 저장본도 같은 인쇄 경로로 확인했다. [편집 중/정확한 저장본의 geometry 대조](live-save-geometry.json)는
16건 모두 글 순서·수량·쪽수와 텍스트 bbox가 같았다. 이 관계 검사는 독립 Print의 대체 증거로 쓰지 않았다.

## 실제 Studio와 확대 판독

실제 파일 입력으로 열고 로컬 글꼴 감지에서는 대체 글꼴 보기를 선택했다. 안쪽 표에 캐럿을 두고
도구 모음 가운데 정렬, 키보드 Undo/Redo, F5 세 번으로 해당 안쪽 표의 두 칸 선택, 오른쪽 정렬을
실행했다. 선택하지 않은 다른 중첩 표의 문단 속성은 보존됐다. UI 조작 증적과 인쇄 fidelity 검증의
글꼴 환경을 구분한다. 인쇄 비교에는 명시적인 HCRBatang 1.980 프로그램을 공급했다.

- [Studio same-line](images/nested-same-line-studio-right.png) / [Studio width-wrap](images/nested-width-wrap-studio-right.png)
- [오른쪽 정렬 확대](images/zoom-nested-same-line-right-hwpx.png)
- [앞 간격·줄간격 확대](images/zoom-nested-width-wrap-before-line-hwpx.png)
- [여백·내어쓰기 확대](images/zoom-nested-width-wrap-indent-hwpx.png)

확대 그림은 왼쪽부터 독립 PDF, Native, fresh WASM이다. 같은 좌표를 2배 확대한 것으로 점수
계산에는 사용하지 않았다. 글줄 시작·끝, 두 중첩 표의 포함·앞뒤 순서, 다음 표·문단 간격과
줄바꿈을 직접 대조했다. 내용 누락·중복·추가 겹침은 관찰하지 않았으며 수식은 없는 입력이다.
`images/`에는 canonical Visual Sweep의 Native/WASM review·overlay도 보존한다.

## 일반 시각 비교 보조값

각 상태의 1쪽을 Native/fresh WASM에서 직접 확대 판독했다. 일반 RGB 임계값과 잉크 영역
보조값도 [원 metric 추출값](normal-review-metrics.json)으로 남긴다. 2px raw 실루엣 gate와
구분하며 낮은 잉크 보조값을 숨기거나 RGB 전체 동일로 해석하지 않는다.

| 상태 | Native pixel / ink proxy (%) | WASM pixel / ink proxy (%) |
| --- | --- | --- |
| 오른쪽 정렬 | 98.76188 / 34.83013 | 98.92588 / 42.50781 |
| 앞 간격·줄간격 | 98.61955 / 27.20923 | 98.90534 / 39.37962 |
| 여백·내어쓰기 | 98.69161 / 31.41109 | 98.92869 / 41.98882 |

모든 페이지의 raw는 100.00%다. 같은 영역 확대에서 글줄 경계·기준선·내용과 소유·간격을 확인했고
RGB/안티앨리어싱 차이 외 새 결함은 관찰하지 않았다.

## 원 기여와 메인터너 보정

원 기여의 Rust·Studio 구현은 보존했다. 메인터너는 새 API 세 개가 빠진 public TypeScript 선언을
검증한 package에서 가져왔다. 원 PR 제출용 보정 `3fa96a6e8d`는 원 head 바로 위에 클래스 메서드
세 개·InitOutput 세 개·raw WASM export 세 개, 총 9줄만 추가한다. 기존 서명·순서는 그대로다.
최신 devel 이력은 contributor fork에 섞지 않는다. [제출 후보의 merge-tree 대조](submission-candidate.json)에서
검증한 통합 후보와 모든 runtime source/test blob이 같고 생성 선언의 순서만 다름을 확인했다.
최종 제출용 선언을 직접 import한 소비자 컴파일도 통과했다.
[네 산출물 동기화](binding-sync.json), [새 base 기능 보존](base-feature-preservation.json),
[기준 WASM의 실제 source 동등성](base-wasm-source-equivalence.json)을 확인했다.

[네 산출물 동기화 기록](binding-sync.json)은 최초 로컬 통합 producer의 해시다. 원 PR 제출용
최소 선언 보정은 최신 base와 병합하면 같은 서명 집합이 되며 기존 생성 순서만 보존한다.

기준/후보는 동일한 원본·Print·글꼴·DPI로 실행했다. Runtime producer는 `producer.json`의
source/base를 clean merge한 결과다. 이후 선언 동기화는 Rust runtime 소스를 바꾸지 않았으며
실행한 JS/WASM/Native 파일 해시를 그대로 기록했다. 새 보정 head의 원격 CI는 아직 실행 전이다.

#8138의 구조 편집 명령과 중첩 셀 스타일 전체 지원은 이번 PR의 해결 범위가 아니다.

## 공개 타입 소비 검사의 재현

저장소의 `output/pr-review/pr8340/public-types-probe.ts`에 다음 내용을 둔다.

```typescript
import type { HwpDocument } from '../../../rhwp-studio/public/rhwp.js';
declare const doc: HwpDocument;
doc.getCellParaPropertiesAtByPath(0, 0, '[]');
doc.applyParaFormatInCellByPath(0, 0, '[]', '{}');
doc.setCellParaShapeIdByPath(0, 0, '[]', 0);
```

`rhwp-studio`에서 `npx tsc --ignoreConfig --noEmit --skipLibCheck --module nodenext
--moduleResolution nodenext --target es2022 ../output/pr-review/pr8340/public-types-probe.ts`를 실행한다.
수정 전에는 세 메서드가 모두 TS2551이었고, 동기화 후에는 성공했다. 빈 경로를 실행하는 검사가
아니라 공개 선언을 소비하는 컴파일 검사다.

전체 Studio 회귀도 [최소 글꼴 환경](font-isolation.json)에서 재실행해 2,713건 통과·기존 2건
건너뜀을 확인했다. 독립 Print에 맞추는 HCRBatang 프로그램과 개인 글꼴 경로는 이 회귀 환경에서
노출하지 않았다. 실행 로그·글꼴 프로그램·WASM 바이너리는 이 증적 디렉터리에 포함하지 않는다.
