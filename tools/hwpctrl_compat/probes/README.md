# 탐침 (게이트 밖)

여기 있는 시나리오는 `run_gate.py` 가 안 읽는다. **아직 안 맞는 것**을 재현 가능한 꼴로
남겨 두는 자리다 — 게이트는 초록이어야 하고, 붉은 것을 게이트에 두면 초록의 뜻이 없어진다.

`run_gate.py --only` 로는 안 돌아간다 — 게이트는 `scenarios/` 만 훑는다. 두 러너를 직접 부른다:

```
python tools/hwpctrl_compat/runner_ocx.py  tools/hwpctrl_compat/probes/<이름>.json --out <출력> --expect-version 12
node   tools/hwpctrl_compat/runner_rhwp.mjs tools/hwpctrl_compat/probes/<이름>.json --out <출력> --impl npm/hwpctrl-ocx/src/index.mjs
python tools/hwpctrl_compat/compare.py <출력>
```

## p2-footnote-chain — 컨트롤 사슬 차례가 문서에 따라 갈린다

`samples/2025 행정업무운영 편람(최종).hwp` 에서 164건 중 **83건이 어긋난다.**

사슬 앞 14개는 맞는다. 그 다음부터 한글은 그리기 개체가 **담은 글 리스트**의 컨트롤들을
rhwp 와 다른 자리에서 흘린다 — rhwp 는 깊이 우선(자기 다음에 자기 속)인데 한글은 본문 층을
더 걸은 뒤에 흘리는 것으로 보인다. 같은 문단(97)에 놓인 `secd`·`cold` 의 앞뒤도 뒤집힌다.

지금까지 사슬은 표본 **둘**로만 검증했고(`2026_oss_rst`·`20250130-hongbo`) 둘 다 맞았다.
셋째 표본이 규칙을 재판했다 — §4.44 와 같은 계열의 교훈이다.

원장에 올린 항목은 이 갈래 문서를 근거로 삼지 않았으므로 선언이 뒤집히지는 않는다. 다만
`HeadCtrl`·`Next` 계열은 **표본 의존**이라는 것이 드러났으니, 사슬을 쓰는 새 규칙을 세울 때는
이 표본을 반드시 함께 돌려야 한다.
