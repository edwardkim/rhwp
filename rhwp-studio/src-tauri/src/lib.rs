#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    // WebView2는 File System Access API picker(showOpenFilePicker/showSaveFilePicker)를
    // 구현하지 않는다 — 함수는 존재하지만 호출하면 다이얼로그가 뜨지 않고 Promise가
    // 영원히 pending으로 남아, 두 번째 호출부터 "File picker already active"로 거부된다.
    // 데스크톱 셸에서는 dialog 플러그인의 네이티브 다이얼로그로 파일을 고르고
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
