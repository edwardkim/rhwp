---
kind: pr-review
status: active
---

# PR #3636 review — Skia 구두점·합성 굵게·오른쪽 정렬

| 항목 | 값 |
| --- | --- |
| 작성자 / base | planet6897 / `devel` |
| head 참고값 | `f2da30e0de64d4d4e40d782cf82e27eabe0b0d97` |
| 범위 | Skia raster text, paragraph right alignment |
| 관련 이슈 | Refs #3386 |
| 권고 | 통합 PR로 반영 |

반각 강제 구두점의 Skia 0.5x 배치를 web canvas와 맞추고, 실제 bold face가 없는 typeface에만
합성 굵기를 적용한 경계가 적절하다. 셀 밖 오른쪽 정렬의 말미 공백 제외도 셀 내부 TAC를
보존한다. 통합 검토에서 말미 공백이 여러 run을 가로지를 때 마지막 run style을 재사용하던
오차를 `fac610dbe`로 보정했고 #1285 회귀 2건을 통과했다.

Native-Skia 라이브러리 58건, picture placeholder 2건, direct PDF export 4건 및 전체
release-test integration을 통과했다. #3386은 이 PR이 `Refs`만 하므로 close하지 않는다.
