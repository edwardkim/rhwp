# @rhwp/hwpctrl — 웹한글컨트롤 호환 층

한컴 **웹한글컨트롤(WebHwpCtrl) API v2.4** 를 rhwp WASM 위에서 호출 호환으로 구현한다.
기존 통합 페이지의 스크립트를 **한 줄도 고치지 않고** 동작시키는 것이 목표다.

```html
<script src="rhwp-hwpctrl.js"></script>
<script>
  HwpCtrl.Open(file, "", "", function () {
    HwpCtrl.PutFieldText("기안자", "홍길동");
    HwpCtrl.SaveAs("기안문.hwp", "Hwp", "");
  });
</script>
```

계획서: [`mydocs/plans/hwpctrl_ocx_full_compat.md`](../../mydocs/plans/hwpctrl_ocx_full_compat.md)

## 현재 상태 — P0 (하니스·스펙만)

구현 코드는 아직 없다. 이 디렉터리에 있는 것은 **정답지와 원장**이다.

| 파일 | 내용 | 출처 |
|---|---|---|
| `spec/webhwpctrl_api.json` | API 122항목 (속성 18·메서드 67·이벤트 3·객체 34) | `samples/hwpctl_API_v2.4.hwp` |
| `spec/actions.json` | Action 312개와 각자의 ParameterSet | `samples/hwpctl_Action_Table__v1.1.hwp` |
| `spec/parameter_sets.json` | ParameterSet 50종 / Item 521개 | `samples/hwpctl_ParameterSetID_Item_v1.2.hwp` |
| `spec/api_ledger.json` | 원장 484항목 — 진척의 유일한 진실 | 위 세 파일 + 오라클 판정 |

**`spec/` 는 손으로 고치지 않는다.** 재생성은
[`tools/hwpctrl_compat/`](../../tools/hwpctrl_compat/README.md) 의 `extract_spec.py`.

## 진척 보고 방식

`api_ledger.json` 의 `summary.done / summary.total` 한 숫자로만 한다. `verified` 는
**오라클 대조가 채우며 사람이 올릴 수 없다** — 근거는 각 항목의 `oracle.scenarios`.

## 기존 studio 층과의 관계

`rhwp-studio/src/hwpctl/` 은 별개이고 **P6 까지 동결**이다. 이 패키지가 원장 100% 에 도달하면
P7 에서 그 층을 철거하고 studio 를 이쪽으로 이관한다(계획서 §6.2).
