"""# !!! WARNING !!!! This is migrated code that needs to be cleaned up. It is
not maintained and will be used again when the tracker is being developed.

Before running server:

    run `echo DATABASE_URL=postgres://username:password@localhost/artifact > .env'
        to create .env file in /artifact directory
    (the database name might be changeable, as long as it is specified in the .env)

Run the artifact server with:
    cargo run --features server,tracker -- -v serve

Then in a separate shell run this script to interact with it by calling:
    python2 scripts/api.py

This script may grow in the future.

"""

from __future__ import print_function

import json
import pprint
import requests
import argparse


def parse_args():
    parser = argparse.ArgumentParser(
        description='run against artifact JSON-RPC')
    parser.add_argument('addr', help='address of artifact server')
    parser.add_argument(
        'method', help='method to use: create|read|update|delete')

    args = parser.parse_args()
    addr = args.addr + '/json-rpc'

    # TODO: call one of the bellow methods depending on method


def read_runs(addr):
    """Get specific runs based on some filters."""
    payload = {
        'jsonrpc': '2.0',
        'id': 1,
        'method': 'GetRuns',
        'params': {'min_epoch': '6', 'max_epoch': '999', 'versions': [
            {'major': '2', 'minor': '7', 'patch': '6'},
            {'major': '6', 'build': '4323'}]}
    }

    print("calling with addr={}, payload={}".format(addr, payload))
    response = requests.get(addr, data=json.dumps(payload))
    try:
        print('json response:')
        pprint.pprint(response.json())
    except:
        print('Got error:\n')
        print(response.text)


def read_all_runs(addr):
    """Get all tracked test cases."""
    payload = {
        'jsonrpc': '2.0',
        'id': 1,
        'method': 'GetAllTestRuns'
    }

    print("calling with addr={}, payload={}".format(addr, payload))
    response = requests.get(addr, data=json.dumps(payload))
    try:
        print('json response:')
        pprint.pprint(response.json())
    except:
        print('Got error:\n')
        print(response.text)


def create_test_run(addr):
    """create a fake test run."""
    payload = {
        'jsonrpc': '2.0',
        'id': 1,
        'method': args.method,
        'params': {'test_name': 'testymctestersons',
                   'passed': 'true',
                   'artifacts':  ['REQ-server'],
                   'epoch': 678.889,
                   'version_id': 15}
    }

    print("calling with addr={}, payload={}".format(addr, payload))
    response = requests.get(addr, data=json.dumps(payload))
    try:
        print('json response:')
        pprint.pprint(response.json())
    except:
        print('Got error:\n')
        print(response.text)
