import pytest # pyright: ignore[reportUnusedImport]
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from .conftest import KossJS


def test_net_socket(koss: KossJS):
    result = koss.eval("typeof require('koss:node/net').Socket")
    assert result == 'function'


def test_net_server(koss: KossJS):
    result = koss.eval("typeof require('koss:node/net').Server")
    assert result == 'function'


def test_net_create_server(koss: KossJS):
    result = koss.eval("typeof require('koss:node/net').createServer")
    assert result == 'function'


def test_net_connect(koss: KossJS):
    result = koss.eval("typeof require('koss:node/net').connect")
    assert result == 'function'


def test_net_create_connection(koss: KossJS):
    result = koss.eval("typeof require('koss:node/net').createConnection")
    assert result == 'function'


def test_net_is_ip_v4(koss: KossJS):
    result = koss.eval("require('koss:node/net').isIP('127.0.0.1')")
    assert result == '4'


def test_net_is_ip_v6(koss: KossJS):
    result = koss.eval("require('koss:node/net').isIP('::1')")
    assert result == '6'


def test_net_is_ip_invalid(koss: KossJS):
    result = koss.eval("require('koss:node/net').isIP('not-an-ip')")
    assert result == '0'


def test_net_is_ipv4(koss: KossJS):
    result = koss.eval("require('koss:node/net').isIPv4('192.168.1.1')")
    assert result == 'true'


def test_net_is_ipv6(koss: KossJS):
    result = koss.eval("require('koss:node/net').isIPv6('::1')")
    assert result == 'true'


def test_net_tcp_connect(koss: KossJS):
    result = koss.eval("typeof require('koss:node/net').connect")
    assert result == 'function'


def test_net_server_listen(koss: KossJS):
    result = koss.eval("""
        var net = require('koss:node/net');
        var server = net.createServer();
        typeof server.listen === 'function'
    """)
    assert result == 'true'


def test_stream_readable(koss: KossJS):
    result = koss.eval("typeof require('koss:node/stream').Readable")
    assert result == 'function'


def test_stream_writable(koss: KossJS):
    result = koss.eval("typeof require('koss:node/stream').Writable")
    assert result == 'function'


def test_stream_duplex(koss: KossJS):
    result = koss.eval("typeof require('koss:node/stream').Duplex")
    assert result == 'function'


def test_stream_transform(koss: KossJS):
    result = koss.eval("typeof require('koss:node/stream').Transform")
    assert result == 'function'


def test_dns_lookup(koss: KossJS):
    result = koss.eval("typeof require('koss:node/dns').lookup")
    assert result == 'function'


def test_dns_lookup_localhost(koss: KossJS):
    result = koss.eval("typeof require('koss:node/dns').lookup")
    assert result == 'function'


def test_tls(koss: KossJS):
    result = koss.eval("typeof require('koss:node/tls').connect")
    assert result == 'function'


def test_tls_server(koss: KossJS):
    result = koss.eval("typeof require('koss:node/tls').createServer")
    assert result == 'function'


def test_http_create_server(koss: KossJS):
    result = koss.eval("typeof require('koss:node/http').createServer")
    assert result == 'function'


def test_https_create_server(koss: KossJS):
    result = koss.eval("typeof require('koss:node/https').createServer")
    assert result == 'function'
