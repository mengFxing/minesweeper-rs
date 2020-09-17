// 定义了几个模块-->同目录的同名rs文件
mod comp_assets;
mod comp_ui;
mod interop;
mod minesweeper;
mod numerics;
mod visual_grid;
mod window_target;
// 使用自己的模块
use interop::{create_dispatcher_queue_controller_for_current_thread, ro_initialize, RoInitType};
use minesweeper::Minesweeper;
use window_target::CompositionDesktopWindowTargetSource;
// 使用winit依赖，一个跨平台的窗口创建和管理 https://crates.io/crates/winit
use winit::{
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
// 使用同工作目录lib，在依赖中指定bindings目录中
use bindings::windows::{foundation::numerics::Vector2, ui::composition::Compositor};

// 定义同步函数run，返回 winrt的result类型
// winrt 提供了windows Api以rust的形式调用的能力 https://crates.io/crates/winrt
fn run() -> winrt::Result<()> {

    // 传播错误的简写：? 运算符
    // ro初始化
    ro_initialize(RoInitType::MultiThreaded)?;

    // 为当前线程创建调度程序队列控制器
    let _controller = create_dispatcher_queue_controller_for_current_thread()?;

    // 创建一个事件循环
    let event_loop = EventLoop::new();

    // 创建一个窗口。借用event_loop
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // 设置标题
    window.set_title("Minesweeper");

    // 创建UI合并器
    let compositor = Compositor::new()?;

    // 创建window目标，借用compositor
    let target = window.create_window_target(&compositor, false)?;

    // 创建可视化容器
    let root = compositor.create_container_visual()?;

    // r#-->原生标识操作符：保持原始字符不进行转义
    // 设置长宽比
    root.set_relative_size_adjustment(Vector2 { x: 1.0, y: 1.0 })?;
    
    // 目标应用设置
    target.set_root(&root)?;

    // 获取窗口内部尺寸
    let window_size = window.inner_size();

    // 对尺寸进行类型转换
    let window_size = Vector2 {
        x: window_size.width as f32,
        y: window_size.height as f32,
    };

    // 新建扫雷游戏
    let mut game = Minesweeper::new(&root, &window_size)?;

    // move 关键字强制闭包获取其使用的环境值的所有权
    // 运行事件循环
    event_loop.run(move |event, _, control_flow| {
        // 解引用，重新赋值
        *control_flow = ControlFlow::Wait;
        // match判断
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let size = Vector2 {
                    x: size.width as f32,
                    y: size.height as f32,
                };
                game.on_parent_size_changed(&size).unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let point = Vector2 {
                    x: position.x as f32,
                    y: position.y as f32,
                };
                game.on_pointer_moved(&point).unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                if state == ElementState::Pressed {
                    game.on_pointer_pressed(button == MouseButton::Right, false)
                        .unwrap();
                }
            }
            _ => (),
        }
    });
}

fn main() {
    let result = run();

    // We do this for nicer HRESULT printing when errors occur.
    if let Err(error) = result {
        error.code().unwrap();
    }
}
