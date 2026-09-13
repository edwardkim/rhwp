# #3587 템플릿 자동화 검증 자료

기존 연구노트·표 문서로 복제·채우기·다른 문서 가져오기를 실행한 최종 검증 자료다.
원본은 `samples/rnote/`, `samples/hwp_table_test.hwp`, `samples/table-in-tbox.hwp`에 있다.

`MANIFEST.json`의 `path`는 저장소 루트 기준 영구 경로, `source`는 조사 당시 로컬 산출 위치다.
SHA가 같은 CLI/MCP/WASM 출력·중간 입력·음성 대조 파일은 한 사본을 재사용한다.
해시는 파일 동일성을 의미하며 정상 조판의 판정을 대신하지 않는다.

| 접두사 | 역할 |
| --- | --- |
| `b-` | 같은 문서 내 문단 블록 반복과 저장·재열기 |
| `c-form-`, `c-rows-` | 고정 양식 및 완결 표 행 채우기 |
| `c-transport-` | CLI/MCP/실제 WASM의 동등 실행·재열기, 파생 HWPX 입력 |
| `d-` | 다른 문서의 표·글상자 가져오기, 자원 이식과 후속 채우기 |
| `gym-` | 기존 제품 API를 이용한 연구노트 시나리오의 단계별 입력·최종 결과 |

한컴 판정은 [최종 보고서](../../mydocs/report/task_m100_3587_report.md)에 지정한 파일·범위에만 적용한다.
특히 모든 transport 입력이나 음성 대조 파일에 한컴 시각 판정이 있는 것은 아니다.
파생 HWPX는 한컴에서 별도로 저장한 독립 원본이 아니다.
제목 표/큰 글상자에 대응하는 새 한컴 기준 PDF는 `pdf/issue3587/`,
연구노트 원본 PDF는 `pdf/rnote/labnote-001-hwp-2020.pdf`에 있다.
요청 profile은 2020이며 실제 변환 runtime과 작업 ID는 Stage 18/20 보고서에 기록되어 있다.

Gym 최종 `gym-hwp-labnote-filled.hwp`, `gym-hwpx-labnote-filled.hwpx`는
메인테이너가 한컴에서 모두 정상 판정했다. 제목/본문 3건은 실험 1/2/3, 온도 21/22/23도다.
이는 Gym 전수 벤치마크나 별도 Studio 결함 #7065/#7084/#7090의 해결 증거가 아니다.
