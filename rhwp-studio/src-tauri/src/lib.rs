#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    // 데스크톱 셸은 File System Access API 대신 네이티브 다이얼로그로 파일을 고른다.
    // WebView2에서 FSA picker도 동작하기는 하지만(2026-08-09 실측), 경로 대신 핸들만
    // 주어 저장이 원본 경로 덮어쓰기로 확정되지 않고, 창 단위 "picker active" 플래그
    // 때문에 열기 요청이 겹치면 "File picker already active"로 실패한다.
    // 그래서 dialog 플러그인의 네이티브 다이얼로그로 파일을 고르고
    // fs 플러그인으로 경로를 직접 읽고 쓴다. dialog 플러그인이 선택된 경로를
    // fs 런타임 스코프에 등록하므로 별도 정적 스코프 설정은 두지 않는다.
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
