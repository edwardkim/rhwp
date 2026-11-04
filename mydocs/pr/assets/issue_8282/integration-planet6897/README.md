# #8282 통합 후보의 독립 Print 대조

2026-11-01 KST, 96 DPI, Native Skia와 fresh WASM Canvas의 전체 쪽을 기존 한컴 2020 Print PDF와 비교했다. 네 입력은 각각 1쪽이며 두 경로 모두 raw 100.00%다. 각 저장본은 의도된 편집 후 HWPX로 생성한 합성 입력이며 독립 Print 결과와 구분한다.

- Native 실행 파일 SHA-256: `4b03859cfa21cdba1a766d5141d260adf70d3c986c6481a3e8440d1ab992ae96`.
- WASM SHA-256: `cee8e58be7191a14febfb73e8b79b2ad59946ac1c7f13332d46ae92157aef55f`.
- 이 증거는 제품 코드 생산 커밋 `6be28314c5`의 이전 통합 후보를 대상으로 한다. 이후 제품 코드가 변경됐으므로 최신 head의 재검증 결과로 취급하지 않는다. 시스템 글꼴을 사용했고 글꼴 파일은 포함하지 않는다.
- 8개 TSV의 출력 PNG와 review·overlay 생성에 쓰인 PNG는 각각 SHA-256이 같다. 네 입력 모두 두 경로의 Print와 비교해 글자처럼 그림, 어울림 개체, 줄 나눔과 다음 문단을 직접 확인했다.

| 입력 | HWPX SHA-256 | Print PDF SHA-256 | Native TSV | WASM TSV |
| --- | --- | --- | --- | --- |
| `tac_picture_square_picture_overlap` | `852872245590f3fccec43db326075c92c0324e91fc8e1b6eab8abaf5d9884554` | `2ff48072e4f122e266d552396699b692f24a9d01e6a95e312ede1dcbbcd3fb9e` | [1쪽](native-picture_overlap.tsv) | [1쪽](wasm-picture_overlap.tsv) |
| `tac_picture_square_textbox` | `6baae6814e620e32f9eebc9a2928a6c66ac8194322588a0293fdf0323fbf3bcd` | `b0cb291e344891951cb95a1643974059203525c04ad19bd110e2f5a792776346` | [1쪽](native-textbox.tsv) | [1쪽](wasm-textbox.tsv) |
| `tac_picture_square_textbox_left_overlap` | `c43a36fdb9f8b55675927c1570c61302dee0114b2c40f339bc3d662437349b80` | `bfee35fd2b2a1579537442fb30a12236c094ab9b0dd5a12ef48fb66de514120a` | [1쪽](native-textbox_left_overlap.tsv) | [1쪽](wasm-textbox_left_overlap.tsv) |
| `tac_picture_square_textbox_overlap` | `403dbebb8a0f36c203a70d6b13afa94879ec65b8d5a1a9da14b2087318f01936` | `149336312c2cf329072d64e04417e1796517ed0295cd44e6145f6d228e8cf567` | [1쪽](native-textbox_overlap.tsv) | [1쪽](wasm-textbox_overlap.tsv) |

대표 입력 `textbox_overlap`의 동일 쪽:

- Native [review](native-review-textbox-overlap.png) · [overlay](native-overlay-textbox-overlap.png)
- fresh WASM [review](wasm-review-textbox-overlap.png) · [overlay](wasm-overlay-textbox-overlap.png)
