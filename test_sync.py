import importlib
import unittest
from unittest.mock import mock_open, patch

sync_script = importlib.import_module("sync")


class TestSyncScript(unittest.TestCase):

    def setUp(self):
        # 模拟 source_data
        self.source_content = [
            {
                "repo": "repo_a",
                "commit": "new_sha_a",
                "path": "path_a",
                "notes": "source_notes",
            },
            {
                "repo": "repo_b",
                "commit": "new_sha_b",
                "path": "path_b",
                "notes": "source_notes_b",
            },
        ]
        # 模拟 target_data
        self.target_content = [
            {
                "repo": "repo_a",
                "commit": "old_sha_a",
                "path": "path_a",
                "notes": "target_notes",
            },
            {
                "repo": "repo_b",
                "commit": "old_sha_b",
                "path": "path_b",
                "notes": "target_notes_b",
            },
        ]

    @patch("os.path.exists", return_value=True)
    @patch("builtins.open", new_callable=mock_open)
    @patch("json.load")
    @patch("json.dump")
    def test_normal_sync(self, mock_json_dump, mock_json_load, mock_file, mock_exists):
        """测试普通的 commit 替换逻辑"""
        # 设置模拟返回值
        mock_json_load.side_effect = [self.source_content, self.target_content]

        # 构造模拟参数
        args = unittest.mock.Mock()
        args.source = "s.json"
        args.target = "t.json"
        args.entire = False
        args.skip = None

        sync_script.main(args)

        # 验证结果：target_data 的 commit 是否被更新
        updated_data = mock_json_dump.call_args[0][0]
        self.assertEqual(updated_data[0]["commit"], "new_sha_a")
        self.assertEqual(
            updated_data[0]["notes"], "target_notes"
        )  # 非 entire 模式，notes 不应变

    @patch("os.path.exists", return_value=True)
    @patch("builtins.open", new_callable=mock_open)
    @patch("json.load")
    @patch("json.dump")
    def test_entire_sync(self, mock_json_dump, mock_json_load, mock_file, mock_exists):
        """测试 --entire 模式，整个对象替换"""
        mock_json_load.side_effect = [self.source_content, self.target_content]

        args = unittest.mock.Mock()
        args.source = "s.json"
        args.target = "t.json"
        args.entire = True
        args.skip = None

        sync_script.main(args)

        updated_data = mock_json_dump.call_args[0][0]
        # 验证 notes 是否也同步了（source 是 source_notes）
        self.assertEqual(updated_data[0]["notes"], "source_notes")

    @patch("os.path.exists", return_value=True)
    @patch("builtins.open", new_callable=mock_open)
    @patch("json.load")
    @patch("json.dump")
    def test_skip_list(self, mock_json_dump, mock_json_load, mock_file, mock_exists):
        """测试 --skip 列表功能"""
        mock_json_load.side_effect = [self.source_content, self.target_content]

        args = unittest.mock.Mock()
        args.source = "s.json"
        args.target = "t.json"
        args.entire = False
        args.skip = ["path_a"]  # 跳过 path_a

        sync_script.main(args)

        updated_data = mock_json_dump.call_args[0][0]
        # path_a 应该保持 old_sha_a
        self.assertEqual(updated_data[0]["commit"], "old_sha_a")
        # path_b 应该更新为 new_sha_b
        self.assertEqual(updated_data[1]["commit"], "new_sha_b")
        # --- 新增：URL 转换工具函数测试 ---

    def test_convert_git_url_logic(self):
        """验证 URL 转换函数的正则逻辑"""
        https_url = "https://github.com/siliconflow/ComfyUI.git"
        ssh_url = "git@github.com:siliconflow/ComfyUI.git"

        # 测试 HTTPS -> SSH
        self.assertEqual(sync_script.convert_git_url(https_url, "ssh"), ssh_url)
        # 测试 SSH -> HTTPS
        self.assertEqual(sync_script.convert_git_url(ssh_url, "https"), https_url)
        # 测试不带 .git 后缀的自动补全
        self.assertEqual(
            sync_script.convert_git_url("https://github.com/a/b", "ssh"),
            "git@github.com:a/b.git",
        )
        # 测试无法识别的 URL (应原样返回)
        self.assertEqual(
            sync_script.convert_git_url("invalid_url", "ssh"), "invalid_url"
        )

    # --- 新增：集成测试 (to-ssh) ---
    @patch("os.path.exists", return_value=True)
    @patch("builtins.open", new_callable=mock_open)
    @patch("json.load")
    @patch("json.dump")
    def test_sync_with_to_ssh(
        self, mock_json_dump, mock_json_load, mock_file, mock_exists
    ):
        """测试同步时强制转换为 SSH 格式"""
        # Source 是 HTTPS，Target 是 HTTPS
        source = [{"repo": "https://github.com/a/b.git", "commit": "new", "path": "p"}]
        target = [{"repo": "https://github.com/a/b.git", "commit": "old", "path": "p"}]
        mock_json_load.side_effect = [source, target]

        args = unittest.mock.Mock()
        args.source = "s.json"
        args.target = "t.json"
        args.entire = False
        args.skip = None
        args.to_ssh = True
        args.to_https = False  # 互斥组模拟

        sync_script.main(args)

        updated_data = mock_json_dump.call_args[0][0]
        # 验证 commit 更新了，且 URL 变成了 SSH
        self.assertEqual(updated_data[0]["commit"], "new")
        self.assertEqual(updated_data[0]["repo"], "git@github.com:a/b.git")

    # --- 新增：集成测试 (to-https) ---
    @patch("os.path.exists", return_value=True)
    @patch("builtins.open", new_callable=mock_open)
    @patch("json.load")
    @patch("json.dump")
    def test_sync_with_to_https(
        self, mock_json_dump, mock_json_load, mock_file, mock_exists
    ):
        """测试同步时强制转换为 HTTPS 格式"""
        # Source 是 SSH，Target 是 SSH
        source = [{"repo": "git@github.com:a/b.git", "commit": "new", "path": "p"}]
        target = [{"repo": "git@github.com:a/b.git", "commit": "old", "path": "p"}]
        mock_json_load.side_effect = [source, target]

        args = unittest.mock.Mock()
        args.source = "s.json"
        args.target = "t.json"
        args.entire = False
        args.skip = None
        args.to_ssh = False
        args.to_https = True

        sync_script.main(args)

        updated_data = mock_json_dump.call_args[0][0]
        # 验证 URL 变成了 HTTPS
        self.assertEqual(updated_data[0]["repo"], "https://github.com/a/b.git")


if __name__ == "__main__":
    unittest.main()
