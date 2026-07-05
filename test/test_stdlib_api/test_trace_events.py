import pytest
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from .conftest import KossJS


def test_trace_events_create_tracing(koss: KossJS):
    result = koss.eval("typeof require('koss:node/trace_events').createTracing({categories: ['node']})")
    assert result == 'object'


def test_trace_events_tracing_class(koss: KossJS):
    result = koss.eval("typeof require('koss:node/trace_events').Tracing")
    assert result == 'function'


def test_trace_events_tracing_enable(koss: KossJS):
    result = koss.eval("""
        var tr = require('koss:node/trace_events').createTracing({categories: ['node']});
        tr.enable();
        tr.enabled
    """)
    assert result == 'true'


def test_trace_events_tracing_disable(koss: KossJS):
    result = koss.eval("""
        var tr = require('koss:node/trace_events').createTracing({categories: ['node']});
        tr.enable();
        tr.disable();
        tr.enabled
    """)
    assert result == 'false'


def test_trace_events_tracing_categories(koss: KossJS):
    result = koss.eval("""
        var tr = require('koss:node/trace_events').createTracing({categories: ['node', 'http']});
        tr.categories
    """)
    assert result == 'node,http'


def test_trace_events_get_enabled_categories(koss: KossJS):
    result = koss.eval("typeof require('koss:node/trace_events').getEnabledCategories")
    assert result == 'function'


def test_trace_events_enabled_categories_none(koss: KossJS):
    result = koss.eval("require('koss:node/trace_events').getEnabledCategories()")
    assert result == 'undefined' or result == ''


def test_trace_events_tracing_requires_categories(koss: KossJS):
    with pytest.raises(Exception):
        koss.eval("require('koss:node/trace_events').createTracing({categories: []})")
