# #1470 새 개체 캡션의 번호·물리 줄 검증 입력

기존 Hancom2020 저장 빈 문서 `samples/hwpx/hancom-hwp/blank_hwpx.hwp`에서 공개 API로
표/그림3개를 삽입했다. 캡션 전체, 중간 캡션만 삭제, 마지막 표의 왼쪽 캡션 변형이다.
원본 빈 문서는 변경하지 않았다. `tests/cases/issue_1470_authored_caption_numbering.rs`에 같은
공개 편집 연산과 번호 소유·저장 왕복 검사를 둔다. 그림은 의도적으로1px 투명 PNG다.

독립 기준은 Hancom2020 11.0.0.9136 / managed direct DLL host / 32bit,
Print method0 / hancom2020_pdf_driver_one_up이다. 입력 전처리는
HWPX는none, HWP는scripts_disabled이며, 각각 같은 실제 입력 SHA에 연결된 독립 출력이다.
요청 제한1800초이며 모두1쪽이다. 새 생성 입력과 이전 실패 입력을 바꾸어 쓰지 않았다.
표·그림의 번호 종류는 각각 TABLE/PICTURE이며, 가운데 캡션만 지워도 개체는 남으므로
표시1·3을 유지한다. 저장 number 캐시를 표시 assigned_number로 덮어쓰지 않는다.

[전쪽 TSV·대표 review/overlay](../../../mydocs/pr/assets/issue7445/completed-20261009/stage24-20261021/README.md).
10개 입력×1쪽의 Native/fresh WASM raw2px 최저는 각각100.00000%다.
이는 엄격 RGB 일치가 아니다. 캡션 위치와 내용·원래 개체3개 보존을 직접 대조했고,
검정 괘선의 래스터 진하기 차이는 남는다.

| 입력 | 입력 SHA256 | 독립 Print PDF | PDF SHA256 | 변환 job |
|---|---|---|---|---|
| `tables-all.hwpx` | `35e5702a6054cbf4311a771a0a3faf6942c041da54156dacd9c170201021897d` | [tables-all-hwpx-2020.pdf](../../../pdf/issue1470_authored_caption_numbering/tables-all-hwpx-2020.pdf) | `f7567b3bab53b356b3801b0e254babc041236da679a23f9675d6fa8e17f3bacd` | `6cc92d9a-0c0b-42c7-8724-95e37ab18166` |
| `tables-all.hwp` | `eeaded2d82304d37e8094caf49d6774a3700782ca576c05c0c9c8e0d44237f16` | [tables-all-hwp-2020.pdf](../../../pdf/issue1470_authored_caption_numbering/tables-all-hwp-2020.pdf) | `857797a18e209992c6e17e2d17c27cdbdfebe4833f6968cad40c25cca76bdb78` | `b8f27df2-b3d0-455d-b85c-83a4ff7ddb24` |
| `tables-caption-removed.hwpx` | `9752163cfbc0b0b339ea38d6c3f1a1f247ff1515c78cb15cd0752ffc4e43578a` | [tables-caption-removed-hwpx-2020.pdf](../../../pdf/issue1470_authored_caption_numbering/tables-caption-removed-hwpx-2020.pdf) | `db3f2c82ef558265eb5d610bbd555b76b5fb1ef8a724509a3f8f47c684212273` | `4938216b-8d99-471e-8ac4-1b32a9431c8c` |
| `tables-caption-removed.hwp` | `c5bb25260921eb288f843b1d55b0bd69bbd9abf6ed332876bccdec328cd11fdf` | [tables-caption-removed-hwp-2020.pdf](../../../pdf/issue1470_authored_caption_numbering/tables-caption-removed-hwp-2020.pdf) | `2abe3989c480a1b0340e1f55fb8a75dec532894d6e620ae0f5090edf5ee40612` | `f7f58c01-780d-440e-b32a-783a6ff18b16` |
| `pictures-all.hwpx` | `90ffcfe89d3530ec6f9edc9ac4ae049e95cf0350a1871b51fd6602d2009c6f9a` | [pictures-all-hwpx-2020.pdf](../../../pdf/issue1470_authored_caption_numbering/pictures-all-hwpx-2020.pdf) | `4163e727c837c2d7ae83469285a2a0d39ceef3899f5b6c64b8e144210a7373e4` | `7ae06a97-d69d-40cd-9926-bf64199e6cf3` |
| `pictures-all.hwp` | `4c326ea66de00a03be8b773281519f9495c7e3c4ec1513a468420562de52b592` | [pictures-all-hwp-2020.pdf](../../../pdf/issue1470_authored_caption_numbering/pictures-all-hwp-2020.pdf) | `57ded8283ffce04022a4e106eb0afc2e7e34ff3c89df69e80d67b97253466b90` | `f2060fe7-5c66-42b7-9f94-7ff27053c936` |
| `pictures-caption-removed.hwpx` | `3cb0898a1257471b8d98e6c7c2b1f539d03ac9c847cb938e5f83586b34a5c140` | [pictures-caption-removed-hwpx-2020.pdf](../../../pdf/issue1470_authored_caption_numbering/pictures-caption-removed-hwpx-2020.pdf) | `fdac48e1efc99dbdae4b60b3231875e17c582dfaab915a6763703e688e239312` | `143ef42f-f6e5-49da-b87e-14fdd1c88aaa` |
| `pictures-caption-removed.hwp` | `e7c00a42726c4d577ed0d82db7ec526a876223d2f3f6a58c4d87e59293d63173` | [pictures-caption-removed-hwp-2020.pdf](../../../pdf/issue1470_authored_caption_numbering/pictures-caption-removed-hwp-2020.pdf) | `d9d8be451d8de8ecebd0cd9066e95f8e8ddc0b88bba0392cbea640a756f61704` | `f5fb9e57-771b-41fc-8a06-43d464649eaa` |
| `tables-caption-left.hwpx` | `f1368a7439003ee52c92f28b7a7288e892aef924325e6b73182310701ba0928b` | [tables-caption-left-hwpx-2020.pdf](../../../pdf/issue1470_authored_caption_numbering/tables-caption-left-hwpx-2020.pdf) | `6287d318c8cf2dda0a7fc8512728f4580f151fd11213eba3d0924c3f02cac2c6` | `68565516-af46-4748-b483-84894ec71ba6` |
| `tables-caption-left.hwp` | `fe7c84a6e5f7664acdf5b52a2d07b936a86b9089f1a820724a09651f2c9088b9` | [tables-caption-left-hwp-2020.pdf](../../../pdf/issue1470_authored_caption_numbering/tables-caption-left-hwp-2020.pdf) | `7d23ffe745c24939a0661901a61470343ec03494b74fa5cb458df5fa4c58774b` | `0bdd1db6-e191-4363-a6de-bbbc57aabc71` |

## 실제 전처리 모드

- `tables-all-hwpx`: `none`
- `tables-all-hwp`: `scripts_disabled`
- `tables-caption-removed-hwpx`: `none`
- `tables-caption-removed-hwp`: `scripts_disabled`
- `pictures-all-hwpx`: `none`
- `pictures-all-hwp`: `scripts_disabled`
- `pictures-caption-removed-hwpx`: `none`
- `pictures-caption-removed-hwp`: `scripts_disabled`
- `tables-caption-left-hwpx`: `none`
- `tables-caption-left-hwp`: `scripts_disabled`
