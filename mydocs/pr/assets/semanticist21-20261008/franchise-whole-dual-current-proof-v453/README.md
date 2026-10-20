# 가맹사업 전142쪽 최종 대조

생산 소스: `ddf17f86476293b114b6236bfa6cf1ebc9d9d474`. Native/fresh WASM 전142쪽 모두 최저68쪽93.16816%, 미달·누락0, 독립 Print 쪽수142/142다. 전142쪽 current RGBA가 같으며 60/68/118쪽의 양쪽 review6개를 직접 열었다. 이전 같은 소스의17개 경계·확대 판독과 전체 Native 비교를 연결한다.

- 원본: `samples/issue5966/1130000-202100008_franchise_review_report.hwp`
- 독립 Print: `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue5966_franchise_review_report_1130000-202100008_franchise_review_report-2020.pdf`
- 입력/Print·binary·pkg·글꼴 해시와 전체 TSV는 ignored v453의 run manifest, whole-native/fresh-results 및 franchise-current-whole-dual-proof JSON에 보존한다. TSV는 원본 다운로드 링크로 제출하며 Git에 커밋하지 않는다.

| 쪽 | Native/fresh 전체 TSV | 직접 판독 |
| --- | --- | --- |
| 60 | 99.99886% | 세 판례의 셀/본문 소속·내용 보존·하단 포함 |
| 68 | 93.16816% | 심사지침·인용문·65) 각주; 밑줄/획 표현 차이는 남음 |
| 118 | 99.99852% | 중첩 프레임 시작/끝·구속 제목 뒤 상자·정렬 복구 |

대표 PNG만 게시한다. 출력 겹침 패널의 반올림100%와 전체 TSV 원값을 구분한다.

## 실제 fresh 명령

```bash
venv/bin/python scripts/visual_sweep.py --rhwp-bin /home/tsjang/rhwp/output/pr-review/semanticist21-20261008/nested-frame-alignment-provenance-v453/nested-frame-provenance-rhwp --embed-fonts full --font-path output/pr-review/semanticist21-20261008/verified-franchise-original-human-v176 --font-path output/pr-review/semanticist21-20261008/verified-declared-faces-v91 --font-path /usr/local/share/fonts/hancom-office-2020/Hwp --font-path /usr/local/share/fonts/hancom-office-2020/All --font-path /usr/local/share/fonts/hancom-office-2020/Install --font-path output/pr-review/semanticist21-20261008/verified-font-supply --font-path /usr/local/share/fonts/hwp-convert-mcp-survey --font-path output/pr-review/semanticist21-20261008/verified-ms-font-supply-v61/faces --file-target franchise5966 samples/issue5966/1130000-202100008_franchise_review_report.hwp pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue5966_franchise_review_report_1130000-202100008_franchise_review_report-2020.pdf --out /home/tsjang/rhwp/output/pr-review/semanticist21-20261008/nested-frame-alignment-provenance-v453/franchise-fresh --compat 2022 --silhouette-only --font-environment output/pr-review/semanticist21-20261008/franchise-glyph-fallback-v428/font-environment.json --wasm-pkg /home/tsjang/rhwp/pkg
```

처리 순서 보정은 기존142쪽 SVG를 유지하고63쪽 캡처 뒤부터 새 Node에 적용했다. 1/68/118쪽의 전후 캡처는 같은 RGBA·글꼴 기록이며 전142쪽은 최종 Native와 같은 RGBA로 확인했다. 이미 실행 중인 Python은 기존 검사를 유지했으며 전쪽을 재내보내지 않았다. 문서당 Chrome 재사용은 후속 배치 경로로 따로 검증했다.
