use eframe::egui;

use crate::executor::Executor;
use crate::i18n::{I18n, Language};
use crate::logger::Logger;
use crate::model::{Action, Category, Item, CATEGORIES, ITEMS};

pub struct App {
    search_query: String,
    selected_category: Option<&'static str>,
    i18n: I18n,
    logger: Logger,
    executor: Executor,
    show_logs: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 加载中文字体
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "lxgw".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/LXGWWenKaiLite-Regular.ttf")).into(),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "lxgw".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "lxgw".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        Self {
            search_query: String::new(),
            selected_category: None,
            i18n: I18n::new(),
            logger: Logger::new(),
            executor: Executor::new(),
            show_logs: true,
        }
    }

    fn get_item_name(&self, item: &Item) -> &str {
        match self.i18n.lang {
            Language::Chinese => item.name,
            Language::English => item.name_en,
        }
    }

    fn get_item_desc(&self, item: &Item) -> &str {
        match self.i18n.lang {
            Language::Chinese => item.description,
            Language::English => item.description_en,
        }
    }

    fn get_category_name(&self, cat: &Category) -> &str {
        match self.i18n.lang {
            Language::Chinese => cat.name,
            Language::English => cat.name_en,
        }
    }

    fn filter_items(&self) -> Vec<&'static Item> {
        ITEMS.iter().filter(|item| {
            // 分类过滤
            if let Some(cat) = self.selected_category {
                if item.category != cat {
                    return false;
                }
            }

            // 搜索过滤
            if !self.search_query.is_empty() {
                let query = self.search_query.to_lowercase();
                let name_match = item.name.to_lowercase().contains(&query)
                    || item.name_en.to_lowercase().contains(&query);
                let desc_match = item.description.contains(&query)
                    || item.description_en.to_lowercase().contains(&query);
                let tag_match = item.tags.iter().any(|t| t.to_lowercase().contains(&query));

                if !name_match && !desc_match && !tag_match {
                    return false;
                }
            }

            true
        }).collect()
    }

    fn get_action_button_text(&self, action: &Action) -> &'static str {
        match action {
            Action::Download { .. } | Action::GitHubRelease { .. } | Action::WebScrape { .. } => self.i18n.t("download"),
            Action::PowerShell { .. } => self.i18n.t("execute"),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 顶栏
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // 搜索框
                ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text(self.i18n.t("search"))
                        .desired_width(200.0)
                );

                ui.separator();

                // 打开下载目录
                if ui.button(self.i18n.t("open_downloads")).clicked() {
                    let _ = open::that(self.executor.get_download_dir_path());
                }

                // 查看日志
                if ui.button(self.i18n.t("view_logs")).clicked() {
                    self.logger.open_log_dir();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 语言切换
                    if ui.button(self.i18n.lang.label()).clicked() {
                        self.i18n.lang = self.i18n.lang.toggle();
                    }
                });
            });
        });

        // 底部日志区
        egui::TopBottomPanel::bottom("log_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(self.i18n.t("logs"));
                if ui.small_button(if self.show_logs { "▼" } else { "▶" }).clicked() {
                    self.show_logs = !self.show_logs;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(self.i18n.t("clear_logs")).clicked() {
                        self.logger.clear();
                    }
                });
            });

            if self.show_logs {
                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for entry in self.logger.get_entries() {
                            let color = if entry.is_error {
                                egui::Color32::RED
                            } else {
                                egui::Color32::GRAY
                            };
                            ui.colored_label(color, format!("[{}] {}", entry.time, entry.message));
                        }
                    });
            }
        });

        // 左侧分类
        egui::SidePanel::left("category_panel")
            .resizable(false)
            .default_width(100.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // "全部" 按钮
                    let all_selected = self.selected_category.is_none();
                    if ui.selectable_label(all_selected, self.i18n.t("all")).clicked() {
                        self.selected_category = None;
                    }

                    ui.separator();

                    // 分类列表
                    for cat in CATEGORIES {
                        let selected = self.selected_category == Some(cat.id);
                        if ui.selectable_label(selected, self.get_category_name(cat)).clicked() {
                            self.selected_category = Some(cat.id);
                        }
                    }
                });
            });

        // 中间条目列表
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let items = self.filter_items();

                for item in items {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.strong(self.get_item_name(item));
                                    if item.require_admin {
                                        ui.label("🛡");
                                    }
                                });
                                ui.label(
                                    egui::RichText::new(self.get_item_desc(item))
                                        .small()
                                        .color(egui::Color32::GRAY)
                                );
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // 检查是否正在下载
                                if let Some(progress) = self.executor.get_progress(item.id) {
                                    ui.add(
                                        egui::ProgressBar::new(progress)
                                            .desired_width(80.0)
                                            .text(format!("{:.0}%", progress * 100.0))
                                    );
                                } else {
                                    // 普通按钮
                                    let btn_text = self.get_action_button_text(&item.action);
                                    if ui.button(btn_text).clicked() {
                                        self.logger.log(&format!("执行: {}", item.name));
                                        self.executor.execute(item.id, &item.action, item.require_admin, &self.logger);
                                    }
                                }
                            });
                        });
                    });
                    ui.add_space(4.0);
                }
            });
        });

        // 持续刷新以更新进度
        ctx.request_repaint();
    }
}
