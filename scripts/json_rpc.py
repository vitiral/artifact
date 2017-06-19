import requests


class JsonRpc(requests.Session):
    """This class serves no purpose other than providing very basic checks to
    conform to the JSON-RPC standard. The mandatory Request members are:
    jsonrpc, id and method. [1]

      - jsonrpc must be set to '2.0'
      - id can be any string or integral value and is preferred to be not Null.
      - method must be provided by the caller.

    A Session object will also keep track of Cookies and provides basic authentication for the session life time.

    [1] http://www.jsonrpc.org/specification#request_object

    """
    rpc_path = '/json-rpc'

    def __init__(self, host):
        super(JsonRpc, self).__init__()
        self.address = host + self.rpc_path
        self._id = 0

    def request(self, http_method, url, json=None, **kwargs):
        """Overrides the super class to insert mandatory fields, then calls it
        with those additions. You are not supposed to call this method
        directly, the super class will take care of then when calling post(),
        get() … etc. on this object.

        :param http_method:
        :param url:
        :param json: all fields to be sent, must at least contain 'method'
        :type json: dict
        :param kwargs: all other optional fields to configure the Request
        :rtype: requests.Response

        """
        # raise error as 'method' and 'params' are mandatory and have to be defined
        if json is None:
            raise ValueError('json parameter undefined, cannot be None')
        if 'method' not in json:
            raise ValueError('json-rpc method is empty, must be defined')

        # set jsonrpc and id to default values without overriding existing values.
        self._id += 1
        json.setdefault('jsonrpc', '2.0')
        json.setdefault('id', self._id)

        return super(JsonRpc, self).request(http_method, url, json=json, **kwargs)
