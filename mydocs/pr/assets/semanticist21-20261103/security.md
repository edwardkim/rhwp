# 새 fixture의 보안 검증

신규 `samples/` 파일은 0개다. 새·수정된 검증 fixture 24건은 동일 Native API로 아래 세 정책을 실행했으며 72회 모두 clean이다. classifier·탐지 기준·allowlist는 바꾸지 않았다.

- Hidden text: `HiddenTextOptions::default()`.
- Injection: Low 이상, fields 포함, 실제 capabilities의 MCP 이름 164개 전체.
- Unicode: 정규 security corpus 검사와 같은 함수·문서/표/글상자/수식 순회.

최소 글꼴 파일 시스템 격리에서 실제 release rlib를 호출했다. 테스트용 경로를 samples로 가장한 symlink나 sample 예외는 추가하지 않았다.

| 검증 입력 | SHA-256 | Hidden / Injection / Unicode |
| --- | --- | --- |
| `tests/fixtures/hancom_rotated/rotated.hwp` | `06968097d9f9e2b0e9d4159dd838e04591be204794b82d695a0ee1fe6e9ec2a3` | 0 / 0 / 0 |
| `tests/fixtures/hancom_rotated/rotated.hwpx` | `7e9762de41fa0ec1c38304e42fb31144e741b7b44d0bd2dc9dda40360e09df7b` | 0 / 0 / 0 |
| `tests/fixtures/hancom_table_grow/grow3-after.hwpx` | `48611171d47efd415d88f932e76b3687c9b9b9e45271fea32c6e54100fff28b7` | 0 / 0 / 0 |
| `tests/fixtures/hancom_table_grow/grow3-before.hwpx` | `fd15d23befba672bfe13261aa5151c973d19e2ee504931a4a0fa4f3aabf823f5` | 0 / 0 / 0 |
| `tests/fixtures/header_overflow/header-overflow.hwp` | `7ed7d9fcf74012e0e7c5dbfdb5943d54920202e577ce2f39b49da03f3a1c8546` | 0 / 0 / 0 |
| `tests/fixtures/header_overflow/header-overflow.hwpx` | `d8338073c5b23bfbf13d08e0b54a1498863dcf304e301c1cf4f8de7b2ba3d0ec` | 0 / 0 / 0 |
| `tests/fixtures/nested_wide_clip/nested-wide-hancom.hwpx` | `566ca9c8fb99d916eb4190eb2c2b07c67c3fcde374bbca5f329a38cdf789ad04` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/forms-before.hwpx` | `0453f21c1af1d3ecee0520044917f56d126ac8372150b8bd8e6e7c353bd71415` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/forms-edited.hwpx` | `5c3a81a073df5693837b62ef41f3848879c7e57a72394333f6cc7fc7543fc711` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/grow-nested-unchanged.hwpx` | `1a27ebb0a45da16558b37494507edfbbbb8ba8e62ad265fc3fb294355030c28f` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/grow-nested.hwpx` | `eaa6a17f56f80b2869fdcbab993220dd198b7511576a340004a7c46e0f35522b` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/no-lineseg-empty.hwpx` | `79df85cd755245717c3fec799a66a33e1fd8459a3fece2f63fe5e6377cb27ade` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/no-lineseg-text.hwpx` | `63b9997bd8d35eeabe599ed21f17608ee061f80312fb6a2d12562d8587c169fa` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/polygon-270.hwpx` | `07a367f2a2c5b00cc1945dd7355c701ad9a093a39ddd151e1d87340bebc9dde5` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/polygon-90.hwpx` | `fa78819178e579a453f9c02978926a1a5eb22625db9288d117c9386d5de38929` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/polygon-hflip.hwpx` | `501b1dc56f960d5a825728f3fc420fa72c371415f25ca96bace8cd7c67cb0764` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/polygon-resize.hwpx` | `8b0387e44139652a27b968368241d7e43de266c6f63b3282deaf577fdaa40437` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/polygon-vflip.hwpx` | `4b41f0564711624a9e569cfb1ca124a89377888f8cd368dfdfa1db8bbeefeb2f` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/spacing-after.hwpx` | `ba8b3204b123a1fe8fbd87f3916a4698a4c046a7a55575d29cbeeb3103b5ea1f` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/spacing-before.hwpx` | `edbf65bdf2a50caa53f9ad285f6433b70f927a5ecf9142a240d269a26ad776b4` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/spacing.hwpx` | `2720e9840e51f031b3dfec482f75a1f88c018276069e0d08882d1e672ac7f66a` | 0 / 0 / 0 |
| `tests/fixtures/semanticist21-20261103/no-lineseg-wrap.hwpx` | `e17ce3b14c21242843db591bd6d728a4869e4480186ce8642b77aea52314ce32` | 0 / 0 / 0 |

| `mydocs/pr/assets/semanticist21-20261103/saved/notice-edited.hwpx` | `567ec4aabba351a3936285479ff31d5e8d11570a033e770e1662347f7b7ce3c0` | 0 / 0 / 0 |
| `mydocs/pr/assets/semanticist21-20261103/saved/grow-nested-edited.hwpx` | `4eb9db7414cd35336ec1a965e62b45783af477bcb150fae05fe7ce86b0e5717c` | 0 / 0 / 0 |

실제 두 backend 저장본은 같은 SHA-256이므로 입력당 한 사본을 보존했다. 최종 두 저장 입력까지 포함한 24개 전체를 같은 세 정책으로 다시 실행해 72회 clean을 확인했다.

## 전체 회귀 첫 실행의 설정 오류

`RHWP_SECURITY_SWEEP_SAMPLES_JSON`은 `samples/` 전용인데 첫 실행에서 `tests/fixtures/` 21건을 전달했다. 그 결과 12,464건 중 12,463 PASS·해당 경로 전제 1 FAIL이었다. 제품 실패나 보안 finding으로 해석하지 않았으며 실패 실행을 숨기지 않았다. 신규 samples가 없으므로 이 환경변수를 해제하고 전체 회귀를 다시 실행해 12,465 PASS·45 skip·0 FAIL을 확인했다. 추가된 그림 단독 마지막 줄 회귀까지 포함한 결과다.
