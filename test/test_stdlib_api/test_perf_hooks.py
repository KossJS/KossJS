import pytest # pyright: ignore[reportUnusedImport]
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from .conftest import KossJS


def test_perf_hooks_performance(koss: KossJS):
    result = koss.eval("typeof require('koss:node/perf_hooks').performance")
    assert result == 'object'


def test_perf_hooks_performance_now(koss: KossJS):
    result = koss.eval("typeof require('koss:node/perf_hooks').performance.now()")
    assert result == 'number'


def test_perf_hooks_performance_entry(koss: KossJS):
    result = koss.eval("typeof require('koss:node/perf_hooks').PerformanceEntry")
    assert result == 'function'


def test_perf_hooks_performance_mark(koss: KossJS):
    result = koss.eval("typeof require('koss:node/perf_hooks').PerformanceMark")
    assert result == 'function'


def test_perf_hooks_performance_measure(koss: KossJS):
    result = koss.eval("typeof require('koss:node/perf_hooks').PerformanceMeasure")
    assert result == 'function'


def test_perf_hooks_performance_observer(koss: KossJS):
    result = koss.eval("typeof require('koss:node/perf_hooks').PerformanceObserver")
    assert result == 'function'


def test_perf_hooks_constants(koss: KossJS):
    result = koss.eval("typeof require('koss:node/perf_hooks').constants")
    assert result == 'object'


def test_perf_hooks_histogram(koss: KossJS):
    result = koss.eval("typeof require('koss:node/perf_hooks').createHistogram()")
    assert result == 'object'


def test_perf_hooks_timerify(koss: KossJS):
    result = koss.eval("require('koss:node/perf_hooks').timerify(function(){return 1;})()")
    assert result == '1'
