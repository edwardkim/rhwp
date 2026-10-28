# 작업 2493 단계 2 - 브라우저 확장 원격 HML fetch 완결

## 범위

- #2511이 노출한 원격 HML link 경로를 Safari의 별도 background 구현까지 완성한다.
- 기존 보안 gate를 보존한다. 즉 `.hml` suffix만으로 임의의 HTML이나 JSON 응답을 허용해서는 안 된다.

## 발견

- Chrome과 Firefox는 binary signature gate 없이 URL policy를 적용하므로 #2511 뒤 원격 HML 경로가
  통과한다.
- Safari background fetch validation은 HWP CFB와 HWPX ZIP signature만 받아들인다.
- HML은 XML이므로 유효한 원격 HML 문서는 viewer URL까지 도달하지만 WASM HML parser가 bytes를 받기
  전에 거부된다.

## 검증 계획

1. 공유 document signature policy에 Rust parser와 호환되는 HML prefix 검사를 추가하고 Safari
   background script보다 먼저 불러온다.
2. 유효한 HML과 HTML/JSON 위장 입력에 대한 회귀 검증을 추가한다.
3. 확장을 설치하거나 publish하지 않고 extension build와 영향받는 테스트를 실행한다.

## 결과

- `rhwp-shared/security/file-signature.js`는 제한된 UTF-8/UTF-16 prefix가 nonempty `Version`
  attribute가 있는 `HWPML` root로 해독될 때만 HML을 인식한다.
- Safari는 `background.js`보다 먼저 helper를 불러와 `dist`에 복사하고, 자동 URL policy에서 `.hml`을
  허용한다.
- 회귀 검증은 UTF-8, UTF-16, 저장소의 HML sample 두 개를 수용하며 HTML, JSON, 유효하지 않은 HWPML
  lookalike는 거부한다.
- Chrome과 Firefox production build는 완료됐다. Safari `dist` 생성도 완료됐으며, signed build는
  로컬 Mac certificate 때문에 막혀 있지만 `CODE_SIGNING_ALLOWED=NO` Xcode build는 성공했다.
