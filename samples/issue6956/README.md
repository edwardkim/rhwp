# #6956 — 형광펜 표지 왕복 보존

`3024739-exposure-algorithm-markpen.hwpx` — 환경부 「환경유해인자 위해성평가를 위한 절차와
방법 등에 관한 지침」 [별표 4]. 코퍼스의 형광펜 문서 18건 중 **가장 작다**(11KB).

형광펜 표지가 **두 모양**으로 들어 있어 경계를 같이 잰다.

```xml
<!-- ① hp:t 안 (이 이슈가 닫는 축) — 2쌍 -->
<hp:run charPrIDRef="5"><hp:t>□ <hp:markpenBegin color="#FFFFFF"/>노출량 산정 알고리즘<hp:markpenEnd/></hp:t></hp:run>

<!-- ② run 바로 밑에서 표를 감싼다 (별도 축) — 1쌍 -->
<hp:run charPrIDRef="14"><hp:markpenBegin color="#FFFFFF"/><hp:tbl …/><hp:t><hp:markpenEnd/></hp:t></hp:run>
```

## 계약

①은 색까지 그대로 왕복한다. ②는 `hp:t` 밖(컨트롤 형제 자리)이라 이 축이 다루지 않는다 —
왕복에서 여전히 사라지고, 그 사실을 시험이 함께 못박는다.

## 모수

코퍼스 HWPX 3,418건 중 형광펜을 쓰는 문서 **18건 · 표지 59개**. 이 수정으로 **17/18** 이
개수·색까지 보존되고, 남는 1건이 이 문서의 ② 한 쌍이다.
