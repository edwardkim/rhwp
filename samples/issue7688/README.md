# #7688 검증 입력과 독립 기준

기존 samples 입력은 원래 경로를 재사용한다. 이번 폴더의 PDF는 검증에 사용한 한컴 Print PDF와 바이트가 동일하며 기존 기준 PDF를 덮어쓰지 않는다.

MCP 기준은 Hancom 2020 11.0.0.9136, `pdf_print_method=0`, `hancom2020_pdf_driver_one_up`이다. #7333은 이미 추적된 한컴 PDF를 재사용한다.

| 대상 | 원문 | Print PDF | 원문 SHA-256 | PDF SHA-256 |
| --- | --- | --- | --- | --- |
| paragraph-line-basic | [lseg-01-basic.hwp](../lseg-01-basic.hwp) | [paragraph-line-basic-2020.pdf](reference/paragraph-line-basic-2020.pdf) | `86e607233a2218928fabe5969155116a46d4ab4400db1402fe45b0082149496c` | `6d99965c7ddc80fb5db24d770fe39d62e3db7f83c46fd9d9d18260e58d1dc040` |
| paragraph-basic | [lseg-05-tab.hwp](../lseg-05-tab.hwp) | [paragraph-basic-2020.pdf](reference/paragraph-basic-2020.pdf) | `0066108df04204324c0802414087563e7673e30f3d142dd014af14e318153d2c` | `91e39742d77c1dbc4986f96401d6dea564a110ebb925a1bc1bf3c62239097d12` |
| table-core | [hwp_table_test.hwp](../hwp_table_test.hwp) | [table-core-2020.pdf](reference/table-core-2020.pdf) | `dc3e57d7577447ae4b47f59bdb0f499a5ac29066b62af8a650bcdd8d70eee32a` | `c04ef8b2819529f83fe3e6043683982600535432426c8fb58f6e5cd31ea00963` |
| image-crop | [pic-crop-01.hwp](../pic-crop-01.hwp) | [image-crop-2020.pdf](reference/image-crop-2020.pdf) | `3b5afbe4cdb18faa49452a9cbed770d5caa27aed2f83b708e9ab8516c65ebf4c` | `c9d29de7680e5642a24818a6924ef5f42b18e1194766b693d636853d8782d16d` |
| equation-inline | [eq-01.hwp](../eq-01.hwp) | [equation-inline-2020.pdf](reference/equation-inline-2020.pdf) | `d4b5e4730da12395c953d16c026c979adfdf3235eee4be605cef0ab758a7e5ec` | `24ea8abd8d016149bdcc2df1e0e20fd04072ede95275fa09e12c61c3f6bf7780` |
| shape-group | [group-drawing-02.hwp](../group-drawing-02.hwp) | [shape-group-2020.pdf](reference/shape-group-2020.pdf) | `4a21523e19c2f191b7f6a23fff59c2b0b7838f42f2cc2a6d21ba3f28795181a7` | `b40a7187b099c10f4a6d866050f6cf39f80304648082ded969a38b744db89c6d` |
| scan | [evaluation_form_200dpi_scan.hwp](../issue3239/evaluation_form_200dpi_scan.hwp) | [scan-2020.pdf](reference/scan-2020.pdf) | `34e5ca1df74196c2ddcfa3b24cbda9cdb0099441dfc7e066d15a780d52668367` | `755f52297eec1a7a2e43ff07e09dace32cb18b86810f394298a94606ae007eb0` |
| sizes-1 | [control-1.hwpx](arrow-controls/control-1.hwpx) | [sizes-1-2020.pdf](reference/sizes-1-2020.pdf) | `e944bf6ce5b6edb7a80897370386690d038731dff2e2ac83ef9370a059b78f07` | `eceddbf1a29db893340bea3d96e26f72c9b383cb98b8133771ff2b9f30dd67d8` |
| sizes-2 | [control-2.hwpx](arrow-controls/control-2.hwpx) | [sizes-2-2020.pdf](reference/sizes-2-2020.pdf) | `1ea2cecb68affdfc2920b103a6fd4ab687c34ae1db09cd53cd01688c39406bd4` | `1906dbdb1c2364e0730d45fe4da3d6131973a53d18e1fea2eef1653f226dad04` |
| widths | [control-3.hwpx](arrow-controls/control-3.hwpx) | [widths-2020.pdf](reference/widths-2020.pdf) | `35e343d271763dac9f2355cfff35ac294180459212490ca8a6fe1f9322bb89e7` | `ad7726fdd4cd9b9f8e4e21c6aab5bef0005dc16c4013147addb4dc67d551fad9` |
| short | [control-short.hwpx](arrow-controls/control-short.hwpx) | [short-2020.pdf](reference/short-2020.pdf) | `8431f0f4446822cad43897292817aa718bb3b4e586b5fcf245fab162eccee986` | `5a9aecb3da24a8de2cb14c8a4705225b847a06bae9ef9323c1be5a3c702ecde0` |
| compound | [control-compound.hwpx](arrow-controls/control-compound.hwpx) | [compound-2020.pdf](reference/compound-2020.pdf) | `ab309908bce072332ab3b635a4a99ae58b1afe2806aada41871baefd321b804d` | `1bd27989056bead55663b4b8735a822e613519ce00c4fe59865dfd285570d6f4` |
| issue7333 | [aaaaaa.hwp](../issue7333/aaaaaa.hwp) | [aaaaaa-2020.pdf](../../pdf/issue7333/aaaaaa-2020.pdf) | `ab88150db2eb4652c945e6773ed40f61806782b6ca85694eb5f8c8ec390f96c5` | `fbc0ff34b909e8824e775872ff47ac7d6c904d2cb74504fbe9e7a8411b2c597b` |

화살촉 대조군은 `group-drawing-02.hwp`를 CLI `export-hwpx`로 변환한 뒤 선 스타일의 크기 코드·선 두께만 변경했다. control-1/2는 크기 0..7, control-3은 크기 8 및 두께 대조다. control-short는 저장 scale matrix로 선 길이를 변경했고, control-compound는 복합선 경로를 확인한다. 저장 줄·그룹·원문 내용은 대조군 생성에서 유지했다. 수동 변경 입력의 기대 형상은 rhwp 출력이 아닌 각각의 한컴 Print에서 관측했다.

상세 생성·독립 관측·정식 회귀 및 남은 범위는 [5단계 기록](../../mydocs/working/task_m100_7688_stage5.md)과 [최종 검증 기록](../../mydocs/working/task_m100_7688_stage7.md)을 따른다. 원본·검증 당시 임시 파일과 복사 hash 대조는 ignored `output/pr-review/renderer-backend-audit-20261009/final/submission-inputs.json`에 있다.
