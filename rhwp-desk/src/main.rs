// rhwp-desk — HWP/HWPX 문서를 위한 에이전트 워크벤치 (M0+M1).
// 진입점은 셸 부트스트랩만 담당하고 로직은 모듈에 있다.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod caps;
mod commands;
mod engine;
mod journal;
mod planner;
mod planner_net;
mod tool_ontology;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            app.manage(commands::AppState { data_dir });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_engine,
            commands::load_capabilities,
            commands::run_tool,
            commands::render_page,
            commands::read_journal,
            commands::startup_args,
            commands::path_kind,
            commands::list_documents,
            commands::probe_local_llm,
            commands::planner_list_models,
            commands::planner_test,
            commands::planner_chat,
            commands::load_mcp_tools,
            commands::mcp_to_openai_tools,
            commands::map_tool_call,
            commands::secret_set,
            commands::secret_exists,
            commands::secret_delete,
            tool_ontology::tool_ontology,
        ])
        .run(tauri::generate_context!())
        .expect("rhwp-desk 실행 실패");
}
