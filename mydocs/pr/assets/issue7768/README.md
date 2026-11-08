# #7768 납세담보 확인서 — 통합 브랜치 재확인

원본: `samples/hwp3-table-caption.hwp`, SHA-256
`e8ac29656b1ca41f96b1dd9f6ef88be294c61165e2daf40f75873814291795e1`.
독립 한컴 Print: `mydocs/pr/assets/pr_6191/hancom/issue_6078_hancom_2020.pdf`,
SHA-256 `48940a3579d0fb3a9545c68e0b3d00da1094c9eea67773a1d781b958c7319305`, 전체1쪽.

| 출력 | 전체 쪽수 | 최저 실루엣 | 미달·누락 | 실제 review gate |
| --- | ---: | ---: | ---: | --- |
| Native | 1 | 92.73966% | 0 | passed |
| fresh WASM | 1 | 92.73966% | 0 | passed |

고정 base Native도92.73966%다. 앞선 #7568에서 통합된 배치의 무회귀를 확인했고
이를 이번 통합의 신규 시각 개선으로 보고하지 않는다. Native/fresh WASM 전1쪽 raster bytes는 같다.

- Native 완료 빌드 source: `7dd9572b7fecbc602c9e5ab47c4a6c9d6422b4bd`.
  실행 파일 SHA-256: `f04e0eafe1a7cfc0ea726452236d501924fd488cd80c5e7e4f3e2c1af87ae046`.
- optimized fresh WASM 빌드 source: `03a987f4aeded1d2b03415e7edecac5c833ca5cd`.
  WASM SHA-256: `e235c944e4921b80a30e347bf28f74503b0be02ce6398c41d280cf79bfb72cd5`.
  JS SHA-256: `57805b611572e0d2bf9d3a871e801698450d9a8785e7bc524ebc422f7ac924a7`.
  root `pkg/`와 Studio `public/` 바이트 일치 및 새 브라우저 초기화 응답을 확인했다.
- Print profile, compat2022,96dpi,`--embed-fonts full`.
  글꼴 공급은 `/usr/local/share/fonts/hancom-office-2020/All`, `Install`, `Hwp`의 실제 디렉터리다.
- rebase 뒤 추가 검사를 하지 않는 사용자 지시에 따라 Native 완료 빌드를 재사용했다.
  SVG 조판·parser·model·layout·typeset의 해당 경로는 동일하며, 이후 upstream 추가는
  Native Skia/Sixel 및 확장 프로그램 경로다. 중단된 재빌드는 성공으로 세지 않는다.

[Native review](native-review.png) / [overlay](native-overlay.png),
[fresh WASM review](wasm-review.png) / [overlay](wasm-overlay.png)를 같은 페이지로 생성했다.
양 경로의 [캡션](native-caption-zoom.png)·[체크박스](native-checkboxes-zoom.png)·
[후행 용지 규격](native-paper-spec-zoom.png)을 같은 배율로 확대했다.
캡션·표·후행 줄의 순서와 용지 내 포함, 추천기관 체크박스3개의 보존을 직접 확인했다.
글꼴 획·문자 간격과 인 표기의 모양 차이는 남으며 완전한 일치를 주장하지 않는다.
기존 관계형 회귀 `issue_6078_caption_table_and_paper_spec_keep_reading_order`는
최종 전체 회귀10,215PASS/실패0에 포함된다.

TSV·로그·중간 JSON·source/입력/글꼴 해시는 ignored
`output/pr-review/planet6897-20261008/linux-validation-20261011/`에 보존한다.
전쪽 TSV 원문은 통합 PR 본문에 Native/fresh WASM 각각 포함한다.
TSV-only의 `not_evaluated`를 바꾸지 않고 별도 실제 review/overlay의 canonical gate를 연결했다.

`canvaskit-type7.png`는 이 문서의 캡처가 아니라 별도의 synthetic PaintOp 진단이다.
실제 CanvasKit software surface에서 Type7 리더34개 점 구간과 실선·일반 점선 대조군을 확인했다.
이 진단을 독립 한컴 문서 일치나 정식 새 회귀 검사로 보고하지 않는다.
