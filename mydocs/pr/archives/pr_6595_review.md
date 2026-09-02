# PR #6595 검토 - 글자처럼 그림의 아래 캡션 배치

- 원 PR head: `a0596bf0d7ae6866d24e189bba8b36bfda6787ea`
- 통합 기준: `upstream/devel` `51043f5f8d0453b9bc929233de443fa60cb3df4b`
- 통합 후보: cherry-pick `67bdcd8ae`를 포함한 `9088bd705cafd004d703fcf4fa1a40002e9e3bee`와 현재 로컬 보정
- reviewer: `jangster77` 요청 완료

## 판정: 승인

`#6593`의 글자처럼 그림 아래 캡션이 그림 잉크 상자가 아니라 그림 상자 바닥을 기준으로 흐르도록 한 변경이다.
통합 후보의 focused 회귀와 전체 nextest에 차단 finding이 없다.

## 검증 및 증적

- 전체 nextest: `8951 passed`, `46 skipped`.
- `samples/issue6575/156489219_satellite_pm_release.hwp` 5쪽과 `pdf/156489219_satellite_pm_release-2020.pdf` 직접 비교를 생성했다. page diff는 `13.27%`이며, 전체 페이지 수치로 캡션 결함 외 범위를 판정하지 않는다.
- [stable review PNG](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6595_issue6593_p005.png), [manifest](../assets/pr_6595_6631_jeong_sik_integration_20260902/manifest.tsv)

원 PR은 직접 merge하지 않고 승인된 통합 PR에서만 수용한다.
