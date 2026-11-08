# #7642 구조 수식의 독립 한컴 Print 대조군

Issue: #7642

## 입력과 출력

서로 다른 EQALIGN 왼쪽 폭·역순 행, 빈 행렬 칸/행, 누락 인자/빈 템플릿, CHOOSE/BINOM, LONGDIV, COLOR를 30쪽으로 분리한다. 각 쪽은 이름 문단 → 수식만 있는 문단 → 후속 이름 문단 순서다. 표시 없는 수식 줄이 본문 글자 때문에 우연히 높이를 얻는 경우를 구별한다. 첫 쪽의 sectionPr 문단은 보존한다.

입력 HWPX의 수식 저장 폭·높이·기준선과 LineSeg를 비우고, 한컴 2020 MCP의 정상 HWP 저장 뒤 같은 저장본을 Print로 출력했다. RHWP의 export-hwpx는 저장 프레임 분석에만 사용했으며 기준 PDF 생성에 사용하지 않았다. 원 카탈로그의 실패 쪽을 이 대조군으로 대체하지 않는다.

| 파일 | SHA-256 |
| --- | --- |
| `structural-input.hwpx` | `d1c2b6b531cabf25827d8c186ca166fe449a1502222ebf4ba20e68a681634671` |
| `structural-hancom-2020.hwp` | `9cb3eef2652d2caf9c786c7c4cc69ecdb1b1621eb0d749ca8c89e3fbd93cf3fd` |
| `structural-hancom-2020.pdf` | `aafc722759d43419b6619ca238fca1efad3f96ed7d15d8a409aeff776529c792` |

입력 생성 template: `/home/tsjang/rhwp/output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage117-full-equation-catalog-print/hancom-normalized-extract.hwpx`; SHA-256 `69153efafdbbaa7e536990c6c2d64bd0c4194ef5983e7d41502cc369ba3fd35d`.

한컴 버전은 `11.0.0.9136`, `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`, Print 쪽수는 30이다.

MCP 정상 저장 job: `d457ba5c-0d7f-4d67-bdd5-cac845db4311`; Print job: `c582d1d6-bb34-4928-be4d-2ef811497af2`. 세부 변환 영수증·실행 로그·입력 생성 스크립트는 ignored output에 보존한다. 접근 정보는 기록하지 않는다.

## 독립 관측

- EQALIGN은 글자 폭·분수 폭·역순 행에서 각 행의 왼쪽 원점을 공유한다. 짧은 행을 최대 왼쪽 폭으로 옮기지 않는다.
- 빈 행렬/명시적 빈 칸은 반 em의 논리 폭을 예약한다. 누락 PILE 인자는 빈 템플릿과 달리 무효 크기·기준선 캐시를 남기며 유효한 호스트 빈 줄은 점유한다.
- CHOOSE는 앞 항과 뒤 항을 모두 조합 본체에 포함한다. LONGDIV는 제수 아래 행, 몫 위 행, 피제수 아래 행과 나눗셈 괘선을 출력한다. 복합 피제수에서 몫은 오른쪽에 정렬된다.
- COLOR의 이름/미지원 색 인자는 표시하지 않고 본체를 검정으로 출력한다. 숫자 RGB 인자는 실제 지정 색으로 출력한다.

## 검증 상태

새 구현의 Native/fresh WASM 전쪽 TSV와 확대 판독은 진행 중이다. 원 카탈로그와 전체 문서 검증도 미완료이며 이 파일의 존재를 시각 통과 증거로 간주하지 않는다.
