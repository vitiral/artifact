"""Interact with the artifact server.

TODO: this is still a work in progress. We need:
    - an Artifact data type with from_dict and to_dict methods
      and an `__init__` that we can use for making artifacts
    - all methods should ONLY accept and return the Artifact
      data type
    - better command line interface for interacting with the API

"""

from __future__ import print_function

import argparse
import code
import pprint

from py_helpers.json_rpc import JsonRpc


class CrudApi(JsonRpc):
    """Class for interacting with the artifact CRUD server API."""

    def create_artifacts(self, params):
        """create a list of artifacts."""
        payload = {
            'method': 'CreateArtifacts',
            'params': params,
        }
        return self.put(self.address, json=payload)

    def read_artifacts(self, params=None):
        """read the artifacts in a list."""
        payload = {
            'method': 'ReadArtifacts',
            'params': params,
        }
        r = self.put(self.address, json=payload)
        r.raise_for_status()
        return r.json()

    def update_artifacts(self, params):
        """update the list of artifacts.

        Artifacts must already exist.

        """
        payload = {
            'method': 'UpdateArtifacts',
            'params': params,
        }
        return self.put(self.address, json=payload)

    def delete_artifacts(self, params):
        """delete the list of artifact ids."""
        payload = {
            'method': 'DeleteArtifacts',
            'params': params,
        }
        return self.put(self.address, json=payload)


def parse_args(args=None):
    """parse cmdline arguments."""
    parser = argparse.ArgumentParser()
    parser.add_argument(
        'address', nargs='?',
        default="http://127.0.0.1:5373", help='address of artifact server')

    parser.add_argument(
        '-i', '--interactive',
        help='open python shell to interact with the server',
        action='store_true')

    return parser.parse_args(args=args)


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


# pylint: disable=unused-variable
def start_interactive(api):
    """start an interactive shell for the API.

    :param CrudApi api: the CrudApi session object.

    """
    # make pretty-printing easier
    pp = pprint.pprint

    create = api.create_artifacts
    read = api.read_artifacts
    update = api.update_artifacts
    delete = api.delete_artifacts

    local = locals()
    exports = globals().copy()
    exports.update(local)

    # completion for global and local namespace
    readline_setup(exports)
    # dir() will only show locals()
    code.interact(banner=__doc__, local=local)


def main():
    """execute as a script."""
    args = parse_args()
    api = CrudApi(args.address)
    if args.interactive:
        start_interactive(api)
    else:
        raise NotImplementedError('sorry, the rest is not implemented, yet')


if __name__ == '__main__':
    main()
