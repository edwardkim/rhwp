# #8856 참조 번호 paint 칸·빈 그림 요청과 Native 검사 연결

stage12 `2c5e66b4d9` 뒤의 보정이다. 일반 참조와 제어 전용 표 호스트, 측정의 폭 예산,
실제 Native/SVG/Canvas가 같은 참조 스타일을 사용하게 했다. 본문 run의 양쪽 정렬·탭·자간을
paint가 쓰지 않는 참조 칸에 복사하지 않는다. 새 관계형2건은 실제2FAIL→2PASS,
기존 관련21건도 통과했다. 최소 글꼴95쪽은 Native97.60552%/fresh WASM97.63529%다.

원본 sample10의 비어 있는 그림 데이터3개는 소속·칸만 보존하고 decode 대상에서 제외한다.
실제 브라우저의 빈/손상 payload 대조는1PASS1FAIL→2PASS다. 손상된 비어 있지 않은 PNG 오류는 유지한다.
Native 그림자의 기존 exporter 검사도 기존 CI 모듈로 옮기고 공용 fixture를 공유했다.
현재 Native CI 진입점의22건과 suite Node 계약 검사를 통과했다. workflow/판정 정책 변경은 없다.

[같은95쪽 전후 증적·생산·검사 범위](../pr/assets/issue8856/stage13-note-reference-paint-cell/README.md)

숫자 glyph는 PDF의 Gulim과 현재 휴먼명조가 다르다. font descriptor 이름만으로 판단하지 않고
실제 내장 프로그램3글리프의 윤곽·hmtx를 설치 Gulim과 메모리에서 대조했다. 글꼴 소유 경로는
다음 단계에서 확인한다. 현재는 PR 준비 완료·이슈 전체 해결·새 head 전체 통과가 아니다.
임시 증적·로그는 ignored `output/issue8856/note-marker-paint-contract/`에 보존한다.
