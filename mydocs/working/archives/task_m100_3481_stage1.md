---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-27
---

# Task #3481 Stage 1 — HWP3 비밀번호 암호 문서 복호화

Issue: [#3481](https://github.com/edwardkim/rhwp/issues/3481)

## 확인한 형식과 범위

- 제공된 작업지시자 제작 참조 구현으로 실제 HWP3 암호 fixture의 복호화와 암호 플래그 해제를
  확인했다. 이 구현은 작업지시자 지시에 따라 MIT 기준으로 사용하며, 별도 라이선스 검토·고지는
  범위에 넣지 않는다.
- HWP3 문서 정보의 암호 여부는 오프셋 96, 압축 여부는 오프셋 124, 정보 블록 길이는 오프셋
  126에 있다. 암호 본문은 정보 블록 뒤부터 시작한다.
- 이번 범위는 압축 HWP3 암호 본문만이다. UTF-16LE 비밀번호→DES 키 유도, DES-ECB 복호화, raw
  DEFLATE와 CRC32/ISIZE trailer 검증을 수행한다. 비압축 HWP3 암호 본문은 명시적으로 거부한다.
- 실제 fixture의 raw DEFLATE 평문 앞 256바이트는 HWP3 암호 확인용 prefix이며, 글꼴 테이블은
  그 직후부터 시작한다. 독립 복호화 도구처럼 prefix를 보존한 파일을 일반 parser에 그대로 넘기면
  글꼴 수를 잘못 읽으므로, rhwp 내부 열기 경로에서는 검증 뒤 prefix를 제외한 본문만 전달한다.
- HWP3 전용 복호화·판별은 `src/parser/hwp3/`에만 둔다. 상위 공통 parser는 HWP3도 기존
  `parse_document_with_password` 경로로 전달하며, Studio는 기존 `openWithPassword` API와
  암호 대화상자를 재사용한다.

## 안전성 계약

- 비밀번호, 원문 암호문, 복호화 평문을 로그·문서·최근 문서·저장소에 보관하지 않는다.
- 오입력과 손상된 암호문은 동일한 일반 오류로 처리하고, raw DEFLATE 확장 결과에는 HWP5와 같은
  512 MiB 상한을 적용한다.
- 복호화한 HWP3는 원본이 암호 문서였다는 IR 메타데이터만 유지하고, HWP 저장은 기존 정책대로
  평문 HWP를 만든다.

## 검증 계획

1. HWP3 crypto 단위 테스트: 키 유도, 정상 복호화, 오입력, 압축 해제 상한.
2. 실제 fixture 통합 테스트: 무암호 거부, 오입력 거부, 성공 열기, 공용 `HwpDocument`, CLI stdin,
   저장 후 평문 재열기.
3. 기존 Studio의 범용 암호 열기 계약과 실제 HWP3 fixture E2E를 최신 WASM으로 확인한다.
4. Rust format·focused test·release-test 전체 테스트·IR field sweep을 실행하고, fixture 결과가
   baseline에 새로 기록돼야 하면 근거와 함께 갱신한다.

## 구현 및 현재 검증 결과

- `src/parser/hwp3/crypto.rs`에 HWP3 전용 키 유도·DES-ECB·raw DEFLATE/trailer·512 MiB 상한을
  추가했다. 오입력과 손상은 기존 HWP5와 같은 일반 오류로 귀결되며, 비밀번호는 호출 범위 밖에
  보관하지 않는다.
- HWP3 암호 플래그가 있는 일반 열기는 `ParseError::EncryptedDocument`로 매핑하고, 공용
  `parse_document_with_password` → `DocumentCore` → WASM `openWithPassword` 경로에서만
  복호화한다. Studio의 기존 대화상자·재입력·비보존 계약을 HWP3에도 적용한다.
- 실제 fixture는 1구역·24페이지·365문단·내장 이미지 2개로 열리고, HWP 저장 뒤에는 비밀번호 없이
  재열린다. HWP3 crypto 단위 테스트 4건과 실제 fixture Rust 통합 테스트 2건을 통과했다.
- Studio E2E는 실제 HWP3와 HWP5 fixture 모두에 대해 취소·오입력·Enter 성공·저장소 비보존을
  같은 계약으로 검증하도록 확장했다. 암호 입력이 속한 모달을 기준으로 접근성 속성을 읽어 다른
  비동기 모달과의 선택자 혼입도 막았다.
- 새 fixture를 포함한 IR field sweep은 803개 샘플에서 통과했고, 생성 TSV는 기존
  `tests/fixtures/ir_field_sweep_baseline.tsv`와 byte-for-byte 동일했다. 그러므로 baseline에는
  추가할 발산·예외 항목이 없다.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --profile release-test --tests`, HWP3 crypto focused test, 실제 fixture focused test를
  모두 통과했다. 모든 Rust 검증은 `CARGO_INCREMENTAL=0`과 검토 전용 target으로 수행했다.
- `wasm-pack build --target web --out-dir pkg` 후 Studio의 node contract test 4건, 실제 headless
  Chrome HWP3/HWP5 암호 열기 E2E, E2E manifest 검사, Studio node test 674건과 production build,
  Chrome·Firefox 확장 production build를 모두 통과했다. 두 확장 빌드는 새 `pkg` WASM을 각각
  `dist/wasm/`에 복사해 검증했다.
- 용어는 fixture 이름·기록·보관 문서를 포함해 `HWP3`로 통일했고, 저장소 전체에 이전 형식 표기가
  남지 않음을 확인했다.
