# tools/form_fill — fields / fill-fields / batch fill 픽스처

M-fill (#5481). 기존 CLI 계약만 데이터로 고정한다.

- `fields --json` 조사 봉투
- `edit fill-fields` 단건 채움 · `이름[N]` · `notFound` · `ambiguous`
- `--dry-run` (파일 없음) · `--verify` (재파싱, 불일치 시 exit 3)
- `batch fill` JSONL/CSV · `--name-field` · 행 단위 실패 격리
- T07 / #4781 첫 필드 `홍길동` **복제 금지**

DocumentCore 채움 로직을 발명하지 않는다. 새 rhwp 하위명령을 추가하지 않는다.
`gym/` 과 다른 라이브 시트는 만지지 않는다.

```bash
python tools/form_fill/fatten_form_fill.py
python tools/form_fill/test_form_fill.py
python tools/form_fill/test_fatten_form_fill.py
```

정본 함수는 `form_fill.py` 의 `parse_field_key` / `plan_fill` / `fill_envelope` /
`detect_honggildong_clone` / `batch_fill` 이다. 픽스처 JSON 은 이 함수로 다시
계산해 검증한다.
