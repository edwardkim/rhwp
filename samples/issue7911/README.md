# #7911 PDF 가져오기 저장본

원본은 [`nested_table_text_caption-2020.pdf`](../../pdf/issue5875/nested_table_text_caption-2020.pdf)다.
이 폴더의 HWP/HWPX는 기본 비활성 `pdf-import`를 켠 Native CLI로 전체 12쪽을 직접 가져온 결과다.
2026-10-16 통합본으로 다시 생성했으며 이전 사용자 승인 보정본과 바이트가 동일하다.
생성 소스·실행·Native/Docker WASM 해시는 [4단계 보고](../../mydocs/working/task_m100_7911_stage4.md)에 있다.

```sh
# pdf-import 활성 검증 바이너리에서 실행
rhwp import-pdf pdf/issue5875/nested_table_text_caption-2020.pdf output/imported.hwp --json
rhwp import-pdf pdf/issue5875/nested_table_text_caption-2020.pdf output/imported.hwpx --json
```

독립 한컴 Print는 `pdf/issue7911/`에 보존한다. 원본 PDF 비교와 저장본 재조판 비교는 다른 기준이다.
원본 PDF와의 Native/fresh WASM 전쪽 최저는 90.14609%/90.16832%다. 저장 HWP의 한컴 재조판
1·11쪽 차이는 메인테이너가 승인한 후속 B10-1이며, 이 자료를 정상 출력 golden으로 등록하지 않는다.

| 입력/기준 | SHA-256 |
| --- | --- |
| `samples/issue7911/pdf-import-nested-table.hwp` | `ac245bae37b408b5c4af69ab39f6ef53505a47433034fc34288714704a107d99` |
| `pdf/issue7911/pdf-import-hwp-hancom2020.pdf` | `6236fba4e84bdc4c35e0407f77def18bf2760e16b14ec6755fb473fe4217fd2d` |
| `samples/issue7911/pdf-import-nested-table.hwpx` | `032e87dc63af02de4330e6f19b0578abaabf521d17f38015b1a9d3da9523603b` |
| `pdf/issue7911/pdf-import-hwpx-hancom2020.pdf` | `e66f78cd6ba8f0dad082b7dfabcc54ab27cacda2c7812c428469c94b3c2e84c3` |

## 독립 Print 출처

- HWP: job `88b2e367-697c-498c-b9af-ed7e574dd6bb`, engine `11.0.0.9136`, `hancom2020_pdf_driver_one_up`, 12쪽.
- HWPX: job `ce3a0d75-cb00-44a5-be0a-4adc5cc4c454`, engine `11.0.0.9136`, `hancom2020_pdf_driver_one_up`, 12쪽.

위 Print는 3단계 보정본에서 실제 생성한 결과를 동일 저장본에 재사용한 것이다. Creator 필드의
형식별 버전 문자열 차이와 실제 변환 engine 버전을 구분한다. 새 Print를 실행했다고 보고하지 않는다.
3쪽 오른쪽 crop·11쪽 그림 10 표시는 메인테이너가 직접 확인했다. 파일 구조·내용 보존 증거와
전체 한컴 재조판 완성을 혼동하지 않는다.
