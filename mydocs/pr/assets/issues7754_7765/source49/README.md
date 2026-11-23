# 실제 리베이스 후보와 원 글꼴 프로그램의 완료된 검증

코드 후보 `8f9fd0414ffcfb720fa4a601c79c4590480de0e8`, base
`8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`의 실제 Source105 빌드다. 후보 Native
`4c84e4b409dad93cae9057f4a1a5d5470a8fa5e799af27c2dcb6fb1b0d0aba3b`, 최적화 fresh
WASM `d02f8e1661a0eb51fbbe33212998755fb1f5c221ae1fac78b2083297461d9896`이며
잘못 재사용된 Source88 후보와 구분한다. 실제 컴파일 명령·전후 소스 해시와 수정 시각의
갱신 근거는 producer 영수증에 있다. Native/fresh WASM의 8개씩 작은 경계를 통과했다.

화학물질 문서는 103쪽 전쪽이 Native·fresh WASM 각각 100.00%다. 원 TTF·검증 TTC·
원 CFF 영문 프로그램을 명시 공급했다. 공급 목록에 없던 실제 글꼴 바이트를 보완하여
기존의 outline 재생 경로가 같은 프로그램을 소비한다. renderer·마스크·DPI·원본·PDF나
기대값을 수정해 수치를 맞추지 않았다. HFT 본문은 별도 한컴 전 글자표의 원 윤곽으로
보완했으며 대상 문서는 글꼴 생성 입력으로 사용하지 않았다.

개요 문서는 base·후보의 Native·fresh WASM 4조건 모두 9쪽 전쪽 100.00%다. 역사적
`4ade...` JSON은 정본 명령의 입력이 아니라 Native가 자동 생성했던 출력 snapshot이다.
그 파일의 바이트 복구를 주장하지 않는다. 정본과 같은 공개 원본 WOFF2·Native 변환본
해시, 정확히 일치하는 원 CFF 공급 및 독립 Print의 실제 TTF glyph 프로그램을 확인했다.
같은 이름의 DotumChe TTF는 개요에서 32 glyph가 달라 원본 자격이 없었다. 독립적으로
확인한 Gulim TTC 3번 face는 215개 glyph의 윤곽·전진폭·LSB가 모두 같아 선택했다.
화학물질의 관측 1,436개 TTF glyph도 모두 일치했다. 따라서 원 TTF → 검증 TTC → CFF의
순서를 유지하며 단순한 family 이름 일치로 원본 자격을 판정하지 않는다.

필수 편람 Native HWPX 383쪽 gate도 실제 Source105로 통과했다. 암호 원본은 두 출력
경로의 24쪽이 평문과 PNG 바이트까지 같고 전쪽 98% 이상이다. 새 전체 Rust·보호
46조건 전체 취합·WASM 원본 전쪽·Studio의 지표 준비 호출 검증 등 남은 실행은 완료 후
별도 근거로 확정한다. 이 폴더만으로 PR 준비 완료를 선언하지 않는다.

개인 글꼴은 Git에 포함하지 않았다. 보호 정본의 하한·TSV를 바꾸지 않았다. 디스크 부족을
해소할 때는 실제 base와 최신 Source105, 활성 Cargo/target, 모든 PDF·PNG·TSV·소스·
실행 로그를 보존하고, 거부된 중복 또는 사용이 끝난 구 producer 실행 파일만 확인 후
정리했다. 삭제 목록과 재현용 SHA는 영수증에 있다. 최신 지시는 PR 준비 후 대기이며
원격 push·PR 생성·병합을 수행하지 않는다.
