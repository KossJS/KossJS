"""FS 沙箱测试 - 验证 FS 能力位对 __koss_fs_* 全局函数的控制

修复项 C1：低层 __koss_fs_* 全局函数此前无条件注册且不检查能力位，
导致 KOSS_CAP_SANDBOX（零权限）实例仍可读写/删除任意文件。
现改为按 FS_READ/FS_WRITE/FS_DELETE/FS_MKDIR/FS_RENAME 门控。
"""
import os
import pytest  # pyright: ignore[reportUnusedImport]
from kossjs_interface import KossJS


# 通过 JS try/catch 返回哨兵字符串，避免依赖错误如何传播到 Python。
_PROBE = "(function(){{try{{{body};return 'ok';}}catch(e){{return 'denied';}}}})()"


def _probe(js: str) -> str:
    return _PROBE.format(body=js)


# ── 拒绝：无能力位时应抛错 ────────────────────────────────────────────

def test_fs_read_denied_in_sandbox():
    """KOSS_CAP_SANDBOX 下 __koss_fs_read 应抛出权限错误。"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        assert js.eval(_probe("__koss_fs_read('nonexistent.txt')")) == "denied"
    finally:
        js.destroy()


def test_fs_write_denied_in_sandbox():
    """KOSS_CAP_SANDBOX 下 __koss_fs_write 应抛出权限错误。"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_SANDBOX)
    try:
        assert js.eval(_probe("__koss_fs_write('x.txt','data')")) == "denied"
    finally:
        js.destroy()


def test_fs_write_denied_with_read_only():
    """仅授予 FS_READ 时，写操作仍应被拒绝。"""
    js = KossJS(capabilities=KossJS.FS_READ)
    try:
        assert js.eval(_probe("__koss_fs_read('nonexistent.txt')")) == "ok"
        assert js.eval(_probe("__koss_fs_write('x.txt','data')")) == "denied"
        assert js.eval(_probe("__koss_fs_mkdir('d',0)")) == "denied"
        assert js.eval(_probe("__koss_fs_unlink('x.txt')")) == "denied"
    finally:
        js.destroy()


# ── 放行：授予对应能力位后应可用 ──────────────────────────────────────

def test_fs_read_allowed_with_cap():
    """授予 FS_READ 后 __koss_fs_read 不再抛权限错误（缺失文件返回 code!=0 字符串）。"""
    js = KossJS(capabilities=KossJS.FS_READ)
    try:
        assert js.eval(_probe("__koss_fs_read('nonexistent.txt')")) == "ok"
    finally:
        js.destroy()


def test_fs_roundtrip_with_caps(tmp_path):
    """授予 FS_READ|FS_WRITE 后可写入并读回文件内容（验证完整路径）。"""
    target = str(tmp_path / "koss_fs_test.txt").replace("\\", "/")
    js = KossJS(capabilities=KossJS.FS_READ | KossJS.FS_WRITE)
    try:
        write_ok = js.eval(_probe(f"__koss_fs_write('{target}','hello-koss')"))
        assert write_ok == "ok"
        assert os.path.exists(target)
        with open(target, "r", encoding="utf-8") as f:
            assert f.read() == "hello-koss"
    finally:
        js.destroy()


def test_fs_available_with_all_caps():
    """默认 KOSS_CAP_ALL 实例的 fs 全局函数不应被门控拦截（无回归）。"""
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        assert js.eval(_probe("__koss_fs_read('nonexistent.txt')")) == "ok"
        assert js.eval(_probe("__koss_fs_stat('nonexistent.txt')")) == "ok"
    finally:
        js.destroy()
