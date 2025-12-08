/// 动作类型
#[derive(Clone)]
pub enum Action {
    Download {
        url: &'static str,
        filename: Option<&'static str>,
        sha256: Option<&'static str>,
    },
    /// 从 GitHub Release 下载最新版本
    GitHubRelease {
        /// 仓库名，格式: "owner/repo"
        repo: &'static str,
        /// 匹配 asset 文件名
        asset_pattern: &'static str,
        /// 保存的文件名
        filename: Option<&'static str>,
    },
    /// 从网页爬取下载链接
    WebScrape {
        /// URL
        page_url: &'static str,
        /// 匹配第一个链接
        link_regex: &'static str,
        /// 保存的文件名
        filename: Option<&'static str>,
    },
    PowerShell {
        script: &'static str,
        hidden: bool,
    },
}

/// 条目定义
#[derive(Clone)]
pub struct Item {
    pub id: &'static str,
    pub name: &'static str,
    pub name_en: &'static str,
    pub description: &'static str,
    pub description_en: &'static str,
    pub category: &'static str,
    pub tags: &'static [&'static str],
    pub action: Action,
    pub require_admin: bool,
}

/// 分类定义
pub struct Category {
    pub id: &'static str,
    pub name: &'static str,
    pub name_en: &'static str,
}

/// 内置分类
pub const CATEGORIES: &[Category] = &[
    Category { id: "common", name: "常用", name_en: "Common" },
    Category { id: "tool", name: "工具", name_en: "Tools" },
    Category { id: "shell", name: "Shell命令", name_en: "Shell Script" },
];

/// 内置条目
pub const ITEMS: &[Item] = &[
    // ==================== common (常用) ====================
    Item {
        id: "chrome",
        name: "Google Chrome",
        name_en: "Google Chrome",
        description: "下载 Chrome 浏览器安装包",
        description_en: "Download Chrome browser installer",
        category: "common",
        tags: &["browser", "浏览器", "google"],
        action: Action::Download {
            url: "https://dl.google.com/dl/chrome/install/googlechromestandaloneenterprise64.msi",
            filename: Some("ChromeSetup.msi"),
            sha256: None,
        },
        require_admin: false,
    },
    Item {
        id: "qq",
        name: "QQ",
        name_en: "QQ",
        description: "下载 QQ PC 客户端",
        description_en: "Download QQ PC client",
        category: "common",
        tags: &["chat", "聊天", "qq", "腾讯"],
        action: Action::WebScrape {
            page_url: "https://cdn-go.cn/qq-web/im.qq.com_new/latest/rainbow/windowsConfig.js",
            link_regex: r#""ntDownloadX64Url":"([^"]+)""#,
            filename: Some("QQSetup.exe"),
        },
        require_admin: false,
    },
    Item {
        id: "wechat",
        name: "微信",
        name_en: "WeChat",
        description: "下载微信 PC 客户端",
        description_en: "Download WeChat PC client",
        category: "common",
        tags: &["chat", "聊天", "微信", "wechat"],
        action: Action::Download {
            url: "https://dldir1v6.qq.com/weixin/Windows/WeChatSetup.exe",
            filename: Some("WeChatSetup.exe"),
            sha256: None,
        },
        require_admin: false,
    },
    Item {
        id: "steam",
        name: "Steam",
        name_en: "Steam",
        description: "下载 Steam 游戏平台",
        description_en: "Download Steam gaming platform",
        category: "common",
        tags: &["game", "游戏", "steam"],
        action: Action::Download {
            url: "https://cdn.akamai.steamstatic.com/client/installer/SteamSetup.exe",
            filename: Some("SteamSetup.exe"),
            sha256: None,
        },
        require_admin: false,
    },
    Item {
        id: "teamspeak",
        name: "Teamspeak3",
        name_en: "Teamspeak3",
        description: "下载 Teamspeak3 客户端",
        description_en: "Download TeamSpeak installer",
        category: "common",
        tags: &["voice", "语音", "teamspeak"],
        action: Action::WebScrape {
            page_url: "https://www.teamspeak.com/en/downloads/",
            link_regex: r#"href="(https://files\.teamspeak-services\.com/releases/client/[\d.]+/TeamSpeak3-Client-win64-[\d.]+\.exe)""#,
            filename: Some("TeamSpeak.exe"),
        },
        require_admin: false,
    },
    // ==================== tool (工具) ====================
    Item {
        id: "7zip",
        name: "7-Zip",
        name_en: "7-Zip",
        description: "下载 7-Zip 压缩工具",
        description_en: "Download 7-Zip compression tool",
        category: "tool",
        tags: &["zip", "压缩", "7zip"],
        action: Action::GitHubRelease {
            repo: "ip7z/7zip",
            asset_pattern: "*-x64.exe",
            filename: Some("7zip.exe"),
        },
        require_admin: false,
    },
    Item {
        id: "vscode",
        name: "VS Code",
        name_en: "VS Code",
        description: "下载 VS Code 编辑器",
        description_en: "Download VS Code editor",
        category: "tool",
        tags: &["editor", "编辑器", "code"],
        action: Action::Download {
            url: "https://code.visualstudio.com/sha/download?build=stable&os=win32-x64-user",
            filename: Some("VSCodeSetup.exe"),
            sha256: None,
        },
        require_admin: false,
    },
    Item {
        id: "everything",
        name: "Everything",
        name_en: "Everything",
        description: "下载 Everything 文件搜索工具",
        description_en: "Download Everything file search tool",
        category: "tool",
        tags: &["search", "搜索", "文件", "everything"],
        action: Action::WebScrape {
            page_url: "https://www.voidtools.com/downloads/",
            link_regex: r#"href="(/Everything-[\d.]+\.x64-Setup\.exe)""#,
            filename: Some("Everything.exe"),
        },
        require_admin: false,
    },
    Item {
        id: "snipaste",
        name: "Snipaste",
        name_en: "Snipaste",
        description: "下载 Snipaste 截图贴图工具",
        description_en: "Download Snipaste screenshot tool",
        category: "tool",
        tags: &["screenshot", "截图", "贴图", "snipaste"],
        action: Action::Download {
            url: "https://dl.snipaste.com/win-x64",
            filename: Some("Snipaste.zip"),
            sha256: None,
        },
        require_admin: false,
    },
    Item {
        id: "clash_verge",
        name: "Clash Verge",
        name_en: "Clash Verge",
        description: "下载 Clash Verge 代理工具",
        description_en: "Download Clash Verge proxy tool",
        category: "tool",
        tags: &["proxy", "代理", "clash", "vpn"],
        action: Action::GitHubRelease {
            repo: "clash-verge-rev/clash-verge-rev",
            asset_pattern: "*x64-setup.exe",
            filename: Some("ClashVerge.exe"),
        },
        require_admin: false,
    },
    Item {
        id: "huorong",
        name: "火绒安全",
        name_en: "Huorong Security",
        description: "下载火绒安全软件",
        description_en: "Download Huorong Security software",
        category: "tool",
        tags: &["security", "安全", "杀毒", "火绒"],
        action: Action::Download {
            url: "https://down-tencent.huorong.cn/sysdiag-all-x64-6.0.8.3-2025.12.06.1.exe",
            filename: Some("HuorongSetup.exe"),
            sha256: None,
        },
        require_admin: false,
    },
    Item {
        id: "spacesniffer",
        name: "Space Sniffer",
        name_en: "Space Sniffer",
        description: "下载 Space Sniffer 磁盘分析工具",
        description_en: "Download Space Sniffer disk analyzer",
        category: "tool",
        tags: &["disk", "磁盘", "spacesniffer"],
        action: Action::GitHubRelease {
            repo: "redtrillix/SpaceSniffer",
            asset_pattern: "*.zip",
            filename: Some("SpaceSniffer.zip"),
        },
        require_admin: false,
    },
    Item {
        id: "steamtools",
        name: "Steam++",
        name_en: "SteamTools",
        description: "下载 Steam++ 工具",
        description_en: "Download SteamTools",
        category: "tool",
        tags: &["steam", "加速", "steamtools"],
        action: Action::GitHubRelease {
            repo: "BeyondDimension/SteamTools",
            asset_pattern: "*_win_x64.exe",
            filename: Some("SteamTools.exe"),
        },
        require_admin: false,
    },
    Item {
        id: "tubatool",
        name: "图吧工具箱",
        name_en: "Tubatool",
        description: "下载图吧工具箱",
        description_en: "Download Tubatool",
        category: "tool",
        tags: &["hardware", "硬件", "检测"],
        action: Action::Download {
            url: "https://apac.tualatin.club/%E5%9B%BE%E5%90%A7%E5%B7%A5%E5%85%B7%E7%AE%B12025.07R2%E5%AE%89%E8%A3%85%E5%8C%85.exe",
            filename: Some("tubatool.exe"),
            sha256: None,
        },
        require_admin: false,
    },
    Item {
        id: "officetool",
        name: "Office Tool Plus",
        name_en: "Office Tool Plus",
        description: "下载 Office Tool Plus",
        description_en: "Download Office Tool Plus",
        category: "tool",
        tags: &["office", "微软", "办公"],
        action: Action::Download {
            url: "https://otp.landian.vip/redirect/download.php?site=github&type=runtime&arch=x64",
            filename: Some("otp.zip"),
            sha256: None,
        },
        require_admin: false,
    },
    // ==================== shell (Shell命令) ====================
    // DNS 相关
    Item {
        id: "set_dns_google",
        name: "设置 DNS (Google)",
        name_en: "Set DNS (Google)",
        description: "将 DNS 设置为 8.8.8.8 和 8.8.4.4",
        description_en: "Set DNS to 8.8.8.8 and 8.8.4.4",
        category: "shell",
        tags: &["dns", "network", "网络", "google"],
        action: Action::PowerShell {
            script: r#"$adapters = Get-NetAdapter | Where-Object {$_.Status -eq 'Up'}; foreach ($adapter in $adapters) { Set-DnsClientServerAddress -InterfaceIndex $adapter.ifIndex -ServerAddresses ('8.8.8.8','8.8.4.4') }; Write-Host 'DNS 设置完成 (Google)'"#,
            hidden: false,
        },
        require_admin: true,
    },
    Item {
        id: "set_dns_ali",
        name: "设置 DNS (阿里)",
        name_en: "Set DNS (Alibaba)",
        description: "将 DNS 设置为 223.5.5.5 和 223.6.6.6",
        description_en: "Set DNS to 223.5.5.5 and 223.6.6.6",
        category: "shell",
        tags: &["dns", "network", "网络", "阿里", "国内"],
        action: Action::PowerShell {
            script: r#"$adapters = Get-NetAdapter | Where-Object {$_.Status -eq 'Up'}; foreach ($adapter in $adapters) { Set-DnsClientServerAddress -InterfaceIndex $adapter.ifIndex -ServerAddresses ('223.5.5.5','223.6.6.6') }; Write-Host 'DNS 设置完成 (阿里)'"#,
            hidden: false,
        },
        require_admin: true,
    },
    Item {
        id: "reset_dns",
        name: "重置 DNS",
        name_en: "Reset DNS",
        description: "将 DNS 重置为自动获取 (DHCP)",
        description_en: "Reset DNS to automatic (DHCP)",
        category: "shell",
        tags: &["dns", "network", "网络", "重置", "reset"],
        action: Action::PowerShell {
            script: r#"$adapters = Get-NetAdapter | Where-Object {$_.Status -eq 'Up'}; foreach ($adapter in $adapters) { Set-DnsClientServerAddress -InterfaceIndex $adapter.ifIndex -ResetServerAddresses }; Write-Host 'DNS 已重置为自动获取'"#,
            hidden: false,
        },
        require_admin: true,
    },
    Item {
        id: "flush_dns",
        name: "刷新 DNS 缓存",
        name_en: "Flush DNS Cache",
        description: "清除本地 DNS 解析缓存",
        description_en: "Clear local DNS resolver cache",
        category: "shell",
        tags: &["dns", "network", "网络", "刷新", "flush"],
        action: Action::PowerShell {
            script: "ipconfig /flushdns; Write-Host 'DNS 缓存已刷新'",
            hidden: false,
        },
        require_admin: true,
    },
    // 系统优化
    Item {
        id: "clean_temp",
        name: "清理临时文件",
        name_en: "Clean Temp Files",
        description: "清理系统和用户临时文件",
        description_en: "Clean system and user temp files",
        category: "shell",
        tags: &["clean", "清理", "temp", "临时文件"],
        action: Action::PowerShell {
            script: r#"Remove-Item -Path "$env:TEMP\*" -Recurse -Force -ErrorAction SilentlyContinue; Remove-Item -Path "C:\Windows\Temp\*" -Recurse -Force -ErrorAction SilentlyContinue; Write-Host '临时文件清理完成'"#,
            hidden: false,
        },
        require_admin: true,
    },
    Item {
        id: "disable_telemetry",
        name: "禁用遥测",
        name_en: "Disable Telemetry",
        description: "禁用 Windows 遥测数据收集",
        description_en: "Disable Windows telemetry data collection",
        category: "shell",
        tags: &["privacy", "隐私", "telemetry", "遥测"],
        action: Action::PowerShell {
            script: r#"Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection" -Name "AllowTelemetry" -Value 0 -Type DWord -Force; Stop-Service DiagTrack -Force -ErrorAction SilentlyContinue; Set-Service DiagTrack -StartupType Disabled -ErrorAction SilentlyContinue; Write-Host '遥测已禁用'"#,
            hidden: false,
        },
        require_admin: true,
    },
    Item {
        id: "visual_effects",
        name: "优化视觉效果",
        name_en: "Optimize Visual Effects",
        description: "自定义视觉效果：保留字体平滑、缩略图、透明选择框、窗口和鼠标阴影",
        description_en: "Custom visual effects: keep font smoothing, thumbnails, translucent selection, window and cursor shadows",
        category: "shell",
        tags: &["visual", "视觉", "性能", "优化", "performance"],
        action: Action::PowerShell {
            script: r#"Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects' -Name 'VisualFXSetting' -Value 3 -Type DWord; Set-ItemProperty -Path 'HKCU:\Control Panel\Desktop' -Name 'FontSmoothing' -Value '2' -Type String; Set-ItemProperty -Path 'HKCU:\Control Panel\Desktop' -Name 'FontSmoothingType' -Value 2 -Type DWord; Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' -Name 'IconsOnly' -Value 0 -Type DWord; Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' -Name 'ListviewAlphaSelect' -Value 1 -Type DWord; Set-ItemProperty -Path 'HKCU:\Control Panel\Desktop' -Name 'UserPreferencesMask' -Value ([byte[]](0x9E,0x3E,0x07,0x80,0x12,0x00,0x00,0x00)) -Type Binary; Set-ItemProperty -Path 'HKCU:\Control Panel\Desktop\WindowMetrics' -Name 'MinAnimate' -Value '0' -Type String; Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' -Name 'TaskbarAnimations' -Value 0 -Type DWord; Set-ItemProperty -Path 'HKCU:\Control Panel\Desktop' -Name 'DragFullWindows' -Value '0' -Type String; Add-Type 'using System; using System.Runtime.InteropServices; public class WinAPI { [DllImport(\"user32.dll\")] public static extern bool SystemParametersInfo(uint a, uint b, IntPtr c, uint d); }'; [WinAPI]::SystemParametersInfo(0x103F, 0, [IntPtr]::Zero, 0x02); Write-Host '视觉效果已优化'"#,
            hidden: false,
        },
        require_admin: true,
    },
    // 激活工具
    Item {
        id: "mas_activate",
        name: "MAS 激活工具",
        name_en: "MAS Activation",
        description: "Microsoft Activation Scripts",
        description_en: "Microsoft Activation Scripts",
        category: "shell",
        tags: &["activate", "激活", "windows", "office"],
        action: Action::PowerShell {
            script: "irm https://get.activated.win | iex",
            hidden: false,
        },
        require_admin: true,
    },
    Item {
        id: "spotx",
        name: "SpotX (Spotify去广告)",
        name_en: "SpotX (Spotify Ad-free)",
        description: "Spotify 去广告补丁",
        description_en: "Spotify ad-free patch",
        category: "shell",
        tags: &["spotify", "音乐", "去广告", "spotx"],
        action: Action::PowerShell {
            script: r#"iex "& { $(iwr -useb 'https://raw.githubusercontent.com/SpotX-Official/SpotX/refs/heads/main/run.ps1') } -new_theme""#,
            hidden: false,
        },
        require_admin: true,
    },
];
