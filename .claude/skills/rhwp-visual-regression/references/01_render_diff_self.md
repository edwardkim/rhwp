# 01 — render-diff 자기 라운드트립

가장 싼 점검이다. 파일을 하나 주고, 원본 IR 과 직렬화→재로드 IR 을
같은 렌더러로 그린 뒤 노드 bbox 를 비교한다.

```bash
rhwp render-diff samples/form-01.hwp --via hwpx
```

레시피 06 실측:

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

종료 코드 0. `--via hwpx` 는 HWP5 원본을 HWPX 로 변환했다가 다시
렌더링한다. `--via hwp` 는 HWP 어댑터 경로다. 기본은 hwpx.

특정 페이지만:

```bash
rhwp render-diff samples/form-01.hwp -p 0 --via hwpx
```

1쪽 문서라 결과는 같다. `-p` 가 비교 범위 밖이면 **exit 2** 다.
빈 PASS 로 위장하지 않는다.

## 이것이 재는 것 / 안 재는 것

재는 것: rhwp 가 그린 원본 vs rhwp 가 그린 왕복. 내부 회귀.

안 재는 것: 한컴 PDF 충실도. 자기 라운드트립 PASS ≠ 한컴과 같다.

## JSON

```bash
rhwp render-diff samples/form-01.hwp --via hwpx --json
```

`mode` 는 `"roundtrip"`, `sourceB` 는 null, `via` 는 `"hwpx"` 또는
`"hwp"`. `status: PASS` 이면 exit 0.

## 언제 쓰나

- 변환 파이프라인에 넣기 전 싼 스모크
- CI 상시 기준선 (A==A 와 함께)
- 편집 전후를 보기 전에 "도구 자체가 흔들리지 않는가"
