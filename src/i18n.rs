#[derive(Clone, Copy, PartialEq)]
pub enum Language {
    Chinese,
    English,
}

impl Language {
    pub fn toggle(&self) -> Self {
        match self {
            Language::Chinese => Language::English,
            Language::English => Language::Chinese,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Language::Chinese => "EN",
            Language::English => "中",
        }
    }
}

pub struct I18n {
    pub lang: Language,
}

impl I18n {
    pub fn new() -> Self {
        Self { lang: Language::English }
    }

    pub fn t(&self, key: &'static str) -> &'static str {
        match self.lang {
            Language::Chinese => match key {
                "search" => "搜索...",
                "open_downloads" => "打开下载目录",
                "view_logs" => "查看日志",
                "logs" => "日志",
                "clear_logs" => "清空",
                "download" => "下载",
                "execute" => "执行",
                "open" => "打开",
                "admin_required" => "需要管理员权限",
                "downloading" => "下载中...",
                "all" => "全部",
                _ => key,
            },
            Language::English => match key {
                "search" => "Search...",
                "open_downloads" => "Open Downloads",
                "view_logs" => "View Logs",
                "logs" => "Logs",
                "clear_logs" => "Clear",
                "download" => "Download",
                "execute" => "Execute",
                "open" => "Open",
                "admin_required" => "Admin required",
                "downloading" => "Downloading...",
                "all" => "All",
                _ => key,
            },
        }
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}
