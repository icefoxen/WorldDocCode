Assume the client is running their own IPFS system so we don’t have to worry about that.

    Identity server: This can be just an HTTP server that returns a user’s GPG public key as a document. As a testbed this
doesn’t need anything to be dynamic.

    Name server: This will also be an HTTP server with a little REST interface. You either GET a name to get the content
address associated with the name, or POST to an existing name to update it with a new content address. Your POST request will
contain: a user id, a datetime, an IPFS address to update to, and a signature of the other elements composed. Errors will
occur if: the user is not authorized, the signature does not match. Errors should occur (but won’t for the prototype) if the
IPFS address does not exist or points to some invalid sort of data.

    Client: Minimum possible thing, probably just a command-line thing like curl. Will take a server name and either fetch the
latest message or post a new message. Once that works it will be configurable to fetch the last N messages.


