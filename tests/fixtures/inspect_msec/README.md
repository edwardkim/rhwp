# inspect 3축 계약 픽스처 (M-sec / #5476)

이 디렉터리는 `inspect hidden-text` · `inspect injection` · `inspect unicode`
의 **기존 CLI 계약**을 봉투·예외·행렬로 고정한다.

새 탐지 규칙을 발명하지 않는다. 악성 `.hwp` 를 커밋하지 않는다.
라이브 바이너리를 부르지 않는다. 소비자는 키 존재·분기 필드·exit 규약만 본다.

## 생성

```bash
python tools/inspect_msec/gen_msec_fixtures.py
python tools/inspect_msec/test_msec_fixtures.py
```

## 건수

- 성공 봉투: 194
- 예외 봉투: 22
- 합계: 216
- hidden-text: 48
- injection: 102
- unicode: 44

## 권위

- `src/document_core/queries/hidden_text.rs`
- `src/document_core/queries/injection_scan.rs`
- `src/document_core/text_security.rs`
- `src/main.rs` `inspect_command`
- `tests/hidden_text_contract.rs`
- `tests/injection_scan_contract.rs`
- `tests/unicode_deception_contract.rs`

## 하지 않는 것

- DocumentCore 판정 로직 변경
- 새 kind / 새 토큰 / 새 코드포인트
- gym / canvaskit / serializer / layout-anomaly / oracle /
  render_backend / proptest / fidelity_compare / hwp5-inventory / page-count
