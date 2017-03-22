from __future__ import print_function

import json
import pprint
import requests
import argparse


parser = argparse.ArgumentParser(description='run against artifact JSON-RPC')
parser.add_argument('addr', help='address of artifact server')
parser.add_argument('method', help='method to use. Default=GetArtifacts',
                    default='GetArtifacts', nargs='?')

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
