# 08 — A==A 결정성

```bash
rhwp render-diff 산출.hwp 산출.hwp
```

레시피 06 실측:

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
status: PASS
```

항상 PASS, maxDisp 0.00 이어야 한다. 같은 바이트를 두 번 렌더했는데
노드가 움직이면 **문서 회귀가 아니라 도구 비결정성**이다.

CI 에 상시 기준선으로 심는다. 이 한 줄이 깨지면 전후 비교 숫자를
믿을 수 없다.

자기 라운드트립(`render-diff <파일> --via hwpx`)과 혼동하지 않는다.
왕복은 직렬화 경로를 한 번 탄다. A==A 는 그 경로조차 타지 않는다.

CI 한 줄 예:

```bash
rhwp render-diff "$OUT" "$OUT"; test $? -eq 0
```

`--json` 이어도 `status` 는 PASS, `maxDisp` 는 0.0, `regression` 은 false.
같은 입력을 두 번 연속으로 돌려 봉투가 바이트 단위로 같은지까지 보면
게이트의 결정성 전제가 닫힌다.
