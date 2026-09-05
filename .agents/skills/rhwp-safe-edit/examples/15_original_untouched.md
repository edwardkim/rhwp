# 15 — 원본이 그대로인지 증명하기

모든 층의 공통 사후 조건. 주장이 아니라 해시/바이트 대조.

권위: [verify_loops.md](../references/verify_loops.md) §9,
`tests/edit_fill_fields_contract.rs` (dry-run 은 파일을 만들지 않는다).

## 1. 실행 전

```bash
sha256sum samples/field-01.hwp > /tmp/orig.sha
# PowerShell
Get-FileHash samples/field-01.hwp -Algorithm SHA256 | Select-Object Hash
```

산출 경로가 남아 있으면 지운다. 이전 잔재를 이번 실패의 산출물로 오인하지 않기 위함.

## 2. 수행

02 선확인 + 03 실행 (01 편 또는 07 편). 실패 표본(09)도 좋다.

## 3. 실행 후

```bash
sha256sum -c /tmp/orig.sha
cmp samples/field-01.hwp samples/field-01.hwp
```

원본 경로의 해시가 같아야 한다.

`-o` 가 원본과 같았다면 이 대조는 의미가 없다 — 그 호출 자체가 규약 위반이다.
`redact --in-place` 는 사용자가 명시한 덮어쓰기이므로 이 편의 대상이 아니다.

## 4. `run` 실패 후

선검증 실패·단언 실패·CAS 실패는 산출 경로를 만들지 않는다.
`test ! -e out/plan_result.hwp` (잔재를 지운 뒤).

## 5. 체크리스트

- [ ] 실행 전 해시를 떠 두었다
- [ ] `-o` ≠ input
- [ ] 실행 후 원본 해시가 같다
- [ ] 실패 표본에서도 같다
