#!/usr/bin/python3

import re
import json

# "| 知乎每日精选 | [https://www.zhihu.com/rss](https://www.zhihu.com/rss) | [查看](https://webfollow.cc/channel/www.zhihu.com/rss) |"
def parse(input_string):
    # 提取“知乎每日精选”
    title = input_string.split("|")[1].strip()

    # 提取链接"https://www.zhihu.com/rss"
    link = re.search(r"\[([^\]]+)\]\(([^)]+)\)", input_string).group(2)

    if len(title) == 0 or len(link) == 0:
        return None

    return {"name": title, "url": link}

if __name__=='__main__':
    datas = []
    with open("./top-rss-list.md") as f:
        for line in f:
            item = parse(line)
            if item == None:
                continue

            datas.append(item)

    json_output = json.dumps(datas, ensure_ascii=False)
    print(json_output)

    with open("./top-rss-list.json", mode='w') as f:
        f.write(json_output)

