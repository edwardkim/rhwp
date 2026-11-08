# #7642 고정폭 공백 회귀 원본의 재검증

실패 검사 `issue_6597_figure_space_advance`는 실물 절대 위치 대신 일반 공백과
고정폭 공백의 폭 비율을 검사한다. 다만 현재 출력의 TextLine 선택 결과가 빈
배열이어서 실패했다. 아직 기대값을 바꾸거나 새 회귀의 통과를 선언하지 않았다.

- code source: `fcd86c8ae30515dfdac3cdbaedaa466f3b81ebe5`
- 원본: `samples/issue6060/30307_local_service_reform.hwp`
- 입력 SHA-256: `cc8668819b79ed51408a93c0ad7876569bf67e8fed56dbfb7febdd0c9a91cded`
- 독립 한컴 2020 Print: `pdf/issue6060/30307_local_service_reform-2020.pdf`
- PDF SHA-256: `0c1baf52bddc2c1e204d12dce8e26e5fd2078f6a5c4eb4d1b7d9caa6309f549f`
- fresh WASM: `ab407d8560f8d8bb339ad5ad3143a9f819f27c13fb8cfe039aefe19e98ff5b71`
- print profile, compat 2022, 96dpi, 전체 글꼴 바이트 공급이다.

| 실제 출력 | 전체 쪽수 | 최저 일치율 | 90% 미만 |
| --- | ---: | ---: | ---: |
| Native | 13/13 | 93.19077% | 0 |
| fresh WASM | 13/13 | 93.19077% | 0 |

| 대표 쪽 | Native | fresh WASM |
| --- | --- | --- |
| 3 | [review](native168-p003-review.png) | [review](wasm168-p003-review.png) |
| 5 | [review](native168-p005-review.png) | [review](wasm168-p005-review.png) |

5쪽의 글머리·표·본문 순서와 내용 보존을 대조했다. 글자 형태·굵기 차이는 점수와
별도로 남아 있다. 실제 공급한 `돋움` 글꼴 별칭의 바이트는 `Noto Sans KR ExtraLight`
이며 HY중고딕·굴림·맑은 고딕은 각각 실제 이름과 대응한다. 이를 원본 돋움의
완전한 공급으로 보고하지 않는다. Unicode cmap에 따옴표와 해당 한글은 있지만
cmap 검사만으로 실제 브라우저 선택과 글자의 모양을 입증하지 않는다. 고정폭
공백의 원본 관측과 새 검사 소속·내용 보존, 확대 판독은 계속 확인해야 한다.

전쪽 TSV·실행 명령·빌드/입력 해시·글꼴 공급/복원 증거는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`의
`stage168-figure-space-original-*`에 있다. 평균/대표 점수를 전체 판정으로 쓰지
않으며 이 입력의 통과를 55개 원본 전체 검증 완료로 바꾸어 보고하지 않는다.
