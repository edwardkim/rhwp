# Task #2099 Stage 2 — 옛한글 자모 폰트 우선순위 보강

## 배경

Stage 1에서 U+F53A ``를 `ᄒᆞᆫ` 자모 시퀀스로 확장하도록 복구했다.

추가 검증 파일:

- `samples/한글문서파일형식_5.0_revision1.3.hwp`: 68쪽 배포본, 사용자 재현 화면과 동일 계열 파일.

## 확인

- 텍스트 추출 결과 1쪽 제목은 raw U+F53A ``로 저장되어 있다.
- Stage 1 적용 후 SVG에는 raw U+F53A가 남지 않고 `ᄒᆞᆫ`이 출력된다.
- 하지만 SVG/Canvas 폰트 체인에서 `Source Han Serif K Old Hangul`이 일반 고딕 폰트 뒤에 있어,
  브라우저가 옛한글 자모 클러스터를 먼저 일반 한글 폰트로 잡으면 화면상 아래아 조합이 깨질 수 있다.
- 같은 파일의 제목 폰트 `한컴산뜻돋움`은 웹폰트 매핑이 없어 영문 폭과 간격도 한컴 기준과 차이가 커진다.

## 수정 방침

- 렌더 시점에 옛한글 자모 클러스터를 감지하고, 해당 클러스터만 `Source Han Serif K Old Hangul`을
  font-family 체인의 최우선으로 배치한다.
- 일반 한글/영문 클러스터는 기존 font-family 체인을 유지해 일반 텍스트 영향 범위를 줄인다.
- `한컴산뜻돋움`을 기존 한컴 돋움 CDN 폰트 매핑에 추가해 사용자 재현 파일의 제목/영문 폭을 개선한다.

## 검증 계획

- `cargo test --test issue_2099_araea_pua`
- `cargo test --lib pua`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `wasm-pack build --target web --out-dir pkg`
- 사용자 재현 파일 1쪽 SVG에서 `ᄒᆞᆫ` 클러스터의 font-family가 `Source Han Serif K Old Hangul` 우선인지 확인.

## 검증 결과

- `samples/한글문서파일형식_5.0_revision1.3.hwp`는 68쪽 distribution 문서다.
- `export-text` 결과 1쪽 제목 첫 글자는 raw U+F53A ``로 저장되어 있다.
- `export-svg` 결과 raw U+F53A는 남지 않고 `ᄒᆞᆫ`으로 출력된다.
- `ᄒᆞᆫ` 클러스터의 SVG font-family는 `Source Han Serif K Old Hangul` 우선으로 출력된다.
- `localhost:7700` 브라우저 검증:
  - 68쪽 로드.
  - `한컴산뜻돋움` loaded.
  - `Source Han Serif K Old Hangul` loaded.
  - 콘솔 error/warn 0건.
  - 대표 제목/영문 라인 증적: `mydocs/pr/assets/pr_2128_issue2099_revision13_browser_title_crop.png`.
