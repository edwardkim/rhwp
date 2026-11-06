# PR #8616 실제 Studio 기호 색상 보정

## 원인과 독립 근거

2026-11-06 KST, [원 PR #8616](https://github.com/edwardkim/rhwp/pull/8616)의 글자 겹치기
14모양을 실제 Studio Canvas2D·CanvasKit의 screen/print에서 확인했다. 원 PR을 적용한
`2fb1103` 이전은 모양 0~4만 지원하며, 이 PR의 추가 모양 13은
`src/model/control.rs`의 U+267B BLACK UNIVERSAL RECYCLING SYMBOL이다.
같은 원본 `samples/char_overlap/hancom-mac-shapes.hwpx`와 독립
`samples/char_overlap/hancom-mac-shapes.pdf`의 해당 기호는 검정이다.

수정 전 실제 Canvas2D는 `#000000`으로 paint를 요청했지만 컬러 이모지로 그렸다.
`Batang, AppleMyungjo, Noto Serif KR` 다음에 바로 자동 이모지 후보가 와서 생긴 차이다.
CanvasKit과 독립 Print는 검정이었다. 기존 Native/fresh WASM 직접 출력 100.00%만으로
Studio의 별도 CSS 후보 소비도 정확하다고 판단하지 않았다.

## 생산 결과부터 실제 소비까지

`font-substitution.ts`의 공통 후보 생성 → `wasm-bridge.ts`의 로컬 글꼴 승인·별칭 필터 →
CSS 후보 병합 → 실제 `CanvasRenderingContext2D.font` → `fillText`를 추적했다.
누락한 후보를 단순히 마지막에 붙이면 기존 generic 뒤에 들어가 선택되지 않을 수 있었다.
기존 요청·별칭 순서를 유지하면서 빠진 후보를 다음 공통 후보 앞에 삽입하도록 보정했다.
로컬 글꼴 승인 필터와 문서의 명시적 대체 정책은 유지했다.

기존 공개 Symbols2의 cmap에는 U+267B가 없다. 기존 공개 D2Coding Regular/Bold의 cmap에는
`uni267B`가 있으므로, Symbols2 다음·자동 이모지 앞에 D2Coding을 함께 공급했다.
명시적으로 컬러 이모지를 요청한 경우 원래 face가 첫 후보인 계약은 유지한다.
새 글꼴 프로그램을 추가하지 않았다. 기존 `THIRD_PARTY_LICENSES.md`와
`assets/fonts/licenses/D2Coding-OFL.txt`, `NotoSansSymbols2-OFL.txt`의 공개 공급을 재사용한다.

## 회귀 검사와 실제 실행

- 후보 누락의 실제 기존 동작 2건 FAIL → 공통 순서·별칭·명시적 이모지 계약 5건 PASS.
- 기존 후보 문자열 검사 3건은 새 공개 후보 순서를 반영했다. 기존 글꼴 승인·별칭 검사는 유지했다.
- 기존 음성 대조의 코드 주입 위치는 새 병합 형태로 옮겼다. 원래의 `ENGINE_ALIAS_LOST`
  검출 목적을 유지했고, 제품 경로·음성 대조를 포함한 집중 Node 26건이 통과했다.
- 새 `char-overlap-monochrome-fallback.test.mjs`는 실제 Studio PageRenderer에서 전체 페이지를
  읽는다. 겹침 문서의 원래 검정 내용은 색상 픽셀 0, 덧말 문서의 원래 파란 내용은 색상 픽셀
  양수로 검사한다. 특정 좌표·영역·시스템 글꼴 존재 여부에 의존하지 않는다.
- 최소 공개 시스템 글꼴 5개만 보이는 bwrap 환경에서 실제 브라우저 검사를 실행해 exit 0을
  확인했다. 겹침의 색상 픽셀 0·잉크 2,387, 덧말의 색상 픽셀 98·잉크 1,821이었다.
  로컬 비공개 OTF 없이 기존 공개 웹 글꼴을 실제로 공급했다.
- 두 원본 HWPX는 각 1쪽이며 Native/fresh WASM 직접 출력과 독립 Print의 raw는 각각
  100.00%다. [현재 전쪽 원본 TSV·SHA](validation-r6/measurements.md)를 연결한다.

## 재사용 경계

Rust 제품 source `86709ac16711820e61baa3b87e2c4ddaac23573e`, Native binary와 최적화
WASM SHA는 그대로다. 직접 PNG exporter는 수정한 Studio 글꼴 로더·CSS 병합을 사용하지 않는다.
따라서 기존 128입력·301쪽 Native/fresh WASM과 보호 PR #8389/#8397 비교를 재사용하고,
바뀐 실제 Studio 소비 경로·전체 Node·production build·글꼴 로딩 수명 검사는 새로 실행한다.
전쪽 점수·쪽수와 실제 Studio 판독 결과를 혼동하지 않는다.

초기 필수 공급은 실제 U+267B를 제공하는 D2Coding만 추가해 기존 4개 파일 batch 안에서
끝나게 했다. 사용하지 않는 Symbols2까지 초기 필수로 추가했을 때 발생했던 다음 batch의
가상 타이머 대기는 수정했으며, 기존 5초 제한 기대값을 바꾸지 않고 로더 수명 14건을 통과했다.
현재 소스의 전체 Node는 2,744건 PASS·실패/skip 0, 실제 exit 0이다.
최종 브라우저 12캡처·production build 결과는 완료 후 검증 요약에 추가한다.
원본 PDF를 재출력하거나 원본 내용을 변경하지 않았다.
