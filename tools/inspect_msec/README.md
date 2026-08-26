# inspect_msec — M-sec 3축 계약 픽스처 (#5476)

`inspect hidden-text` · `inspect injection` · `inspect unicode` 의
**기존** 토큰·코드포인트·예외 경로를 봉투와 행렬로 풀어 놓는다.

새 탐지 규칙을 발명하지 않는다. DocumentCore 를 건드리지 않는다.
라이브 `rhwp` 를 부르지 않는다. 악성 표본을 커밋하지 않는다.

## 명령

```bash
python tools/inspect_msec/gen_msec_fixtures.py
python tools/inspect_msec/test_msec_fixtures.py
```

산출: `tests/fixtures/inspect_msec/`
작업 기록: `mydocs/working/inspect_msec/` · `mydocs/working/m_sec_inspect_fatten.md`

## 권위

- `src/document_core/queries/hidden_text.rs`
- `src/document_core/queries/injection_scan.rs`
- `src/document_core/text_security.rs`
- `src/main.rs` `inspect_command`
- `tests/hidden_text_contract.rs`
- `tests/injection_scan_contract.rs`
- `tests/unicode_deception_contract.rs`

여기 표에 없는 kind·토큰·코드포인트를 생성기에 넣지 않는다.
