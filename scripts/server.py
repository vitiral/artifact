"""
Before running server:
    set up and run postgres server with database `artifact` and table `test_name`
    run `echo DATABASE_URL=postgres://username:password@localhost/artifact > .env'
        to create .env file in /artifact directory
    (the database name might be changeable, as long as it is specified in the .env)

Run the artifact server with:
    cargo run --features server -- -v serve

When running this script in interactive mode you can access all functions through:
    create, read, update and delete
or directly through the object:
    api
"""

from __future__ import print_function

import argparse
import code
import pprint

from JsonRpc import JsonRpc


class CrudApi(JsonRpc):
    def create_art(self, params):
        payload = {
            'method': 'CreateArtifacts',
            'params': params,
        }
        self.post(self.host, json=payload)

    def read_art(self, params):
        payload = {
            'method': 'ReadArtifacts',
            'params': params,
        }
        self.post(self.host, json=payload)

    def update_art(self, params):
        payload = {
            'method': 'UpdateArtifacts',
            'params': params,
        }
        self.post(self.host, json=payload)

    def delete_art(self, params):
        payload = {
            'method': 'DeleteArtifacts',
            'params': params,
        }
        self.post(self.host, json=payload)


def make_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument('address', help='address of artifact server')
    parser.add_argument('-i', '--interactive', help='open python shell to interact with the server',
                        action='store_true')

    subparsers = parser.add_subparsers(help='run against artifact JSON-RPC')

    # create = subparsers.add_parser('create')
    # create.add_argument('')

    return parser


def start_interactive(api):
    create = api.create_art
    read = api.read_art
    update = api.update_art
    delete = api.delete_art
    code.interact(banner=__doc__, local=locals())


def main():
    parser = make_parser()
    args = parser.parse_args()
    api = CrudApi(args.address)
    if args.interactive:
        start_interactive(api)
    else:
        raise SystemExit('sorry, the rest is not implemented, yet')
#    print("calling addr={} with payload={}".format(api.host, payload))

#    response = requests.get(addr, json=payload)
#    try:
#        print('json response:')
#        pprint.pprint(response.json())
#    except:
#        print('Got error:\n')
#        print(response.text)


if __name__ == '__main__':
    main()
