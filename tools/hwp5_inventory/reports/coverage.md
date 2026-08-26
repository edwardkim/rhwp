# HWP5 inventory fatten coverage

- claim: `M-hwp5`
- issue: #5469
- cases: **63**
- tags: 45
- controls: 38

## Family

| family | cases |
|---|---:|
| `ctrl` | 1 |
| `docinfo` | 12 |
| `equation` | 1 |
| `field` | 10 |
| `form` | 1 |
| `note` | 5 |
| `page` | 5 |
| `para` | 8 |
| `shape` | 9 |
| `table` | 11 |

## Failure class

| class | cases |
|---|---:|
| `A` | 5 |
| `B` | 30 |
| `C` | 8 |
| `D` | 6 |
| `E` | 12 |
| `F` | 2 |

## Hancom judgment

| judgment | cases |
|---|---:|
| 열림 + 조판 실패 | 31 |
| 파일 손상 | 25 |
| 파일 읽기 오류 | 6 |
| 성공 | 1 |

## 하지 않은 것

- 시리얼라이저 페이지 수 로직 (#4882 석)
- canvaskit_policy / pdf / layout-anomaly / oracle_public / render_backend / proptest / fidelity_compare
- gym/
