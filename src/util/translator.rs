use crate::config;
use std::collections::HashMap;

pub fn tr(text: &str) -> String {
    if config::ui().language == "cn" {
        return text.to_string();
    }

    let mut items: HashMap<&str, &str> = HashMap::new();
    items.insert("出错", "Error");
    items.insert("原因", "Reason");
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
    items.insert("正在重试...", "Retrying...");
    items.insert("正在下载...", "Downloading...");
    items.insert("创建账户成功", "Create account success");
    items.insert("创建账户失败", "Create account failed");
    items.insert("密码错误", "Wrong password");
    items.insert("修改密码成功", "Change password success");
    items.insert("非法输入", "Invalid input");
    items.insert("是否删除全部？", "Delete all entrys or not?");
    items.insert("刷新...", "Flush...");

    items.insert("没有订阅", "No Rss");
    items.insert("Rss名称和图标", "Rss name and icon");
    items.insert("请输入Rss名称", "Please input Rss name");
    items.insert("Rss源地址", "Rss URL");
    items.insert("请输入Rss源地址", "Please input Rss URL");
    items.insert("Rss源格式", "Rss format");
    items.insert("已经启用代理", "Enabled proxy");
    items.insert("未启用代理", "Disable proxy");
    items.insert("已经关注", "Star");
    items.insert("未关注", "Not star");
    items.insert("图标库", "Icons");

    items.insert("警告", "Warning");
    items.insert("订阅", "Rss");
    items.insert("收藏夹", "Favorite");
    items.insert("添加", "Add");
    items.insert("设置", "Setting");

    items.insert("没有数据", "No Data");
    items.insert("没有消息", "No Message");

    if let Some(txt) = items.get(text) {
        return txt.to_string();
    }

    text.to_string()
}
