import pytest
from kossjs_interface import KossJS


def test_default_audit_mask_is_zero():
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        assert js.get_audit_mask() == 0
    finally:
        js.destroy()


def test_set_audit_mask():
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        js.set_audit_mask(KossJS.FS_READ | KossJS.NET_FETCH)
        assert js.get_audit_mask() == (KossJS.FS_READ | KossJS.NET_FETCH)
    finally:
        js.destroy()


def test_audit_mask_ignores_ungranted_capabilities():
    js = KossJS(capabilities=KossJS.FS_READ)
    try:
        js.set_audit_mask(KossJS.FS_READ | KossJS.NET_FETCH)
        mask = js.get_audit_mask()
        assert mask == KossJS.FS_READ
    finally:
        js.destroy()


def test_audit_mask_zero_disables_audit():
    js = KossJS(capabilities=KossJS.KOSS_CAP_ALL)
    try:
        js.set_audit_mask(0)
        assert js.get_audit_mask() == 0
    finally:
        js.destroy()
