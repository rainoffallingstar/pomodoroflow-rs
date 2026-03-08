// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pomoflow_rs::PomodoroAppManager;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use tauri::Manager;

// 导入命令模块
mod commands;
use commands::*;

fn main() {
    // 设置 panic hook
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("=== PANIC OCCURRED ===");
        eprintln!("Location: {:?}", panic_info.location());
        eprintln!("Payload: {:?}", panic_info.payload());
        eprintln!("======================");

        // 记录到文件
        let log_path = dirs::data_dir()
            .map(|p| p.join("pomoflow-rs/crash.log"))
            .unwrap_or_else(|| std::path::PathBuf::from("crash.log"));

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let panic_msg = format!("[{}] PANIC: {:?}\n", timestamp, panic_info);

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
            let _ = file.write_all(panic_msg.as_bytes());
        }
    }));

    tauri::Builder::default()
        // 初始化全局应用管理器
        .setup(|app| {
            let app_handle = app.handle().clone();
            println!("🔧 Tauri Setup 开始 - 同步初始化模式");

            // 使用 block_on 同步等待初始化完成，确保状态在命令调用前已设置
            let result = tauri::async_runtime::block_on(async move {
                println!("🔄 开始初始化应用管理器...");
                let app_handle_for_events = app_handle.clone();

                match initialize_app_manager_sync(app_handle).await {
                    Ok(app_manager_arc) => {
                        println!("✅ 应用管理器初始化完成");

                        // 启动事件系统（异步，不阻塞）
                        let app_manager_for_events = app_manager_arc.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = start_event_system(app_handle_for_events, app_manager_for_events).await {
                                eprintln!("⚠️  事件系统启动失败: {}", e);
                            }
                        });

                        Ok(app_manager_arc)
                    }
                    Err(e) => {
                        eprintln!("❌ 应用管理器初始化失败: {}", e);
                        Err(e)
                    }
                }
            });

            match result {
                Ok(_) => {
                    println!("✅ Tauri Setup 完成，状态已就绪");
                    if let Err(e) = app.emit_all("initialization-complete", "ok") {
                        eprintln!("Failed to emit initialization-complete: {}", e);
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Tauri Setup 失败: {}", e);
                    let err_msg = format!("Failed to initialize app: {}", e);
                    if let Err(emit_err) = app.emit_all("initialization-error", err_msg.clone()) {
                        eprintln!("Failed to emit initialization-error: {}", emit_err);
                    }
                    Err(err_msg.into())
                }
            }
        })
        // 注册 Tauri 命令
        .invoke_handler(tauri::generate_handler![
            // 番茄钟命令
            start_pomodoro,
            pause_pomodoro,
            reset_pomodoro,
            skip_pomodoro_phase,
            get_pomodoro_session,
            update_pomodoro_config,
            // 待办事项命令
            create_todo,
            update_todo,
            delete_todo,
            toggle_todo_status,
            set_todo_status,
            link_todo_github,
            clear_todo_github_link,
            get_todos,
            get_todo_stats,
            // 标签命令
            get_tags,
            create_tag,
            delete_tag,
            assign_tag_to_todo,
            remove_tag_from_todo,
            get_todo_tags,
            // 配置管理命令
            get_user_config,
            get_github_sync_config,
            save_user_config,
            // 同步命令
            run_github_sync
        ])
        .run(tauri::generate_context!())
        .map_err(|e| {
            eprintln!("Failed to run Tauri application: {}", e);
            std::process::exit(1);
        });
}

/// 初始化全局应用管理器（同步版本，在 setup 中使用 block_on 调用）
async fn initialize_app_manager_sync(
    app_handle: tauri::AppHandle,
) -> Result<Arc<Mutex<PomodoroAppManager>>, Box<dyn std::error::Error>> {
    use std::time::Instant;

    let total_start = Instant::now();
    println!("🚀 Initializing PomodoroFlow-Rs Tauri App (同步模式)...");

    // 创建应用管理器实例
    let manager_start = Instant::now();
    let mut app_manager = PomodoroAppManager::new().await?;
    let manager_elapsed = manager_start.elapsed();
    println!("   ├── 应用管理器创建: {:.2?}", manager_elapsed);

    // 启动应用服务
    let service_start = Instant::now();
    println!("   ├── 开始调用 app_manager.start()...");
    let service_result = app_manager.start().await;
    let service_elapsed = service_start.elapsed();

    match service_result {
        Ok(_) => {
            println!("   ├── 核心服务启动成功: {:.2?}", service_elapsed);
        }
        Err(e) => {
            println!("   ├── 核心服务启动失败 (耗时: {:.2?}): {}", service_elapsed, e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    }

    // 将应用管理器存储到全局状态（关键：必须在 setup 返回前完成）
    let app_manager_arc = Arc::new(Mutex::new(app_manager));
    app_handle.manage(app_manager_arc.clone());

    let total_elapsed = total_start.elapsed();

    // 打印启动报告
    println!("\n📊 性能统计:");
    println!("   ├── 应用管理器创建: {:.2?}", manager_elapsed);
    println!("   ├── 核心服务启动: {:.2?}", service_elapsed);
    println!("   └── 总初始化时间: {:.2?}", total_elapsed);

    if total_elapsed.as_millis() > 3000 {
        println!("⚠️  注意：初始化时间超过3秒，建议进一步优化");
    } else if total_elapsed.as_millis() > 1000 {
        println!("📈 初始化时间在可接受范围内");
    } else {
        println!("🎉 初始化性能优秀！");
    }

    println!("✅ PomodoroFlow-Rs Tauri App initialized successfully!");

    Ok(app_manager_arc)
}

/// 启动事件系统后台任务
async fn start_event_system(
    app_handle: tauri::AppHandle,
    app_manager: Arc<Mutex<PomodoroAppManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle_clone = app_handle.clone();

    // 启动后台任务，每1秒检查一次番茄钟状态并发射事件
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        let mut last_session: Option<pomoflow_rs::core::pomodoro::PomodoroSession> = None;
        let mut error_count = 0;
        const MAX_ERRORS: u32 = 10;

        loop {
            interval.tick().await;

            // 使用异步锁
            let result = {
                let guard = app_manager.lock().await;
                guard.get_pomodoro_session().await
            };

            match result {
                Ok(session) => {
                    error_count = 0; // 重置错误计数

                    if let Some(pomodoro_session) = session {
                        // 如果番茄钟正在运行，每秒都发射事件
                        // 如果未运行，只在状态变化时发射
                        let should_emit = if pomodoro_session.is_running {
                            true
                        } else {
                            match &last_session {
                                Some(last) => {
                                    pomodoro_session.is_running != last.is_running ||
                                    pomodoro_session.remaining != last.remaining ||
                                    pomodoro_session.phase != last.phase
                                }
                                None => true,
                            }
                        };

                        // 发射更新事件
                        if should_emit {
                            if let Err(e) = app_handle_clone.emit_all("pomodoro-tick", &pomodoro_session) {
                                eprintln!("Failed to emit pomodoro-tick event: {}", e);
                            }
                        }

                        // 通过“上一拍运行中 + 阶段发生变化”的边沿检测阶段完成。
                        // 兼容后端自动启动下一阶段（阶段切换时仍保持 is_running=true）。
                        if let Some(last) = &last_session {
                            let phase_completed =
                                last.is_running &&
                                (pomodoro_session.phase != last.phase || pomodoro_session.cycle_count != last.cycle_count);

                            if phase_completed {
                                if let Err(e) = app_handle_clone
                                    .emit_all("pomodoro-phase-completed", &pomodoro_session) {
                                    eprintln!("Failed to emit pomodoro-phase-completed event: {}", e);
                                }

                                let phase_str = format!("{:?}", pomodoro_session.phase);
                                if let Err(e) = app_handle_clone.emit_all("show-pomodoro-notification", &phase_str) {
                                    eprintln!("Failed to emit show-pomodoro-notification event: {}", e);
                                }
                            }
                        }

                        // 更新最后状态
                        last_session = Some(pomodoro_session.clone());
                    } else {
                        // 无活动会话
                        last_session = None;
                    }
                }
                Err(e) => {
                    error_count += 1;
                    eprintln!("Failed to get pomodoro session (error #{}/{}): {}", error_count, MAX_ERRORS, e);

                    // 如果连续错误太多，暂停一段时间
                    if error_count >= MAX_ERRORS {
                        eprintln!("Too many consecutive errors, pausing event system for 30 seconds");
                        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                        error_count = 0;
                    }
                }
            }
        }
    });

    Ok(())
}
