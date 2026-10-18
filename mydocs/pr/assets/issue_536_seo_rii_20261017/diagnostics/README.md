# 독립 극한 이름 진단 입력 — 배치 미해결

이 파일들은 HyhwpEQ ASCII `lim`/`Lim`과 PUA 본체의 실제 한컴 Print 글꼴·em 크기를
관찰한 재현 입력이다. 정상 렌더링 fixture나 golden으로 등록하지 않는다.
정식 회귀 `issue_536_equation_limit_name`은 이 파일들을 읽지 않고, Native/fresh WASM 모두
90% 이상 확인한 기존 `samples/equation-lim.hwp`만 사용한다.

| 글자 크기 | 고정 devel Native | 이름 보정 Native | fresh WASM |
| --- | ---: | ---: | --- |
| 11pt | 50.51151% | 52.57069% | 미실행 |
| 16pt | 30.00000% | 40.52670% | 미실행 |

devel은 `fe5725dc384689b45d60ef8ea4c83138bc2701d8`, 보정 renderer는
`d9a0b1c7919f3d7e5f5c6fef9e01c38472f7ce80`이다. 낮은 출력은 해결되지 않았다.
이 수치를 정상 기대값으로 고정하거나 전체 수식 배치 해결·승인 근거로 사용하지 않는다.
이름의 글꼴 계약과 별도로 단독 입력의 배치·저장 크기/기준선 원인 조사가 남아 있다.

## 생성·독립 출력

기존 `samples/hwpx/ref/ref_empty.hwpx`에 공개 CLI로 수식을 넣었다.
공용 target에서 빌드한 `rhwp`로 아래 두 크기를 각각 실행한다.

```bash
rhwp_review_target_dir=/home/tsjang/rhwp/target/pr-review
"${rhwp_review_target_dir}/release-test/rhwp" edit insert-equation \
  samples/hwpx/ref/ref_empty.hwpx \
  --script 'lim _{x rarrow 0} x + Lim _{x rarrow 0} x' \
  --font-size 1100 -o /tmp/limit-1100.hwpx --json
# 16pt 대조군은 --font-size 1600과 다른 출력 경로를 사용한다.
```

`limit-1100.hwpx`와 `limit-1600.hwpx`는 이 실제 생성본이다.
각 `*-hancom-saved.hwp`는 같은 입력을 한컴 2020으로 다시 저장한 대조군이다.
대응하는 독립 Print PDF는 [진단 PDF 경로](../../../../../pdf/seo-rii-20261017/diagnostics/)에 있다.
Hancom MCP engine 2020, 전처리 없음, PrintMethod=0 / 1-up으로 출력했다.

Print의 ToUnicode와 Tf에서 이름/본문 em은 11pt에서 110/92, 16pt에서 160/133이었다.
한컴 Save 전후 script·크기·baseline·LineSeg 및 Print 페이지 content stream은 같았다.
따라서 Save가 이 입력의 저장 cache를 정규화했다거나 전체 배치가 정상이라는 증거로 쓰지 않는다.

입력/PDF 해시·job ID·실행 결과는 [통합 검토 4단계](../../../../working/task_m100_536_integration_review_stage4.md)와
[5단계](../../../../working/task_m100_536_integration_review_stage5.md), ignored 검증 기록에 연결한다.
완료 시 ignored 기록은 output 밖으로 해시 대조 보존한다.
