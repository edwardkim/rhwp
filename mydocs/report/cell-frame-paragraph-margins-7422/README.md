# #7422 — 재조합 경로의 칸 상자가 문단 좌우 여백을 뺀다

제품 SHA: `bd0660d31`. 도입 전 비교 기준은 그 부모(`#7414` head `3d18a31ec`)다.
두 checkout 을 각각 `cargo build --release --bin rhwp` 로 빌드해 같은 입력에 적용했다.

Windows 11 / Chromium(webfont rasterizer) / Poppler 25.07.0 / 96dpi.
`scripts/visual_sweep.py` 의 compare·overlay·review 를 산출해 직접 판독했다.

## 대상 — `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx` 37쪽

기준 PDF `pdf/78494-virtual-convergence-industry-decree-2020.pdf`.

### [78494_p37_before_after_oracle.png](78494_p37_before_after_oracle.png) — 직접 판독

위부터 **도입 전 rhwp / 수정 후 rhwp / 한/글 정본**이다(정본은 세로 어긋남을 보정해 잘랐다).

```text
한/글 정본     … 지원센터의      / 설치 목적 … 가능한 / 법인 또는 단체일 것
도입 전 rhwp   … 지원센터의 설치 / 목적 … 법인 또는   / 단체일 것
수정 후 rhwp   … 지원센터의      / 설치 목적 … 가능한 / 법인 또는 단체일 것
```

정본 37쪽의 **첫 줄**이 `법인 또는 단체일 것` 한 줄이고, 수정 후가 그와 같다. 도입 전은
앞 줄이 `법인 또는` 까지 먹어 `단체일 것` 만 남았다. 이 줄이 정식 회귀 검사의 대상이다.

### 전체 쪽 비교

| 파일 | 상태 | 2px 이웃 관용 내용 실루엣 |
| --- | --- | ---: |
| [78494_p37_before_review.png](78494_p37_before_review.png) | 도입 전 | 46.82492% |
| [78494_p37_after_review.png](78494_p37_after_review.png) | 수정 후 | 46.94879% |

각 PNG 는 **왼쪽 rhwp / 가운데 한/글 / 오른쪽 overlay** 다. 이 쪽은 정본에서 같은 칸이
앞 쪽부터 이어지므로 세로가 통째로 어긋나 있고, 그 어긋남이 픽셀 비교를 지배한다.
**점수를 판정으로 쓰지 않는다** — 위 직접 판독이 판정이다. 남은 세로 어긋남은 이 변경의
범위 밖이며 미해결이다.

## 함께 바뀐 나머지 두 쪽

`samples/**` 1,148건을 두 빌드로 `export-svg` 해 산출 해시를 비교하면 **3문서**가 바뀌고
**쪽 수가 바뀐 문서는 0건**이다. 위 문서를 뺀 둘은 다음과 같다.

| 문서 | 쪽 | 실루엣 |
| --- | ---: | --- |
| `samples/task2146/21761835_jeonjik_exemption_table.hwp` | 3 (6쪽 전부 변화) | 54.99556% → 55.36099% |
| `samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp` | 9 | 29.83422% → 29.90611% |

둘 다 소폭 개선이고 회귀는 없다. 두 문서의 낮은 절대 일치율은 이 변경 이전부터 있던
차이이며, 그 원인 분석은 이 변경의 범위가 아니다.

## 이 증적이 말하지 않는 것

- 저장 줄이 **있는** 문단은 그 사다리를 그대로 쓰므로 이 변경의 대상이 아니다.
- WASM 병행 캡처는 실행하지 않았다. 위 수치는 Native 빌드 기준이다.
- `#7413`(칸 안 여백 축소)이 만드는 `78494` 54쪽의 한 글자 차이는 이 변경으로 바뀌지 않는다.
