# 빈 문단 뒤 서식 변경 및 실제 저장 쪽 경계

PR #7468의 두 편집 API (`apply_char_format_native`, `set_char_shape_id_native`)를
검증하는 공개 합성 입력입니다. 개인정보나 외부 문서 내용은 포함하지 않습니다.

`edited.hwp`는 `saved/blank2010.hwp`에서 `Title`, `A`, `B`, 빈 문단, `C`, `D`를
순서대로 삽입한 문서입니다. Title에 20pt를 적용한 뒤 문단을 삽입하여 그 서식을
상속하고, B와 C에 13pt/bold를 적용했습니다. `deleted.hwp`는 그 상태에서 빈 문단을
삭제한 결과입니다. 두 API로 생성한 HWP 바이트가 각각 동일함을 확인했습니다.
회귀 검사는 출력에서 읽은 절대 좌표 대신 입력의 160% 줄간격과 글자 크기로 계산한
상대 줄 전진, 텍스트 순서, 소속 문단/쪽, 삭제 전후 이동 및 저장·재열기를 확인합니다.

`stored-control.hwp`는 실제 한컴 2020이 다시 저장한 2쪽 문서입니다. 생성 절차는
blank2010에 `Line 001`을 삽입하고 10pt를 적용한 다음, 같은 서식을 상속하는 새 문단에
`Line 002`부터 `Line 070`까지 삽입하는 것입니다. 수동 쪽 나눔, line-seg tag나 위치를
조작하지 않았습니다. 이 합성 HWP를 한컴으로 열어 HWP로 다시 저장했습니다.
0-based 문단 40의 원본 저장 vpos는 양수이며 문단 41의 원본 저장 vpos는 0입니다.
검사는 이 실제 쪽 경계 양쪽을 두 API로 편집하고 모든 70줄의 소속 쪽/문단과 상대
위치를 편집 전 및 재열기 후와 비교합니다.

대응 PDF는 `pdf/pr7468-{edited,deleted,stored-control}-print-2020.pdf`입니다.
각 HWP를 독립 한컴 Print로 변환했고 `pdf_print_method=0`,
`pdf_output_mode=hancom2020_pdf_driver_one_up`, 한컴 `11.0.0.9136`을 확인했습니다.
최종 프로덕션 소스의 Native와 새로 최적화 빌드한 WASM으로 전체 4쪽을 비교한 뒤에만
이 fixture와 검사를 추가했습니다. 두 backend 모두 2px 관용 내용 실루엣 최저
99.95857%였으며, 이는 획 모양의 완전한 일치를 뜻하지 않습니다.

| 입력 | SHA-256 | 독립 Print job |
| --- | --- | --- |
| edited.hwp | `ccfe2eb35dacdc84dd8340b156d6dc00beb6192cfae80bf13147f27241ce7858` | `5c2ac960-023b-4326-a339-5343dd57bfc7` |
| deleted.hwp | `fbc3d8703b785432882562133d80dfebf80762ab4fd5f8d78a5b06bda1e0d686` | `aaf381fa-2385-4ebc-80c0-f71e25e90635` |
| stored-control.hwp | `3cbdcef6542378de4ff54d1cce09ab254119ccaba2ffeb5be0c6be12a865cba7` | `49dc6a3f-2eb0-493e-8845-0d202aabdac4` |

stored-control의 한컴 HWP 저장 job은 `d23f75aa-2bdb-4510-b129-cc39f6e42c45`입니다.
PDF SHA-256은 아래와 같습니다.

- edited: `e189dd73ce3bc7b1bc9022f756d5af209640daf526e52d1f18c81c4a7bc03aab`
- deleted: `1df77f19d934463b4410bedc7b91bae8716d6ea01059297fce50634767ad67d2`
- stored-control: `08cf3f48ff60f3d4914a06f41a01b86f2567dfe96219055455606b1587fcc74b`

이 실제 저장 경계 검사는 이전 PR의 sample16 로드 시 64쪽만 확인하던 검사를
강화합니다. sample16의 기존 65/64쪽 및 일부 쪽 시각 미달은 upstream에서 이미
[이슈 #7445](https://github.com/edwardkim/rhwp/issues/7445)로 이관되었습니다.
이번 변경은 그 이슈를 해결하거나 기대값을 65로 완화하지 않습니다. 해당 문서의
기존 검사는 유지하고, 여기서는 전쪽 시각 검증을 통과한 실제 저장 경계를 사용합니다.
