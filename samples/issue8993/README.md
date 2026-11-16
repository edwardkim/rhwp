# samples/issue8993 — 저장 프레임이 본문을 넘는 RowBreak 표

이 폴더는 [#8993](https://github.com/edwardkim/rhwp/issues/8993)의 정식 회귀 fixture다.

행정안전부령 「지방자치단체 회계기준에 관한 규칙」(부령 제124호, 2009.12.31, HWP, 19쪽) 원본이다.
14쪽은 빈 문단 넷 뒤에 [별지 제1호 서식] 재정상태보고서 표가 온다.

| 항목 | 값 |
| --- | --- |
| 표 | 52행×13열, 비-TAC, 자리차지, 문단 기준 세로 0, 쪽나눔 RowBreak |
| 선언 높이 | 59368HU(791.6px) |
| 저장 프레임 | 14쪽 130.1px → 921.7px(본문 914.6px 를 넘는다) |
| 뒤 문단 | 저장 vpos 0(되감김) |

저장 프레임 자체가 본문을 넘으므로 원본도 이 자리에서 행을 나눴다. 한/글 engine 2020 Print 는
14쪽에 「Ⅱ. 장기차입부채」 묶음까지, 15쪽에 「Ⅲ. 기타비유동부채」부터 「부채와 순자산 총계」까지 둔다.

- 회귀 계약: 전체 19쪽. 14쪽 = 표 머리 … 「장기차입부채」, 15쪽 = 「기타비유동부채」 … 「부채와 순자산 총계」.
- 바이트 정본: [MANIFEST.json](MANIFEST.json)의 SHA-256과 크기
- 기준 엔진: **2020** — `rhwp info --json` 의 `lastSavedWith.product` 가 없음(`7.5.8.527`)이라
  저장소 정책의 2022 이하 버킷이다.

기준 PDF: [`pdf/issue8993/31675-rowbreak-overflowing-frame-2020.pdf`](../../pdf/issue8993/31675-rowbreak-overflowing-frame-2020.pdf)
— 196,869 bytes, SHA-256 `abad2bc7b5691e5bd6c4b3a5fbaae0eb95b21662f6bc5a3a7472d9027acbb73d`,
Creator `Hwp 2022 12.0.0.4547`(engine 2020 profile), Producer `Hancom PDF 1.3.0.547`, 19쪽.
