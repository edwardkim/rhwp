# P4 첫 합성 절편의 독립 PDF 원본

`generate_composition_sources.py --output <로컬 경로>`는 제품 코드와 독립적으로
원본 연산자·기대 소속 주석을 만든다. 아래 14개는 Native/fresh Docker WASM의
PDF·HWP·HWPX 선행 시각 조건과 변경 영역 직접 판독 뒤 정식 검사에 등록했다.
원본 선분의 길이 × 0.5pt에서 경계 그림의 면적을 정하며, 공통 배경의 면적·자식 표
영역은 명시적 사각형 연산자에서 정한다. 구현 helper 결과를 기대값으로 재인용하지 않는다.

| 원본 | SHA-256 | Native 전 형식 최저 % | WASM 전 형식 최저 % |
| --- | --- | --- | --- |
| cell-nested.pdf | 79631eabb9786d075c513774bcef1d8a484bcfb53ae73c11128b2182d0618b90 | 99.98107 | 99.98086 |
| cell-picture-crossing.pdf | 0f52d80f08d579aa3a8b5c29f4073b66bf226f2c5d720b725a6cbc9bd051c930 | 100.00000 | 100.00000 |
| cell-picture-first.pdf | c70ea86d888cf412a4a434a2779648dcab825783b042ec6e34543f95a920ad24 | 99.99115 | 99.99108 |
| cell-picture-overlap.pdf | 444f80f9ef5856f0a2f999e3624081753c80156de9bf37fd6675013d7e4640b8 | 100.00000 | 100.00000 |
| cell-picture.pdf | c17cb37701c3f535b24059ac91f0e31ff34ee710310ea89e25314edafa377da9 | 100.00000 | 100.00000 |
| cell-two-pictures.pdf | 33fad600a876513d0b68e1805e6912178e851d9b830a1c38b016ebf6bf04dc52 | 100.00000 | 100.00000 |
| nested-periodic-background.pdf | 5a6e80dae0e0f7ad4b7d4e3f7030f8cc8460cbf01f47a8ce067edd55b6a6ca9a | 100.00000 | 100.00000 |
| nested-periodic-both.pdf | 5a184a678519612c499fd13295503e05014d1c675ef7c2356e3ef9609127fe31 | 99.95913 | 99.95846 |
| nested-periodic-child.pdf | b2ac4affe41d8daa1ba2387ac6edbfce532cc8166f904d694007e6ad8bd32b99 | 99.96083 | 99.96022 |
| nested-periodic-deep.pdf | 5cc41c9f5aef01841290e9f8516f2f8b6fb3fff20d88248c96271bfca9080a9a | 99.96642 | 99.96585 |
| nested-periodic-first.pdf | 16415af50a3f151b50c1f5a5942e2fd5b5c1b53e02420686dc1d8b6f622a2d3c | 99.98640 | 99.98619 |
| nested-periodic-image.pdf | 22aac8117cb8eee973d7c480a2a91302fdbcbb34c69443ff2cff505cfe479bfa | 99.96569 | 99.96523 |
| nested-periodic-parent.pdf | a1cdef0da49941f27ad175dc33ca711b85666ba935c875da378e12d12b6f307c | 99.95984 | 99.95920 |
| nested-periodic-siblings.pdf | 8a0510df67559ba25c06116b596f80a5f2908deeceebcebfddea40f193e3a0f2 | 99.98792 | 99.98774 |

producer 기준: `4768e389f` 위 P4 변경. 전체 source 지문·명령·전쪽 TSV는
[stage4 보고](../../mydocs/working/task_m100_7998_stage4.md)에 연결했다.
Native SHA-256: `4407cc5b5b016e796e64c2a5c1b452910369f6fa90e5c159255825970892491f`.
fresh Docker WASM SHA-256:
`c7a6c40044be28a27f1a2d953798a602c1ba5884f6685fda031a10ec9fab02d1`.

첫 절편 당시 `--include-compound-hold`로 PDF dash 연산자 원본 7개를 별도 보존했고,
위 첫 절편 회귀에 포함하지 않았다. 두 번째 절편에서 같은 원본의 복합 경계 보존을
구현·검증했다. 현재 생성기 옵션은 `--include-compound`이며 과거 이름도 alias로 지원한다.
[복합 경계 원본과 검증](compound-sources.md)을 따른다. 서로 다른 원본의 증거를 대체하지 않는다.
경계 침범/교차 2개는 내용 보존을 검사하는 반례이며 작성 객체 복원 goldens가 아니다.
기준 PDF는 이 독립 원본이며 한컴 열기·인쇄와 최신 devel 비교는 P5에 남는다.
같은 backend의 이전 P3 PNG 132쪽은 각각 byte-identical이었다. 새 중첩 원본의
이전 positional 출력 대비 미세 하락은 stage4와 검토 화면에 보존한다.
