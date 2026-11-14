# 최소 글꼴 집중 회귀

2026-11-14 KST. bwrap에서 사용자 HOME·상용 글꼴·시스템 추가 글꼴·Visual Sweep 웹폰트 캐시를 숨겼다. Fontconfig 노출 5개 프로그램은 `exposed-fonts.txt`를 따른다. 자동 수집 suite의 컴파일에는 라이선스를 확인한 명시적 fixture `ttfs/redistributable/happiness-sans/HappinessSansVF.ttf`도 허용했다. 이는 ambient 시스템 글꼴 요구가 아니다.

최종 집중 회귀 12건은 전부 통과했다. 첫 시도에서는 새 worktree의 ignored suite wrapper를 준비하지 않아 컴파일하지 못했고, `--prepare` 뒤에는 명시적 Happiness fixture를 격리에서 허용하지 않아 컴파일하지 못했다. 두 실행 준비 문제를 보정한 최종 실행 결과만 통과로 계산했다. 제품 assertion 실패로 분류하지 않았다. nextest 0.9.137의 권장 버전 0.9.140 경고는 유지했으며 도구를 임의로 갱신하지 않았다.
