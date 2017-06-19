"""Before running server: set up and run postgres server with database
`artifact` and table `test_name` run `echo
DATABASE_URL=postgres://username:password@localhost/artifact > .env' to create.

.env file in /artifact directory (the database name might be changeable, as
long as it is specified in the .env)

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

from json_rpc import JsonRpc


class CrudApi(JsonRpc):
    def create_artifact(self, params):
        payload = {
            'method': 'CreateArtifacts',
            'params': params,
        }
        self.put(self.address, json=payload)

    def read_artifact(self, params):
        payload = {
            'method': 'ReadArtifacts',
            'params': params,
        }
        self.put(self.address, json=payload)

    def update_artifact(self, params):
        payload = {
            'method': 'UpdateArtifacts',
            'params': params,
        }
        self.put(self.address, json=payload)

    def delete_artifact(self, params):
        payload = {
            'method': 'DeleteArtifacts',
            'params': params,
        }
        self.put(self.address, json=payload)


def make_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument('address', help='address of artifact server')
    parser.add_argument('-i', '--interactive', help='open python shell to interact with the server',
                        action='store_true')

    subparsers = parser.add_subparsers(help='run against artifact JSON-RPC')

    # create = subparsers.add_parser('create')
    # create.add_argument('')

    return parser


def readline_setup(exports):
    """setup readline completion, if available.

    :param exports: the namespace to be used for completion
    :return: True on success

    """
    try:
        import readline
    except ImportError:
        # no completion for you.
        readline = None
        return False
    else:
        import rlcompleter
        readline.set_completer(
            rlcompleter.Completer(namespace=exports).complete)
        return True


def start_interactive(api):
    """start an interactive shell for the API.

    :param api: the CrudApi session object.

    """
    create = api.create_artifact
    read = api.read_artifact
    update = api.update_artifact
    delete = api.delete_artifact

    local = locals()
    exports = globals().copy()
    exports.update(local)

    # completion for global and local namespace
    readline_setup(exports)
    # dir() will only show locals()
    code.interact(banner=__doc__, local=local)


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
