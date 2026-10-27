# 표 조각과 글줄의 실제 점유·쪽 소유 회귀 증적

## 검증 범위와 결과

- 구현 source: `646bd3538a9f13b4b31ca9b08ea05580d06e593a`.
- 고정 base: `5e2b9bfbf08573a31fa1e11a2445ff63fc0755c0`.
- 원본 36개, 독립 Print 36개, 각 64쪽. Native058·최적화한 fresh WASM026 모두
  **전쪽 raw 실루엣 100%**, 독립 PDF와 전체 쪽수 일치, 같은 base 대비 0.1pp 이상 하락 없음.
- `fine/`의 review와 standalone overlay 4쌍을 직접 확대 판독했다. 표 시작·마지막 행,
  앞·뒤 본문을 같은 PDF 좌표로 비교했다. 페이지·행·본문 소유와 누락·중복도 검사한다.
- 이 자료는 이 36개 입력의 회귀 추가 선행 근거다. 전체 통합 PR의 완료 판정이 아니다.

`provenance.json`은 파일별 입력/PDF SHA-256·크기·인쇄 job·쪽수·실행 명령·producer 해시·
글꼴 공급과 고정 base의 동일성 근거를 담는다. 실행 로그와 중간 JSON은 ignored output에만 둔다.
`tsv/{base,candidate}/{native,wasm}/`는 각 실행의 TSV를 바이트 그대로 복사한 원본이다.

## 원 해결 PR과 실제 계약의 대조

| 원 PR·실제 통합 | 확인한 계약과 현재 처리 |
| --- | --- |
| [#8055](https://github.com/edwardkim/rhwp/pull/8055) | 분할 표 뒤 문단 앞 간격의 중복 소비를 주장했다. 그러나 축약 factory의 정책은 `None`이다. `pr8055-split-successor.hwpx`를 변경 없이 보존한다. 이 입력의 Print는 표를 통째 첫쪽에 두고 뒤 본문을 다음 쪽에 둔다. 실제 분할 간격 계약에는 별도의 한컴 저장 RowBreak와 그 자체의 Print를 사용한다. 원 교육부 실물 문서가 검증됐다는 의미는 아니다. |
| [#2108](https://github.com/edwardkim/rhwp/pull/2108), [#2135](https://github.com/edwardkim/rhwp/pull/2135) | 온전한 선언 프레임에 들어가는 실제 저장 표를 메트릭 팽창으로 과분할하지 않는다. 빈 문자열도 실제 저장 글줄이 있으면 물리 높이를 점유한다. 기존 선언-fit 정상 검사 4개를 그대로 실행해 통과했다. |
| [#7141](https://github.com/edwardkim/rhwp/pull/7141) → 실제 통합 [#7188](https://github.com/edwardkim/rhwp/pull/7188) | 표 조각의 바깥여백·실제 점유와 다음 내용 예약을 같은 프레임으로 소비한다. 단일 셀·rowspan 등 별도 소유를 새 행 prefix 규칙에 섞지 않는다. 기존 조각 상자 검사 10개를 유지해 통과했다. |
| [#3740](https://github.com/edwardkim/rhwp/pull/3740), [#7882](https://github.com/edwardkim/rhwp/pull/7882) | EOF에서 이미 확정한 다음 쪽 개체·각주를 배출한다. 다음 쪽 큐 활성화와 활성 개체가 있는 마지막 단 확정 두 소비 지점을 모두 확인했다. |

## 입력 생성과 독립 기대값

| 입력 종류 | 개수 | 출처·차이 |
| --- | ---: | --- |
| 원 #8055 축약 입력 | 1 | 작성자 재현 절차로 만든 HWPX. `None`을 분할 정책으로 바꾸거나 저장 줄을 새로 주입하지 않는다. |
| 공개 API NoSplit | 12 | 2행/60행, 앞 본문 유무, 쪽 제한 유무, HWP/HWPX. 실제 공개 편집 API와 내보내기를 사용했다. |
| 공개 API RowBreak/Cell | 20 | 2행/60행, 쪽 제한 유무, 앞 본문 유무, HWP/HWPX. 정책을 바꾼 뒤 실제 내보내기로 계산 줄 정보를 갱신했다. |
| 한컴 저장 RowBreak | 1 | 한컴이 저장한 HWP의 실제 빈 글줄·부분 선언 프레임. 이 저장본 자체를 다시 Print했다. |
| 한컴 저장 NoSplit | 2 | 한컴 저장본의 실제 호스트 글줄·뒤 본문 사다리. 두 저장본 각각을 Print했다. |

모든 기준 PDF는 한컴 2020 `11.0.0.9136`의 `PrintMethod=0`·1-up이다. HWP 입력의
`scripts_disabled` 전처리 여부를 파일별로 기록했다. HWPX→HWP 저장은 저장 구조를 확인하기
위한 별도 단계이며, 해당 저장본의 기준 PDF는 그 저장본에서 실제 Print한 파일이다.
`--compat 2022`는 RHWP의 조판 경로 선택이며 한컴 인쇄 엔진 연도를 뜻하지 않는다.

`expected.json`의 쪽수·본문·경계 좌표는 RHWP 배치값이 아니라 같은 독립 PDF의 벡터 선과
텍스트 span에서 추출했다. 60행 분할 표는 앞 본문이 없으면 50/10행, 있으면 49/11행이다.
NoSplit의 용지 밖 행은 Print와 같은 물리 쪽에서 잘린다. PDF에 보이는 행 수로 원본 60행을
줄이지 않으며 실제 TableCell 소유에서 전체 행의 보존·비중복을 별도로 검사한다.

수동 XML에서 정책만 변경하고 전역 LineSeg를 남긴 `split-policy-cell` 진단은 이 36개에
포함되지 않는다. 그 진단의 실패 자료는 output에 그대로 남긴다. 다른 저장본의 통과를 그
진단 원본의 일치로 보고하지 않는다.

## 직접 판독과 좌표 회귀

본문의 같은 글자·줄·간격과 표의 같은 경계를 판독했다. 얇은 선은 독립 PDF 벡터의
0.36pt 펜과 RHWP 원본 선폭·중심이 일치한다. PDFium이 얇은 선을 최소 폭으로 래스터화해
PDF가 더 진하게 보인다. 동일 PDF의 별도 래스터 진단에서도 이 차이를 확인했다.
기준 래스터화·gate·펜 폭을 바꿔 점수를 맞추지 않았다. 이는 RGB 완전 일치 주장이 아니다.

양쪽 모두 98% 이상이고 직접 판독을 마친 페이지이므로, 표 좌우·시작·분할 종료 경계와
본문 원점을 독립 PDF의 96 DPI 페이지 좌표에서 **±2px**로 보호한다. 동시에 원본 행의
누락·중복·쪽 소유, 실제 셀의 표 내부 포함, 본문 순서·보존·비겹침 관계를 유지한다.
쪽 제한 NoSplit의 앞쪽 호스트·뒤 본문과 다음 쪽의 표만 있는 EOF도 검사한다.

## 재실행

파일별 정확한 인수는 `provenance.json`의 `cases[].backends`에 있다. 현재 빌드 producer로
바꿀 때도 입력·독립 PDF·96 DPI·글꼴·페이지 범위는 유지한다.

```bash
venv/bin/python scripts/visual_sweep.py --silhouette-only \
  --file-target <이름> <같은 입력> <같은 독립 Print PDF> \
  --rhwp-bin <현재 Native 실행 파일> --compat 2022 --dpi 96 \
  --page-workers 1 --out output/pr-review/<검토명>/native \
  --font-path /usr/share/fonts --font-path /usr/local/share/fonts \
  --font-path /home/tsjang/.local/share/fonts --font-path assets/fonts
```

fresh WASM은 같은 명령에 `--wasm-pkg <현재 최적화한 fresh web package>`를 추가하고
별도 output에 산출한다. 모든 쪽 TSV와 직접 판독을 다시 확인한다. `not_evaluated`는
보조값 산출 상태이며 그 자체가 PR 승인 판정은 아니다.
