# 12 — long-doc-digest

켤 때: `page_count >= 10` (`LONG_DOC_PAGES`). 우선순위 40.
skill `rhwp-doc-triage`.

```
rhwp digest <file> --sections --json
```

confidence: 쪽수 ≥ 20 이면 high, 10–19 는 medium.

## why

`N쪽 장문 — 통째로 읽기 전 요약·절 단위 청킹 권장`

9쪽은 켜지지 않는다. 10쪽은 medium. 20쪽은 high. 임계를 이 스킬이
바꾸지 않는다.

## 통독 금지

장문 메뉴가 켜진 문서에 `export-text` 로 전문을 컨텍스트에 넣지 않는다.
절 단위 digest 와 search 로 좁힌다. 보안이 같이 켜져 있으면 스윕이 먼저다.
9쪽 문서를 장문으로 승격하지 않는다. 임계는 엔진 상수다.
