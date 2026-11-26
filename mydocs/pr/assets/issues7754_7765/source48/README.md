# 공유 Cargo 캐시의 잘못된 후보 산출물 수용 중단

Source88의 base와 후보에는 생산 파일 52개 차이가 있지만 Native 바이너리·실제 deps rlib와
WASM 바이트가 모두 같았다. 후보 Native 22초·WASM Rust 빌드 7초 내외의 로그에는 실제
rhwp 재컴파일이 없었다. 이전 checkout 파일 시각이 base의 새 fingerprint보다 앞서 공유
Cargo target이 base 산출물을 재사용한 것이다. 전후 소스 해시만으로 실제 컴파일을
입증할 수 없음을 확인하고 해당 후보 측정을 수용 증거에서 제외했다.

해당 후보를 소비한 Source90 candidate·91·101·102·103을 종료하고 전체·부분 결과와
PID 경계를 보존했다. Source88의 실제 base 산출물과 이를 쓴 Source90/98 base 수집은
유효하게 유지한다. 소스·기대값·PDF·마스크·정본을 바꾸지 않았다.

Source105는 후보 생산 파일의 바이트 해시를 확인한 채 수정 시각만 갱신해 Native와 최적화
fresh WASM을 재컴파일한다. Cargo의 실제 `Compiling rhwp`와 명령·source 경로·산출물
SHA를 남기고 base 바이너리 재사용을 거부한다. Source107에는 base의 HWPX 53쪽
19.46% 미달을 포함한 실제 경계 4쪽을 추가해 후보의 98% 이상과 함께 재빌드 유효성을
확인한다. 새 Source106/108/109/110이 기존 필수 회귀·편람·보호·전쪽 비교를 수행한다.

사용자의 최신 범위는 PR 준비 후 대기다. 원격 push·PR 생성·병합을 수행하지 않는다.
신명조 독립 글자표는 7쪽 중 6쪽에서 600초 한도로 실패했으므로 동일 독립 입력을
1,800초 한도로 재시도한다. Windows rhwp 직접 PDF 기능 테스트는 계속 제외한다.
