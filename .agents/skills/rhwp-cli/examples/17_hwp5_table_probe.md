# 예제 — 표 probe

명령: `rhwp hwp5-table-probe oracle.hwp generated.hwp --out-dir output/poc/probe`

이 예제는 기존 `hwp5-table-probe` 만 쓴다. 새 CLI 없음. gym 없음.

## 언제

사용자가 "표 probe" 에 해당하는 말을 할 때. 페이지가 있으면 0 기준인지 확인한다.

## 절차

```bash
cargo build --release
rhwp hwp5-table-probe oracle.hwp generated.hwp --out-dir output/poc/probe
```

## 읽는 것

- exit 0 이면 산출 경로 또는 JSON 필드.
- exit 1 이면 missing-file / load-fail / 쓰기 실패. stderr 첫 줄.
- exit 2 이면 사용법·페이지 범위·png skia 부재.
- exit 3 이면 ir-diff --json 또는 convert --verify. 산출물은 남아 있을 수 있다.

## 페이지·단위

- 한컴 N쪽 → `-p N-1`.
- dump 의 `-p` 는 문단. dump-pages 의 `-p` 는 페이지.
- 1px = 75 HWPUNIT. overlay 의 y 와 dump 의 HU 를 1:1 로 두지 말 것.

## 자기 왕복

이 예제가 성공해도 한컴 호환을 선언하지 않는다.

## 저장 계약

oracle/generated 가 필요하면 한컴 저장본을 사용자에게 받는다. 가짜 oracle 금지.

## 다음

레이아웃이면 [17_layout_debug_order.md](../references/17_layout_debug_order.md) 다음 단.
예외면 [21_exception_envelopes.md](../references/21_exception_envelopes.md).
