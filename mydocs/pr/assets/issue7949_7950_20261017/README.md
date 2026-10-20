# #7949·#7950 공개 재현 PDF와 시각 증거

- 작성일: 2026-10-17 (Asia/Seoul)
- 검증 제품 source: `337cde2865bf6c926fdcd53018d420fb19207b24`
- 기준 devel: `552514f423ec91f1b7171f6e6b0882a4efe034f2`
- 실제 Native Skia와 fresh Docker WASM의 Print Canvas, compat 2022, 96dpi
- Native producer SHA-256: `7def5bd1f2cad4d65633f477918953cf49772f79175a099d1682dd4d52bee055`
- WASM SHA-256: `6dc88d238132563e40d332c53898e96ce1272df997d099923e9ec69adf9ee35b`

모든 PDF는 이번 검증을 위해 직접 작성한 공개 가능한 한 쪽 입력이다. 원본 PDF를
Poppler로 렌더한 결과를 독립 기준으로 비교했다. rhwp 출력에서 정답지를 만들지 않았고,
사용자 제공 PDF 원본·개인 정보·글꼴 파일을 이 폴더에 복사하지 않았다.
PDF의 고정 길이 xref 공백을 텍스트 정리로 바꾸지 않도록 이 폴더의 PDF만 binary로 지정했다.

| 원본 입력 | Native 2px 실루엣 | Docker WASM 2px 실루엣 | 입력 SHA-256 |
| --- | --- | --- | --- |
| [explicit-mask.pdf](explicit-mask.pdf) | 100.00000% | 100.00000% | `fac1f596e9d094c455855bcda6eca123fb913134016c99bec7d85005ee815f63` |
| [highres-smask.pdf](highres-smask.pdf) | 100.00000% | 100.00000% | `f3e6bcc53c065f5613b247cd2d8ee22c850bcef0b2ce53cbd3cbbe91e9393d47` |
| [smask-matte.pdf](smask-matte.pdf) | 100.00000% | 100.00000% | `505fb32ca88265aac1c7496674a04e14326a83a7b036564d8333271fd7bac012` |
| [paint-state.pdf](paint-state.pdf) | 99.99778% | 100.00000% | `ce44aa22487fdbf255a7a1f73fbadd0da6512c73ae7444c658236da171b56747` |
| [compound-winding.pdf](compound-winding.pdf) | 100.00000% | 100.00000% | `f2dfb7c237e00bf549b281d17c3805fc72ba9181889714d93b14635182d35d68` |
| [path-image-order.pdf](path-image-order.pdf) | 100.00000% | 100.00000% | `209af48309dfbda15431743b7cca35b070e034c9288e07b9a8aec58797a51159` |
| [contiguous-table.pdf](contiguous-table.pdf) | 99.94191% | 99.89551% | `f3906615e5a9e12a55edeba5ddb408b49c0b71bbe556217ca05cf3ef71954935` |
| [interleaved-table.pdf](interleaved-table.pdf) | 99.94147% | 99.89471% | `81c50716badd8ff5701a1295d69c6603b36299684d6ab8aeefb9f54a0ecec977` |
| [plain-table.pdf](plain-table.pdf) | 98.68496% | 97.75313% | `85e13d457908cd979b0535cba8b805e8c35efe6fbc0be76bb5f47b331a25e4e2` |

각 입력의 `*-native-compare.png`·`*-native-overlay.png`,
`*-wasm-compare.png`·`*-wasm-overlay.png`는 같은 페이지·배율의 새 출력이다.
compare는 rhwp 왼쪽/PDF 오른쪽, overlay는 `visual_sweep.py`의 표준 색 구분을 따른다.
점수와 별개로 마스크의 투명 방향·alpha 단계·Matte 색, 보이는/숨겨진 글자,
Bezier 구멍과 겹침 순서, 표 머리글·본문의 실제 가시성을 직접 대조했다.
실루엣 100%를 모든 색·보간·글리프까지 바이트 동일하다는 의미로 쓰지 않는다.

`table-visible-text-before-after.png`는 같은 원본의 독립 PDF, `d37cc9257`의 글자 가림,
최종 Native/WASM을 같은 배율로 나란히 보여준다. 그 수정 전 producer는 최종 PNG 검사에서
실제 글자 잉크 누락으로 실패하며 최종 producer는 통과한다. 원본 grid 축에서 셀 내부를
정하므로 출력 IR 좌표를 기대값으로 재사용하지 않는다. 배경과 겹치지 않는 일반 표는
semantic table로 남는 정상 대조군이다.

- 정식 회귀: [issue_7950_pdf_paint.rs](../../../../tests/cases/issue_7950_pdf_paint.rs)
- 원인·실제 소비 경로·최종 검증: [stage7](../../../working/task_m100_7949_stage7.md)
- 사양 근거: [PDF Reference 1.7](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf)의 이미지 마스크·graphics state·path painting 계약

재실행은 저장소 루트에서 Native + native-skia, 기본 Docker WASM을 준비하고 다음처럼 수행한다.
`INPUT`은 위 입력 중 하나이고 `BIN`은 공유 review target의 실제 검증 실행 파일이다.
글자가 있는 표 입력은 검증 환경과 같은 실제 공급 글꼴을 지정한다. 폰트를 배포하지는 않는다.

```bash
python scripts/visual_sweep.py --silhouette-only \
  --hwp "$INPUT" --pdf "$INPUT" --key public-paint \
  --compat 2022 --dpi 96 --rhwp-bin "$BIN" \
  --out output/pr-review/pdf-default-mask/native
python scripts/visual_sweep.py --silhouette-only \
  --hwp "$INPUT" --pdf "$INPUT" --key public-paint \
  --compat 2022 --dpi 96 --rhwp-bin "$BIN" --wasm-pkg pkg \
  --out output/pr-review/pdf-default-mask/wasm
```

전체 TSV·실행 로그·중간 JSON은 ignored `output/pr-review/issue7949/integration-20261017/`에
보존했다. PNG·PDF는 새로운 기준값/golden이 아니며 기존 baseline을 완화하지 않았다.
