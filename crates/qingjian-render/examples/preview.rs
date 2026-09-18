//! 离线预览：`cargo run --release -p qingjian-render --example preview -- --out target/render-preview`
//! 把样例帧按浅 / 深色、竖 / 横排画成 PNG，与各平台原生候选窗截图并排比；`--measure` 只量几段文字的宽度与原生对数；
//! 末尾列出验收行每个字形落到了哪家字体。不是日常工具，改渲染器时拿来核对。

use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use qingjian_render::{FontLibrary, Renderer, Shadow, Theme};

#[path = "../tests/scenes/mod.rs"]
mod scenes;

#[derive(Parser)]
struct Args {
    /// PNG 输出目录。
    #[arg(long, default_value = "target/render-preview")]
    out: PathBuf,

    /// 点 → 像素倍数（Retina 为 2）。
    #[arg(long, default_value_t = 2.0)]
    scale: f32,

    /// 中日字形回退用的 locale。
    #[arg(long, default_value = "zh-CN")]
    locale: String,

    /// 不画阴影（对照壳自己带系统阴影的截图时用）。
    #[arg(long)]
    no_shadow: bool,

    /// 只量几段文字的宽度（点），不出图；与 AppKit 的 NSAttributedString.size() 对数。
    #[arg(long)]
    measure: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "qingjian_render=debug".into()),
        )
        .init();
    let args = Args::parse();
    std::fs::create_dir_all(&args.out)?;

    let started = Instant::now();
    let library = FontLibrary::system(&args.locale)?;
    println!(
        "字体库：{:?}，界面字体 {}",
        started.elapsed(),
        library.ui_family()
    );
    println!("已加载字族：{}", library.families().join(" / "));
    let mut renderer = Renderer::new(library);
    if args.measure {
        for text in [
            "int. hello · int. hi",
            "hello",
            "ni'hao",
            "1/6",
            "你好",
            "phr. you change",
            "int. ",
            "·",
            " · ",
            "hi",
            "你好像",
            "開発する",
        ] {
            let widths: Vec<String> = [11.0, 12.0, 16.0]
                .into_iter()
                .map(|size| format!("{size}pt={:.2}", renderer.measure_points(text, size)))
                .collect();
            println!("{text:<24} {}", widths.join("  "));
        }
        return Ok(());
    }
    let shadow = (!args.no_shadow).then_some(Shadow::mac_panel());

    let samples = scenes::candidate_scenes();
    for (theme_name, theme) in [("light", Theme::light()), ("dark", Theme::dark())] {
        for (scene, frame, layout) in &samples {
            let started = Instant::now();
            let rendered = renderer.render(frame, *layout, &theme, args.scale, shadow.as_ref())?;
            let elapsed = started.elapsed();
            let path = args.out.join(format!("{scene}-{theme_name}.png"));
            rendered.pixmap.save_png(&path)?;
            let (w, h) = rendered.content_size_points();
            println!(
                "{:<28} {:>4.0}×{:<4.0}pt  {:>8.2?}  {}",
                format!("{scene}-{theme_name}"),
                w,
                h,
                elapsed,
                path.display()
            );
        }
    }

    // Windows 的悬浮状态条：三格
    let cells = scenes::status_cells();
    for (theme_name, theme) in [("light", Theme::light()), ("dark", Theme::dark())] {
        let status = renderer.render_status(&cells, &theme, args.scale, shadow.as_ref())?;
        let path = args.out.join(format!("status-{theme_name}.png"));
        status.rendered.pixmap.save_png(&path)?;
        let (w, h) = status.rendered.content_size_points();
        println!(
            "{:<28} {:>4.0}×{:<4.0}pt  格边界 {:?}  {}",
            format!("status-{theme_name}"),
            w,
            h,
            status.cell_edges,
            path.display()
        );
    }

    for probe in [
        "青简 hello 🙂 日本語 骨直曜",
        "開発(かいはつ)する",
        "int. hello · int. hi",
    ] {
        println!(
            "「{probe}」各字形字体：{}",
            renderer.trace_families(probe, &Theme::light()).join(" → ")
        );
    }
    Ok(())
}
