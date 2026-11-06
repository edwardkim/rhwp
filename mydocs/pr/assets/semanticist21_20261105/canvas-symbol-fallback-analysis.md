# PR #8616 실제 Studio 기호 공급 보정

## 원인과 독립 근거

2026-11-06 KST, [원 PR #8616](https://github.com/edwardkim/rhwp/pull/8616)의 글자 겹치기
14모양을 실제 Studio Canvas2D·CanvasKit의 screen/print에서 확인했다. 원 PR 적용 커밋
`2fb1103d8195261593769fef5255cc9f5a1aaee2`의 이전은 모양 0~4만 지원한다. 추가 모양 13은
`src/model/control.rs`의 U+267B BLACK UNIVERSAL RECYCLING SYMBOL이다.
같은 원본 `samples/char_overlap/hancom-mac-shapes.hwpx`와 독립
`samples/char_overlap/hancom-mac-shapes.pdf`의 해당 기호는 검정이며 전각 테두리 크기다.

기존 실제 Canvas2D는 `#000000`으로 paint를 요청해도 자동 컬러 이모지를 선택했다.
`Batang, AppleMyungjo, Noto Serif KR` 다음에 바로 자동 이모지 후보가 있었다.
Native/fresh WASM 직접 exporter가 100.00%라는 사실만으로 Studio의 별도 CSS 소비도
같다고 판정하지 않고, 실제 Studio 화면을 추가 대조해 발견했다.

## 생산부터 실제 소비까지

`font-substitution.ts`의 공통 후보 생성 → `wasm-bridge.ts`의 로컬 승인·별칭 필터 →
CSS 후보 병합 → 실제 `CanvasRenderingContext2D.font` → `fillText`를 추적했다.
누락 후보를 마지막에만 붙이면 기존 generic 뒤에 들어가 소비되지 않을 수 있었다.
원래 요청·별칭 순서를 유지하면서 빠진 후보를 다음 공통 후보 앞에 넣도록 공통 병합을 보정했다.
로컬 글꼴 승인 필터, 문서의 명시적 치환, 명시적 컬러 이모지의 첫 후보 계약은 유지했다.
문서 ID나 특정 문자만 분기해 크기를 조정하지 않았다.

## 프로그램 coverage와 크기 대조

- 기존 공개 Symbols2에는 U+267B cmap이 없다.
- 기존 D2Coding에는 `uni267B`가 있지만 em 1000에서 전진 500·잉크 경계 `(0,60,500,542)`인
  반각 설계다. 이를 앞에 공급한 R7은 검정은 되었지만 Print보다 작았다. 확대 판독에서 발견해
  완료 근거로 채택하지 않고 R8에서 보정했다.
- 이미 초기 필수로 공급하던 함초롬바탕의 기존 CDN 프로그램에는 U+267B가 있고, em 1000에서
  전진 970·잉크 경계 `(66,-24,903,794)`다. 독립 Print의 글자 경계 높이 8.137329pt와 원 테두리
  높이 9.442078pt도 별도 PDF 문자 상자 읽기로 확인했다.
- 최종 일반 본문 후보는 기존 본문 face 뒤에서 Symbols2 → 기존 함초롬바탕 → D2Coding →
  자동 이모지를 찾는다. 함초롬바탕을 D2Coding보다 먼저 소비해 전각 모양을 보존한다.
- `font-loader.ts`의 필수 글꼴 목록은 기존 3개 그대로다. 새 초기 공급·다운로드 URL·글꼴 프로그램을
  추가하지 않았다. 기존 CDN 경로의 내용은 로컬 분석에만 사용하며 Git에 배포하지 않는다.
  한컴 폰트와 공개 배포 폰트의 경계는 `assets/fonts/FONTS.md`·`THIRD_PARTY_LICENSES.md`를 유지한다.

분석한 기존 CDN 프로그램은 `https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff`이며,
SHA-256 `b3192018e3bd4da76b29b0530f71c79fb6f465a2b7cd41308bc38f9a4ae09a6c`다.
이 프로그램의 다운로드·coverage 확인을 새 라이선스나 재배포 허가로 표현하지 않는다.

## 회귀와 최종 실제 실행

- 후보 누락의 수정 전 실제 동작 2건 FAIL → 공통 후보·별칭·명시적 이모지 순서 5건 PASS.
- 기존 후보 문자열 3건은 추가 후보 순서를 반영했다. 기존 승인·설치 별칭 보호는 유지했다.
- 기존 음성 대조의 주입 위치는 새 병합 코드에 대응시켰으며 `ENGINE_ALIAS_LOST` 검출 목적을
  유지했다. 후보·제품 경로·음성 대조·로더 수명을 포함한 최종 집중 Node 40건 PASS다.
- 최종 소스의 전체 Node 2,744건 PASS·실패/skip 0, production build exit 0이다.
- 실제 Canvas2D·CanvasKit compatibility/default × screen/print 12캡처와 기존 웹폰트 로딩
  브라우저 7시나리오도 exit 0이다. 겹침 6캡처는 색상 픽셀 0, 덧말 6캡처는 원래 파란색을 보존한다.
- 새 `char-overlap-monochrome-fallback.test.mjs`는 실제 PageRenderer의 페이지 잉크와 색상,
  실제 paint의 공개 후보·순서를 검사한다. 최소 시스템 글꼴 5개만 보이는 환경에서도 exit 0이다.
  특정 시스템 폰트의 설치나 정확한 glyph ink 높이를 기대값으로 고정하지 않는다.
  최소 환경의 fallback 자형을 한컴 Print 시각 검증으로 표현하지 않는다.
- 올바른 시각 글꼴 환경에서 최종 Studio Canvas2D와 독립 Print를 전쪽 비교한 결과,
  겹침 HWPX 1쪽·덧말 HWPX 1쪽 모두 raw 100.00000%다. 전체 쪽수도 각각 1쪽으로 일치한다.
  [최종 같은 페이지 review·overlay·TSV와 직접 확대 판독](studio-symbol/README.md)을 보존했다.
- 두 원본의 Native/fresh WASM 직접 exporter raw도 각각 100.00%다.
  [원본·PDF SHA와 각 전체 TSV](validation-r6/measurements.md)에 연결한다.

## 검증 재사용 경계

Rust source `86709ac16711820e61baa3b87e2c4ddaac23573e`, Native binary와 최적화 WASM
SHA는 그대로다. 직접 PNG exporter는 수정한 Studio CSS 후보·병합을 사용하지 않는다.
따라서 128입력·301쪽 Native/fresh WASM과 보호 PR #8389/#8397의 기존 실제 비교를 재사용했다.
실제 Studio 소비 경로와 Node·production build·로딩 수명은 바뀐 소스에서 별도로 재검증했다.
원본 문서·PDF·기대 좌표·기준선·시각 판정 스크립트를 변경하지 않았다.
