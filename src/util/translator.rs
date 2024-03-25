use crate::config;
use std::collections::HashMap;

pub fn tr(text: &str) -> String {
    if config::ui().language == "cn" {
        return text.to_string();
    }

    let mut items: HashMap<&str, &str> = HashMap::new();
    items.insert("出错", "Error");
    items.insert("原因", "Reason");
    items.insert("取消", "Cancel");
    items.insert("确认", "Confirm");
    items.insert("编辑", "Edit");
    items.insert("删除", "Delete");

    items.insert("删除成功", "Delete success");
    items.insert("删除失败", "Delete failed");
    items.insert("添加成功", "Add success");
    items.insert("添加失败", "Add failed");
    items.insert("复制失败", "Copy failed");
    items.insert("复制成功", "Copy success");
    items.insert("清空失败", "Delete failed");
    items.insert("清空成功", "Delete success");
    items.insert("保存失败", "Save failed");
    items.insert("保存成功", "Save success");
    items.insert("重置成功", "Reset success");
    items.insert("刷新成功", "Flush success");
    items.insert("发送失败", "Send failed");
    items.insert("下载成功", "Download success");
    items.insert("下载失败", "Download failed");
    items.insert("加载失败", "Load failed");
    items.insert("密码错误", "Password invalid");
    items.insert("创建账户成功", "Create account success");
    items.insert("创建账户失败", "Create account failed");
    items.insert("密码错误", "Wrong password");
    items.insert("非法输入", "Invalid input");

    items.insert("刷新...", "Refresh...");
    items.insert("正在重试...", "Retrying...");
    items.insert("正在下载...", "Downloading...");

    items.insert("是否删除全部？", "Delete all entrys or not?");
    items.insert("是否删除全部缓存？", "Delete all cache or not?");
    items.insert("清除缓存失败", "Remove cache failed");
    items.insert("清除缓存成功", "Remove cache success");

    items.insert("界 面", "UI");
    items.insert("同 步", "Sync");
    items.insert("代 理", "Proxy");
    items.insert("缓 存", "Cache");
    items.insert("关 于", "About");
    items.insert("帮 助", "Help");

    items.insert("新建", "New");
    items.insert("没有订阅", "No RSS");
    items.insert("RSS名称和图标", "RSS name and icon");
    items.insert("请输入RSS名称", "Please input RSS name");
    items.insert("RSS源地址", "RSS URL");
    items.insert("请输入RSS源地址", "Please input RSS URL");
    items.insert("RSS源格式", "RSS format");
    items.insert("已经Http启用代理", "Enabled Http proxy");
    items.insert("未启Http用代理", "Disable Http proxy");
    items.insert("已经Socks5启用代理", "Enabled Socks5 proxy");
    items.insert("未启Socks5用代理", "Disable Socks5 proxy");
    items.insert("已经收藏", "Star");
    items.insert("未收藏", "Not star");
    items.insert("图标库", "Icons");
    items.insert("请选择条目", "Please select entry");

    items.insert("字体大小", "Font size");
    items.insert("字体样式", "Font family");
    items.insert("选择语言", "Choose language");
    items.insert("同步时间间隔(分钟)", "Sync time interval(minute)");
    items.insert("请输入时间间隔", "Please input time interval");
    items.insert("同步超时(秒)", "Sync timeout(second)");
    items.insert("请输入同步超时", "Please input sync timeout");
    items.insert("已经启用自动同步", "Enabled auto sync");
    items.insert("未启用自动同步", "Disable auto sync");
    items.insert(
        "程序启动时，马上进行一次同步",
        "Starting sync once, when application starting",
    );
    items.insert(
        "程序启动时，不马上进行一次同步",
        "Don't start sync, when application starting",
    );
    items.insert("代理地址", "Proxy address");
    items.insert("代理端口", "Proxy port");

    items.insert("警告", "Warning");
    items.insert("订阅", "RSS");
    items.insert("收藏夹", "Collection");
    items.insert("添加", "Add");
    items.insert("设置", "Setting");

    items.insert("没有数据", "No Data");
    items.insert("没有消息", "No Message");

    if let Some(txt) = items.get(text) {
        return txt.to_string();
    }

    text.to_string()
}
