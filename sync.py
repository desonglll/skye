"""
data structure inside the setup.json.

it seems like path is the identified key.

    {
        "repo": "git@github.com:siliconflow/ComfyUI.git",
        "commit": "8f374716ee98d378d403ebc61250e091ecd3a25c",
        "license": "GPL-3.0 license",
        "notes": "ComfyUI 主体",
        "path": "ComfyUI"
    }

"""

import argparse
import json
import os
import re


def convert_git_url(url, target_format):
    """
    将 Git URL 转换为 SSH 或 HTTPS 格式
    """
    # 提取域名和路径 (例如: github.com 和 siliconflow/ComfyUI.git)
    # 兼容多种输入格式
    ssh_pattern = r"git@([^:]+):([^/]+/[^/]+)"
    https_pattern = r"https://([^/]+)/([^/]+/[^/]+)"

    domain = ""
    repo_path = ""

    if url.startswith("git@"):
        match = re.search(ssh_pattern, url)
        if match:
            domain, repo_path = match.groups()
    elif url.startswith("http"):
        match = re.search(https_pattern, url)
        if match:
            domain, repo_path = match.groups()

    if not domain or not repo_path:
        return url

    if not repo_path.endswith(".git"):
        repo_path += ".git"

    if target_format == "ssh":
        return f"git@{domain}:{repo_path}"
    elif target_format == "https":
        return f"https://{domain}/{repo_path}"

    return url


def main(args):

    skip_paths = set(args.skip) if args.skip else set()

    if not os.path.exists(args.source):
        print(f"file {args.source} not exists.")
        exit(-1)

    if not os.path.exists(args.target):
        print(f"file {args.target} not exists.")
        exit(-1)

    with open(args.source, "r", encoding="utf-8") as f:
        source_data = json.load(f)
        print(f"source data: {len(source_data)} objects")

    with open(args.target, "r", encoding="utf-8") as f:
        target_data = json.load(f)
        print(f"target data: {len(target_data)} objects")

    repos = []

    for data in source_data:
        repos.append(data["repo"])

    print(f"scanned {len(repos)} repos in total.")

    for i, target in enumerate(target_data):

        if target["path"] in skip_paths:
            print(f"\033[0;33mskip path:\033[0;32m {target['path']}\033[0m")
            continue

        for source in source_data:
            if target["path"] == source["path"]:
                print(f"\033[0;33mprocess\033[0;32m {target['repo']}\033[0m")

                if args.entire:
                    target_data[i] = source.copy()
                else:
                    target_data[i]["commit"] = source["commit"]
                    target_data[i]["repo"] = source["repo"]

                current_repo = target_data[i]["repo"]
                if args.to_ssh:
                    target_data[i]["repo"] = convert_git_url(current_repo, "ssh")
                elif args.to_https:
                    target_data[i]["repo"] = convert_git_url(current_repo, "https")

                if current_repo != target_data[i]["repo"]:
                    print(f"  \033[0;34mURL converted:\033[0m {target_data[i]['repo']}")

                break

    with open(args.target, "w", encoding="utf-8") as f:
        json.dump(target_data, f, ensure_ascii=False, indent=4)

    pass


if __name__ == "__main__":

    parser = argparse.ArgumentParser(
        description="Sync dependencies commit to target repo from source repo, use 'path' property for identification."
    )

    parser.add_argument("source", help="setup.json file from source repo")
    parser.add_argument("target", help="setup.json file from target repo")

    parser.add_argument(
        "--entire",
        "-e",
        action="store_true",
        help="replace whole json objects, including repo, commit, description and path.",
    )

    parser.add_argument(
        "--skip",
        "-s",
        nargs="+",
        help="the repo you want to skip, pass a list that contains multiple 'path's. eg: --skip ComfyUI CustomNodes",
    )

    group = parser.add_mutually_exclusive_group()
    group.add_argument(
        "--to-ssh", action="store_true", help="Convert repo URLs to SSH format"
    )
    group.add_argument(
        "--to-https", action="store_true", help="Convert repo URLs to HTTPS format"
    )

    args = parser.parse_args()
    print(f"source: {args.source}")
    print(f"target: {args.target}")
    print(f"entire: {args.entire}")
    main(args=args)

    exit(0)

    pass
