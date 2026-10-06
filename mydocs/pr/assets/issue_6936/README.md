# #6936 최종 시각 증적과 실행 환경

시각 판정: [한컴 / 수정 전 / 수정 후 비교 패널](comparison.png)의 p.1, 12개 행 전체를 직접 확인했다.
돋움·바탕·굴림·새굴림의 일반 face 합성 굵게가 복원되고 함초롬의 실제 Bold face는 유지된다.
원시 PNG·JSON·로그는 `/private/tmp/rhwp-6936-pr-prepare-20260913/archived-evidence`에 보관하며
PR에 포함하지 않는다. 다음은 그 결과에서 확인한 사실을 옮긴 영구 기록이다.

## 입력 커밋 확인

초기 구현·증적 commit은 `01cd447a4`이며, 최종 코드 검증 commit `3ba6d6d3e`에서 아래 입력 6개를
실제로 사용한 파일 및 Git blob과 SHA-256으로 다시 대조했다. 최종 코드로 HWP를 다시 PDF로 내보낸
결과도 커밋된 `after` PDF와 바이트 단위로 동일하다. 따라서 아래 비교 패널과 측정값은 최종 코드에도 적용된다.

| 역할 | 저장소 경로 | SHA-256 |
| --- | --- | --- |
| 합성 원본 | `samples/issue6936/bold-faces.hwpx` | `c751bf79f44411377c2d58db158068cad04e6b1f0fa8d830a6e867d27dc8b59a` |
| 한컴 저장 시각 검증 원본 | `samples/issue6936/bold-faces.hwp` | `eefe5d89c29a8d7c8ba3a3148033fb3c5c2e960205ef1ff927ec36873089a3a7` |
| hancom | `pdf/issue6936-bold-faces-2020.pdf` | `5b0dc916098847ed3c6e3eadae468f33a9810ee58de3e91c1e2bb2f3a7e2ae12` |
| before | `pdf/issue6936-before.pdf` | `f5b390c4c6f347188d6530f4b359adfddca784db0f059aa5027c4b79e8902168` |
| stroke_only | `pdf/issue6936-stroke-only.pdf` | `96ca615af520ff04475b322ec27bd74ecd0051f3b5060ff3d3eb01293772df2d` |
| after | `pdf/issue6936-after.pdf` | `bf620e633f879d1e2f28ff3a46a55c41db51d9b94937cb9011eb78344dbf1ebd` |

## MCP 변환

`hwp2024-mcp-convert` client 20260824-011002, engine **2020**, 한컴 **12.0.0.4605**.
두 작업 모두 `start → status → download`로 성공 상태와 다운로드 결과를 확인했다.

| 입력 | job ID | 시간 | 출력 크기 |
| --- | --- | ---: | ---: |
| `samples/issue6936/bold-faces.hwpx` | `d9df76fa-2f86-4fbb-9225-3f8ebd55bab5` | 5967 ms | 24064 bytes |
| `samples/issue6936/bold-faces.hwp` | `7e4a7036-ed79-464c-9f77-83e15a49d970` | 14538 ms | 51890 bytes |

## 글꼴 환경

macOS arm64 CLI에서 Windows에 설치된 파일을 읽기 전용으로 복사해 `--font-path`로 제공했다.
원본 경로는 `win10-ted:C:/Windows/Fonts/<파일명>`. 라이선스가 있는 글꼴 파일 자체는 배포하지 않는다.

| 파일 | SHA-256 |
| --- | --- |
| `HANBatang.ttf` | `0dbeb7b129fcec63f8d6b7f1d3b5bb991c1d599e725286518ccc598de643f014` |
| `HANBatangB.ttf` | `91da0dde2995484b69fed48b01c27206dbce952a2b4820d70d0264d385b585b5` |
| `HANDotum.ttf` | `38126c8212411cfc4616d2d9463da75e46ae483f1d7d7f5bb2cd681de665c9ea` |
| `HANDotumB.ttf` | `a8270c516d1a0212c2c5953c2c577d094935c9bd7933e5c8eaca5ccb35c719c4` |
| `NGULIM.TTF` | `871b6190046c01ccabf52c80e393af623176c1a30a86a8a297f6f1acfbdd9b43` |
| `batang.ttc` | `84b0ba79a5d1c6012bbccf8d364ddc04a0c8b135d227a159196c558225cf1d89` |
| `gulim.ttc` | `4b9ac63e8920ed1bae29c068025ed30493464c18ee18617887daa18e59189226` |

## PDF 측정

| 산출물 | 페이지 | 공백 제외 문자 | show 연산 | Tr 횟수 |
| --- | ---: | ---: | ---: | --- |
| hancom | 1 | 158 | 60 | `{'2': 20}` |
| before | 1 | 158 | 158 | `{'0': 158}` |
| stroke_only | 1 | 207 | 207 | `{'0': 158, '1': 49}` |
| after | 1 | 158 | 158 | `{'0': 109, '2': 49}` |

한컴은 문자를 묶어 기록하므로 show 수가 rhwp와 다르다. 최종 rhwp는 glyph 158개를 한 번씩 기록하며
합성 굵게 49개에 `Tr 2`를 쓴다. 획만 추가한 대조군은 49자 중복(207자)이다.
New Gulim의 원래 face 복원으로 해당 두 행에서만 원점이 변했다(최대 5.200012pt).
나머지 10개 행의 수정 전후 원점 변화는 0이다. 합성 두께는 한컴의 `w/Tf=2.66/133=0.02`에 근거한다.
전체 페이지 픽셀 동일성 또는 비공개 사내 코퍼스 재현을 주장하지 않는다.

## 재생성

pypdf, pymupdf, Pillow가 있는 Python 환경에서:

```sh
python samples/issue6936/inspect_pdf.py --out-dir /tmp/rhwp-6936-pdf-evidence
```

스크립트는 커밋된 PDF 4개를 읽고 추출·연산·원점 보존을 검증한다. 원본과 대조군의 CLI 생성 방법은
[fixture 설명](../../../../samples/issue6936/README.md)에 있다.

## 쪽수 원장 등록

`tests/fixtures/oracle_page_count_baseline.tsv`에 `samples/issue6936/bold-faces.hwp\t1\t1` 한 행을
추가했다. 한컴 PDF 1페이지와 같은 HWP의 `rhwp info --json` pageCount=1,
lastSavedWith.product=`hancom-office-2022`, printMethodImpliesNup=false를 확인한 직접 대응이다.
기존 코퍼스 원장을 전수 재생성하거나 다른 문서의 기준값을 바꾸지 않았다. 한컴 저장 전 HWPX는
해당 HWP PDF의 별도 oracle 행으로 등록하지 않는다.
