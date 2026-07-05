import pytest # pyright: ignore[reportUnusedImport]
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from .conftest import KossJS


def test_diagnostics_channel_channel(koss: KossJS):
    result = koss.eval("typeof require('koss:node/diagnostics_channel').channel('test')")
    assert result == 'object'


def test_diagnostics_channel_has_subscribers_false(koss: KossJS):
    result = koss.eval("var dc = require('koss:node/diagnostics_channel'); dc.hasSubscribers('test')")
    assert result == 'undefined'


def test_diagnostics_channel_subscribe(koss: KossJS):
    result = koss.eval("""
        var dc = require('koss:node/diagnostics_channel');
        var ch = dc.channel('test1');
        ch.subscribe(function(data){});
        ch.hasSubscribers
    """)
    assert result == 'true'


def test_diagnostics_channel_unsubscribe(koss: KossJS):
    result = koss.eval("""
        var dc = require('koss:node/diagnostics_channel');
        var ch = dc.channel('test2');
        var fn = function(data){};
        ch.subscribe(fn);
        ch.unsubscribe(fn);
        ch.hasSubscribers
    """)
    assert result == 'false'


def test_diagnostics_channel_publish(koss: KossJS):
    result = koss.eval("""
        var dc = require('koss:node/diagnostics_channel');
        var ch = dc.channel('test3');
        var received = null;
        ch.subscribe(function(data) { received = data; });
        ch.publish(42);
        received
    """)
    assert result == '42'


def test_diagnostics_channel_channel_name(koss: KossJS):
    result = koss.eval("require('koss:node/diagnostics_channel').channel('test4').name")
    assert result == 'test4'


def test_diagnostics_channel_reuse_channel(koss: KossJS):
    result = koss.eval("""
        var dc = require('koss:node/diagnostics_channel');
        var a = dc.channel('test5');
        var b = dc.channel('test5');
        a === b
    """)
    assert result == 'true'
