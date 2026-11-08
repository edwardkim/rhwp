# sample10의 로컬 글꼴 브라우저 검증

2026-10-31에 리베이스한 제품 코드 `11def71519`의 fresh WASM을 사용했다. WASM SHA-256은
`5ccf5d7efc0467488729124e95667c020e71671f29e4641f02b3555bc3637040`이며 `pkg`와 Studio의
WASM·JS 해시가 각각 같았다. 실제 HWP3 원본은 `samples/hwp3-sample10.hwp`, 한컴 2020 Print
PDF는 `pdf/issue8371/hwp3-sample10-2020.pdf`다.

headless Chromium의 Studio에서 딸기 OTF 공급 후 문서 열기, 감지 결과 초기화, 목록 검색,
승인 전 대체 글꼴, 브라우저 열거 누락, 현재 문서 글꼴 적용, 관련 글꼴 재감지 시 재조판,
무관한 글꼴 재감지 시 재조판하지 않음, 글꼴 동의·목록·개수 표시를 실제 실행해 통과했다.
공급 시험은 브라우저의 Local Font Access 응답을 테스트에서 대체하고 설치된 원본 파일의 바이트를
전달했으므로, 운영체제가 해당 face를 실제로 열거하는지에 대한 검증은 아니다.
OS 설치 글꼴에 기대지 않는 동봉 fixture의 HFT 윤곽 브라우저 검사도 통과했다.
실제 로컬 공급 시험의 딸기 `HGSTRBRY.otf` SHA-256은
`6844aed54084783e28e7be0da351d08f4c0fe9e16e0844cbad4e4164c47488ca`, 제목의
`TEDNREN.otf`는 `d292d1972ff9de8641c52e3bff41fa0d5ba012c3fd0a1b6514f49e4a5f29fced`다.
글꼴 파일 자체는 포함하지 않는다.

## CanvasKit의 실제 1쪽 재생과 Studio 선택 경계

이 763쪽 문서는 기존 CanvasKit 문서 preflight의 128쪽 상한을 넘는다. 따라서 Studio에서
CanvasKit을 명시해도 기존 정책에 따라 Canvas2D로 전환된다. 전체 문서에서 CanvasKit이
선택됐다고 주장하지 않는다.

같은 원본의 1쪽 레이어를 실제 CanvasKit 렌더러에 직접 전달해 두 로컬 OTF를 준비했다.
딸기 글자와 제목 글자의 실제 trace에서 로컬 face 선택을 확인했고 지원되지 않은 replay 연산은
없었다. 동일 한컴 Print 1쪽과 96 DPI로 대조한 보조 실루엣 raw는 **99.97%**다.
[review](sample10-canvaskit-p001-review.png)와 [overlay](sample10-canvaskit-p001-overlay.png)를
직접 확인했다. 이는 CanvasKit의 한 쪽 소비 경로 확인이며 전체 페이지 WASM TSV를 대체하지 않는다.
딸기만 공급한 초기 진단의 89.02%는 제목 face가 준비되지 않은 상태였으므로 완성 검증으로
사용하지 않고, 두 원본 face를 공급한 뒤 같은 페이지를 다시 측정했다.
