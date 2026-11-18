# 실제 선택된 분할 HFT 프로그램의 paint 소유 보정

저장된 HFT 이름 `명조`는 실제 환경에서 `Myeongjo`로 선택되지만, 기존 윤곽 등록은 저장 이름과
일치할 때만 허용했다. 같은 family의 한글·영문 CFF 조각도 첫 face 하나만 등록하여 나머지는
OS별 FontFace 래스터에 남았다. 원 해결 PR과 통합 근거는 정본 보호 TSV manifest의 동일 입력·PDF에
연결한다. PDF·글꼴·DPI·문턱을 바꾸지 않았다.

문서 언어·글꼴 슬롯의 실제 환경 선택을 등록 경계에 적용하고, 성공한 FontFace와 같은 프로그램을
같은 후보 순서로 보관한다. 첫 cmap 소유 face의 윤곽만 소비하며, 혼합 형식·미지원 윤곽은 기존
painter에 맡긴다. 저장 IR·조판 전진을 바꾸지 않는다. 후보 수·프로그램·세션 바이트 한도와
세대 초기화를 유지하고, 중복 프로그램은 같은 바이트 저장소를 공유한다.

## 실제 확인 결과

- 공개 합성 글꼴의 환경 선택 검사는 기존 제품 코드에서 의도한 원인으로 **1 FAIL**을 확인했다.
  후속 Unicode family 정규화 검사도 보정 전 **1 FAIL**, 현재 최소 글꼴 Native 집중 검사는
  **18 PASS / 0 FAIL**이다. 비 HFT·미선택 family를 거부하며 저장 바이트 보존과 후보 순서·한도·교체를 검사한다.
- Studio TypeScript 검사와 전체 **2,784 PASS / 0 FAIL / 2 skip**을 확인했다.
- 로컬 no-opt fresh WASM의 실제 Chrome 검사 **6 PASS**로 두 조각의 실제 path 소비,
  단일 공급·초기화·Screen/Print와 TTF 대조를 확인했다.
- [집중 비교 TSV](focus-wasm-comparison.tsv)는 기존 하락 10쪽만 재측정한 결과다.
  sample10 6쪽을 복원했으며 **sample10 251·509쪽과 sample16 HWP2024·HWPX 35쪽의 4쪽은 여전히 하락**한다.
  전체 문서 TSV나 PR 준비 완료로 확대하지 않는다.
- sample10 757쪽은 raw **99.41233%**, 공식 표시 **99.41%**로 보호값 **99.40%**를 넘는다.
  [review](sample10-wasm-p757-review.png)와 [overlay](sample10-wasm-p757-overlay.png)를 함께 직접 판독했다.
  [후보 raw](sample10-wasm-p757-raw.png)·[독립 PDF raw](sample10-p757-pdf-raw.png)에 본문·ASCII 도식·쪽 번호를 보존한다.

[검증·producer·hash](validation.json)에 측정한 WASM과 후속 Unicode 비교 보정 소스를 구분했다.
후속 소스 `785194d2df`의 fresh WASM도 다시 빌드해 실제 브라우저 **6 PASS**와 집중 10쪽 재출력을
완료했고, 10쪽 PNG 모두 앞선 측정과 바이트가 같았다. 전쪽·전체 회귀와 남은 하락 보정은 계속 필요하다. 비공개 글꼴과
윤곽 추출 진단 자료는 ignored output에만 보관하며 이 증적에 포함하지 않았다.

## 후속 프레임 보정

위 4쪽 잔여는 이 문서가 기록한 `785194d2df`의 결과다. 후속
[실제 CFF 합성 굵기 보정](../verified-cff-synthetic-bold-20261109/README.md)과
[공통 Print 프레임 복원](../source-print-frame-preservation-20261109/README.md)의 집중 재출력에서
모두 보호값 이상으로 복원했다. 이 후속 집중 결과도 전쪽·전체 회귀 완료를 뜻하지 않는다.
