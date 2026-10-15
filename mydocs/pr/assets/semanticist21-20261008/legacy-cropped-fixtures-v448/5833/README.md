# 수동 축약 fixture #5833의 기존 미달 증거

- base: `552f5228d2dbfd6046c838f14241704f1bcb8a05` (`upstream/devel` 고정).
- 현재 Rust source: `d7a13ef732ffe410b290d2868c6f01fe7060907c`; 도구 변경 head: `f1461af1f1c398d1e299fbbe9959898a772140b4`.
- 기존 활성 검사: `tests/cases/issue_5833_cell_multi_para_float_pics.rs`. base에 입력과 검사 존재를 `git cat-file -e`로 확인했다.
- 원래 입력: `samples/issue5833/cell_multi_para_float_pics.hwp` / SHA-256 `a0e58d445f81797eb3e72e4db7ec4b0c563387ee57185252adaacee71c02ac70`.
- 독립 한컴 2020 Print: `pdf/semanticist21-20261008/remaining-failures-independent-print-v366/float5833-a0e58d445f81-2020.pdf` / SHA-256 `a578dde586dd92c4574a5a1858d45ce0d7420b03163818e15f5577889acc33de`. 같은 축약 입력에서 만든 1쪽 PDF다.
- Native 전체 1쪽: base **6.62281%**, 현재 **6.61305%**; 양쪽 모두 1/1쪽, 미달 1쪽, 누락 0쪽.
- fresh WASM은 이 축약 fixture에서 미실행이다. Native 기존 미달 증거로 이관하며, 양 backend 통과로 보고하지 않는다.
- #5833 원래 이슈는 전체 원본 문서의 다른 범위를 해결한 닫힌 이슈다. 동일 축약 파일·미해결 원점 차이의 열린 이슈를 검색했으나 없었다.

- 이관 이슈: [#7863](https://github.com/edwardkim/rhwp/issues/7863) (#7445 하위 이슈).

## 직접 판독

원본 표를 발췌하고 그림을 1×1 stub으로 바꾼 입력이다. base와 현재 모두 표·캡션 원점이 독립 Print보다 위에 있으며, 현재 하단 밴드도 차이가 남는다.
전체 원본의 해결 결과를 이 축약 입력의 정상 출력 증거로 대신하지 않는다. 이관은 출력 결함의 해결이 아니다.

![base review](base-review_001.png)
![현재 review](current-review_001.png)

## 보존과 복원 조건

공유 parser·codec 입력이므로 원래 `samples/` 파일 및 IR baseline은 바이트 그대로 유지한다.
부적합한 활성 렌더링 검사만 등록 이슈 범위로 이관한다. 이전 검사 본문은 `.rs.txt`로 보존했다.
독립 PDF와 Native/fresh WASM 전쪽 최저 90% 이상, 직접 확대 판독을 마친 뒤 관계형 회귀를 복원한다.
현재 변경으로 발생한 별도 회귀는 구현에서 해결하며 이 이관으로 면제하지 않는다.

## 재현 명령

```bash
venv/bin/python scripts/visual_sweep.py --rhwp-bin /home/tsjang/rhwp/output/pr-review/semanticist21-20261008/regression-root-cause-batch-v448/new-base-rhwp --embed-fonts full --compat 2022 --silhouette-only --out /home/tsjang/rhwp/output/pr-review/semanticist21-20261008/regression-root-cause-batch-v448/new-base-issue5833 --font-path output/pr-review/semanticist21-20261008/verified-declared-faces-v91 --font-path /usr/local/share/fonts/hancom-office-2020/Hwp --font-path /usr/local/share/fonts/hancom-office-2020/All --font-path /usr/local/share/fonts/hancom-office-2020/Install --font-path output/pr-review/semanticist21-20261008/verified-font-supply --font-path /usr/local/share/fonts/hwp-convert-mcp-survey --font-path output/pr-review/semanticist21-20261008/verified-ms-font-supply-v61/faces --file-target issue5833 samples/issue5833/cell_multi_para_float_pics.hwp pdf/semanticist21-20261008/remaining-failures-independent-print-v366/float5833-a0e58d445f81-2020.pdf
```

```bash
venv/bin/python scripts/visual_sweep.py --rhwp-bin /home/tsjang/rhwp/output/pr-review/semanticist21-20261008/regression-root-cause-batch-v448/closed-cell-frame-rhwp --embed-fonts full --compat 2022 --silhouette-only --out /home/tsjang/rhwp/output/pr-review/semanticist21-20261008/regression-root-cause-batch-v448/current-d7-issue5833 --font-path output/pr-review/semanticist21-20261008/verified-declared-faces-v91 --font-path /usr/local/share/fonts/hancom-office-2020/Hwp --font-path /usr/local/share/fonts/hancom-office-2020/All --font-path /usr/local/share/fonts/hancom-office-2020/Install --font-path output/pr-review/semanticist21-20261008/verified-font-supply --font-path /usr/local/share/fonts/hwp-convert-mcp-survey --font-path output/pr-review/semanticist21-20261008/verified-ms-font-supply-v61/faces --file-target issue5833 samples/issue5833/cell_multi_para_float_pics.hwp pdf/semanticist21-20261008/remaining-failures-independent-print-v366/float5833-a0e58d445f81-2020.pdf
```

## 자산 SHA-256

| 자산 | SHA-256 |
| --- | --- |
| `base-overlay_001.png` | `f16fc9278e8bdf36edb686762004fef1174eececb65ff09ec76a116183725b25` |
| `base-review_001.png` | `7e1682b28c12b3b891c7d2897976a7295f7aed09c75e11c7fba5b005917cbd01` |
| `cell_multi_para_float_pics.hwp` | `a0e58d445f81797eb3e72e4db7ec4b0c563387ee57185252adaacee71c02ac70` |
| `current-overlay_001.png` | `2d8a2d25835500d8b1a476aab26262513f8688b3d96be518c523627404ecd84b` |
| `current-review_001.png` | `e385955a09d07848c483336a7cd2509ee6d969cb7e156faf0cdda7d24298baab` |
| `independent-hancom2020-print.pdf` | `a578dde586dd92c4574a5a1858d45ce0d7420b03163818e15f5577889acc33de` |
| `issue_5833_cell_multi_para_float_pics.rs.txt` | `028514c83d3d379ff517c6aeb9a6917173e04c5d4d248a9fe48552c019ca7d7f` |
