# [#4864] 실측 증빙 — 컨텍스트 비용과 본문 복원율

`context-cost-measured.png` 는 `rhwp-agent context-cost <파일...> --json` 의 **출력에서
직접 그린** 표다(수치 하드코딩 없음).

| 문서 | 그대로 싣기(UTF-8) | 문서 본문 | 문자 배수 | 복원율 UTF-8 | 복원율 UTF-16LE |
|---|---|---|---|---|---|
| `samples/hwp3-sample.hwp` | 85,121자 | 21,526자 | 4.0배 | 0.0% | 0.9% |
| `samples/basic/BookReview.hwp` | 136,052자 | 2,297자 | **59.2배** | 0.0% | 44.1% |
| `samples/2022년 국립국어원 업무계획.hwp` | 289,198자 | 33,685자 | 8.6배 | 0.0% | 2.1% |

읽는 법: 파일을 그대로 실으면 본문의 몇 배에 해당하는 문자가 컨텍스트에 들어가면서 본문은
UTF-8 로 한 글자도 복원되지 않는다. 인코딩을 가장 유리하게(UTF-16LE) 찍어 줘도 복원율은
0.9~44.1% 다 — 비싼 것이 아니라, **비싸면서 틀린다**.

복원율은 본문 줄(4자 이상)이 그 디코딩 안에 원문 그대로 있는 비율이다. 짧은 줄은 우연
일치를 만들어 표본에서 제외한다.

재현:

```bash
cargo build --bin rhwp-agent
./target/debug/rhwp-agent context-cost \
  samples/hwp3-sample.hwp \
  samples/basic/BookReview.hwp \
  "samples/2022년 국립국어원 업무계획.hwp" --json
```

같은 입력이면 봉투가 바이트까지 같다 — `tests/agent_context_cost_contract.rs` 의
`measurement_is_deterministic` 가 고정한다.
