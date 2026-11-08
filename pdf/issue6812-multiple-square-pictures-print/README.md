# #6812 다중 그림 입력의 독립 Print 진단 자료

원본은 `samples/issue6797/156160455-social-pig-farm-income.hwp`다. 실패한
`issue_6812_following_text_respects_column_bottom_after_multiple_pictures`와 같이
첫 문단의 첫 그림을 복제하고 세로 오프셋을 4500HU 더했다. 원본을 RHWP로
HWPX 내보낸 뒤 그림 id/instid를 새 값으로 분리했으며 다른 내용을 바꾸지 않았다.
이 진단 저장본은 원본의 실패 증거를 대신하지 않는다.

- HWPX SHA-256: `b7d308b604380836b663a0708d324d2367343de813a789ec8870df809824a31f`
- PDF SHA-256: `4acdcedd336fb4864196769f149291f88925a419ee5e51b69b9a9bd0b40853b2`
- 한컴 2020 MCP job: `ea1ea3c7-4173-47df-8663-32ecd2afbe1d`
- Hancom: `11.0.0.9136`, engine `2020`, PrintMethod `0`,
  `hancom2020_pdf_driver_one_up`, 11쪽

Stage 159 Native는 기존 검사와 같은 3.6px 본문 하단 초과를 재현했다.
독립 Print와 첫 두 쪽은 95.60806 / 45.35332%다. 한컴의 다음 쪽 첫 줄이
RHWP에서는 앞 쪽에 남는다. 아직 통과한 회귀 fixture가 아닌 보정용 진단 자료다.
전체 Native/fresh WASM 90%와 확대 판독 전에는 이를 새 회귀의 기대값으로 쓰지 않는다.

명령·Print receipt·원본 대응과 전후 검증은
`mydocs/working/task_m100_7642_stage162.md` 및 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`
아래 `stage161-square-picture-print`, `stage161-square-picture-before`에 있다.
