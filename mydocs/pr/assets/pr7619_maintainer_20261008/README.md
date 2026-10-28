# PR #7619 메인테이너 시각 검증 — 2026-10-08

검증 source: `b87dd201179d5ff6e1dacdf8b7d18c7ce2b0058d`. 원 PR code head
`0f1153d18c83043c449304996fc462a6b5b15edb`와 src/crates/tests/samples/pdf/Cargo 파일이 동일하다.
검토 base는 `60efc4b7b380b2a183c20c3ee0ab24a60bc2d832`이다.
이 폴더는 동일 코드의 새 시각 출력만 추가하며 기존 기여자 증적을 변경하지 않는다.

## 입력과 기준

모든 입력은 공개 합성 HWPX이고 각각 한컴 2020 Print PDF 1쪽에 대응한다.
문단 기준 입력은 7언어 fontface 선언 대조군이다. 기여자가 보존한 HANGUL-only 원 입력의
89.04%를 이 대조군 통과로 바꾸어 보고하지 않는다. TopAndBottom, 실제 편집 세션,
다음 단 이월, PAPER 정렬 조합 및 선언/실측 표 높이가 다른 경계는 이 4개 입력으로 입증하지 않는다.
[원 이슈](https://github.com/edwardkim/rhwp/issues/7548)는 부분 해결로 계속 열린 상태로 둔다.

| 입력 | 경로 | SHA-256 | 기준 PDF | SHA-256 |
| --- | --- | --- | --- | --- |
| 7619-full | `samples/page_anchored_square/page_anchored_square_reflow.hwpx` | `988763f863459da6fb26874cd963065ba22aee9d111ee2e29e60c68d08911f95` | `pdf/page_anchored_square/page_anchored_square-2020.pdf` | `736a7b58e5ba418b05bddbf0f154f683e3c4c16a0444a70059564084c82a0af7` |
| 7619-narrow | `samples/page_anchored_square/page_anchored_square_reflow_narrow.hwpx` | `2454813ec3a22323bc5a7e4a20fcea3670478e62a88b6318a13ff15aedbb3923` | `pdf/page_anchored_square/page_anchored_square_reflow_narrow-2020.pdf` | `5f99b5d6f1a31e06ad7d9d4c271fb7d907ebe11f13f909881fb046b3c504d2c5` |
| 7619-overlap | `samples/page_anchored_square/page_anchored_square_reflow_allow_overlap.hwpx` | `fe8d2341e919f23978750d936129bb987a0e488f1392f5459578144144018791` | `pdf/page_anchored_square/page_anchored_square_reflow_allow_overlap-2020.pdf` | `1967b0396b0fb69875ea4b2e21ac7b68030265646cbdb34cfc1b8db2655b9149` |
| 7619-lane | `samples/para_square_lane/para_square_lane_reflow.hwpx` | `ec69ea339b29243b5769c73f3df29b1a23f68c8c9f9b1bc180cd3cfa1bac1900` | `pdf/para_square_lane/para_square_lane_reflow-2020.pdf` | `89f93b32457a0774e733ecc9f3eef578ab9269f73481cb0726a72959be504666` |

## 실행과 결과

Native: release-test, `rhwp` SHA-256 `4d53843c50b37e2d74dd1c4d19c0ecaacd1f5f667c0a8517096e44e53ab1399f`.
Fresh WASM: `docker compose -p rhwp --env-file .env.docker run --rm wasm`,
Rust 1.93.0 / wasm-pack 0.15.0. WASM SHA-256
`06c52c5669a306c6b4d2f476d00591cff67eeb36e27b8f43fdbdfe7e6c3e8e7e`;
pkg/Studio public JS·WASM hash 일치를 확인했다.

```sh
python scripts/visual_sweep.py --rhwp-bin <release-test/rhwp> --dpi 96   --embed-fonts=full --font-path /mnt/c/Windows/Fonts --out <output>   --file-target <key> <input.hwpx> <hancom.pdf>
# fresh WASM 경로는 같은 명령에 --wasm-pkg <fresh pkg>를 추가한다.
```

| 입력·쪽 | Native 2px 관용 실루엣 % | fresh WASM % | gate |
| --- | ---: | ---: | --- |
| full p1 | 99.45089 | 99.45089 | passed |
| narrow p1 | 99.35870 | 99.35870 | passed |
| overlap p1 | 99.45089 | 99.45089 | passed |
| lane p1 | 100.00000 | 100.00000 | passed |

전 4쪽을 두 경로로 비교했으며 90% 미만·누락·쪽수 차이가 없다.
8개 review PNG를 각각 직접 판독하여 표 앞뒤 순서·누락·겹침·옆 차선·전폭 복귀를 확인했다.
표 괘선 약 1~2px 차이가 남는다. 값은 엄격 픽셀 일치율이나 사람의 판정 정확도가 아니다.
TSV·로그·중간 JSON은 메인테이너의 ignored output에 보존했고 Git에 포함하지 않는다.
Focused 검사는 새 4건과 기존 정상 대조 8건의 12 PASS / 0 FAIL이다.

[Full CI run](https://github.com/edwardkim/rhwp/actions/runs/37543258011)은
candidate `77f3932a12aced7871b7ecab9c3a118375b155b4`에서 lint·Native Skia·전체 A/B/C/D 테스트가 성공했다.
원 PR head의 devel merge는 자동 merge-tree와 정확히 같고 latest CI의 preflight가 이 candidate를 확인했다.
최신 head의 required checks와 merge 가능 상태는 병합 직전에 별도로 확인한다.
