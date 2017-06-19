import requests


class JsonRpc(requests.Session):
    rpc_path = '/json-rpc'

    def __init__(self, address):
        super(JsonRpc, self).__init__()
        self.host = address + self.rpc_path
        self._id = 0

    def request(self, http_method, url, json=None, **kwargs):
        """
        just overrides the super class to insert mandatory fields,
        then calls it with those additions.
        You are not supposed to call this method directly,
        the super class will take care of then when calling post(), get() …

        :param http_method:
        :param url:
        :param json:
        :param kwargs:
        :return:
        """
        # raise error as 'method' and 'params' are mandatory and have to be defined
        if json is None:
            raise ValueError('json parameter undefined, cannot be None')
        if 'method' not in json:
            raise ValueError('json-rpc method is empty, must be defined')
        if 'params' not in json:
            raise ValueError('json-rpc params is empty, must be defined')

        # set jsonrpc and id to default values
        self._id += 1
        json.setdefault('jsonrpc', '2.0')
        json.setdefault('id', self._id)

        return super(JsonRpc, self).request(http_method, url, json=json, **kwargs)


