# #5706 스킬 검증 — rhwp 실렌더

터미널 창은 증적이 아니다. 아래 PNG 는 	arget/release/rhwp.exe(v0.8.4) 가
export-svg --profile print 로 그린 페이지를 Edge headless 로 찍은 것이다.

## 라우터가 고른 대표 문서

| 스킬 군 | 샘플 | 화면 |
|---|---|---|
| 온보딩·CLI·트리아지 | samples/basic/english.hwp | [english](_renders/english.png) |
| 서식 채움 (전) | samples/basic/BlogForm_BookReview.hwp | [form](_renders/form.png) |
| 서식 채움 (후) | output/filled-book.hwp 4칸 기입 | [form-filled](_renders/form-filled.png) |
| 보안·출처 | samples/basic/request.hwp | [request](_renders/request.png) |
| 표·시각회귀 | samples/basic/issue1994_behindtext_table_20200830.hwp | [table](_renders/table.png) |

스킬별 폴더는 skills/<id>/README.md + page.png.
