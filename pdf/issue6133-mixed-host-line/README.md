# #6133 전체 원본의 수동 Print 기준

- 원본: [full-source-156483831.hwp](../../mydocs/pr/assets/issue_6133_mixed_host_line_20261025/inputs/full-source-156483831.hwp)
- 독립 기준: [full-source-156483831-2020.pdf](full-source-156483831-2020.pdf), 4쪽, 689,957바이트.
- 입력 SHA-256: `b1821af4c2565348b37fafe21a47bc13e24adbeb7697d22a857b91d669132436`.
- PDF SHA-256: `0cf13bad0700f54d775efeb346a38253eb9dd1d914bbd476185bf99984993d5a`.
- 작업지시자가 원본의 내용을 바꾸지 않고 한컴의 Print로 출력해 2026-10-25 제공했다.
  Windows 원본 보관 위치는 `C:\temp\full-source-156483831-2020.pdf`다.
- 파일 이름의 `2020`과 별개로 실제 메타데이터는 `Hwp 2022 12.0.0.4605`,
  `Hancom PDF 1.3.0.550`이다. 실제 출력 정보를 보존하며 연도만으로 기준을 거부하지 않는다.
- Windows/Ubuntu의 PDF SHA-256이 일치한다. 본문에 휴먼명조가 포함되어 있으며, 기존 MCP
  출력의 Gulim 대체와 구분한다. 기존 출력은 과거 실행의 실패 증거로만 보존한다.

## 검증 상태

96 DPI·compat 2022·등록된 시스템 글꼴로 전체 4쪽을 비교했다.

| 런타임 | Native 최저 raw | fresh WASM 최저 raw |
| --- | ---: | ---: |
| `9f79fb8e7c73a41eecb509e780526a7e45a79f93` | 90.70001% | 90.31892% |
| `62656a8c807b79b2620ee9081e88ce27d7a1194b` | 90.70001% | 아래 현재 runtime의 최종 결과 참조 |
| `8dc75860cec88ddf7c3f8aeee230b2ca7e2e5537` (Native224 / fresh063) | 90.70001% | 90.31892% |

실행 명령·입력/producer SHA·전체 TSV는 ignored
`output/pr-review/semanticist21-20261019/manual-full-source-comparison-3494-result.json`에 있다.
2쪽의 fresh review·overlay를 직접 확인했으며, 문단 순서와 줄바꿈은 보존된다.
굵은 글자의 농도와 일부 글자 위치 차이가 남아 있어 이를 정확히 일치한 출력으로 보고하지 않는다.
이 기준 확보만으로 새 회귀의 최종 등록이나 PR 제출 완료를 선언하지 않는다.

현재 runtime의 전쪽 TSV와 포스터 4쪽의 review·overlay, 독립 PDF에서 다시 읽은
좌표·SHA 및 축약본과 대조군을 포함한 18개 정식 검사는
[혼합 그림·TAC 줄 증거](../../mydocs/pr/assets/issue_6133_mixed_host_line_20261025/README.md)에 있다.
포스터 4쪽은 양쪽 raw 100%이며, 본문 1~2쪽의 잔차와 구분한다. 이 정식 검사 범위의
완료를 전체 Rust·원 #8064 corpus·새 base 대비 검증이나 PR 제출 완료로 표현하지 않는다.
