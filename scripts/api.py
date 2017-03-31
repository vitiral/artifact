"""
Before running server:
    set up and run postgres server with database `artifact` and table `test_name`
    run `echo DATABASE_URL=postgres://username:password@localhost/artifact > .env' 
        to create .env file in /artifact directory
    (the database name might be changeable, as long as it is specified in the .env)

Run the artifact server with:
    cargo run --features server -- -v serve

Then in a separate shell run this script to interact with it by calling:
    python2 scripts/api.py

This script may grow in the future.
"""

from __future__ import print_function

import json
import pprint
import requests
import argparse


parser = argparse.ArgumentParser(description='run against artifact JSON-RPC')
parser.add_argument('addr', help='address of artifact server')
parser.add_argument('method', help='method to use. Default=GetArtifacts',
                    default='GetTests', nargs='?')

args = parser.parse_args()
addr = args.addr + '/json-rpc'

payload = {
    'jsonrpc': '2.0',
    'id': 1,
    'method': args.method,
}

print("calling with addr={}, payload={}".format(addr, payload))
response = requests.get(addr, data=json.dumps(payload))
try:
    print('json response:')
    pprint.pprint(response.json())
except:
    print('Got error:\n')
    print(response.text)
