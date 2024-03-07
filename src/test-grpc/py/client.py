import helloworld_pb2 as pb
import helloworld_pb2_grpc as pbg
from client_wrapper import ServiceClient


def run():
    users = ServiceClient(pbg, 'GreeterStub', 'localhost', 50051)
    # Insert example metadata
    metadata = [('ip', '127.0.0.1')]
    res = users.SayHello(pb.HelloRequest(name='World'), metadata)
    print('res:', res)


if __name__ == '__main__':
    run()
