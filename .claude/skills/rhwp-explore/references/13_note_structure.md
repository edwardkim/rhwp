# 13 — note-structure

켤 때: `footnote_count + endnote_count > 0`. 우선순위 45.
skill `rhwp-doc-triage`. confidence high.

```
rhwp explain <file> --json
```

## why

`각주 N개·미주 M개 — 참조 구조를 포함한 문서`

한쪽이 0이어도 문장에 둘 다 적는다. 엔진 개수를 숨기지 않는다.
미주만 있어도 항목이 켜진다.

다음 명령이 `explain` 인 이유는 각주/미주 개수가 explain 봉투에 이미
있기 때문이다. 새 노트 전용 하위명령을 만들지 않는다. 세 축을 섞어
`explore` 를 다시 치는 순환도 하지 않는다. 본문 인용은 explain 의
몫이고 explore why 에는 개수만 남는다.
