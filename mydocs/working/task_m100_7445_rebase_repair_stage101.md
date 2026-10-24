# Stage101: 두 usvg 타입 소비자의 공통 소스 lint 계약

Issue: #7445. 제품 실행 HEAD는 `1f54c3df17747b71c8be9b47b3198add4fa4fd3b`,
고정 base는 `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다.

## 진단 원인과 보정 범위

최종 전체 회귀 뒤 Native Clippy가 `duplicate_mod` 두 건을 검출했다. 메타파일은
`resvg::usvg` 0.47.0, Native PDF는 `usvg` 0.45.1의 실제 Tree/Options/fontdb 타입을
사용한다. 두 소비자는 이미 같은 `pdf_synthetic_bold.rs`와 `pdf_synthetic_italic.rs`를
`super::style_usvg`로 인스턴스화한다. 모듈을 단순 `use`로 합치면 다른 버전의 구체 타입을
선택하게 된다. 판정 구현을 복제하거나 의존성 버전을 바꿀 이유가 아니다.

메타파일의 두 모듈 선언에만 Native 조건의 `#[expect(clippy::duplicate_mod, reason = …)]`를
명시했다. crate 전체의 경고를 끄지 않으며 공유한 함수·선택 규칙·테스트·의존성을 바꾸지 않는다.
WASM은 선언이 하나이므로 해당 expectation을 적용하지 않는다.
[Clippy의 해당 lint 구현](https://raw.githubusercontent.com/rust-lang/rust-clippy/rust-1.93.0/clippy_lints/src/duplicate_mod.rs)에서
개별 선언의 expectation을 처리하는 경로도 확인했다.

## source와 기존 실행 증거의 관계

전쪽 및 실제 소비 경로 검사와 이전 Cargo 실행이 모두 끝난 뒤 보정했다. 변경 전 파일 SHA는
`7b87a2d2ca815205989abf9da355b3a67200bd4d1ae89d6a3a9a9b14a3b50c86`, 변경 후는
`a72b10b2d2c7b382b8147cb1d2ba671b2e259a7830872cd58c50265580258a96`이다.
두 진단 속성을 제거하면 파일 전체 바이트가 변경 전과 정확히 같음을 확인했다.
다른 Rust 제품·테스트·입력·WASM 설정 변경은 없다.

따라서 제품부 HEAD의 전체 회귀·전쪽 출력은 진단 보정 이후에 새로 실행한 결과라고 재표기하지
않는다. 동일한 실행 함수·입력·출력 조건의 기존 증거로 연결하고, 진단 속성은 실제 Native/WASM
컴파일 경로에서 별도로 검증했다. source/body 동일성 기록은 ignored output의
`stage101-dual-usvg-lint/source-applied.json`에 남긴다.

## 보정 후 실제 결과

| 검사 | 결과 |
| --- | --- |
| rustfmt check | PASS |
| Native Clippy, `-D warnings` | PASS |
| WASM32 Clippy, `-D warnings` | PASS |
| workspace all-target Clippy, `-D warnings` | PASS |
| native-skia all-target Clippy, `-D warnings` | PASS |
| 기존 두 소비자의 bold·italic·글꼴 세션 회귀 | 21 PASS / 0 FAIL |

마지막 검사는 현재 manifest로 다섯 논리 case의 실제 suite를 계산해 release-test/native-skia,
빌드 jobs 8·테스트 threads 8로 실행했다. 1,268개 필터 제외를 전체 회귀의 skip에 합산하지 않는다.
가짜 굵기/기울기의 glyph 선택·기준선·advance, 실제 italic의 이중 shear 방지, 실제 Native PNG 및
메타파일 글꼴 등록/해제의 문서·저장 바이트 보존을 확인했다. 새 렌더링 검사나 fixture를 추가하지 않았다.

원 실패와 최종 명령·종료 코드·시간은 ignored output의 `stage95-final-gates/clippy-native.log`와
`stage101-dual-usvg-lint/results.json` 및 각 로그에 보존한다. 이 보정은 시각 문턱이나 품질 검사를
완화하는 예외가 아니다.
