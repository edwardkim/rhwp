# 예제 — 가설은 구현해서 기각

이슈 #5324. playbook 함정 4. gym 아님. F16.

## 정답지

IR 차이 (`--verify` / `ir-diff --json`). 페이지 수 증상은 단서가
아니다.

## 명령

```bash
rhwp export-hwpx 원본.hwp out.hwpx --verify-pages
# exit 4 로 끊기면
rhwp export-hwpx 원본.hwp out.hwpx --verify
rhwp ir-diff 원본.hwp out.hwpx --json
```

## 읽는 법

#3518: provenance 가드가 원인이라 수정안까지 있었으나 페이지 수는
그대로. `char_shapes` 시작 −2 가 진짜 원인이었다. 이 스킬은 패치를
넣지 않는다. 기각 로그를 IT03 으로 남긴다.
